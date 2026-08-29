//! `~/.winmakase/config.toml` — every path and knob the supervisor uses.
//!
//! Nothing about the component set is compiled in. That is not decoration: the
//! supervisor has to be exercisable against stand-in processes, because
//! starting the real kanata/GlazeWM pair on a machine someone is using breaks
//! their desktop (M0 spike, cascade incident).

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::scratchpad::ScratchpadConfig;

/// Written verbatim when no config exists. Kept as text, not serialized from
/// [`Config::default`], so the comments survive — a config file that cannot
/// explain itself gets edited wrong. `default_matches_template` asserts the two
/// never drift apart.
pub const DEFAULT_CONFIG_TOML: &str = r#"# Winmakase supervisor configuration.
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
# Restart the pair when the monitor set changes (glazewm#1233 insurance).
# Off by default: the spike could not reproduce #1233 on 3.10.1, and a bounce
# on every routine teleprompter toggle would be self-inflicted churn. Display
# changes are logged either way.
bounce_on_display_change = false
# Auto-hide the taskbar while the stack runs (Zebar replaces it); whatever
# state the taskbar had is put back on shutdown. Panic restores it too.
hide_taskbar = true

# The rendered kanata config (`winmakase kanata render`): which physical key
# carries the WM chord.
#   caps — Caps Lock = right-Win modifier; scrlk = raw-CapsLock escape hatch.
#   apps — menu key: tap = context menu, hold = WM chord; Caps stays native.
# tap_ms/hold_ms: the apps-mode tap-hold decision window (caps mode has no tap).
[keyboard]
mode = "caps"
tap_ms = 200
hold_ms = 200

# kanata and GlazeWM are a LINKED PAIR: both healthy or both down.
# GlazeWM dying with kanata alive leaves raw Win+letter chords firing OS
# shortcuts at a desktop nobody can tile. Start order is kanata first.
#
# `adopt = true`: a component whose exact executable is already running is
# adopted (watched by pid) instead of started again — the supervisor can take
# over a live desktop without bouncing it. An adopted process keeps whatever
# stdio it had; its output is only captured from its next restart on.

[kanata]
# kanata must run ELEVATED (UIPI: a user-level hook goes deaf while an admin
# window has focus), and this supervisor runs user-level, so kanata is hosted
# by the pre-registered elevated `WinmakaseKanata` scheduled task: `task` makes
# start = `schtasks /run` and stop = `schtasks /end` (both UAC-free for the
# task's owner). `command` is not executed — it names the exact image to watch.
command = "D:/dev/winmakase/spike/tools/kanata/kanata_windows_gui_winIOv2_cmd_allowed_x64.exe"
task = "WinmakaseKanata"
adopt = true

[glazewm]
command = "C:/Program Files/glzr.io/GlazeWM/glazewm.exe"
args = []
adopt = true
# Asking GlazeWM to exit through its own CLI lets glazewm-watcher restore every
# window to where it was. Killing the process skips that and leaves the desktop
# in tiled positions.
stop_command = "C:/Program Files/glzr.io/GlazeWM/cli/glazewm.exe"
stop_args = ["command", "wm-exit"]

# The bar. Optional — delete this section to run without one. Not part of the
# linked pair: zebar dying restarts zebar alone. Its widgets/startup config
# lives in ~/.glzr/zebar, not here.
[zebar]
command = "C:/Program Files/glzr.io/Zebar/zebar.exe"
args = []
adopt = true

# Named scratchpads for `winmakase scratchpad toggle <name>`: the matching
# window is banished to the hidden 'scratch' workspace and summoned back
# floated + centered + focused. The GlazeWM config must define a workspace
# named "scratch" (unkeyed, last in definition order).
#
# `title` is the second match rule: process alone is too coarse when the
# pad's process is a daily driver — a bare `wt` rule grabs whichever
# Windows Terminal it finds first. Launch with a distinctive marker title
# and match it here (case-insensitive substring).
[scratchpad.term]
launch = "wt"
launch_args = ["-w", "-1", "new-tab", "--title", "winmakase-pad"]
process = "WindowsTerminal"
title = "winmakase-pad"
width = "55%"
height = "60%"
"#;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub supervisor: SupervisorConfig,
    /// The rendered kanata config's content — distinct from `[kanata]`, which
    /// is how the component is *hosted*.
    #[serde(default)]
    pub keyboard: KeyboardConfig,
    pub kanata: ComponentConfig,
    pub glazewm: ComponentConfig,
    /// Optional: a missing `[zebar]` section means no bar in the set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zebar: Option<ComponentConfig>,
    /// Named scratchpads for `winmakase scratchpad toggle <name>`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub scratchpad: crate::scratchpad::ScratchpadMap,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentConfig {
    /// The component's executable. Spawned directly — unless `task` is set, in
    /// which case this is only the exact image path to find and watch.
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    /// Host through this pre-registered scheduled task instead of spawning:
    /// start = `schtasks /run`, stop = `schtasks /end`. The elevation door for
    /// kanata — a user-level supervisor can neither spawn nor kill an elevated
    /// process, but a task's owner can run and end its task UAC-free.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    /// Adopt a running instance of `command` (matched on the full image path)
    /// instead of starting a second one. What lets the supervisor slide under
    /// a live desktop without bouncing it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub adopt: bool,
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
            task: None,
            adopt: false,
            stop_command: None,
            stop_args: Vec::new(),
        }
    }

    pub fn with_stop(mut self, command: impl Into<String>, args: &[&str]) -> Self {
        self.stop_command = Some(command.into());
        self.stop_args = args.iter().map(|s| (*s).to_string()).collect();
        self
    }

    pub fn hosted_by_task(mut self, task: impl Into<String>) -> Self {
        self.task = Some(task.into());
        self
    }

    pub fn adopting(mut self) -> Self {
        self.adopt = true;
        self
    }
}

