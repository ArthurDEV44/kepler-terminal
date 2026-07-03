# M2 PTY Recordings

PTY recordings are checked-in replay evidence for live PTY behavior. They keep
M2 tests deterministic when the local shell, ConPTY version or POSIX PTY
behavior differs by host.

## Format

Each recording is JSON with:

- `schema: "hera.pty_recording"`
- `version: 1`
- command, platform, exit, runtime, input and resize metadata
- timestamped `output`, `input`, `resize`, `eof` and `exit` events
- `final_snapshot`

Timestamps are preserved as metadata. Replay ignores timestamps and applies only
output bytes and resize events to `terminal-core`.

## Refresh Flow

Use the CLI harness to capture a local session:

```powershell
cargo run -p terminal-cli -- run --cols 80 --rows 24 --record crates/terminal-fixtures/fixtures/pty/<name>.json -- <program> <args>
```

Then keep the recording small and reviewable:

1. Prefer commands with deterministic output.
2. Keep shell-specific recordings out of the offline pack unless the bytes are
   stable across replay.
3. Run `cargo test -p terminal-fixtures pty_recording`.
4. Run the full workspace gates before marking the PRD story in review.

The checked-in pack intentionally uses tiny synthetic recordings. Live host
coverage belongs in ignored `live-pty-tests`, while offline replay coverage must
not require a shell.
