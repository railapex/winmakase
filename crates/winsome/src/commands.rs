//! Rendering for the read-only verbs: `status` and `logs`.
//!
//! The formatting is separated from the I/O so it can be tested against a
//! fixture health file rather than against a live supervisor.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::health::{ComponentHealth, Health, Status};
use crate::paths::Paths;
use crate::timefmt;

/// How many rotated files back `logs` will reach for older lines.
const MAX_ARCHIVES: usize = 8;

pub fn render_status(health: &Health, path: &Path, supervisor_alive: Option<bool>) -> String {
    let mut out = String::new();
    let sup = &health.supervisor;
    let sup_state = match (sup.running, supervisor_alive) {
        (true, Some(false)) => "GONE",
        (true, _) => "running",
        (false, _) => "stopped",
    };
    let _ = writeln!(
        out,
        "{:<11} {:<11} {:<12} {}",
        "supervisor",
        sup_state,
        format!("pid {}", sup.pid),
        age_phrase(&sup.since, sup.running)
    );

    for (name, c) in &health.components {
        let _ = writeln!(out, "{}", render_component(name, c));
    }

    if sup.running && supervisor_alive == Some(false) {
        let _ = writeln!(
            out,
            "\nWARNING: this file says the supervisor is running, but pid {} is gone.\n\
             It died without shutting down — the components below may be orphaned.",
            sup.pid
        );
    }

    let _ = write!(out, "\n{}", updated_line(&health.updated, path));
    out
}

fn render_component(name: &str, c: &ComponentHealth) -> String {
    let pid = c
        .pid
        .map_or_else(|| "—".to_string(), |p| format!("pid {p}"));
    let mut line = format!(
        "{:<11} {:<11} {:<12} {:<14} restarts {}",
        name,
        c.status.as_str(),
        pid,
        age_phrase(&c.since, c.status == Status::Running),
        c.restarts
    );
    if let Some(code) = c.last_exit_code {
        let _ = write!(line, "   last exit {code}");
    }
    line.trim_end().to_string()
}

fn age_phrase(since: &str, running: bool) -> String {
    match timefmt::elapsed_since(since) {
        Some(d) if running => format!("up {}", timefmt::human_duration(d)),
        Some(d) => format!("for {}", timefmt::human_duration(d)),
        None => "since ?".to_string(),
    }
}

fn updated_line(updated: &str, path: &Path) -> String {
    let age = timefmt::elapsed_since(updated).map_or_else(
        || updated.to_string(),
        |d| format!("{} ago", timefmt::human_duration(d)),
    );
    // State is written on transitions, not on a timer, so an old stamp on a
    // healthy stack is the normal case — say that rather than implying decay.
    format!("state last changed {age}  ({})", path.display())
}

pub fn no_health_state(path: &Path) -> String {
    format!(
        "No health state at {}.\n\
         The supervisor has not run here — start it with `winsome supervise`, or point\n\
         WINSOME_HOME at the stack you meant.",
        path.display()
    )
}

/// Last `lines` lines of a component's log, reaching back through rotated files
/// when the live one is short. Returns the text and the files it drew from.
pub fn tail_log(paths: &Paths, name: &str, lines: usize) -> io::Result<(String, Vec<PathBuf>)> {
    let base = paths.log_for(name);
    if !base.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{}", base.display()),
        ));
    }

    let mut collected: VecDeque<String> = VecDeque::new();
    let mut used = Vec::new();
    // Newest first: the live file, then .1, .2, ...
    for candidate in
        std::iter::once(base.clone()).chain((1..=MAX_ARCHIVES).map(|i| archive(&base, i)))
    {
        if collected.len() >= lines {
            break;
        }
        if !candidate.exists() {
            continue;
        }
        let text = String::from_utf8_lossy(&fs::read(&candidate)?).into_owned();
        let before = collected.len();
        for line in text.lines().rev() {
            if collected.len() >= lines {
                break;
            }
            collected.push_front(line.to_string());
        }
        if collected.len() > before {
            used.push(candidate);
        }
    }

    Ok((collected.into_iter().collect::<Vec<_>>().join("\n"), used))
}

fn archive(base: &Path, n: usize) -> PathBuf {
    let mut s = base.to_path_buf().into_os_string();
    s.push(format!(".{n}"));
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::{Status, SupervisorHealth};
    use crate::rolling_log::RollingLog;
    use crate::testutil::TempDir;
    use std::collections::BTreeMap;

    fn health(running: bool) -> Health {
        let mut components = BTreeMap::new();
        components.insert(
            "glazewm".to_string(),
            ComponentHealth {
                status: Status::Restarting,
                pid: None,
                since: timefmt::now_iso8601(),
                restarts: 2,
                last_exit_code: Some(1),
            },
        );
        components.insert(
            "kanata".to_string(),
            ComponentHealth {
                status: Status::Running,
                pid: Some(4321),
                since: timefmt::now_iso8601(),
                restarts: 0,
                last_exit_code: None,
            },
        );
        Health::new(
            SupervisorHealth {
                running,
                pid: 12345,
                since: timefmt::now_iso8601(),
            },
            components,
        )
    }

    #[test]
    fn status_reads_as_a_table() {
        let text = render_status(&health(true), Path::new("h.json"), Some(true));
        assert!(text.contains("supervisor  running     pid 12345"));
        assert!(text.contains("kanata      running     pid 4321"));
        assert!(text.contains("glazewm     restarting  —"));
        assert!(text.contains("restarts 2"));
        assert!(text.contains("last exit 1"));
        assert!(!text.contains("WARNING"));
    }

    #[test]
    fn a_dead_pid_behind_a_running_file_is_called_out() {
        let text = render_status(&health(true), Path::new("h.json"), Some(false));
        assert!(text.contains("GONE"));
        assert!(text.contains("pid 12345 is gone"));
    }

    #[test]
    fn a_clean_shutdown_is_not_a_warning() {
        let text = render_status(&health(false), Path::new("h.json"), Some(false));
        assert!(text.contains("supervisor  stopped"));
        assert!(!text.contains("WARNING"));
    }

    #[test]
    fn tail_reaches_into_rotated_files() {
        let dir = TempDir::new("tail");
        let paths = Paths::at(dir.path());
        let mut log = RollingLog::open(paths.log_for("kanata"), 400, 3).unwrap();
        for i in 0..100 {
            log.write_line(&format!("line {i:03}")).unwrap();
        }
        // 400-byte files hold ~13 lines each, so 25 lines cannot come from the
        // live file alone.
        let (text, used) = tail_log(&paths, "kanata", 25).unwrap();
        let got: Vec<&str> = text.lines().collect();
        assert_eq!(got.len(), 25);
        assert!(got.last().unwrap().ends_with("line 099"));
        assert!(got.first().unwrap().ends_with("line 075"));
        assert!(used.len() > 1, "25 lines must have spanned a rotation");
    }

    #[test]
    fn tail_of_a_short_log_returns_what_there_is() {
        let dir = TempDir::new("tail-short");
        let paths = Paths::at(dir.path());
        let mut log = RollingLog::open(paths.log_for("supervisor"), 4096, 3).unwrap();
        log.write_line("only line").unwrap();
        let (text, _) = tail_log(&paths, "supervisor", 50).unwrap();
        assert_eq!(text.lines().count(), 1);
    }

    #[test]
    fn tail_of_a_missing_log_is_not_found() {
        let dir = TempDir::new("tail-missing");
        let paths = Paths::at(dir.path());
        let err = tail_log(&paths, "glazewm", 10).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
