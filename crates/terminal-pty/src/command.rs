use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use crate::{PtyError, PtySize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtyCommand {
    program: OsString,
    args: Vec<OsString>,
    cwd: Option<PathBuf>,
    env: BTreeMap<OsString, OsString>,
}

impl PtyCommand {
    #[must_use]
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            env: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    #[must_use]
    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    #[must_use]
    pub fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    #[must_use]
    pub fn program(&self) -> &OsStr {
        &self.program
    }

    #[must_use]
    pub fn args_slice(&self) -> &[OsString] {
        &self.args
    }

    #[must_use]
    pub fn cwd_path(&self) -> Option<&Path> {
        self.cwd.as_deref()
    }

    #[must_use]
    pub fn env_overrides(&self) -> &BTreeMap<OsString, OsString> {
        &self.env
    }

    pub fn validate(&self) -> Result<(), PtyError> {
        if self.program.as_os_str().is_empty() {
            return Err(PtyError::invalid_command(
                "program",
                "program must not be empty",
            ));
        }

        for key in self.env.keys() {
            if key.as_os_str().is_empty() {
                return Err(PtyError::invalid_command(
                    "env",
                    "environment variable name must not be empty",
                ));
            }
        }

        if let Some(cwd) = &self.cwd {
            let metadata = std::fs::metadata(cwd).map_err(|error| PtyError::InvalidCwd {
                path: cwd.clone(),
                message: error.to_string(),
            })?;

            if !metadata.is_dir() {
                return Err(PtyError::InvalidCwd {
                    path: cwd.clone(),
                    message: "path is not a directory".to_owned(),
                });
            }
        }

        Ok(())
    }

    pub(crate) fn to_portable_command(&self) -> Result<portable_pty::CommandBuilder, PtyError> {
        self.validate()?;

        let mut command = portable_pty::CommandBuilder::new(&self.program);
        command.args(&self.args);

        if let Some(cwd) = &self.cwd {
            command.cwd(cwd.as_os_str());
        }

        for (key, value) in &self.env {
            command.env(key, value);
        }

        Ok(command)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtySessionConfig {
    command: PtyCommand,
    size: PtySize,
}

impl PtySessionConfig {
    #[must_use]
    pub const fn new(command: PtyCommand, size: PtySize) -> Self {
        Self { command, size }
    }

    #[must_use]
    pub const fn command(&self) -> &PtyCommand {
        &self.command
    }

    #[must_use]
    pub const fn size(&self) -> PtySize {
        self.size
    }

    pub fn validate(&self) -> Result<(), PtyError> {
        self.command.validate()
    }
}

#[cfg(test)]
mod tests {
    use super::PtyCommand;
    use crate::{PtyError, PtyErrorKind};

    #[test]
    fn empty_program_returns_typed_validation_error() {
        let error = PtyCommand::new("").validate().expect_err("empty must fail");

        assert_eq!(error.kind(), PtyErrorKind::InvalidCommand);
        assert!(matches!(
            error,
            PtyError::InvalidCommand {
                field: "program",
                ..
            }
        ));
    }

    #[test]
    fn missing_cwd_returns_typed_validation_error() {
        let path = std::env::temp_dir().join(format!("hera-missing-cwd-{}", std::process::id()));
        let error = PtyCommand::new("hera-test")
            .cwd(path.clone())
            .validate()
            .expect_err("missing cwd must fail");

        assert_eq!(error.kind(), PtyErrorKind::InvalidCwd);
        assert!(matches!(error, PtyError::InvalidCwd { path: got, .. } if got == path));
    }
}
