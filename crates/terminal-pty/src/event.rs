use crate::PtySize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtyExit {
    code: u32,
    signal: Option<String>,
}

impl PtyExit {
    #[must_use]
    pub fn new(code: u32, signal: Option<String>) -> Self {
        Self { code, signal }
    }

    #[must_use]
    pub const fn code(&self) -> u32 {
        self.code
    }

    #[must_use]
    pub fn signal(&self) -> Option<&str> {
        self.signal.as_deref()
    }

    #[must_use]
    pub const fn success(&self) -> bool {
        self.code == 0 && self.signal.is_none()
    }

    pub(crate) fn from_portable(status: portable_pty::ExitStatus) -> Self {
        Self {
            code: status.exit_code(),
            signal: status.signal().map(ToOwned::to_owned),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyEvent {
    Output(Vec<u8>),
    Eof,
    Exit(PtyExit),
    Resize(PtySize),
}
