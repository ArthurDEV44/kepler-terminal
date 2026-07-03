use std::io::{ErrorKind, Read, Write};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::{
    PtyBackend, PtyChild, PtyChildKiller, PtyError, PtyEvent, PtyExit, PtySession,
    PtySessionConfig, PtySize,
};

pub const M2_DEFAULT_EVENT_CAPACITY: usize = 1024;
pub const M2_MAX_EVENT_CAPACITY: usize = 1024;
pub const M2_DEFAULT_READ_CHUNK_BYTES: usize = 8192;
pub const M2_MAX_READ_CHUNK_BYTES: usize = 64 * 1024;
pub const M2_DEFAULT_COMMAND_TIMEOUT_MS: u64 = 5000;
pub const M2_MAX_COMMAND_TIMEOUT_MS: u64 = 120_000;
pub const M2_DEFAULT_DRAIN_TIMEOUT_MS: u64 = 250;
pub const M2_MAX_DRAIN_TIMEOUT_MS: u64 = 5000;
pub const M2_DEFAULT_WRITE_TIMEOUT_MS: u64 = 1000;
pub const M2_MAX_WRITE_TIMEOUT_MS: u64 = 5000;
pub const M2_DEFAULT_POLL_INTERVAL_MS: u64 = 10;
pub const M2_MAX_EVENTS_PER_TICK: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtyRuntimeConfig {
    event_capacity: usize,
    read_chunk_bytes: usize,
    command_timeout: Duration,
    drain_timeout: Duration,
    write_timeout: Duration,
    poll_interval: Duration,
}

impl PtyRuntimeConfig {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_event_capacity(mut self, event_capacity: usize) -> Result<Self, PtyError> {
        if event_capacity == 0 || event_capacity > M2_MAX_EVENT_CAPACITY {
            return Err(PtyError::invalid_runtime_config(
                "event_capacity",
                format!("must be between 1 and {M2_MAX_EVENT_CAPACITY}"),
            ));
        }

        self.event_capacity = event_capacity;
        Ok(self)
    }

    pub fn with_read_chunk_bytes(mut self, read_chunk_bytes: usize) -> Result<Self, PtyError> {
        if read_chunk_bytes == 0 || read_chunk_bytes > M2_MAX_READ_CHUNK_BYTES {
            return Err(PtyError::invalid_runtime_config(
                "read_chunk_bytes",
                format!("must be between 1 and {M2_MAX_READ_CHUNK_BYTES}"),
            ));
        }

        self.read_chunk_bytes = read_chunk_bytes;
        Ok(self)
    }

    pub fn with_command_timeout(mut self, command_timeout: Duration) -> Result<Self, PtyError> {
        let timeout_ms = duration_millis(command_timeout);
        if timeout_ms == 0 || timeout_ms > M2_MAX_COMMAND_TIMEOUT_MS {
            return Err(PtyError::invalid_runtime_config(
                "command_timeout",
                format!("must be between 1 and {M2_MAX_COMMAND_TIMEOUT_MS}ms"),
            ));
        }

        self.command_timeout = command_timeout;
        Ok(self)
    }

    pub fn with_drain_timeout(mut self, drain_timeout: Duration) -> Result<Self, PtyError> {
        let timeout_ms = duration_millis(drain_timeout);
        if timeout_ms == 0 || timeout_ms > M2_MAX_DRAIN_TIMEOUT_MS {
            return Err(PtyError::invalid_runtime_config(
                "drain_timeout",
                format!("must be between 1 and {M2_MAX_DRAIN_TIMEOUT_MS}ms"),
            ));
        }

        self.drain_timeout = drain_timeout;
        Ok(self)
    }

    pub fn with_write_timeout(mut self, write_timeout: Duration) -> Result<Self, PtyError> {
        let timeout_ms = duration_millis(write_timeout);
        if timeout_ms == 0 || timeout_ms > M2_MAX_WRITE_TIMEOUT_MS {
            return Err(PtyError::invalid_runtime_config(
                "write_timeout",
                format!("must be between 1 and {M2_MAX_WRITE_TIMEOUT_MS}ms"),
            ));
        }

        self.write_timeout = write_timeout;
        Ok(self)
    }

    #[must_use]
    pub const fn event_capacity(self) -> usize {
        self.event_capacity
    }

    #[must_use]
    pub const fn read_chunk_bytes(self) -> usize {
        self.read_chunk_bytes
    }

    #[must_use]
    pub const fn command_timeout(self) -> Duration {
        self.command_timeout
    }