/// `[keyboard]` — what `winmakase kanata render` produces. Which physical key
/// carries the WM chord, and the apps-mode tap-hold window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyboardConfig {
    #[serde(default)]
    pub mode: KeyboardMode,
    #[serde(default = "d_tap_hold_ms")]
    pub tap_ms: u16,
    #[serde(default = "d_tap_hold_ms")]
    pub hold_ms: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyboardMode {
    /// Caps Lock = right-Win modifier (v4.1, live-proven); scrlk = raw hatch.
    #[default]
    Caps,
    /// Menu key: tap = context menu, hold = WM chord; Caps stays native.
    Apps,
}

fn d_tap_hold_ms() -> u16 {
    200
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            mode: KeyboardMode::Caps,
            tap_ms: d_tap_hold_ms(),
            hold_ms: d_tap_hold_ms(),
        }
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
    /// Restart the pair on a monitor-set change (glazewm#1233 insurance).
    /// Default off: the spike could not reproduce the bug, and the occasional
    /// display here is toggled routinely. Events are logged regardless.
    #[serde(default)]
    pub bounce_on_display_change: bool,
    /// Auto-hide the taskbar while the stack runs; restore its prior state on
    /// shutdown. On by default — a hidden taskbar is the product's end state,
    /// and before the supervisor owned this, a reboot brought the taskbar
    /// back even though the logon task brought the stack up.
    #[serde(default = "d_hide_taskbar")]
    pub hide_taskbar: bool,
}

