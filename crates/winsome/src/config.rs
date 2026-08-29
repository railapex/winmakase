//! `~/.winsome/config.toml` — every path and knob the supervisor uses.
//!
//! Nothing about the component set is compiled in. That is not decoration: the
//! supervisor has to be exercisable against stand-in processes, because
//! starting the real kanata/GlazeWM pair on a machine someone is using breaks
//! their desktop (M0 spike, cascade incident).

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Written verbatim when no config exists. Kept as text, not serialized from
/// [`Config::default`], so the comments survive — a config file that cannot
/// explain itself gets edited wrong. `default_matches_template` asserts the two
/// never drift apart.
pub const DEFAULT_CONFIG_TOML: &str = r#"# Winsome supervisor configuration.
# Paths use forward slashes; Windows accepts them and TOML does not eat them.

[supervisor]
# How often the supervisor checks children, timers, and the control file.
poll_ms = 250
# Restart backoff: 1s, 2s, 4s, 8s, 16s, then capped.
backoff_initial_ms = 1000
backoff_max_ms = 30000
# A component that stays up this long is considered recovered; backoff resets.
healthy_reset_secs = 60
# Grace period for a component to exit on its own before it gets killed.
stop_timeout_ms = 5000
# Rolling logs: roll at this size, keep this many files per component
# (the live .log plus its archives — 3 means kanata.log, kanata.log.1, kanata.log.2).
log_max_bytes = 5242880
log_keep_files = 3

# kanata and GlazeWM are a LINKED PAIR: both healthy or both down.
# GlazeWM dying with kanata alive leaves raw Win+letter chords firing OS
# shortcuts at a desktop nobody can tile. Start order is kanata first.

[kanata]
command = "D:/dev/winsome/spike/tools/kanata/kanata_windows_gui_winIOv2_x64.exe"
args = ["--cfg", "D:/dev/winsome/spike/caps.kbd"]

[glazewm]
command = "C:/Program Files/glzr.io/GlazeWM/glazewm.exe"
args = []
# Asking GlazeWM to exit through its own CLI lets glazewm-watcher restore every
# window to where it was. Killing the process skips that and leaves the desktop
# in tiled positions.
stop_command = "C:/Program Files/glzr.io/GlazeWM/cli/glazewm.exe"
stop_args = ["command", "wm-exit"]
"#;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub supervisor: SupervisorConfig,
    pub kanata: ComponentConfig,
    pub glazewm: ComponentConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// Optional graceful-stop command. Absent means "kill the process".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_command: Option<String>,
    #[serde(default)]
    pub stop_args: Vec<String>,
}

impl ComponentConfig {
    pub fn new(command: impl Into<String>, args: &[&str]) -> Self {
        Self {
            command: command.into(),
            args: args.iter().map(|s| (*s).to_string()).collect(),
            stop_command: None,
            stop_args: Vec::new(),
        }
    }

    pub fn with_stop(mut self, command: impl Into<String>, args: &[&str]) -> Self {
        self.stop_command = Some(command.into());
        self.stop_args = args.iter().map(|s| (*s).to_string()).collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupervisorConfig {
    #[serde(default = "d_poll_ms")]
    pub poll_ms: u64,
    #[serde(default = "d_backoff_initial_ms")]
    pub backoff_initial_ms: u64,
    #[serde(default = "d_backoff_max_ms")]
    pub backoff_max_ms: u64,
    #[serde(default = "d_healthy_reset_secs")]
    pub healthy_reset_secs: u64,
    #[serde(default = "d_stop_timeout_ms")]
    pub stop_timeout_ms: u64,
    #[serde(default = "d_log_max_bytes")]
    pub log_max_bytes: u64,
    #[serde(default = "d_log_keep_files")]
    pub log_keep_files: usize,
}

fn d_poll_ms() -> u64 {
    250
}
fn d_backoff_initial_ms() -> u64 {
    1_000
}
fn d_backoff_max_ms() -> u64 {
    30_000
}
fn d_healthy_reset_secs() -> u64 {
    60
}
fn d_stop_timeout_ms() -> u64 {
    5_000
}
fn d_log_max_bytes() -> u64 {
    5 * 1024 * 1024
}
fn d_log_keep_files() -> usize {
    3
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            poll_ms: d_poll_ms(),
            backoff_initial_ms: d_backoff_initial_ms(),
            backoff_max_ms: d_backoff_max_ms(),
            healthy_reset_secs: d_healthy_reset_secs(),
            stop_timeout_ms: d_stop_timeout_ms(),
            log_max_bytes: d_log_max_bytes(),
            log_keep_files: d_log_keep_files(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            supervisor: SupervisorConfig::default(),
            kanata: ComponentConfig::new(
                "D:/dev/winsome/spike/tools/kanata/kanata_windows_gui_winIOv2_x64.exe",
                &["--cfg", "D:/dev/winsome/spike/caps.kbd"],
            ),
            glazewm: ComponentConfig::new("C:/Program Files/glzr.io/GlazeWM/glazewm.exe", &[])
                .with_stop(
                    "C:/Program Files/glzr.io/GlazeWM/cli/glazewm.exe",
                    &["command", "wm-exit"],
                ),
        }
    }
}

impl Config {
    pub fn parse(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// Read the config, writing the commented default first if the file is
    /// absent. Parse failures are loud: a supervisor that silently fell back to
    /// defaults would point at the wrong binaries and say nothing.
    pub fn load_or_create(path: &Path) -> io::Result<Self> {
        if !path.exists() {
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir)?;
            }
            fs::write(path, DEFAULT_CONFIG_TOML)?;
        }
        let text = fs::read_to_string(path)?;
        Self::parse(&text).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {e}", path.display()),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_template() {
        assert_eq!(
            Config::parse(DEFAULT_CONFIG_TOML).unwrap(),
            Config::default()
        );
    }

    #[test]
    fn supervisor_section_is_optional() {
        let cfg = Config::parse(
            r#"
            [kanata]
            command = "k.exe"
            [glazewm]
            command = "g.exe"
            "#,
        )
        .unwrap();
        assert_eq!(cfg.supervisor, SupervisorConfig::default());
        assert!(cfg.kanata.args.is_empty());
        assert_eq!(cfg.glazewm.stop_command, None);
    }

    #[test]
    fn individual_knobs_are_optional() {
        let cfg = Config::parse(
            r#"
            [supervisor]
            poll_ms = 10
            [kanata]
            command = "k.exe"
            [glazewm]
            command = "g.exe"
            "#,
        )
        .unwrap();
        assert_eq!(cfg.supervisor.poll_ms, 10);
        assert_eq!(cfg.supervisor.backoff_max_ms, 30_000);
    }

    #[test]
    fn a_missing_component_is_an_error_not_a_default() {
        let err = Config::parse("[kanata]\ncommand = \"k.exe\"\n").unwrap_err();
        assert!(err.to_string().contains("glazewm"), "got: {err}");
    }

    #[test]
    fn creates_then_reads_back_the_default_file() {
        let dir = crate::testutil::TempDir::new("cfg-create");
        let path = dir.path().join("config.toml");
        let written = Config::load_or_create(&path).unwrap();
        assert_eq!(written, Config::default());
        assert!(fs::read_to_string(&path).unwrap().contains("LINKED PAIR"));
        assert_eq!(Config::load_or_create(&path).unwrap(), Config::default());
    }

    #[test]
    fn a_broken_config_fails_loud() {
        let dir = crate::testutil::TempDir::new("cfg-broken");
        let path = dir.path().join("config.toml");
        fs::write(&path, "[kanata]\ncommand = ").unwrap();
        let err = Config::load_or_create(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }
}