    #[must_use]
    pub const fn drain_timeout(self) -> Duration {
        self.drain_timeout
    }

    #[must_use]
    pub const fn write_timeout(self) -> Duration {
        self.write_timeout
    }
}

impl Default for PtyRuntimeConfig {
    fn default() -> Self {
        Self {
            event_capacity: M2_DEFAULT_EVENT_CAPACITY,
            read_chunk_bytes: M2_DEFAULT_READ_CHUNK_BYTES,
            command_timeout: Duration::from_millis(M2_DEFAULT_COMMAND_TIMEOUT_MS),
            drain_timeout: Duration::from_millis(M2_DEFAULT_DRAIN_TIMEOUT_MS),
            write_timeout: Duration::from_millis(M2_DEFAULT_WRITE_TIMEOUT_MS),
            poll_interval: Duration::from_millis(M2_DEFAULT_POLL_INTERVAL_MS),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtyRunOutcome {
    exit: PtyExit,
    output_bytes: usize,
    output_chunks: usize,
    saw_eof: bool,
    drain_timed_out: bool,
}

impl PtyRunOutcome {
    #[must_use]
    pub const fn exit(&self) -> &PtyExit {
        &self.exit
    }

    #[must_use]
    pub const fn output_bytes(&self) -> usize {
        self.output_bytes
    }

    #[must_use]
    pub const fn output_chunks(&self) -> usize {
        self.output_chunks
    }

    #[must_use]
    pub const fn saw_eof(&self) -> bool {
        self.saw_eof
    }

    #[must_use]
    pub const fn drain_timed_out(&self) -> bool {
        self.drain_timed_out
    }
}

pub struct PtySessionRunner {
    session: PtySession,
    reader: ReaderWorker,
    writer: WriterWorker,
    waiter: WaitWorker,
    killer: Box<dyn PtyChildKiller>,
    config: PtyRuntimeConfig,
    closed: bool,
    pending_exit: Option<PtyExit>,
}

impl PtySessionRunner {
    pub fn spawn(
        backend: &impl PtyBackend,
        config: &PtySessionConfig,
        runtime: PtyRuntimeConfig,
    ) -> Result<Self, PtyError> {
        let session = backend.spawn(config)?;
        Self::new(session, runtime)
    }

    pub fn new(mut session: PtySession, config: PtyRuntimeConfig) -> Result<Self, PtyError> {
        let reader = session.take_reader().ok_or(PtyError::ClosedSession {
            operation: "start_reader",
        })?;
        let reader = ReaderWorker::spawn(reader, config)?;
        let writer = session.take_writer().ok_or(PtyError::ClosedSession {
            operation: "start_writer",
        })?;
        let writer = WriterWorker::spawn(writer)?;
        let child = session.take_child().ok_or(PtyError::ClosedSession {
            operation: "start_child_waiter",
        })?;
        let killer = child.clone_killer();
        let waiter = WaitWorker::spawn(child)?;

        Ok(Self {
            session,
            reader,
            writer,
            waiter,
            killer,
            config,
            closed: false,
            pending_exit: None,
        })
    }

    pub fn write_input(&mut self, bytes: &[u8]) -> Result<(), PtyError> {
        if bytes.len() > crate::M2_MAX_WRITE_CHUNK_BYTES {
            return Err(PtyError::WriteTooLarge {
                requested: bytes.len(),
                max: crate::M2_MAX_WRITE_CHUNK_BYTES,
            });
        }

        self.refresh_exit_state()?;
        if self.closed {
            return Err(PtyError::ClosedSession {
                operation: "write_input",
            });
        }

        self.writer.write(bytes, self.config.write_timeout)
    }

    pub fn close_input(&mut self) {
        self.writer.close();
    }

    pub fn resize(&mut self, size: PtySize) -> Result<(), PtyError> {
        if self.closed {
            return Err(PtyError::ClosedSession {
                operation: "resize",
            });
        }

        self.session.resize(size)
    }

    pub fn run_until_exit(
        &mut self,
        mut emit: impl FnMut(PtyEvent),
    ) -> Result<PtyRunOutcome, PtyError> {
        self.run_until_exit_with_responses(|event| {
            emit(event);
            Ok(None)
        })
    }

    pub fn run_until_exit_with_responses(
        &mut self,
        mut handle_event: impl FnMut(PtyEvent) -> Result<Option<Vec<u8>>, PtyError>,
    ) -> Result<PtyRunOutcome, PtyError> {
        let command_deadline = Instant::now() + self.config.command_timeout;
        let mut drain_deadline = None;
        let mut exit = None;
        let mut saw_eof = false;
        let mut output_bytes = 0usize;
        let mut output_chunks = 0usize;

        loop {
            for _ in 0..M2_MAX_EVENTS_PER_TICK {
                if exit.is_none() {
                    self.refresh_exit_state()?;
                    if let Some(status) = self.pending_exit.take() {
                        self.handle_event(PtyEvent::Exit(status.clone()), &mut handle_event)?;
                        exit = Some(status);
                        drain_deadline = Some(Instant::now() + self.config.drain_timeout);
                    }
                }

                if exit.is_none() && Instant::now() >= command_deadline {
                    return self.timeout_error();
                }

                let Some(message) = self.reader.try_recv()? else {
                    break;
                };

                match message {
                    ReaderMessage::Output(bytes) => {
                        output_bytes = output_bytes.saturating_add(bytes.len());
                        output_chunks = output_chunks.saturating_add(1);
                        self.handle_event(PtyEvent::Output(bytes), &mut handle_event)?;
                    }
                    ReaderMessage::Eof => {
                        saw_eof = true;
                        self.handle_event(PtyEvent::Eof, &mut handle_event)?;
                    }
                }
            }

            if exit.is_none() {
                self.refresh_exit_state()?;
                if let Some(status) = self.pending_exit.take() {
                    self.handle_event(PtyEvent::Exit(status.clone()), &mut handle_event)?;
                    exit = Some(status);
                    drain_deadline = Some(Instant::now() + self.config.drain_timeout);
                }
            }

            if let Some(status) = exit.clone() {
                if saw_eof {
                    self.reader.join();
                    self.waiter.join_if_finished();
                    return Ok(PtyRunOutcome {
                        exit: status,
                        output_bytes,
                        output_chunks,
                        saw_eof,
                        drain_timed_out: false,
                    });
                }

                if drain_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                    self.reader.join_if_finished();
                    self.waiter.join_if_finished();
                    return Ok(PtyRunOutcome {
                        exit: status,
                        output_bytes,
                        output_chunks,
                        saw_eof,
                        drain_timed_out: true,
                    });
                }
            } else if Instant::now() >= command_deadline {
                return self.timeout_error();
            }

            match self.reader.recv_timeout(self.config.poll_interval)? {
                Some(ReaderMessage::Output(bytes)) => {
                    output_bytes = output_bytes.saturating_add(bytes.len());
                    output_chunks = output_chunks.saturating_add(1);
                    self.handle_event(PtyEvent::Output(bytes), &mut handle_event)?;
                }
                Some(ReaderMessage::Eof) => {
                    saw_eof = true;
                    self.handle_event(PtyEvent::Eof, &mut handle_event)?;
                }
                None => {}
            }

            if exit.is_none() {
                self.refresh_exit_state()?;
                if let Some(status) = self.pending_exit.take() {
                    self.handle_event(PtyEvent::Exit(status.clone()), &mut handle_event)?;
                    exit = Some(status);
                    drain_deadline = Some(Instant::now() + self.config.drain_timeout);
                }
            }
        }
    }

    fn refresh_exit_state(&mut self) -> Result<(), PtyError> {
        if self.pending_exit.is_none() {
            if let Some(status) = self.waiter.try_recv()? {
                self.closed = true;
                self.writer.close();
                self.pending_exit = Some(status);
            }
        }

        Ok(())
    }

    fn timeout_error<T>(&mut self) -> Result<T, PtyError> {
        self.closed = true;
        self.writer.close();
        let timeout_ms = duration_millis(self.config.command_timeout);
        let error = match self.killer.kill() {
            Ok(()) => PtyError::timeout("wait", timeout_ms),
            Err(error) => PtyError::timeout_with_source("wait", timeout_ms, error),
        };
        self.reader.join_if_finished();
        self.waiter.join_if_finished();
        Err(error)
    }

    fn handle_event(
        &mut self,
        event: PtyEvent,
        handle_event: &mut impl FnMut(PtyEvent) -> Result<Option<Vec<u8>>, PtyError>,
    ) -> Result<(), PtyError> {
        if let Some(response) = handle_event(event)? {
            self.writer.write(&response, self.config.write_timeout)?;
        }

        Ok(())
    }
}

impl Drop for PtySessionRunner {
    fn drop(&mut self) {
        self.session.close_input();
        self.writer.close();
        if !self.closed {
            let _ = self.killer.kill();
        }
        self.writer.join_if_finished();
        self.reader.join_if_finished();
        self.waiter.join_if_finished();
    }
}

enum ReaderMessage {
    Output(Vec<u8>),
    Eof,
}

enum ReaderEnvelope {
    Event(ReaderMessage),
    Error(String),
}

struct ReaderWorker {
    receiver: Receiver<ReaderEnvelope>,
    handle: Option<JoinHandle<()>>,
    eof_seen: bool,
}

impl ReaderWorker {
    fn spawn(reader: Box<dyn Read + Send>, config: PtyRuntimeConfig) -> Result<Self, PtyError> {
        let (sender, receiver) = mpsc::sync_channel(config.event_capacity);
        let read_chunk_bytes = config.read_chunk_bytes;
        let handle = thread::Builder::new()
            .name("hera-pty-reader".to_owned())
            .spawn(move || reader_worker(reader, sender, read_chunk_bytes))
            .map_err(|error| PtyError::backend("spawn_reader", error))?;

        Ok(Self {
            receiver,
            handle: Some(handle),
            eof_seen: false,
        })
    }

    fn try_recv(&mut self) -> Result<Option<ReaderMessage>, PtyError> {
        match self.receiver.try_recv() {
            Ok(ReaderEnvelope::Event(message)) => Ok(self.track_eof(message)),
            Ok(ReaderEnvelope::Error(message)) => Err(PtyError::backend("read_output", message)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Ok(self.disconnected_eof()),
        }
    }

    fn recv_timeout(&mut self, timeout: Duration) -> Result<Option<ReaderMessage>, PtyError> {
        match self.receiver.recv_timeout(timeout) {
            Ok(ReaderEnvelope::Event(message)) => Ok(self.track_eof(message)),
            Ok(ReaderEnvelope::Error(message)) => Err(PtyError::backend("read_output", message)),
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(None),
            Err(mpsc::RecvTimeoutError::Disconnected) => Ok(self.disconnected_eof()),
        }
    }

    fn track_eof(&mut self, message: ReaderMessage) -> Option<ReaderMessage> {
        if matches!(message, ReaderMessage::Eof) {
            if self.eof_seen {
                None
            } else {
                self.eof_seen = true;
                Some(message)
            }
        } else {
            Some(message)
        }
    }

    fn disconnected_eof(&mut self) -> Option<ReaderMessage> {
        if self.eof_seen {
            None
        } else {
            self.eof_seen = true;
            Some(ReaderMessage::Eof)
        }
    }

    fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    fn join_if_finished(&mut self) {
        if self.handle.as_ref().is_some_and(JoinHandle::is_finished) {
            self.join();
        }
    }
}

enum WriterCommand {
    Write(Vec<u8>, SyncSender<Result<(), PtyError>>),
    Close,
}

struct WriterWorker {
    sender: Option<SyncSender<WriterCommand>>,
    handle: Option<JoinHandle<()>>,
}

impl WriterWorker {
    fn spawn(writer: Box<dyn Write + Send>) -> Result<Self, PtyError> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = thread::Builder::new()
            .name("hera-pty-writer".to_owned())
            .spawn(move || writer_worker(writer, receiver))
            .map_err(|error| PtyError::backend("spawn_writer", error))?;

        Ok(Self {
            sender: Some(sender),
            handle: Some(handle),
        })
    }

    fn write(&mut self, bytes: &[u8], timeout: Duration) -> Result<(), PtyError> {
        if bytes.len() > crate::M2_MAX_WRITE_CHUNK_BYTES {
            return Err(PtyError::WriteTooLarge {
                requested: bytes.len(),
                max: crate::M2_MAX_WRITE_CHUNK_BYTES,
            });
        }

        let sender = self.sender.as_ref().ok_or(PtyError::ClosedSession {
            operation: "write_input",
        })?;
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        sender
            .send(WriterCommand::Write(bytes.to_vec(), response_sender))
            .map_err(|_| PtyError::ClosedSession {
                operation: "write_input",
            })?;

        match response_receiver.recv_timeout(timeout) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                Err(PtyError::timeout("write_input", duration_millis(timeout)))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(PtyError::ClosedSession {
                operation: "write_input",
            }),
        }
    }

    fn close(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.try_send(WriterCommand::Close);
        }
    }

    fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    fn join_if_finished(&mut self) {
        if self.handle.as_ref().is_some_and(JoinHandle::is_finished) {
            self.join();
        }
    }
}

fn writer_worker(mut writer: Box<dyn Write + Send>, receiver: Receiver<WriterCommand>) {
    while let Ok(command) = receiver.recv() {
        match command {
            WriterCommand::Write(bytes, response_sender) => {
                let result = writer
                    .write_all(&bytes)
                    .and_then(|()| writer.flush())
                    .map_err(|error| PtyError::backend("write_input", error));
                let _ = response_sender.send(result);
            }
            WriterCommand::Close => break,
        }
    }
}

enum WaitEnvelope {
    Exit(PtyExit),
    Error(String),
}

struct WaitWorker {
    receiver: Receiver<WaitEnvelope>,
    handle: Option<JoinHandle<()>>,
}

impl WaitWorker {
    fn spawn(child: Box<dyn PtyChild>) -> Result<Self, PtyError> {
        let (sender, receiver) = mpsc::channel();
        let handle = thread::Builder::new()
            .name("hera-pty-wait".to_owned())
            .spawn(move || child_wait_worker(child, sender))
            .map_err(|error| PtyError::backend("spawn_child_waiter", error))?;

        Ok(Self {
            receiver,
            handle: Some(handle),
        })
    }