fn d_hide_taskbar() -> bool {
    true
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

impl SupervisorConfig {
    /// Clamp values that would break the supervisor rather than merely tune it,
    /// returning what changed so the caller can log it.
    ///
    /// A hand-edited zero is the case that matters: a zero backoff turns
    /// restart-on-crash into a busy loop respawning a keyboard remapper as fast
    /// as the OS allows. Clamping and saying so beats refusing to start the
    /// desktop over a typo.
    pub fn sanitize(&mut self) -> Vec<String> {
        let mut notes = Vec::new();
        let mut clamp = |name: &str, value: &mut u64, floor: u64| {
            if *value < floor {
                notes.push(format!(
                    "supervisor.{name} = {value} is below the minimum {floor} — using {floor}"
                ));
                *value = floor;
            }
        };
        // Floors, not just non-zero. A 1ms backoff is still ~13 respawns a
        // second of a keyboard remapper — thrash, not recovery — and a 1ms poll
        // is a spin loop. Below these the value is indistinguishable from the
        // zero it was clamped from.
        clamp("poll_ms", &mut self.poll_ms, 10);
        clamp("backoff_initial_ms", &mut self.backoff_initial_ms, 100);
        clamp("log_max_bytes", &mut self.log_max_bytes, 4096);
        // Zero here resets the backoff on every tick, which is backoff in name
        // only — a crash loop would restart forever at the initial delay.
        clamp("healthy_reset_secs", &mut self.healthy_reset_secs, 1);

        if self.backoff_max_ms < self.backoff_initial_ms {
            notes.push(format!(
                "supervisor.backoff_max_ms = {} is below backoff_initial_ms = {} — raising it to match",
                self.backoff_max_ms, self.backoff_initial_ms
            ));
            self.backoff_max_ms = self.backoff_initial_ms;
        }
        if self.log_keep_files == 0 {
            notes.push(
                "supervisor.log_keep_files = 0 would discard every log — using 1".to_string(),
            );
            self.log_keep_files = 1;
        }
        notes
    }
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
            bounce_on_display_change: false,
            hide_taskbar: d_hide_taskbar(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            supervisor: SupervisorConfig::default(),
            keyboard: KeyboardConfig::default(),
            kanata: ComponentConfig::new(
                "D:/dev/winmakase/spike/tools/kanata/kanata_windows_gui_winIOv2_cmd_allowed_x64.exe",
                &[],
            )
            .hosted_by_task("WinmakaseKanata")
            .adopting(),
            glazewm: ComponentConfig::new("C:/Program Files/glzr.io/GlazeWM/glazewm.exe", &[])
                .adopting()
                .with_stop(
                    "C:/Program Files/glzr.io/GlazeWM/cli/glazewm.exe",
                    &["command", "wm-exit"],
                ),
            zebar: Some(
                ComponentConfig::new("C:/Program Files/glzr.io/Zebar/zebar.exe", &[]).adopting(),
            ),
            scratchpad: BTreeMap::from([(
                "term".to_string(),
                ScratchpadConfig {
                    launch: "wt".to_string(),
                    launch_args: ["-w", "-1", "new-tab", "--title", "winmakase-pad"]
                        .map(String::from)
                        .to_vec(),
                    process: "WindowsTerminal".to_string(),
                    title: Some("winmakase-pad".to_string()),
                    width: Some("55%".to_string()),
                    height: Some("60%".to_string()),
                },
            )]),
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
        assert_eq!(cfg.kanata.task, None);
        assert!(!cfg.kanata.adopt, "adoption is opt-in");
        assert_eq!(cfg.glazewm.stop_command, None);
        assert_eq!(cfg.zebar, None, "the bar is optional");
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
    fn keyboard_section_is_optional_and_modes_parse() {
        let cfg = Config::parse(
            r#"
            [kanata]
            command = "k.exe"
            [glazewm]
            command = "g.exe"
            "#,
        )
        .unwrap();
        assert_eq!(cfg.keyboard, KeyboardConfig::default());
        assert_eq!(cfg.keyboard.mode, KeyboardMode::Caps);

        let cfg = Config::parse(
            r#"
            [keyboard]
            mode = "apps"
            tap_ms = 150
            [kanata]
            command = "k.exe"
            [glazewm]
            command = "g.exe"
            "#,
        )
        .unwrap();
        assert_eq!(cfg.keyboard.mode, KeyboardMode::Apps);
        assert_eq!(cfg.keyboard.tap_ms, 150);
        assert_eq!(cfg.keyboard.hold_ms, 200, "hold keeps its default");

        // An unknown mode is a loud parse error, not a silent caps fallback.
        let err = Config::parse(
            r#"
            [keyboard]
            mode = "dvorak"
            [kanata]
            command = "k.exe"
            [glazewm]
            command = "g.exe"
            "#,
        )
        .unwrap_err();
        assert!(err.to_string().contains("dvorak"), "got: {err}");
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
    fn a_zeroed_config_is_clamped_and_reported() {
        let mut s = SupervisorConfig {
            poll_ms: 0,
            backoff_initial_ms: 0,
            backoff_max_ms: 0,
            healthy_reset_secs: 0,
            stop_timeout_ms: 0,
            log_max_bytes: 0,
            log_keep_files: 0,
            bounce_on_display_change: false,
            hide_taskbar: false,
        };
        let notes = s.sanitize();
        // Not merely non-zero: fast enough to be a spin is still broken.
        assert!(s.poll_ms >= 10);
        assert!(s.backoff_initial_ms >= 100);
        assert!(s.backoff_max_ms >= s.backoff_initial_ms);
        assert!(s.log_max_bytes >= 4096);
        assert_eq!(s.log_keep_files, 1);
        // Every clamp is named, so the log says what was ignored and why.
        assert!(s.healthy_reset_secs >= 1);
        assert_eq!(notes.len(), 6, "{notes:#?}");
        assert!(notes.iter().any(|n| n.contains("backoff_initial_ms")));
        assert!(notes.iter().any(|n| n.contains("log_keep_files")));
        // Sanitizing twice is a no-op.
        assert!(s.sanitize().is_empty());
    }

    #[test]
    fn a_sane_config_is_left_alone() {
        let mut s = SupervisorConfig::default();
        assert!(s.sanitize().is_empty());
        assert_eq!(s, SupervisorConfig::default());
    }

    #[test]
    fn a_cap_below_the_initial_delay_is_raised() {
        let mut s = SupervisorConfig {
            backoff_initial_ms: 2_000,
            backoff_max_ms: 500,
            ..SupervisorConfig::default()
        };
        let notes = s.sanitize();
        assert_eq!(s.backoff_max_ms, 2_000);
        assert_eq!(notes.len(), 1);
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
