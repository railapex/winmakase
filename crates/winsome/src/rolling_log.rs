//! Size-rolling append log.
//!
//! DESIGN.md § Logging: "A failure that leaves no log line is a supervisor
//! bug." The M0 spike's worst bug was only diagnosable because a console
//! happened to be open; component output goes to a file from now on.
//!
//! `<name>.log` is live, `<name>.log.1` the newer archive, `<name>.log.2` the
//! older. `keep` counts every file including the live one.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::timefmt;

pub struct RollingLog {
    path: PathBuf,
    max_bytes: u64,
    keep: usize,
    file: File,
    size: u64,
}

impl RollingLog {
    pub fn open(path: impl Into<PathBuf>, max_bytes: u64, keep: usize) -> io::Result<Self> {
        let path = path.into();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        let size = file.metadata()?.len();
        Ok(Self {
            path,
            max_bytes: max_bytes.max(1),
            keep: keep.max(1),
            file,
            size,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Append one timestamped line, rolling first if this line would push the
    /// file past `max_bytes`.
    pub fn write_line(&mut self, line: &str) -> io::Result<()> {
        let stamped = format!("{} {}\n", timefmt::now_iso8601(), line);
        let len = stamped.len() as u64;
        if self.size > 0 && self.size + len > self.max_bytes {
            self.roll()?;
        }
        self.file.write_all(stamped.as_bytes())?;
        // Flushed per line: an unflushed buffer is exactly the log line that
        // goes missing when the thing being diagnosed takes the process down.
        self.file.flush()?;
        self.size += len;
        Ok(())
    }

    fn archive(&self, n: usize) -> PathBuf {
        let mut s = self.path.clone().into_os_string();
        s.push(format!(".{n}"));
        PathBuf::from(s)
    }

    fn roll(&mut self) -> io::Result<()> {
        let last = self.keep - 1;
        if last == 0 {
            // keep = 1: no archives, the live file starts over.
            self.file = File::create(&self.path)?;
            self.size = 0;
            return Ok(());
        }
        let _ = fs::remove_file(self.archive(last));
        for i in (1..last).rev() {
            let (from, to) = (self.archive(i), self.archive(i + 1));
            if from.exists() {
                fs::rename(&from, &to)?;
            }
        }
        // Renaming the live file works while our handle is open: std opens with
        // FILE_SHARE_DELETE. Reopening reseats the handle on the fresh path
        // before the next write, and drops the old one on assignment.
        fs::rename(&self.path, self.archive(1))?;
        self.file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        self.size = 0;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    fn line_count(p: &Path) -> usize {
        fs::read_to_string(p).unwrap().lines().count()
    }

    #[test]
    fn every_line_is_timestamped() {
        let dir = TempDir::new("log-stamp");
        let path = dir.path().join("supervisor.log");
        let mut log = RollingLog::open(&path, 1024, 3).unwrap();
        log.write_line("kanata started (pid 4321)").unwrap();
        let text = fs::read_to_string(&path).unwrap();
        let (stamp, rest) = text.trim_end().split_once(' ').unwrap();
        assert!(timefmt::parse_iso8601(stamp).is_some(), "got {stamp:?}");
        assert_eq!(rest, "kanata started (pid 4321)");
    }

    #[test]
    fn rolls_at_the_size_cap_and_keeps_three_files() {
        let dir = TempDir::new("log-roll");
        let path = dir.path().join("kanata.log");
        let first = dir.path().join("kanata.log.1");
        let second = dir.path().join("kanata.log.2");
        let mut log = RollingLog::open(&path, 512, 3).unwrap();
        for i in 0..200 {
            log.write_line(&format!("line {i:04} padding padding padding"))
                .unwrap();
        }
        assert!(path.exists() && first.exists() && second.exists());
        assert!(
            !dir.path().join("kanata.log.3").exists(),
            "keep = 3 must not leave a fourth file"
        );
        assert!(fs::metadata(&first).unwrap().len() <= 512 + 128);
        // Newest content is live, the oldest surviving lines are in .2.
        let live = fs::read_to_string(&path).unwrap();
        assert!(live.contains("line 0199"));
        assert!(!live.contains("line 0000"));
    }

    #[test]
    fn rolls_at_five_megabytes_with_the_shipped_default() {
        let dir = TempDir::new("log-5mb");
        let path = dir.path().join("kanata.log");
        let archive = dir.path().join("kanata.log.1");
        let cap = crate::config::SupervisorConfig::default().log_max_bytes;
        assert_eq!(cap, 5 * 1024 * 1024);
        let mut log = RollingLog::open(&path, cap, 3).unwrap();
        let payload = "x".repeat(4096);
        // ~6.3MB written: enough to cross the cap exactly once.
        for _ in 0..1536 {
            log.write_line(&payload).unwrap();
        }
        assert!(archive.exists(), "5MB of output must have rolled");
        assert!(fs::metadata(&archive).unwrap().len() <= cap);
        assert!(fs::metadata(&path).unwrap().len() < cap);
        assert_eq!(
            line_count(&path) + line_count(&archive),
            1536,
            "rolling must not drop lines"
        );
    }

    #[test]
    fn reopening_appends_rather_than_truncating() {
        let dir = TempDir::new("log-reopen");
        let path = dir.path().join("supervisor.log");
        RollingLog::open(&path, 1024, 3)
            .unwrap()
            .write_line("first run")
            .unwrap();
        RollingLog::open(&path, 1024, 3)
            .unwrap()
            .write_line("second run")
            .unwrap();
        assert_eq!(line_count(&path), 2);
    }

    #[test]
    fn keep_one_truncates_in_place() {
        let dir = TempDir::new("log-keep1");
        let path = dir.path().join("k.log");
        let mut log = RollingLog::open(&path, 128, 1).unwrap();
        for i in 0..50 {
            log.write_line(&format!("line {i}")).unwrap();
        }
        assert!(!dir.path().join("k.log.1").exists());
        assert!(fs::metadata(&path).unwrap().len() <= 128 + 64);
    }
}