    fn try_recv(&mut self) -> Result<Option<PtyExit>, PtyError> {
        match self.receiver.try_recv() {
            Ok(WaitEnvelope::Exit(exit)) => {
                self.join();
                Ok(Some(exit))
            }
            Ok(WaitEnvelope::Error(message)) => Err(PtyError::backend("wait", message)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(mpsc::TryRecvError::Disconnected) => Ok(None),
        }
    }

    fn join(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    fn join_if_finished(&mut self) {
        if self.handle.as_ref().is_some_and(JoinHandle::is_finished) {
            self.join();
        }
    }
}

fn child_wait_worker(mut child: Box<dyn PtyChild>, sender: mpsc::Sender<WaitEnvelope>) {
    let message = match child.wait() {
        Ok(exit) => WaitEnvelope::Exit(exit),
        Err(error) => WaitEnvelope::Error(error.to_string()),
    };
    let _ = sender.send(message);
}

fn reader_worker(
    mut reader: Box<dyn Read + Send>,
    sender: SyncSender<ReaderEnvelope>,
    read_chunk_bytes: usize,
) {
    let mut buffer = vec![0; read_chunk_bytes];

    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                let _ = sender.send(ReaderEnvelope::Event(ReaderMessage::Eof));
                break;
            }
            Ok(read) => {
                let bytes = buffer[..read].to_vec();
                if sender
                    .send(ReaderEnvelope::Event(ReaderMessage::Output(bytes)))
                    .is_err()
                {
                    break;
                }
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => {
                let _ = sender.send(ReaderEnvelope::Error(error.to_string()));
                break;
            }
        }
    }
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use std::io::{self, Cursor, Read, Write};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::{M2_MAX_DRAIN_TIMEOUT_MS, PtyRuntimeConfig, PtySessionRunner};
    use crate::{
        M2_MAX_WRITE_CHUNK_BYTES, PtyChild, PtyChildKiller, PtyError, PtyErrorKind, PtyEvent,
        PtyExit, PtyPlatformMetadata, PtySession, PtySize, session::PtySessionBackend,
    };

