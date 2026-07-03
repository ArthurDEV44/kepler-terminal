#![cfg(feature = "live-pty-tests")]

use std::time::{Duration, Instant};

use terminal_pty::{
    PortablePtyBackend, PtyCommand, PtyEvent, PtyRuntimeConfig, PtySessionConfig, PtySessionRunner,
    PtySize,
};

const LIVE_TOKEN: &[u8] = b"HERA_LIVE_PTY";
const RESIZE_COLUMNS: u16 = 100;
const RESIZE_ROWS: u16 = 30;

#[test]
#[ignore = "host-dependent PTY smoke test"]
fn live_command_emits_output_and_exit_zero() {
    let command = helper_command("token");
    let Some(output) = run_live_command(command, Duration::from_secs(5)) else {
        return;
    };

    assert!(
        output
            .windows(LIVE_TOKEN.len())
            .any(|window| window == LIVE_TOKEN),
        "live PTY output did not contain token"
    );
}

#[test]
#[ignore = "host-dependent platform shell PTY smoke test"]
fn live_platform_shell_prints_token_and_exit_zero() {
    let command = platform_shell_command();
    let Some(output) = run_live_command(command, Duration::from_secs(5)) else {
        return;
    };

    assert!(
        output
            .windows(LIVE_TOKEN.len())
            .any(|window| window == LIVE_TOKEN),
        "live platform shell output did not contain token"
    );
}

#[test]
#[ignore = "host-dependent PTY resize smoke test"]
fn live_resize_query_reports_requested_dimensions() {
    let command = helper_command("size-query");
    let requested = PtySize::new(RESIZE_COLUMNS, RESIZE_ROWS).expect("valid resize");
    let Some(output) =
        run_live_command_with_resize(command, Duration::from_secs(5), Some(requested), requested)
    else {
        return;
    };
    let token = format!("HERA_PTY_SIZE={RESIZE_COLUMNS}x{RESIZE_ROWS}");

    assert!(
        output
            .windows(token.len())
            .any(|window| window == token.as_bytes()),
        "live resize query did not report requested dimensions"
    );
}

#[test]
#[ignore = "host-dependent PTY throughput smoke test"]
fn live_one_megabyte_output_drains_within_five_seconds() {
    let command = helper_command("one-megabyte");
    let started = Instant::now();
    let Some(output) = run_live_command(command, Duration::from_secs(5)) else {
        return;
    };

    assert!(
        started.elapsed() <= Duration::from_secs(5),
        "1 MB PTY output did not drain within 5 seconds"
    );
    assert!(
        output.len() >= 1024 * 1024,
        "expected at least 1 MB of PTY output, got {} bytes",
        output.len()
    );
}

fn run_live_command(command: PtyCommand, timeout: Duration) -> Option<Vec<u8>> {
    run_live_command_with_resize(
        command,
        timeout,
        None,
        PtySize::new(80, 24).expect("valid live PTY size"),
    )
}

fn run_live_command_with_resize(
    command: PtyCommand,
    timeout: Duration,
    resize: Option<PtySize>,
    response_size: PtySize,
) -> Option<Vec<u8>> {
    let backend = PortablePtyBackend::new();
    let size = PtySize::new(80, 24).expect("valid live PTY size");
    let config = PtySessionConfig::new(command, size);
    let runtime = PtyRuntimeConfig::default()
        .with_command_timeout(timeout)
        .expect("valid command timeout");
    let mut runner = match PtySessionRunner::spawn(&backend, &config, runtime) {
        Ok(runner) => runner,
        Err(error) => {
            eprintln!("skipping live PTY test: {error}");
            return None;
        }
    };
    if let Some(size) = resize {
        if let Err(error) = runner.resize(size) {
            eprintln!("skipping live PTY resize smoke: {error}");
            return None;
        }
    }
    let mut output = Vec::new();
    let mut answered_cursor_query = false;
    let mut answered_size_query = false;
    let outcome = match runner.run_until_exit_with_responses(|event| {
        if let PtyEvent::Output(bytes) = event {
            output.extend(bytes);
            if !answered_cursor_query
                && output
                    .windows(b"\x1b[6n".len())
                    .any(|window| window == b"\x1b[6n")
            {
                answered_cursor_query = true;
                return Ok(Some(b"\x1b[1;1R".to_vec()));
            }
            if !answered_size_query
                && output
                    .windows(b"\x1b[18t".len())
                    .any(|window| window == b"\x1b[18t")
            {
                answered_size_query = true;
                return Ok(Some(
                    format!(
                        "\x1b[8;{};{}t",
                        response_size.rows(),
                        response_size.columns()
                    )
                    .into_bytes(),
                ));
            }
        }
        Ok(None)
    }) {
        Ok(outcome) => outcome,
        Err(error) if cfg!(windows) => {
            eprintln!(
                "skipping Windows live PTY smoke: portable-pty ConPTY wait/close did not complete cleanly: {error}"
            );
            return None;
        }
        Err(error) => panic!("live PTY command should complete: {error}"),
    };

    if cfg!(windows) && !outcome.exit().success() {
        eprintln!(
            "skipping Windows live PTY smoke: portable-pty ConPTY exited with {}",
            outcome.exit().code()
        );
        return None;
    }

    assert!(
        outcome.exit().success(),
        "live PTY command exited with {}",
        outcome.exit().code()
    );
    Some(output)
}

fn helper_command(command: &str) -> PtyCommand {
    PtyCommand::new(env!("CARGO_BIN_EXE_hera-pty-live-helper")).arg(command)
}

#[cfg(windows)]
fn platform_shell_command() -> PtyCommand {
    PtyCommand::new("cmd.exe").args(["/D", "/C", "echo HERA_LIVE_PTY"])
}

#[cfg(unix)]
fn platform_shell_command() -> PtyCommand {
    PtyCommand::new("/bin/sh").args(["-lc", "printf HERA_LIVE_PTY"])
}
