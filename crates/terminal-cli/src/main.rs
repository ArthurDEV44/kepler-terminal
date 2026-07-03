//! Debug CLI boundary for Hera M1.

#![forbid(unsafe_code)]

use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use terminal_core::Terminal;
use terminal_fixtures::{
    FixtureRunner, M1_MAX_FIXTURE_INPUT_BYTES, M1_MAX_SNAPSHOT_BYTES, deserialize_snapshot,
    first_snapshot_difference, serialize_snapshot_pretty, snapshot_terminal,
};

fn main() -> ExitCode {
    let outcome = run(std::env::args_os().skip(1).collect());

    if !outcome.stdout.is_empty() {
        println!("{}", outcome.stdout);
    }
    if !outcome.stderr.is_empty() {
        eprintln!("{}", outcome.stderr);
    }

    ExitCode::from(outcome.code)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandOutcome {
    code: u8,
    stdout: String,
    stderr: String,
}

impl CommandOutcome {
    fn success(stdout: impl Into<String>) -> Self {
        Self {
            code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    fn failure(code: u8, stderr: impl Into<String>) -> Self {
        Self {
            code,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }
}

fn run(args: Vec<OsString>) -> CommandOutcome {
    let Some(command) = args.first().and_then(|value| value.to_str()) else {
        return CommandOutcome::failure(2, usage());
    };

    match command {
        "inject" => inject_command(&args[1..]),
        "replay" => replay_command(&args[1..]),
        "compare" => compare_command(&args[1..]),
        "run" => CommandOutcome::failure(1, "PTY execution is M2"),
        _ => CommandOutcome::failure(2, usage()),
    }
}

fn inject_command(args: &[OsString]) -> CommandOutcome {
    let Some(path) = one_path_arg(args) else {
        return CommandOutcome::failure(2, "usage: terminal-cli inject <file>");
    };

    let bytes = match read_bytes_capped(&path, M1_MAX_FIXTURE_INPUT_BYTES) {
        Ok(bytes) => bytes,
        Err(message) => return CommandOutcome::failure(1, message),
    };

    let mut terminal = Terminal::with_default_dimensions();
    terminal.advance_bytes(&bytes);
    let snapshot = snapshot_terminal(&mut terminal);

    match serialize_snapshot_pretty(&snapshot) {
        Ok(snapshot) => CommandOutcome::success(snapshot),
        Err(error) => CommandOutcome::failure(1, error.to_string()),
    }
}

fn replay_command(args: &[OsString]) -> CommandOutcome {
    let Some(path) = one_path_arg(args) else {
        return CommandOutcome::failure(2, "usage: terminal-cli replay <fixture>");
    };

    match FixtureRunner::run_pack_path(&path) {
        Ok(reports) => {
            let mut lines = reports
                .iter()
                .map(|report| format!("fixture {}: pass", report.name()))
                .collect::<Vec<_>>();
            lines.push(format!("{} fixtures passed", reports.len()));
            CommandOutcome::success(lines.join("\n"))
        }
        Err(error) => CommandOutcome::failure(1, error.to_string()),
    }
}

fn compare_command(args: &[OsString]) -> CommandOutcome {
    if args.len() != 2 {
        return CommandOutcome::failure(2, "usage: terminal-cli compare <a> <b>");
    }

    let left = match read_snapshot(&PathBuf::from(&args[0])) {
        Ok(snapshot) => snapshot,
        Err(message) => return CommandOutcome::failure(1, message),
    };
    let right = match read_snapshot(&PathBuf::from(&args[1])) {
        Ok(snapshot) => snapshot,
        Err(message) => return CommandOutcome::failure(1, message),
    };

    match first_snapshot_difference(&left, &right) {
        Some(difference) => CommandOutcome::failure(1, difference.to_string()),
        None => CommandOutcome::success("snapshots match"),
    }
}

fn read_snapshot(path: &Path) -> Result<terminal_fixtures::TerminalSnapshot, String> {
    let bytes = read_bytes_capped(path, M1_MAX_SNAPSHOT_BYTES)?;
    deserialize_snapshot(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn read_bytes_capped(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("{}: expected a regular file", path.display()));
    }
    if metadata.len() > limit as u64 {
        return Err(format!(
            "{}: file is {} bytes, maximum is {limit}",
            path.display(),
            metadata.len()
        ));
    }

    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut reader = file.take(limit as u64 + 1);
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;

    if bytes.len() > limit {
        return Err(format!(
            "{}: file exceeded maximum of {limit} bytes while reading",
            path.display()
        ));
    }

    Ok(bytes)
}

fn one_path_arg(args: &[OsString]) -> Option<PathBuf> {
    (args.len() == 1).then(|| PathBuf::from(&args[0]))
}

fn usage() -> &'static str {
    "usage: terminal-cli <inject|replay|compare|run> ..."
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    fn temp_dir(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        fs::create_dir_all(&path).expect("temp dir should be writable");
        path
    }

    #[test]
    fn run_command_reports_pty_deferred() {
        let outcome = run(args(&["run", "echo", "hello"]));

        assert_eq!(outcome.code, 1);
        assert_eq!(outcome.stderr, "PTY execution is M2");
    }

    #[test]
    fn inject_prints_deterministic_snapshot_json() {
        let path = std::env::temp_dir().join(format!("hera-cli-inject-{}.txt", std::process::id()));
        fs::write(&path, b"abc").expect("temp input should be writable");

        let outcome = run(vec![
            OsString::from("inject"),
            path.clone().into_os_string(),
        ]);
        let _ = fs::remove_file(&path);

        assert_eq!(outcome.code, 0);
        assert!(outcome.stdout.contains("\"columns\""));
        assert!(outcome.stdout.contains("\"ch\": \"a\""));
        assert!(outcome.stdout.contains("\"ch\": \"b\""));
        assert!(outcome.stdout.contains("\"ch\": \"c\""));
    }

    #[test]
    fn replay_reports_pass_and_assertion_failure() {
        let dir = temp_dir("hera-cli-replay");
        let pass_fixture = dir.join("pass.json");
        let fail_fixture = dir.join("fail.json");
        fs::write(
            &pass_fixture,
            r#"{"fixtures":[{"name":"ok","terminal":{"columns":2,"rows":1},"chunks":[{"bytes":[97]}],"expected":{"viewport_lines":["a"]}}]}"#,
        )
        .expect("pass fixture should be writable");
        fs::write(
            &fail_fixture,
            r#"{"fixtures":[{"name":"bad","terminal":{"columns":2,"rows":1},"chunks":[{"bytes":[97]}],"expected":{"viewport_lines":["b"]}}]}"#,
        )
        .expect("fail fixture should be writable");

        let pass = run(vec![
            OsString::from("replay"),
            pass_fixture.clone().into_os_string(),
        ]);
        let fail = run(vec![
            OsString::from("replay"),
            fail_fixture.clone().into_os_string(),
        ]);
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(pass.code, 0);
        assert!(pass.stdout.contains("fixture ok: pass"));
        assert_eq!(fail.code, 1);
        assert!(fail.stderr.contains("snapshot mismatch"));
    }

    #[test]
    fn compare_reports_match_and_first_difference() {
        let dir = temp_dir("hera-cli-compare");
        let input_a = dir.join("a.bin");
        let input_b = dir.join("b.bin");
        let snapshot_a = dir.join("a.json");
        let snapshot_b = dir.join("b.json");
        fs::write(&input_a, b"abc").expect("input a should be writable");
        fs::write(&input_b, b"abd").expect("input b should be writable");

        let injected_a = run(vec![
            OsString::from("inject"),
            input_a.clone().into_os_string(),
        ]);
        let injected_b = run(vec![
            OsString::from("inject"),
            input_b.clone().into_os_string(),
        ]);
        fs::write(&snapshot_a, &injected_a.stdout).expect("snapshot a should be writable");
        fs::write(&snapshot_b, &injected_b.stdout).expect("snapshot b should be writable");

        let same = run(vec![
            OsString::from("compare"),
            snapshot_a.clone().into_os_string(),
            snapshot_a.clone().into_os_string(),
        ]);
        let different = run(vec![
            OsString::from("compare"),
            snapshot_a.into_os_string(),
            snapshot_b.into_os_string(),
        ]);
        let _ = fs::remove_dir_all(&dir);

        assert_eq!(same.code, 0);
        assert_eq!(same.stdout, "snapshots match");
        assert_eq!(different.code, 1);
        assert!(different.stderr.contains("$.viewport_rows[0].cells[2].ch"));
    }
}