    type ResizeLog = Arc<Mutex<Vec<PtySize>>>;
    type WriterBytes = Arc<Mutex<Vec<u8>>>;

    #[derive(Debug)]
    struct FakeChild {
        state: Arc<Mutex<FakeChildState>>,
    }

    #[derive(Debug)]
    struct FakeChildState {
        exit_after_polls: Option<usize>,
        polls: usize,
        kill_calls: usize,
        exit_code: u32,
    }

    impl FakeChild {
        fn exits_after(
            exit_after_polls: usize,
            exit_code: u32,
        ) -> (Self, Arc<Mutex<FakeChildState>>) {
            let state = Arc::new(Mutex::new(FakeChildState {
                exit_after_polls: Some(exit_after_polls),
                polls: 0,
                kill_calls: 0,
                exit_code,
            }));
            (
                Self {
                    state: Arc::clone(&state),
                },
                state,
            )
        }

        fn never_exits() -> (Self, Arc<Mutex<FakeChildState>>) {
            let state = Arc::new(Mutex::new(FakeChildState {
                exit_after_polls: None,
                polls: 0,
                kill_calls: 0,
                exit_code: 0,
            }));
            (
                Self {
                    state: Arc::clone(&state),
                },
                state,
            )
        }
    }

    impl PtyChild for FakeChild {
        fn try_wait(&mut self) -> Result<Option<PtyExit>, PtyError> {
            let mut state = self.state.lock().expect("fake child state");
            state.polls += 1;

            if state
                .exit_after_polls
                .is_some_and(|exit_after| state.polls >= exit_after)
            {
                Ok(Some(PtyExit::new(state.exit_code, None)))
            } else {
                Ok(None)
            }
        }

