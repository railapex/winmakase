//! Process handles: spawning children, adopting processes we did not spawn,
//! and the trait the supervisor manages them through.
//!
//! Two kinds of handle exist because the two components are hosted two ways
//! (DESIGN.md § elevation split): GlazeWM is a plain spawned child of the
//! user-level supervisor, while kanata runs elevated via the pre-registered
//! `WinsomeKanata` scheduled task — the supervisor cannot be its parent
//! without itself being elevated, which would make everything GlazeWM ever
//! launches elevated too.

use std::ffi::c_void;
use std::io::{self, BufRead};
use std::io::{BufReader, Read};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
    TerminateProcess, WaitForSingleObject,
};

use crate::config::ComponentConfig;
use crate::rolling_log::RollingLog;

/// Standard access right; `WaitForSingleObject` fails without it, and
/// windows-sys only exposes the constant under its file-system feature.
const SYNCHRONIZE: u32 = 0x0010_0000;

/// The child gets its own process group, so a Ctrl+C at the supervisor's
/// console does not reach it — the supervisor decides how its components die,
/// including asking GlazeWM to exit through its CLI so window positions are
/// restored. No console window either: DESIGN.md § Logging says components run
/// headless and their output goes to the log files, not to windows.
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// One non-blocking look at a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Poll {
    Running,
    /// The exit code when the OS will still tell us; `None` for a process we
    /// could not read a code from (adopted, or the read failed).
    Exited(Option<i32>),
}

/// What the supervisor holds instead of a bare `Child`.
///
/// The trait exists for the second implementation, not for ceremony: an
/// elevated, task-hosted kanata has no `Child`, and the test suite needs a
/// handle whose failures can be scripted (`try_wait` errors are unreachable
/// with a real child).
pub trait ProcessHandle: Send {
    fn pid(&self) -> u32;

    /// Non-blocking status check.
    fn poll(&mut self) -> io::Result<Poll>;

    /// Initiate termination and confirm it within `patience`.
    /// `Ok(())` means the process is confirmed gone.
    fn kill(&mut self, patience: Duration) -> io::Result<()>;

    /// One word for log lines: how this process is held.
    fn kind(&self) -> &'static str;
}

pub struct RunningChild {
    pub handle: Box<dyn ProcessHandle>,
    pub started: Instant,
}

impl RunningChild {
    pub fn pid(&self) -> u32 {
        self.handle.pid()
    }
}

// -- spawned children --------------------------------------------------------

struct SpawnedChild {
    child: Child,
}

impl ProcessHandle for SpawnedChild {
    fn pid(&self) -> u32 {
        self.child.id()
    }

    fn poll(&mut self) -> io::Result<Poll> {
        match self.child.try_wait()? {
            Some(status) => Ok(Poll::Exited(status.code())),
            None => Ok(Poll::Running),
        }
    }

    fn kill(&mut self, patience: Duration) -> io::Result<()> {
        self.child.kill()?;
        // TerminateProcess is asynchronous on paper; in practice the wait
        // returns immediately. The bound is here so a wedged process cannot
        // hang the shutdown budget.
        match wait_for_exit(self, patience) {
            Some(_) => Ok(()),
            None => Err(io::Error::other("still alive after kill")),
        }
    }

    fn kind(&self) -> &'static str {
        "spawned"
    }
}

/// Spawn a component as our child, wiring stdout and stderr into `log`.
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
        handle: Box::new(SpawnedChild { child }),
        started: Instant::now(),
    })
}

// -- external processes ------------------------------------------------------

/// A raw process handle that closes itself. `HANDLE` is a pointer type, so the
/// Send is asserted by hand: a process handle is process-global state, not
/// thread-local.
struct OwnedProcessHandle(*mut c_void);

unsafe impl Send for OwnedProcessHandle {}

impl Drop for OwnedProcessHandle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0) };
    }
}

/// A process the supervisor watches but did not spawn: an already-running
/// component it adopted, or one started through an elevated scheduled task.
///
/// Its stdout goes wherever it was already going — the supervisor cannot
/// capture stdio it never held. The log line that adopts it says so.
pub struct ExternalProcess {
    pid: u32,
    handle: OwnedProcessHandle,
    /// When set, `kill` goes through `schtasks /end /tn <task>` — the only
    /// path a user-level supervisor has to an elevated process. The Task
    /// Scheduler service does the terminating, and a task's owner may end it
    /// without elevation.
    task: Option<String>,
}

impl ExternalProcess {
    /// Open a watch handle on `pid`. Fails rather than guesses: a component we
    /// cannot watch is a component we must not pretend to supervise.
    pub fn open(pid: u32, task: Option<String>) -> io::Result<Self> {
        let raw =
            unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, 0, pid) };
        if raw.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!(
                    "cannot open pid {pid} to watch it (Win32 error {})",
                    unsafe { GetLastError() }
                ),
            ));
        }
        Ok(Self {
            pid,
            handle: OwnedProcessHandle(raw),
            task,
        })
    }
}

impl ProcessHandle for ExternalProcess {
    fn pid(&self) -> u32 {
        self.pid
    }

    fn poll(&mut self) -> io::Result<Poll> {
        match unsafe { WaitForSingleObject(self.handle.0, 0) } {
            WAIT_TIMEOUT => Ok(Poll::Running),
            WAIT_OBJECT_0 => {
                let mut code: u32 = 0;
                let got = unsafe { GetExitCodeProcess(self.handle.0, &mut code) };
                // i32 round-trip matches what ExitStatus::code reports for a
                // spawned child; a huge unsigned NTSTATUS comes out negative
                // either way.
                Ok(Poll::Exited((got != 0).then_some(code as i32)))
            }
            _ => Err(io::Error::other(format!(
                "cannot read process state for pid {} (Win32 error {})",
                self.pid,
                unsafe { GetLastError() }
            ))),
        }
    }

