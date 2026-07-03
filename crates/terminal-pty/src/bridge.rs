use std::fmt;

use crate::{
    PtyBackend, PtyError, PtyEvent, PtyRuntimeConfig, PtySessionConfig, PtySessionRunner, PtySize,
};

pub trait PtyEventSink {
    type Error;

    fn apply_output(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;

    fn resize(&mut self, size: PtySize) -> Result<(), Self::Error>;

    fn current_size(&self) -> Option<PtySize> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyBridgeError<SinkError> {
    Pty(PtyError),
    Sink(SinkError),
}

impl<SinkError> fmt::Display for PtyBridgeError<SinkError>
where
    SinkError: fmt::Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pty(error) => write!(formatter, "{error}"),
            Self::Sink(error) => write!(formatter, "PTY event sink failed: {error}"),
        }
    }
}

impl<SinkError> std::error::Error for PtyBridgeError<SinkError> where
    SinkError: fmt::Debug + fmt::Display
{
}

pub struct PtyBridge<Sink> {
    sink: Sink,
}

impl<Sink> PtyBridge<Sink> {
    #[must_use]
    pub const fn new(sink: Sink) -> Self {
        Self { sink }
    }

    #[must_use]
    pub const fn sink(&self) -> &Sink {
        &self.sink
    }

    #[must_use]
    pub fn sink_mut(&mut self) -> &mut Sink {
        &mut self.sink
    }
}