        fn wait(&mut self) -> Result<PtyExit, PtyError> {
            loop {
                {
                    let state = self.state.lock().expect("fake child state");
                    if state.kill_calls > 0 {
                        return Ok(PtyExit::new(1, Some("killed".to_owned())));
                    }
                    if state.exit_after_polls.is_some() {
                        return Ok(PtyExit::new(state.exit_code, None));
                    }
                }

                std::thread::sleep(Duration::from_millis(1));
            }
        }

        fn process_id(&self) -> Option<u32> {
            Some(7)
        }

        fn kill(&mut self) -> Result<(), PtyError> {
            self.state.lock().expect("fake child state").kill_calls += 1;
            Ok(())
        }

        fn clone_killer(&self) -> Box<dyn PtyChildKiller> {
            Box::new(FakeChildKiller {
                state: Arc::clone(&self.state),
            })
        }
    }

    #[derive(Debug)]
    struct FakeChildKiller {
        state: Arc<Mutex<FakeChildState>>,
    }

    impl PtyChildKiller for FakeChildKiller {
        fn kill(&mut self) -> Result<(), PtyError> {
            self.state.lock().expect("fake child state").kill_calls += 1;
            Ok(())
        }
    }

    struct FakeBackend {
        resize_log: ResizeLog,
    }

    impl PtySessionBackend for FakeBackend {
        fn resize(&self, size: PtySize) -> Result<(), PtyError> {
            self.resize_log.lock().expect("fake resize log").push(size);
            Ok(())
        }
    }

    struct FailingResizeBackend;

    impl PtySessionBackend for FailingResizeBackend {
        fn resize(&self, _size: PtySize) -> Result<(), PtyError> {
            Err(PtyError::backend("resize", "fake resize failed"))
        }
    }

    #[derive(Default)]
    struct CapturingWriter {
        bytes: WriterBytes,
        fail: bool,
    }

    impl Write for CapturingWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.fail {
                Err(io::Error::new(ErrorKind::BrokenPipe, "writer closed"))
            } else {
                self.bytes
                    .lock()
                    .expect("captured bytes")
                    .extend_from_slice(bytes);
                Ok(bytes.len())
            }
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct SlowWriter;

    impl Write for SlowWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            std::thread::sleep(Duration::from_millis(50));
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    use std::io::ErrorKind;

    fn session(
        reader: impl Read + Send + 'static,
        child: FakeChild,
    ) -> (PtySession, ResizeLog, WriterBytes) {
        let resize_log = Arc::new(Mutex::new(Vec::new()));
        let writer_bytes = Arc::new(Mutex::new(Vec::new()));
        let size = PtySize::new(80, 24).expect("valid size");
        let session = PtySession::new(
            Box::new(reader),
            Box::new(CapturingWriter {
                bytes: Arc::clone(&writer_bytes),
                fail: false,
            }),
            Box::new(child),
            PtyPlatformMetadata::new("fake", Some(7), size),
            Box::new(FakeBackend {
                resize_log: Arc::clone(&resize_log),
            }),
        );

        (session, resize_log, writer_bytes)
    }

    #[test]
    fn emits_ordered_output_eof_and_exit_events() {
        let (child, _state) = FakeChild::exits_after(1, 0);
        let (session, _resize_log, _writer_bytes) = session(Cursor::new(b"hello".to_vec()), child);
        let mut runner =
            PtySessionRunner::new(session, PtyRuntimeConfig::default()).expect("runner starts");
        let mut events = Vec::new();

        let outcome = runner
            .run_until_exit(|event| events.push(event))
            .expect("run should complete");

        assert_eq!(outcome.exit().code(), 0);
        assert_eq!(outcome.output_bytes(), 5);
        assert!(outcome.saw_eof());
        assert!(events.iter().any(|event| matches!(
            event,
            PtyEvent::Output(bytes) if bytes == b"hello"
        )));
        assert!(events.iter().any(|event| matches!(event, PtyEvent::Eof)));
        assert!(
            events
                .iter()
                .any(|event| matches!(event, PtyEvent::Exit(exit) if exit.code() == 0))
        );
    }

