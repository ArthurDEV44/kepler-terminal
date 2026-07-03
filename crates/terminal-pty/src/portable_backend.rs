use std::fmt;

use crate::{
    PtyBackend, PtyChild, PtyChildKiller, PtyError, PtyExit, PtyPlatformMetadata, PtySession,
    PtySessionConfig, PtySize, session::PtySessionBackend,
};

pub struct PortablePtyBackend {
    system: Box<dyn portable_pty::PtySystem + Send>,
    backend_name: &'static str,
}

impl PortablePtyBackend {
    #[must_use]
    pub fn new() -> Self {
        Self {
            system: portable_pty::native_pty_system(),
            backend_name: native_backend_name(),
        }
    }

    #[cfg(test)]
    fn with_system(
        system: Box<dyn portable_pty::PtySystem + Send>,
        backend_name: &'static str,
    ) -> Self {
        Self {
            system,
            backend_name,
        }
    }
}

impl Default for PortablePtyBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyBackend for PortablePtyBackend {
    fn spawn(&self, config: &PtySessionConfig) -> Result<PtySession, PtyError> {
        config.validate()?;
        let command = config.command().to_portable_command()?;
        let portable_size = config.size().to_portable();

        let portable_pty::PtyPair { master, slave } = self
            .system
            .openpty(portable_size)
            .map_err(|error| PtyError::backend("openpty", error))?;

        let reader = master
            .try_clone_reader()
            .map_err(|error| PtyError::backend("try_clone_reader", error))?;
        let writer = master
            .take_writer()
            .map_err(|error| PtyError::backend("take_writer", error))?;
        let child = slave
            .spawn_command(command)
            .map_err(|error| PtyError::backend("spawn_command", error))?;
        let process_id = child.process_id();

        Ok(PtySession::new(
            reader,
            writer,
            Box::new(PortablePtyChild { inner: child }),
            PtyPlatformMetadata::new(self.backend_name, process_id, config.size()),
            Box::new(PortablePtySessionBackend { master }),
        ))
    }
}

struct PortablePtySessionBackend {
    master: Box<dyn portable_pty::MasterPty + Send>,
}

impl PtySessionBackend for PortablePtySessionBackend {
    fn resize(&self, size: PtySize) -> Result<(), PtyError> {
        self.master
            .resize(size.to_portable())
            .map_err(|error| PtyError::backend("resize", error))
    }
}

struct PortablePtyChild {
    inner: Box<dyn portable_pty::Child + Send + Sync>,
}

impl fmt::Debug for PortablePtyChild {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PortablePtyChild")
            .finish_non_exhaustive()
    }
}

impl PtyChild for PortablePtyChild {
    fn try_wait(&mut self) -> Result<Option<PtyExit>, PtyError> {
        self.inner
            .try_wait()
            .map(|status| status.map(PtyExit::from_portable))
            .map_err(|error| PtyError::backend("try_wait", error))
    }

    fn wait(&mut self) -> Result<PtyExit, PtyError> {
        self.inner
            .wait()
            .map(PtyExit::from_portable)
            .map_err(|error| PtyError::backend("wait", error))
    }

    fn process_id(&self) -> Option<u32> {
        self.inner.process_id()
    }

    fn kill(&mut self) -> Result<(), PtyError> {
        self.inner
            .kill()
            .map_err(|error| PtyError::backend("kill", error))
    }

    fn clone_killer(&self) -> Box<dyn PtyChildKiller> {
        Box::new(PortablePtyChildKiller {
            inner: self.inner.clone_killer(),
        })
    }
}

struct PortablePtyChildKiller {
    inner: Box<dyn portable_pty::ChildKiller + Send + Sync>,
}

impl fmt::Debug for PortablePtyChildKiller {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PortablePtyChildKiller")
            .finish_non_exhaustive()
    }
}

impl PtyChildKiller for PortablePtyChildKiller {
    fn kill(&mut self) -> Result<(), PtyError> {
        self.inner
            .kill()
            .map_err(|error| PtyError::backend("kill", error))
    }
}

#[cfg(windows)]
const fn native_backend_name() -> &'static str {
    "conpty"
}

#[cfg(unix)]
const fn native_backend_name() -> &'static str {
    "unix"
}