impl<Sink> PtyBridge<Sink>
where
    Sink: PtyEventSink,
{
    pub fn start_session(
        &mut self,
        backend: &impl PtyBackend,
        config: &PtySessionConfig,
        runtime: PtyRuntimeConfig,
    ) -> Result<PtySessionRunner, PtyBridgeError<Sink::Error>> {
        self.sink
            .resize(config.size())
            .map_err(PtyBridgeError::Sink)?;
        PtySessionRunner::spawn(backend, config, runtime).map_err(PtyBridgeError::Pty)
    }

    pub fn apply_event(&mut self, event: &PtyEvent) -> Result<(), PtyBridgeError<Sink::Error>> {
        match event {
            PtyEvent::Output(bytes) => self.sink.apply_output(bytes).map_err(PtyBridgeError::Sink),
            PtyEvent::Resize(size) => self.sink.resize(*size).map_err(PtyBridgeError::Sink),
            PtyEvent::Eof | PtyEvent::Exit(_) => Ok(()),
        }
    }

    pub fn resize_session(
        &mut self,
        runner: &mut PtySessionRunner,
        size: PtySize,
    ) -> Result<(), PtyBridgeError<Sink::Error>> {
        let previous_size = self.sink.current_size();
        self.sink.resize(size).map_err(PtyBridgeError::Sink)?;
        if let Err(error) = runner.resize(size) {
            if let Some(previous_size) = previous_size {
                let _ = self.sink.resize(previous_size);
            }
            return Err(PtyBridgeError::Pty(PtyError::partial_resize(size, error)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::fmt;
    use std::io::{self, Cursor, Write};
    use std::sync::{Arc, Mutex};

    use terminal_core::{Terminal, TerminalConfig, TerminalError};

    use super::{PtyBridge, PtyBridgeError, PtyEventSink};
    use crate::{
        PtyBackend, PtyChild, PtyChildKiller, PtyError, PtyErrorKind, PtyEvent, PtyExit,
        PtyPlatformMetadata, PtyRuntimeConfig, PtySession, PtySessionConfig, PtySize,
        session::PtySessionBackend,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct SinkError(String);

    impl fmt::Display for SinkError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(&self.0)
        }
    }

    struct CoreSink {
        terminal: Terminal,
    }

    impl CoreSink {
        fn new(columns: usize, rows: usize) -> Self {
            Self {
                terminal: Terminal::new(columns, rows).expect("terminal dimensions are valid"),
            }
        }

        fn viewport_text(&mut self) -> String {
            self.terminal.render_snapshot().viewport_rows()[0]
                .cells()
                .iter()
                .map(terminal_core::RenderCell::ch)
                .collect()
        }
    }

    impl PtyEventSink for CoreSink {
        type Error = SinkError;

        fn apply_output(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
            self.terminal.advance_bytes(bytes);
            Ok(())
        }

        fn resize(&mut self, size: PtySize) -> Result<(), Self::Error> {
            self.terminal
                .resize(usize::from(size.columns()), usize::from(size.rows()))
                .map_err(|error| SinkError(error.to_string()))
        }

        fn current_size(&self) -> Option<PtySize> {
            let dimensions = self.terminal.dimensions();
            PtySize::new(
                u16::try_from(dimensions.columns()).ok()?,
                u16::try_from(dimensions.rows()).ok()?,
            )
            .ok()
        }
    }

    #[derive(Debug)]
    struct FakeChild;

    impl PtyChild for FakeChild {
        fn try_wait(&mut self) -> Result<Option<PtyExit>, PtyError> {
            Ok(Some(PtyExit::new(0, None)))
        }

        fn wait(&mut self) -> Result<PtyExit, PtyError> {
            Ok(PtyExit::new(0, None))
        }

        fn process_id(&self) -> Option<u32> {
            Some(9)
        }

        fn kill(&mut self) -> Result<(), PtyError> {
            Ok(())
        }

        fn clone_killer(&self) -> Box<dyn PtyChildKiller> {
            Box::new(FakeChildKiller)
        }
    }

    #[derive(Debug)]
    struct FakeChildKiller;

    impl PtyChildKiller for FakeChildKiller {
        fn kill(&mut self) -> Result<(), PtyError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct NullWriter;

    impl Write for NullWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct FakeSessionBackend {
        resize_log: Arc<Mutex<Vec<PtySize>>>,
        fail_resize: bool,
    }

    impl PtySessionBackend for FakeSessionBackend {
        fn resize(&self, size: PtySize) -> Result<(), PtyError> {
            if self.fail_resize {
                Err(PtyError::backend("resize", "backend refused resize"))
            } else {
                self.resize_log.lock().expect("resize log").push(size);
                Ok(())
            }
        }
    }

    struct FakeBackend {
        spawn_calls: Cell<usize>,
        resize_log: Arc<Mutex<Vec<PtySize>>>,
        fail_resize: bool,
    }

    impl FakeBackend {
        fn new(resize_log: Arc<Mutex<Vec<PtySize>>>, fail_resize: bool) -> Self {
            Self {
                spawn_calls: Cell::new(0),
                resize_log,
                fail_resize,
            }
        }
    }

    impl PtyBackend for FakeBackend {
        fn spawn(&self, config: &PtySessionConfig) -> Result<PtySession, PtyError> {
            self.spawn_calls.set(self.spawn_calls.get() + 1);
            Ok(PtySession::new(
                Box::new(Cursor::new(Vec::new())),
                Box::<NullWriter>::default(),
                Box::new(FakeChild),
                PtyPlatformMetadata::new("fake", Some(9), config.size()),
                Box::new(FakeSessionBackend {
                    resize_log: Arc::clone(&self.resize_log),
                    fail_resize: self.fail_resize,
                }),
            ))
        }
    }

    fn config(size: PtySize) -> PtySessionConfig {
        PtySessionConfig::new(crate::PtyCommand::new("hera-test"), size)
    }

    #[test]
    fn output_events_are_applied_to_terminal_core() {
        let mut bridge = PtyBridge::new(CoreSink::new(5, 1));

        bridge
            .apply_event(&PtyEvent::Output(b"hello".to_vec()))
            .expect("output should apply");

        assert_eq!(bridge.sink_mut().viewport_text(), "hello");
    }

    #[test]
    fn chunked_output_matches_concatenated_core_replay() {
        let chunks = [b"he".as_slice(), b"ll".as_slice(), b"o".as_slice()];
        let mut chunked = PtyBridge::new(CoreSink::new(5, 1));
        for chunk in chunks {
            chunked
                .apply_event(&PtyEvent::Output(chunk.to_vec()))
                .expect("chunk should apply");
        }

        let mut concatenated = CoreSink::new(5, 1);
        concatenated.apply_output(b"hello").expect("output applies");

        assert_eq!(
            chunked.sink_mut().viewport_text(),
            concatenated.viewport_text()
        );
    }

    #[test]
    fn malformed_utf8_is_left_to_terminal_core_parser_policy() {
        let mut bridge = PtyBridge::new(CoreSink::new(4, 1));

        bridge
            .apply_event(&PtyEvent::Output(vec![0xf0, 0x28, 0x8c, 0x28]))
            .expect("malformed UTF-8 should not panic");

        assert!(
            bridge
                .sink()
                .terminal
                .actions()
                .iter()
                .any(|action| matches!(
                    action,
                    terminal_core::TerminalAction::Print(printable)
                        if printable.ch() == char::REPLACEMENT_CHARACTER
                ))
        );
    }

    #[test]
    fn core_resize_is_validated_before_pty_spawn() {
        let resize_log = Arc::new(Mutex::new(Vec::new()));
        let backend = FakeBackend::new(Arc::clone(&resize_log), false);
        let size = PtySize::new(100, 30).expect("valid PTY size");
        let terminal = Terminal::with_config(
            TerminalConfig::new(80, 24).expect("valid initial terminal size"),
        );
        struct RejectingSink {
            terminal: Terminal,
        }
        impl PtyEventSink for RejectingSink {
            type Error = TerminalError;

            fn apply_output(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
                self.terminal.advance_bytes(bytes);
                Ok(())
            }

            fn resize(&mut self, _size: PtySize) -> Result<(), Self::Error> {
                self.terminal.resize(0, 24)
            }
        }
        let mut bridge = PtyBridge::new(RejectingSink { terminal });

        let error = match bridge.start_session(&backend, &config(size), PtyRuntimeConfig::default())
        {
            Ok(_) => panic!("core resize failure should stop spawn"),
            Err(error) => error,
        };

        assert!(matches!(error, PtyBridgeError::Sink(_)));
        assert_eq!(backend.spawn_calls.get(), 0);
        assert!(resize_log.lock().expect("resize log").is_empty());
    }

    #[test]
    fn resize_updates_core_then_pty_backend() {
        let resize_log = Arc::new(Mutex::new(Vec::new()));
        let backend = FakeBackend::new(Arc::clone(&resize_log), false);
        let mut bridge = PtyBridge::new(CoreSink::new(80, 24));
        let size = PtySize::new(80, 24).expect("valid size");
        let mut runner = bridge
            .start_session(&backend, &config(size), PtyRuntimeConfig::default())
            .expect("session starts");
        let resized = PtySize::new(100, 30).expect("valid resize");

        bridge
            .resize_session(&mut runner, resized)
            .expect("resize should work");

        assert_eq!(
            bridge.sink().terminal.dimensions(),
            terminal_core::Dimensions::new(100, 30).expect("valid dimensions")
        );
        assert_eq!(&*resize_log.lock().expect("resize log"), &[resized]);
    }

    #[test]
    fn pty_resize_failure_after_core_resize_returns_partial_resize() {
        let resize_log = Arc::new(Mutex::new(Vec::new()));
        let backend = FakeBackend::new(Arc::clone(&resize_log), true);
        let mut bridge = PtyBridge::new(CoreSink::new(80, 24));
        let size = PtySize::new(80, 24).expect("valid size");
        let mut runner = bridge
            .start_session(&backend, &config(size), PtyRuntimeConfig::default())
            .expect("session starts");
        let resized = PtySize::new(100, 30).expect("valid resize");

        let error = bridge
            .resize_session(&mut runner, resized)
            .expect_err("PTY resize should fail after core resize");

        assert!(matches!(
            error,
            PtyBridgeError::Pty(error) if error.kind() == PtyErrorKind::PartialResize
        ));
        assert_eq!(
            bridge.sink().terminal.dimensions(),
            terminal_core::Dimensions::new(80, 24).expect("rolled back dimensions")
        );
        assert!(resize_log.lock().expect("resize log").is_empty());
    }

    #[test]
    fn final_core_dimensions_match_last_successful_pty_resize() {
        let resize_log = Arc::new(Mutex::new(Vec::new()));
        let backend = FakeBackend::new(Arc::clone(&resize_log), false);
        let mut bridge = PtyBridge::new(CoreSink::new(80, 24));
        let mut runner = bridge
            .start_session(
                &backend,
                &config(PtySize::new(80, 24).expect("valid size")),
                PtyRuntimeConfig::default(),
            )
            .expect("session starts");
        let first = PtySize::new(100, 30).expect("valid resize");
        let second = PtySize::new(120, 40).expect("valid resize");

        bridge
            .resize_session(&mut runner, first)
            .expect("first resize");
        bridge
            .resize_session(&mut runner, second)
            .expect("second resize");

        assert_eq!(
            bridge.sink().terminal.dimensions(),
            terminal_core::Dimensions::new(120, 40).expect("valid dimensions")
        );
        assert_eq!(&*resize_log.lock().expect("resize log"), &[first, second]);
    }
}