    #[test]
    fn drains_output_after_child_exit_until_eof() {
        let (child, _state) = FakeChild::exits_after(1, 0);
        let (session, _resize_log, _writer_bytes) = session(Cursor::new(b"late".to_vec()), child);
        let config = PtyRuntimeConfig::default()
            .with_read_chunk_bytes(1)
            .expect("valid chunk size");
        let mut runner = PtySessionRunner::new(session, config).expect("runner starts");
        let mut output = Vec::new();

        let outcome = runner
            .run_until_exit(|event| {
                if let PtyEvent::Output(bytes) = event {
                    output.extend(bytes);
                }
            })
            .expect("run should complete");

        assert_eq!(outcome.exit().code(), 0);
        assert_eq!(output, b"late");
    }

    #[test]
    fn responds_to_cursor_position_query_from_child_output() {
        let (child, _state) = FakeChild::exits_after(1, 0);
        let (session, _resize_log, writer_bytes) = session(Cursor::new(b"\x1b[6n".to_vec()), child);
        let mut runner =
            PtySessionRunner::new(session, PtyRuntimeConfig::default()).expect("runner starts");

        runner
            .run_until_exit_with_responses(|event| {
                if matches!(event, PtyEvent::Output(bytes) if bytes == b"\x1b[6n") {
                    Ok(Some(b"\x1b[1;1R".to_vec()))
                } else {
                    Ok(None)
                }
            })
            .expect("run should complete");

        assert_eq!(
            &*writer_bytes.lock().expect("captured writer bytes"),
            b"\x1b[1;1R"
        );
    }

    #[test]
    fn non_zero_exit_is_reported_with_output() {
        let (child, _state) = FakeChild::exits_after(1, 42);
        let (session, _resize_log, _writer_bytes) = session(Cursor::new(b"error".to_vec()), child);
        let mut runner =
            PtySessionRunner::new(session, PtyRuntimeConfig::default()).expect("runner starts");
        let mut output = Vec::new();

        let outcome = runner
            .run_until_exit(|event| {
                if let PtyEvent::Output(bytes) = event {
                    output.extend(bytes);
                }
            })
            .expect("run should complete");

        assert_eq!(outcome.exit().code(), 42);
        assert_eq!(output, b"error");
    }

    #[test]
    fn command_timeout_kills_child() {
        let (child, state) = FakeChild::never_exits();
        let (session, _resize_log, _writer_bytes) = session(Cursor::new(Vec::new()), child);
        let config = PtyRuntimeConfig::default()
            .with_command_timeout(Duration::from_millis(1))
            .expect("valid timeout");
        let mut runner = PtySessionRunner::new(session, config).expect("runner starts");

        let error = runner
            .run_until_exit(|_event| {})
            .expect_err("must time out");

        assert_eq!(error.kind(), PtyErrorKind::Timeout);
        assert_eq!(error.operation(), Some("wait"));
        assert_eq!(state.lock().expect("fake child state").kill_calls, 1);
    }

    #[test]
    fn write_input_sends_bytes_and_rejects_oversized_chunks() {
        let (child, _state) = FakeChild::exits_after(10, 0);
        let (session, _resize_log, writer_bytes) = session(Cursor::new(Vec::new()), child);
        let mut runner =
            PtySessionRunner::new(session, PtyRuntimeConfig::default()).expect("runner starts");

        runner.write_input(b"input\r").expect("write should work");
        let too_large = vec![b'x'; M2_MAX_WRITE_CHUNK_BYTES + 1];
        let error = runner
            .write_input(&too_large)
            .expect_err("oversized input must fail");

        assert_eq!(error.kind(), PtyErrorKind::WriteTooLarge);
        assert_eq!(
            &*writer_bytes.lock().expect("captured writer bytes"),
            b"input\r"
        );
    }

    #[test]
    fn write_input_timeout_does_not_block_caller() {
        let (child, _state) = FakeChild::exits_after(10, 0);
        let resize_log = Arc::new(Mutex::new(Vec::new()));
        let size = PtySize::new(80, 24).expect("valid size");
        let session = PtySession::new(
            Box::new(Cursor::new(Vec::new())),
            Box::new(SlowWriter),
            Box::new(child),
            PtyPlatformMetadata::new("fake", Some(7), size),
            Box::new(FakeBackend { resize_log }),
        );
        let config = PtyRuntimeConfig::default()
            .with_write_timeout(Duration::from_millis(1))
            .expect("valid write timeout");
        let mut runner = PtySessionRunner::new(session, config).expect("runner starts");

        let error = runner
            .write_input(b"input\r")
            .expect_err("slow writer should time out");

        assert_eq!(error.kind(), PtyErrorKind::Timeout);
        assert_eq!(error.operation(), Some("write_input"));
    }