    fn kill(&mut self, patience: Duration) -> io::Result<()> {
        if let Some(task) = self.task.clone() {
            // Never let the ask eat the whole patience; the confirmation wait
            // below is the part that matters.
            let ask = (patience / 2).max(Duration::from_millis(200));
            match run_to_completion("schtasks", &args(&["/end", "/tn", &task]), ask) {
                Ok(Some(s)) if !s.success() => {
                    return Err(io::Error::other(format!(
                        "schtasks /end /tn {task} exited with {s}"
                    )));
                }
                Ok(None) => {
                    return Err(io::Error::other(format!(
                        "schtasks /end /tn {task} did not finish in time"
                    )));
                }
                Ok(Some(_)) => {}
                Err(e) => {
                    return Err(io::Error::other(format!(
                        "schtasks /end /tn {task} failed to run: {e}"
                    )));
                }
            }
        } else {
            let raw = unsafe { OpenProcess(PROCESS_TERMINATE, 0, self.pid) };
            if raw.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    format!(
                        "cannot open pid {} to terminate it (Win32 error {})",
                        self.pid,
                        unsafe { GetLastError() }
                    ),
                ));
            }
            let terminate = OwnedProcessHandle(raw);
            if unsafe { TerminateProcess(terminate.0, 1) } == 0 {
                return Err(io::Error::other(format!(
                    "TerminateProcess failed for pid {} (Win32 error {})",
                    self.pid,
                    unsafe { GetLastError() }
                )));
            }
        }

        match wait_for_exit(self, patience) {
            Some(_) => Ok(()),
            None => Err(io::Error::other("still alive after kill")),
        }
    }

    fn kind(&self) -> &'static str {
        if self.task.is_some() {
            "task-hosted"
        } else {
            "adopted"
        }
    }
}

fn args(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| (*s).to_string()).collect()
}

// -- shared helpers ----------------------------------------------------------

/// Run a short-lived command (a component's `stop_command`, a `schtasks` verb)
/// and wait for it.
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
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(Some(status)),
            Ok(None) => {}
            Err(_) => return Ok(None),
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(25));
    }
}

/// Poll a handle until it exits or `timeout` runs out.
/// `Some(code)` when it exited; a poll error also ends the wait as `None`.
pub fn wait_for_exit(
    handle: &mut dyn ProcessHandle,
    timeout: Duration,
) -> Option<Option<i32>> {
    let deadline = Instant::now() + timeout;
    loop {
        match handle.poll() {
            Ok(Poll::Exited(code)) => return Some(code),
            Ok(Poll::Running) => {}
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

    #[test]
    fn a_spawned_child_polls_running_then_exited_with_its_code() {
        let dir = crate::testutil::TempDir::new("spawn-poll");
        let log = Arc::new(Mutex::new(
            RollingLog::open(dir.path().join("x.log"), 1024, 2).unwrap(),
        ));
        // `cmd /c exit 7`: exits almost immediately with a known code.
        let cfg = ComponentConfig::new("cmd", &["/c", "exit 7"]);
        let mut running = spawn(&cfg, log).unwrap();
        let code = wait_for_exit(running.handle.as_mut(), Duration::from_secs(10))
            .expect("the child exits");
        assert_eq!(code, Some(7));
        assert_eq!(running.handle.poll().unwrap(), Poll::Exited(Some(7)));
    }

    #[test]
    fn an_adopted_process_is_watched_and_killable() {
        // A process we did not spawn through `spawn()`: raw std Command.
        let mut child = Command::new("cmd")
            .args(["/c", "ping -n 30 127.0.0.1 > nul"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .unwrap();
        let mut ext = ExternalProcess::open(child.id(), None).unwrap();
        assert_eq!(ext.pid(), child.id());
        assert_eq!(ext.poll().unwrap(), Poll::Running);
        assert_eq!(ext.kind(), "adopted");

        ext.kill(Duration::from_secs(5)).unwrap();
        assert!(matches!(ext.poll().unwrap(), Poll::Exited(_)));
        let _ = child.wait();
    }

    #[test]
    fn an_adopted_process_reports_its_natural_exit_code() {
        let mut child = Command::new("cmd")
            .args(["/c", "exit 9"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .unwrap();
        let pid = child.id();
        // Open before it exits or after — a handle keeps the exit code alive
        // either way; opening can only fail once the pid is fully gone.
        let mut ext = match ExternalProcess::open(pid, None) {
            Ok(e) => e,
            // Lost the race: the process was reaped before we opened it.
            Err(_) => {
                let _ = child.wait();
                return;
            }
        };
        let code = wait_for_exit(&mut ext, Duration::from_secs(10)).expect("exits");
        assert_eq!(code, Some(9));
        let _ = child.wait();
    }

    #[test]
    fn opening_a_dead_pid_fails_loud() {
        let mut child = Command::new("cmd")
            .args(["/c", "exit 0"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .unwrap();
        let pid = child.id();
        child.wait().unwrap();
        // The handle table entry lingers briefly; poll until the open fails or
        // the opened handle reads Exited — either is a correct answer, what is
        // banned is "Running".
        match ExternalProcess::open(pid, None) {
            Err(_) => {}
            Ok(mut ext) => assert!(matches!(ext.poll().unwrap(), Poll::Exited(_))),
        }
    }
}
