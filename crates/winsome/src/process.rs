//! Spawning children and piping their output into the rolling logs.

use std::io::{self, BufRead};
use std::io::{BufReader, Read};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::config::ComponentConfig;
use crate::rolling_log::RollingLog;

/// The child gets its own process group, so a Ctrl+C at the supervisor's
/// console does not reach it — the supervisor decides how its components die,
/// including asking GlazeWM to exit through its CLI so window positions are
/// restored. No console window either: DESIGN.md § Logging says components run
/// headless and their output goes to the log files, not to windows.
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub struct RunningChild {
    pub child: Child,
    pub pid: u32,
    pub started: Instant,
}

/// Spawn a component, wiring stdout and stderr into `log`.
pub fn spawn(cfg: &ComponentConfig, log: Arc<Mutex<RollingLog>>) -> io::Result<RunningChild> {
    let mut child = Command::new(&cfg.command)
        .args(&cfg.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
        .spawn()?;

    if let Some(out) = child.stdout.take() {
        pump(out, Arc::clone(&log), "out");
    }
    if let Some(err) = child.stderr.take() {
        pump(err, log, "err");
    }

    Ok(RunningChild {
        pid: child.id(),
        child,
        started: Instant::now(),
    })
}

/// Run a short-lived command (a component's `stop_command`) and wait for it.
pub fn run_to_completion(
    command: &str,
    args: &[String],
    timeout: Duration,
) -> io::Result<Option<ExitStatus>> {
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW)
        .spawn()?;
    Ok(wait_for_exit(&mut child, timeout))
}

/// Poll a child until it exits or `timeout` runs out.
pub fn wait_for_exit(child: &mut Child, timeout: Duration) -> Option<ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Some(status),
            Ok(None) => {}
            Err(_) => return None,
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(Duration::from_millis(25));
    }
}

/// Render a command the way it would be typed, for the log line.
pub fn render(cfg: &ComponentConfig) -> String {
    let mut s = quote(&cfg.command);
    for a in &cfg.args {
        s.push(' ');
        s.push_str(&quote(a));
    }
    s
}

fn quote(s: &str) -> String {
    if s.contains(' ') {
        format!("\"{s}\"")
    } else {
        s.to_string()
    }
}

/// One thread per pipe, copying lines into the component's log.
///
/// Bytes are decoded lossily rather than strictly: a tool that emits one
/// non-UTF-8 byte must not cost us the rest of its output. "A failure that
/// leaves no log line is a supervisor bug."
fn pump<R: Read + Send + 'static>(reader: R, log: Arc<Mutex<RollingLog>>, tag: &'static str) {
    thread::spawn(move || {
        let mut buf = BufReader::new(reader);
        let mut line = Vec::new();
        loop {
            line.clear();
            match buf.read_until(b'\n', &mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    while matches!(line.last(), Some(b'\n' | b'\r')) {
                        line.pop();
                    }
                    let text = String::from_utf8_lossy(&line);
                    // A poisoned lock means a logging thread panicked; keep
                    // writing anyway rather than going silent.
                    let mut guard = log.lock().unwrap_or_else(|e| e.into_inner());
                    let _ = guard.write_line(&format!("[{tag}] {text}"));
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_command_line_with_quoting() {
        let cfg = ComponentConfig::new("C:/Program Files/glzr.io/glazewm.exe", &["--config", "a"]);
        assert_eq!(
            render(&cfg),
            "\"C:/Program Files/glzr.io/glazewm.exe\" --config a"
        );
        assert_eq!(
            render(&ComponentConfig::new("kanata.exe", &[])),
            "kanata.exe"
        );
    }

    #[test]
    fn spawning_a_missing_binary_is_an_error_not_a_panic() {
        let dir = crate::testutil::TempDir::new("spawn-missing");
        let log = Arc::new(Mutex::new(
            RollingLog::open(dir.path().join("x.log"), 1024, 2).unwrap(),
        ));
        let cfg = ComponentConfig::new("winsome-no-such-binary-b4d.exe", &[]);
        assert!(spawn(&cfg, log).is_err());
    }
}
