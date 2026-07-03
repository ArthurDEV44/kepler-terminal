use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtyErrorKind {
    InvalidSize,
    InvalidCommand,
    InvalidCwd,
    InvalidRuntimeConfig,
    Backend,
    ClosedSession,
    WriteTooLarge,
    Timeout,
    PartialResize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PtyError {
    InvalidSize {
        columns: u16,
        rows: u16,
        max_columns: u16,
        max_rows: u16,
    },
    InvalidCommand {
        field: &'static str,
        message: String,
    },
    InvalidCwd {
        path: PathBuf,
        message: String,
    },
    InvalidRuntimeConfig {
        field: &'static str,
        message: String,
    },
    Backend {
        operation: &'static str,
        source: String,
    },
    ClosedSession {
        operation: &'static str,
    },
    WriteTooLarge {
        requested: usize,
        max: usize,
    },
    Timeout {
        operation: &'static str,
        timeout_ms: u64,
        source: Option<String>,
    },
    PartialResize {
        columns: u16,
        rows: u16,
        operation: &'static str,
        source: String,
    },
}

impl PtyError {
    #[must_use]
    pub const fn kind(&self) -> PtyErrorKind {
        match self {
            Self::InvalidSize { .. } => PtyErrorKind::InvalidSize,
            Self::InvalidCommand { .. } => PtyErrorKind::InvalidCommand,
            Self::InvalidCwd { .. } => PtyErrorKind::InvalidCwd,
            Self::InvalidRuntimeConfig { .. } => PtyErrorKind::InvalidRuntimeConfig,
            Self::Backend { .. } => PtyErrorKind::Backend,
            Self::ClosedSession { .. } => PtyErrorKind::ClosedSession,
            Self::WriteTooLarge { .. } => PtyErrorKind::WriteTooLarge,
            Self::Timeout { .. } => PtyErrorKind::Timeout,
            Self::PartialResize { .. } => PtyErrorKind::PartialResize,
        }
    }

    #[must_use]
    pub const fn operation(&self) -> Option<&'static str> {
        match self {
            Self::Backend { operation, .. }
            | Self::ClosedSession { operation }
            | Self::Timeout { operation, .. }
            | Self::PartialResize { operation, .. } => Some(operation),
            Self::InvalidSize { .. }
            | Self::InvalidCommand { .. }
            | Self::InvalidCwd { .. }
            | Self::InvalidRuntimeConfig { .. }
            | Self::WriteTooLarge { .. } => None,
        }
    }

    #[must_use]
    pub fn source_message(&self) -> Option<&str> {
        match self {
            Self::Backend { source, .. } | Self::PartialResize { source, .. } => Some(source),
            Self::Timeout {
                source: Some(source),
                ..
            } => Some(source),
            Self::InvalidSize { .. }
            | Self::InvalidCommand { .. }
            | Self::InvalidCwd { .. }
            | Self::InvalidRuntimeConfig { .. }
            | Self::ClosedSession { .. }
            | Self::WriteTooLarge { .. }
            | Self::Timeout { source: None, .. } => None,
        }
    }

    pub(crate) fn backend(operation: &'static str, source: impl fmt::Display) -> Self {
        Self::Backend {
            operation,
            source: source.to_string(),
        }
    }

    pub(crate) fn invalid_command(field: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidCommand {
            field,
            message: message.into(),
        }
    }

    pub(crate) fn invalid_runtime_config(field: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidRuntimeConfig {
            field,
            message: message.into(),
        }
    }

    pub(crate) fn timeout(operation: &'static str, timeout_ms: u64) -> Self {
        Self::Timeout {
            operation,
            timeout_ms,
            source: None,
        }
    }

    pub(crate) fn timeout_with_source(
        operation: &'static str,
        timeout_ms: u64,
        source: Self,
    ) -> Self {
        Self::Timeout {
            operation,
            timeout_ms,
            source: Some(source.to_string()),
        }
    }

    pub(crate) fn partial_resize(size: crate::PtySize, source: Self) -> Self {
        let operation = source.operation().unwrap_or("resize");
        Self::PartialResize {
            columns: size.columns(),
            rows: size.rows(),
            operation,
            source: source.to_string(),
        }
    }
}

impl fmt::Display for PtyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize {
                columns,
                rows,
                max_columns,
                max_rows,
            } => write!(
                formatter,
                "invalid PTY dimensions: {columns} columns, {rows} rows; maximum is {max_columns} columns and {max_rows} rows"
            ),
            Self::InvalidCommand { field, message } => {
                write!(formatter, "invalid PTY command {field}: {message}")
            }
            Self::InvalidCwd { path, message } => {
                write!(formatter, "invalid PTY cwd {}: {message}", path.display())
            }
            Self::InvalidRuntimeConfig { field, message } => {
                write!(formatter, "invalid PTY runtime config {field}: {message}")
            }
            Self::Backend { operation, source } => {
                write!(formatter, "PTY operation {operation} failed: {source}")
            }
            Self::ClosedSession { operation } => {
                write!(formatter, "PTY session is closed during {operation}")
            }
            Self::WriteTooLarge { requested, max } => write!(
                formatter,
                "PTY input write is {requested} bytes, maximum chunk is {max}"
            ),
            Self::Timeout {
                operation,
                timeout_ms,
                source,
            } => {
                write!(
                    formatter,
                    "PTY operation {operation} timed out after {timeout_ms}ms"
                )?;
                if let Some(source) = source {
                    write!(formatter, "; cleanup failed: {source}")?;
                }
                Ok(())
            }
            Self::PartialResize {
                columns,
                rows,
                operation,
                source,
            } => write!(
                formatter,
                "PTY resize to {columns}x{rows} partially failed during {operation}: {source}"
            ),
        }
    }
}

impl std::error::Error for PtyError {}
