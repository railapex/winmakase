// Shared test helpers. Included via `#[path = "common/mod.rs"] mod common;` in each test
// binary (cargo integration tests each compile as a separate crate, so this isn't picked up
// automatically the way a `mod.rs` under a subdirectory would be).
#![allow(dead_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// RAII scratch directory for tests, in place of a `tempfile` dependency (this crate's
/// dependencies are deliberately limited to toml/serde/serde_json). Copied verbatim from
/// `crates/winsome/src/testutil.rs`'s pattern — cleans up on `Drop`, including during a
/// panic unwind, so a failing `isLightTheme` marker-file test doesn't leak its directory the
/// way a manual cleanup call (skipped when an earlier assertion panics) would.
static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(label: &str) -> Self {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            env::temp_dir().join(format!("winsome-theme-test-{}-{n}-{label}", process::id()));
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
        // Best effort: on Windows a just-killed child can hold a pipe open for a moment. A
        // leaked scratch dir is not worth failing a test over.
        let _ = fs::remove_dir_all(&self.path);
    }
}
