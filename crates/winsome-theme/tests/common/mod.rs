// Shared test helpers. Included via `#[path = "support.rs"] mod support;` in each test
// binary (cargo integration tests each compile as a separate crate, so this isn't picked up
// automatically the way a `mod.rs` under a subdirectory would be).
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

/// Node's tests use `mkdtempSync`/`rmSync` for a scratch theme directory. This crate's
/// dependencies are deliberately limited to toml/serde/serde_json (no `tempfile`), so this
/// hand-rolls the same thing with a nanosecond-timestamp suffix for uniqueness.
pub fn make_temp_dir(label: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let dir = std::env::temp_dir().join(format!("winsome-theme-{label}-{nanos}"));
    fs::create_dir_all(&dir).expect("create temp theme dir");
    dir
}

pub fn cleanup_temp_dir(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}
