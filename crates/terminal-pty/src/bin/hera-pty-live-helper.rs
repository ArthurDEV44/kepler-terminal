use std::io::{self, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(command) = std::env::args().nth(1) else {
        return ExitCode::from(2);
    };

    match command.as_str() {
        "token" => {
            print!("HERA_LIVE_PTY");
            ExitCode::SUCCESS
        }
        "one-megabyte" => {
            let chunk = vec![b'x'; 8192];
            let mut stdout = io::stdout().lock();
            for _ in 0..128 {
                if stdout.write_all(&chunk).is_err() {
                    return ExitCode::from(1);
                }
            }
            ExitCode::SUCCESS
        }
        "size-query" => {
            let mut stdout = io::stdout().lock();
            if stdout.write_all(b"\x1b[18t").is_err() || stdout.flush().is_err() {
                return ExitCode::from(1);
            }

            let mut response = Vec::new();
            for byte in io::stdin().lock().bytes().take(64) {
                let Ok(byte) = byte else {
                    return ExitCode::from(1);
                };
                response.push(byte);
                if byte == b't' {
                    break;
                }
            }

            let Some((columns, rows)) = parse_size_response(&response) else {
                return ExitCode::from(1);
            };
            print!("HERA_PTY_SIZE={columns}x{rows}");
            ExitCode::SUCCESS
        }
        _ => ExitCode::from(2),
    }
}

fn parse_size_response(response: &[u8]) -> Option<(u16, u16)> {
    let response = std::str::from_utf8(response).ok()?;
    let raw = response.strip_prefix("\x1b[8;")?.strip_suffix('t')?;
    let (rows, columns) = raw.split_once(';')?;
    Some((columns.parse().ok()?, rows.parse().ok()?))
}
