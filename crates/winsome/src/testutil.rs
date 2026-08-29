//! Scratch directories and scripted process handles for the test suites, in
//! place of a `tempfile`/mock dependency. Public so the integration tests (a
//! separate crate) can use them; not part of the CLI's surface.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::process::{Poll, ProcessHandle};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(label: &str) -> Self {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = env::temp_dir().join(format!("winsome-test-{}-{n}-{label}", process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        // Best effort: on Windows a just-killed child can hold a pipe open for
        // a moment. A leaked scratch dir is not worth failing a test over.
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// One scripted answer from a [`FakeHandle`] poll.
pub enum FakePoll {
    Running,
    Exited(Option<i32>),
    /// A status read that fails — the case a real `Child` can never produce
    /// on demand, and the reason this fake exists.
    Error(&'static str),
}

/// A process handle whose behaviour is a script. Polls consume the script in
/// order; the final entry repeats forever.
pub struct FakeHandle {
    pid: u32,
    script: Vec<FakePoll>,
    next: usize,
    /// Whether kill() reports success, and what polls say afterwards.
    killable: bool,
}

impl FakeHandle {
    pub fn new(pid: u32, script: Vec<FakePoll>) -> Self {
        Self {
            pid,
            script,
            next: 0,
            killable: true,
        }
    }

    pub fn unkillable(mut self) -> Self {
        self.killable = false;
        self
    }
}

impl ProcessHandle for FakeHandle {
    fn pid(&self) -> u32 {
        self.pid
    }

    fn poll(&mut self) -> io::Result<Poll> {
        let i = self.next.min(self.script.len().saturating_sub(1));
        self.next = self.next.saturating_add(1);
        match self.script.get(i) {
            Some(FakePoll::Running) => Ok(Poll::Running),
            Some(FakePoll::Exited(code)) => Ok(Poll::Exited(*code)),
            Some(FakePoll::Error(msg)) => Err(io::Error::other(*msg)),
            None => Ok(Poll::Exited(None)),
        }
    }

    fn kill(&mut self, _patience: Duration) -> io::Result<()> {
        if self.killable {
            self.script = vec![FakePoll::Exited(None)];
            self.next = 0;
            Ok(())
        } else {
            Err(io::Error::other("scripted: unkillable"))
        }
    }

    fn kind(&self) -> &'static str {
        "fake"
    }
}
