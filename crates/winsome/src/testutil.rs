//! Scratch directories for the test suites, in place of a `tempfile`
//! dependency. Public so the integration tests (a separate crate) can use it;
//! not part of the CLI's surface.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

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
