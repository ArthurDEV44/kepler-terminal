//! PTY runtime boundary for Hera M2.
//!
//! This crate owns process IO and platform PTY adapters. `terminal-core` stays
//! responsible for terminal state, parser semantics and snapshots, and must not
//! depend on this crate.

#![forbid(unsafe_code)]

mod bridge;
mod command;
mod error;
mod event;
mod portable_backend;
mod runtime;
mod session;
mod size;

pub use bridge::{PtyBridge, PtyBridgeError, PtyEventSink};
pub use command::{PtyCommand, PtySessionConfig};
pub use error::{PtyError, PtyErrorKind};
pub use event::{PtyEvent, PtyExit};
pub use portable_backend::PortablePtyBackend;
pub use runtime::{
    M2_DEFAULT_COMMAND_TIMEOUT_MS, M2_DEFAULT_DRAIN_TIMEOUT_MS, M2_DEFAULT_EVENT_CAPACITY,
    M2_DEFAULT_READ_CHUNK_BYTES, M2_DEFAULT_WRITE_TIMEOUT_MS, M2_MAX_COMMAND_TIMEOUT_MS,
    M2_MAX_DRAIN_TIMEOUT_MS, M2_MAX_EVENT_CAPACITY, M2_MAX_EVENTS_PER_TICK,
    M2_MAX_READ_CHUNK_BYTES, M2_MAX_WRITE_TIMEOUT_MS, PtyRunOutcome, PtyRuntimeConfig,
    PtySessionRunner,
};
pub use session::{
    M2_MAX_WRITE_CHUNK_BYTES, PtyBackend, PtyChild, PtyChildKiller, PtyPlatformMetadata, PtySession,
};
pub use size::{M2_MAX_PTY_COLUMNS, M2_MAX_PTY_ROWS, PtySize};

#[cfg(test)]
mod tests {
    const LIB_SOURCE: &str = include_str!("lib.rs");
    const BRIDGE_SOURCE: &str = include_str!("bridge.rs");
    const COMMAND_SOURCE: &str = include_str!("command.rs");
    const ERROR_SOURCE: &str = include_str!("error.rs");
    const EVENT_SOURCE: &str = include_str!("event.rs");
    const PORTABLE_BACKEND_SOURCE: &str = include_str!("portable_backend.rs");
    const RUNTIME_SOURCE: &str = include_str!("runtime.rs");
    const SESSION_SOURCE: &str = include_str!("session.rs");
    const SIZE_SOURCE: &str = include_str!("size.rs");

    #[test]
    fn public_api_does_not_expose_portable_pty_types() {
        let public_marker = ["pub", " "].concat();
        let portable_path = ["portable_", "pty::"].concat();

        for source in [
            LIB_SOURCE,
            BRIDGE_SOURCE,
            COMMAND_SOURCE,
            ERROR_SOURCE,
            EVENT_SOURCE,
            PORTABLE_BACKEND_SOURCE,
            RUNTIME_SOURCE,
            SESSION_SOURCE,
            SIZE_SOURCE,
        ] {
            for line in source.lines() {
                assert!(
                    !(line.contains(&public_marker) && line.contains(&portable_path)),
                    "public Hera PTY API must not expose portable-pty types: {line}"
                );
            }
        }
    }
}
