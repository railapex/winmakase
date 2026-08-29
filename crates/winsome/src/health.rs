//! `~/.winsome/state/health.json` — what the supervisor believes right now.
//!
//! Rewritten on every state change, never on a timer, so the `updated` stamp is
//! evidence of the last transition rather than proof of life. Zebar's health dot
//! reads this file; `schema` exists so a bar built against v1 can say "I don't
//! know this shape" instead of rendering something wrong.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::timefmt;

pub const SCHEMA: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    /// Child process is alive.
    Running,
    /// Exited; a restart is scheduled (backoff is counting down).
    Restarting,
    /// Not running and nothing scheduled — shut down, or stopped on purpose.
    Stopped,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Running => "running",
            Status::Restarting => "restarting",
            Status::Stopped => "stopped",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: Status,
    pub pid: Option<u32>,
    /// When the component entered this status.
    pub since: String,
    /// Restarts since the supervisor started. The first start is not a restart.
    pub restarts: u32,
    pub last_exit_code: Option<i32>,
}

impl ComponentHealth {
    pub fn stopped_now() -> Self {
        Self {
            status: Status::Stopped,
            pid: None,
            since: timefmt::now_iso8601(),
            restarts: 0,
            last_exit_code: None,
        }
    }

    pub fn set(&mut self, status: Status, pid: Option<u32>) {
        self.status = status;
        self.pid = pid;
        self.since = timefmt::now_iso8601();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisorHealth {
    pub running: bool,
    pub pid: u32,
    pub since: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    pub schema: u32,
    pub updated: String,
    pub supervisor: SupervisorHealth,
    /// Keyed by component name, ordered so diffs of the file stay readable.
    pub components: BTreeMap<String, ComponentHealth>,
}

impl Health {
    pub fn new(
        supervisor: SupervisorHealth,
        components: BTreeMap<String, ComponentHealth>,
    ) -> Self {
        Self {
            schema: SCHEMA,
            updated: timefmt::now_iso8601(),
            supervisor,
            components,
        }
    }

    pub fn read(path: &Path) -> io::Result<Self> {
        let text = fs::read_to_string(path)?;
        serde_json::from_str(&text).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {e}", path.display()),
            )
        })
    }

    /// Write via temp + rename. Zebar polls this file; a torn read would show a
    /// health dot for a state that never existed.
    pub fn write(&self, path: &Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        fs::rename(&tmp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::TempDir;

    fn sample() -> Health {
        let mut components = BTreeMap::new();
        components.insert(
            "kanata".to_string(),
            ComponentHealth {
                status: Status::Running,
                pid: Some(4321),
                since: "2026-08-28T16:40:00Z".to_string(),
                restarts: 0,
                last_exit_code: None,
            },
        );
        components.insert(
            "glazewm".to_string(),
            ComponentHealth {
                status: Status::Restarting,
                pid: None,
                since: "2026-08-28T16:41:00Z".to_string(),
                restarts: 2,
                last_exit_code: Some(1),
            },
        );
        Health::new(
            SupervisorHealth {
                running: true,
                pid: 999,
                since: "2026-08-28T16:39:00Z".to_string(),
            },
            components,
        )
    }

    #[test]
    fn shape_is_what_zebar_will_read() {
        let json = serde_json::to_value(sample()).unwrap();
        assert_eq!(json["schema"], 1);
        assert_eq!(json["supervisor"]["running"], true);
        assert_eq!(json["components"]["kanata"]["status"], "running");
        assert_eq!(json["components"]["kanata"]["pid"], 4321);
        assert_eq!(json["components"]["glazewm"]["status"], "restarting");
        assert_eq!(
            json["components"]["glazewm"]["pid"],
            serde_json::Value::Null
        );
        assert_eq!(json["components"]["glazewm"]["restarts"], 2);
        assert_eq!(json["components"]["glazewm"]["last_exit_code"], 1);
        assert!(timefmt::parse_iso8601(json["updated"].as_str().unwrap()).is_some());
    }

    #[test]
    fn round_trips_through_the_file() {
        let dir = TempDir::new("health-rt");
        let path = dir.path().join("state").join("health.json");
        let h = sample();
        h.write(&path).unwrap();
        assert_eq!(Health::read(&path).unwrap(), h);
        assert!(
            !path.with_extension("json.tmp").exists(),
            "the temp file must be renamed away, not left behind"
        );
    }

    #[test]
    fn a_corrupt_file_fails_loud() {
        let dir = TempDir::new("health-corrupt");
        let path = dir.path().join("health.json");
        fs::write(&path, "{ not json").unwrap();
        assert_eq!(
            Health::read(&path).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn set_stamps_the_transition() {
        let mut c = ComponentHealth::stopped_now();
        c.since = "2020-01-01T00:00:00Z".to_string();
        c.set(Status::Running, Some(17));
        assert_eq!(c.status, Status::Running);
        assert_eq!(c.pid, Some(17));
        assert!(timefmt::elapsed_since(&c.since).unwrap().as_secs() < 5);
    }
}
