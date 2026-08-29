//! Where Winmakase keeps its things.
//!
//! Everything lives under `~/.winmakase`. `WINMAKASE_HOME` overrides the root —
//! that is what the test suite drives, and it is also the escape hatch for
//! running a second stack side by side without touching the live one.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    home: PathBuf,
}

impl Paths {
    /// `WINMAKASE_HOME`, else `%USERPROFILE%/.winmakase`, else `$HOME/.winmakase`.
    pub fn resolve() -> io::Result<Self> {
        if let Some(h) = env::var_os("WINMAKASE_HOME") {
            return Ok(Self::at(h));
        }
        let profile = env::var_os("USERPROFILE")
            .or_else(|| env::var_os("HOME"))
            .ok_or_else(|| {
                io::Error::other(
                    "cannot locate the home directory: neither WINMAKASE_HOME, USERPROFILE, nor HOME is set",
                )
            })?;
        Ok(Self::at(PathBuf::from(profile).join(".winmakase")))
    }

    pub fn at(home: impl Into<PathBuf>) -> Self {
        Self { home: home.into() }
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn config(&self) -> PathBuf {
        self.home.join("config.toml")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.home.join("logs")
    }

    pub fn state_dir(&self) -> PathBuf {
        self.home.join("state")
    }

    pub fn health(&self) -> PathBuf {
        self.state_dir().join("health.json")
    }

    /// Control file the supervisor polls; see [`crate::control`].
    pub fn control(&self) -> PathBuf {
        self.state_dir().join("control")
    }

    /// Where the supervisor writes a reload's outcome (first line: the
    /// request token, so a stale result cannot be mistaken for this one).
    pub fn reload_result(&self) -> PathBuf {
        self.state_dir().join("reload-result")
    }

    pub fn log_for(&self, name: &str) -> PathBuf {
        self.logs_dir().join(format!("{name}.log"))
    }

    pub fn ensure_dirs(&self) -> io::Result<()> {
        fs::create_dir_all(self.logs_dir())?;
        fs::create_dir_all(self.state_dir())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_hangs_off_the_root() {
        let p = Paths::at(r"C:\winmakase-home");
        assert_eq!(p.config(), Path::new(r"C:\winmakase-home\config.toml"));
        assert_eq!(
            p.health(),
            Path::new(r"C:\winmakase-home\state\health.json")
        );
        assert_eq!(p.control(), Path::new(r"C:\winmakase-home\state\control"));
        assert_eq!(
            p.log_for("kanata"),
            Path::new(r"C:\winmakase-home\logs\kanata.log")
        );
    }
}