    #[test]
    fn write_input_after_exit_returns_closed_session() {
        let (child, _state) = FakeChild::exits_after(1, 0);
        let (session, _resize_log, _writer_bytes) = session(Cursor::new(Vec::new()), child);
        let mut runner =
            PtySessionRunner::new(session, PtyRuntimeConfig::default()).expect("runner starts");

        runner
            .run_until_exit(|_event| {})
            .expect("run should complete");
        let error = runner
            .write_input(b"late\r")
            .expect_err("closed session must reject input");

        assert_eq!(error.kind(), PtyErrorKind::ClosedSession);
    }

    #[test]
    fn write_input_after_unobserved_child_exit_returns_closed_session() {
        let (child, _state) = FakeChild::exits_after(1, 0);
        let (session, _resize_log, _writer_bytes) = session(Cursor::new(Vec::new()), child);
        let mut runner =
            PtySessionRunner::new(session, PtyRuntimeConfig::default()).expect("runner starts");
        std::thread::sleep(Duration::from_millis(5));

        let error = runner
            .write_input(b"late\r")
            .expect_err("closed session must reject input");

        assert_eq!(error.kind(), PtyErrorKind::ClosedSession);
    }

    #[test]
    fn resize_delegates_to_session_backend() {
        let (child, _state) = FakeChild::exits_after(10, 0);
        let (session, resize_log, _writer_bytes) = session(Cursor::new(Vec::new()), child);
        let mut runner =
            PtySessionRunner::new(session, PtyRuntimeConfig::default()).expect("runner starts");
        let size = PtySize::new(100, 30).expect("valid size");

        runner.resize(size).expect("resize should work");

        assert_eq!(&*resize_log.lock().expect("resize log"), &[size]);
    }

    #[test]
    fn runtime_config_rejects_unbounded_event_capacity() {
        let error = PtyRuntimeConfig::default()
            .with_event_capacity(1025)
            .expect_err("capacity above cap must fail");

        assert_eq!(error.kind(), PtyErrorKind::InvalidRuntimeConfig);
    }

    #[test]
    fn runtime_config_rejects_unbounded_drain_timeout() {
        let error = PtyRuntimeConfig::default()
            .with_drain_timeout(Duration::from_millis(M2_MAX_DRAIN_TIMEOUT_MS + 1))
            .expect_err("drain timeout above cap must fail");

        assert_eq!(error.kind(), PtyErrorKind::InvalidRuntimeConfig);
    }

    #[test]
    fn bounded_channel_drains_large_output_without_internal_event_buffer() {
        let (child, _state) = FakeChild::exits_after(1, 0);
        let input = vec![b'x'; 1024 * 1024];
        let (session, _resize_log, _writer_bytes) = session(Cursor::new(input), child);
        let config = PtyRuntimeConfig::default()
            .with_event_capacity(1)
            .expect("valid capacity")
            .with_read_chunk_bytes(4096)
            .expect("valid chunk size");
        let mut runner = PtySessionRunner::new(session, config).expect("runner starts");

        let outcome = runner
            .run_until_exit(|_event| {})
            .expect("run should complete");

        assert_eq!(outcome.output_bytes(), 1024 * 1024);
        assert!(outcome.output_chunks() >= 256);
    }

    #[test]
    fn partial_resize_error_preserves_requested_dimensions() {
        let (child, _state) = FakeChild::exits_after(10, 0);
        let writer_bytes = Arc::new(Mutex::new(Vec::new()));
        let size = PtySize::new(80, 24).expect("valid size");
        let session = PtySession::new(
            Box::new(Cursor::new(Vec::new())),
            Box::new(CapturingWriter {
                bytes: writer_bytes,
                fail: false,
            }),
            Box::new(child),
            PtyPlatformMetadata::new("fake", Some(7), size),
            Box::new(FailingResizeBackend),
        );
        let mut runner =
            PtySessionRunner::new(session, PtyRuntimeConfig::default()).expect("runner starts");
        let requested = PtySize::new(100, 30).expect("valid size");
        let source = runner.resize(requested).expect_err("resize must fail");
        let error = PtyError::partial_resize(requested, source);

        assert_eq!(error.kind(), PtyErrorKind::PartialResize);
        assert_eq!(error.operation(), Some("resize"));
        assert!(
            error
                .source_message()
                .is_some_and(|message| message.contains("fake resize failed"))
        );
    }
}