#[cfg(not(any(unix, windows)))]
const fn native_backend_name() -> &'static str {
    "native"
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::io::{self, Cursor, Read, Write};
    use std::sync::{Arc, Mutex};

    use portable_pty::{Child, ChildKiller, MasterPty, PtyPair, PtySystem, SlavePty};

    use super::PortablePtyBackend;
    use crate::{PtyBackend, PtyCommand, PtyError, PtyErrorKind, PtySessionConfig, PtySize};

    #[derive(Debug, Default)]
    struct FakeState {
        open_calls: usize,
        take_writer_calls: usize,
        last_open_size: Option<portable_pty::PtySize>,
        last_resize: Option<portable_pty::PtySize>,
        captured_argv: Vec<OsString>,
        captured_cwd: Option<OsString>,
        captured_empty_env: Option<OsString>,
        writer_bytes: Vec<u8>,
    }

    struct FakeSystem {
        state: Arc<Mutex<FakeState>>,
        fail_open: bool,
    }

    impl PtySystem for FakeSystem {
        fn openpty(&self, size: portable_pty::PtySize) -> anyhow::Result<PtyPair> {
            let mut state = self.state.lock().expect("fake state lock");
            state.open_calls += 1;
            state.last_open_size = Some(size);

            if self.fail_open {
                anyhow::bail!("fake open failed");
            }

            Ok(PtyPair {
                master: Box::new(FakeMaster {
                    state: Arc::clone(&self.state),
                }),
                slave: Box::new(FakeSlave {
                    state: Arc::clone(&self.state),
                }),
            })
        }
    }

    struct FakeMaster {
        state: Arc<Mutex<FakeState>>,
    }

    impl MasterPty for FakeMaster {
        fn resize(&self, size: portable_pty::PtySize) -> Result<(), anyhow::Error> {
            self.state.lock().expect("fake state lock").last_resize = Some(size);
            Ok(())
        }

        fn get_size(&self) -> Result<portable_pty::PtySize, anyhow::Error> {
            Ok(portable_pty::PtySize::default())
        }

        fn try_clone_reader(&self) -> Result<Box<dyn Read + Send>, anyhow::Error> {
            Ok(Box::new(Cursor::new(b"fake output".to_vec())))
        }

        fn take_writer(&self) -> Result<Box<dyn Write + Send>, anyhow::Error> {
            let mut state = self.state.lock().expect("fake state lock");
            state.take_writer_calls += 1;
            Ok(Box::new(FakeWriter {
                state: Arc::clone(&self.state),
            }))
        }

        #[cfg(unix)]
        fn process_group_leader(&self) -> Option<libc::pid_t> {
            None
        }

        #[cfg(unix)]
        fn as_raw_fd(&self) -> Option<portable_pty::unix::RawFd> {
            None
        }

        #[cfg(unix)]
        fn tty_name(&self) -> Option<std::path::PathBuf> {
            None
        }
    }

    struct FakeWriter {
        state: Arc<Mutex<FakeState>>,
    }

    impl Write for FakeWriter {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.state
                .lock()
                .expect("fake state lock")
                .writer_bytes
                .extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct FakeSlave {
        state: Arc<Mutex<FakeState>>,
    }

    impl SlavePty for FakeSlave {
        fn spawn_command(
            &self,
            command: portable_pty::CommandBuilder,
        ) -> Result<Box<dyn Child + Send + Sync>, anyhow::Error> {
            let mut state = self.state.lock().expect("fake state lock");
            state.captured_argv = command.get_argv().clone();
            state.captured_cwd = command.get_cwd().cloned();
            state.captured_empty_env = command.get_env("HERA_EMPTY").map(ToOwned::to_owned);

            Ok(Box::new(FakeChild { waited: false }))
        }
    }

    #[derive(Debug)]
    struct FakeChild {
        waited: bool,
    }

    impl Child for FakeChild {
        fn try_wait(&mut self) -> io::Result<Option<portable_pty::ExitStatus>> {
            Ok(self
                .waited
                .then(|| portable_pty::ExitStatus::with_exit_code(0)))
        }

        fn wait(&mut self) -> io::Result<portable_pty::ExitStatus> {
            self.waited = true;
            Ok(portable_pty::ExitStatus::with_exit_code(0))
        }

        fn process_id(&self) -> Option<u32> {
            Some(4242)
        }

        #[cfg(windows)]
        fn as_raw_handle(&self) -> Option<std::os::windows::io::RawHandle> {
            None
        }
    }

    impl ChildKiller for FakeChild {
        fn kill(&mut self) -> io::Result<()> {
            Ok(())
        }

        fn clone_killer(&self) -> Box<dyn ChildKiller + Send + Sync> {
            Box::new(Self {
                waited: self.waited,
            })
        }
    }

    fn backend(state: Arc<Mutex<FakeState>>, fail_open: bool) -> PortablePtyBackend {
        PortablePtyBackend::with_system(Box::new(FakeSystem { state, fail_open }), "fake")
    }

    fn config(command: PtyCommand) -> PtySessionConfig {
        PtySessionConfig::new(command, PtySize::new(80, 24).expect("valid pty size"))
    }

    #[test]
    fn spawn_returns_session_handles_and_platform_metadata() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let backend = backend(Arc::clone(&state), false);
        let command = PtyCommand::new("hera-test");

        let mut session = backend.spawn(&config(command)).expect("spawn should work");
        let mut output = String::new();
        session
            .reader()
            .expect("reader should be present")
            .read_to_string(&mut output)
            .expect("fake reader should read");

        assert_eq!(output, "fake output");
        assert_eq!(session.metadata().backend(), "fake");
        assert_eq!(session.metadata().process_id(), Some(4242));
        assert_eq!(session.metadata().size().columns(), 80);
        assert!(session.reader().is_some());
        assert!(session.writer().is_some());
    }

    #[test]
    fn backend_maps_openpty_failure_to_operation_and_source() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let backend = backend(state, true);
        let command = PtyCommand::new("hera-test");

        let error = backend
            .spawn(&config(command))
            .expect_err("fake open should fail");

        assert_eq!(error.kind(), PtyErrorKind::Backend);
        assert_eq!(error.operation(), Some("openpty"));
        assert!(
            error
                .source_message()
                .is_some_and(|msg| msg.contains("fake open"))
        );
    }

    #[test]
    fn invalid_cwd_is_rejected_before_opening_pty() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let backend = backend(Arc::clone(&state), false);
        let missing = std::env::temp_dir().join(format!("hera-pty-missing-{}", std::process::id()));
        let command = PtyCommand::new("hera-test").cwd(missing);

        let error = backend
            .spawn(&config(command))
            .expect_err("missing cwd must fail");

        assert_eq!(error.kind(), PtyErrorKind::InvalidCwd);
        assert_eq!(state.lock().expect("fake state lock").open_calls, 0);
    }

    #[test]
    fn writer_is_taken_once_from_backend_and_once_from_public_session() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let backend = backend(Arc::clone(&state), false);
        let command = PtyCommand::new("hera-test");
        let mut session = backend.spawn(&config(command)).expect("spawn should work");

        let mut writer = session.take_writer().expect("writer should be present");
        assert!(session.take_writer().is_none());
        writer
            .write_all(b"input\r")
            .expect("fake write should work");

        let state = state.lock().expect("fake state lock");
        assert_eq!(state.take_writer_calls, 1);
        assert_eq!(state.writer_bytes, b"input\r");
    }

    #[test]
    fn command_builder_preserves_argv_cwd_and_empty_env_override() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let backend = backend(Arc::clone(&state), false);
        let cwd = std::env::current_dir().expect("cwd should exist");
        let command = PtyCommand::new("hera-test")
            .arg("arg with spaces")
            .arg("semi;colon")
            .cwd(&cwd)
            .env("HERA_EMPTY", "");

        let _session = backend.spawn(&config(command)).expect("spawn should work");
        let state = state.lock().expect("fake state lock");

        assert_eq!(
            state.captured_argv,
            vec![
                OsString::from("hera-test"),
                OsString::from("arg with spaces"),
                OsString::from("semi;colon"),
            ]
        );
        assert_eq!(state.captured_cwd, Some(cwd.into_os_string()));
        assert_eq!(state.captured_empty_env, Some(OsString::new()));
    }

    #[test]
    fn resize_delegates_to_backend_with_character_dimensions() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let backend = backend(Arc::clone(&state), false);
        let command = PtyCommand::new("hera-test");
        let session = backend.spawn(&config(command)).expect("spawn should work");
        let size = PtySize::new(100, 30).expect("valid pty size");

        session.resize(size).expect("resize should work");

        let resize = state
            .lock()
            .expect("fake state lock")
            .last_resize
            .expect("resize should be recorded");
        assert_eq!(resize.cols, 100);
        assert_eq!(resize.rows, 30);
    }

    #[test]
    fn child_exit_is_mapped_to_hera_exit_type() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let backend = backend(state, false);
        let command = PtyCommand::new("hera-test");
        let mut session = backend.spawn(&config(command)).expect("spawn should work");

        let exit = session
            .child()
            .expect("child should be present")
            .wait()
            .expect("wait should work");

        assert_eq!(exit.code(), 0);
        assert!(exit.success());
    }

    #[test]
    fn empty_program_is_rejected_before_opening_pty() {
        let state = Arc::new(Mutex::new(FakeState::default()));
        let backend = backend(Arc::clone(&state), false);
        let command = PtyCommand::new("");

        let error = backend
            .spawn(&config(command))
            .expect_err("empty program must fail");

        assert!(matches!(error, PtyError::InvalidCommand { .. }));
        assert_eq!(state.lock().expect("fake state lock").open_calls, 0);
    }
}
