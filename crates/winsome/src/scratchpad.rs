//! `winsome scratchpad` — omarchy mod+S (dropdown-terminal pattern), built on
//! the spike-proven primitives (spike/summon.ps1, source-verified
//! 2026-08-28): windows park on the hidden `scratch` workspace and come back
//! floated, centered, shown on top, focused.
//!
//! Two verbs share the machinery: `toggle <name>` is the config-driven named
//! pad (match by process; summon, banish, or launch); `summon` is the
//! anonymous rwin+s key — rescue the focused window if it is not presentable
//! (DWM-cloaked, Win32-invisible, or parked on a non-displayed workspace),
//! else pull the most recent window out of scratch, else do nothing quietly.
//!
//! The GlazeWM config must define a workspace named `scratch` (unkeyed, last
//! in definition order so monitor fallback never grabs it) — the rig's does.

use std::collections::BTreeMap;
use std::io;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::glazewm::{Client, Node, Workspace, windows_under};

/// The hidden parking workspace's name, by convention with the GlazeWM config.
pub const SCRATCH_WORKSPACE: &str = "scratch";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScratchpadConfig {
    /// Command to launch when no matching window exists anywhere.
    pub launch: String,
    #[serde(default)]
    pub launch_args: Vec<String>,
    /// Window match: the process name, compared case-insensitively.
    pub process: String,
    /// Floating size for the summoned window, e.g. "55%" — passed to
    /// `set-floating --width/--height`. Omitted = GlazeWM's default placement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
}

pub type ScratchpadMap = BTreeMap<String, ScratchpadConfig>;

/// What a toggle should do, decided purely from the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// A match is parked in scratch — bring the most recently focused one out.
    Summon { window_id: String },
    /// A match is out in the world — park it (untargeted by focus, invisible).
    Banish { window_id: String },
    /// No match anywhere — start it.
    Launch,
}

pub fn decide(workspaces: &[Workspace], process: &str) -> Action {
    let matches_process = |w: &crate::glazewm::Node| {
        w.process_name
            .as_deref()
            .is_some_and(|p| p.eq_ignore_ascii_case(process))
    };

    if let Some(scratch) = workspaces.iter().find(|w| w.name == SCRATCH_WORKSPACE) {
        let parked = windows_under(&scratch.children);
        let parked_matching: Vec<&str> = parked
            .iter()
            .filter(|w| matches_process(w))
            .map(|w| w.id.as_str())
            .collect();
        // Most recently focused first, the way summon.ps1 picks.
        if let Some(id) = scratch
            .child_focus_order
            .iter()
            .find(|id| parked_matching.contains(&id.as_str()))
        {
            return Action::Summon {
                window_id: id.clone(),
            };
        }
        if let Some(id) = parked_matching.first() {
            return Action::Summon {
                window_id: (*id).to_string(),
            };
        }
    }

    for ws in workspaces.iter().filter(|w| w.name != SCRATCH_WORKSPACE) {
        if let Some(w) = windows_under(&ws.children)
            .into_iter()
            .find(|w| matches_process(w))
        {
            return Action::Banish {
                window_id: w.id.clone(),
            };
        }
    }

    Action::Launch
}

/// Execute one toggle.
pub fn toggle(client: &Client, cfg: &ScratchpadConfig) -> io::Result<String> {
    let workspaces = client.query_workspaces()?;
    match decide(&workspaces, &cfg.process) {
        Action::Summon { window_id } => {
            summon(client, &workspaces, &window_id, cfg)?;
            Ok(format!("summoned {} from scratch", cfg.process))
        }
        Action::Banish { window_id } => {
            client.command_for(&window_id, &["move", "--workspace", SCRATCH_WORKSPACE])?;
            Ok(format!("banished {} to scratch", cfg.process))
        }
        Action::Launch => {
            launch(cfg)?;
            // Float it once it materializes, so the first toggle looks like
            // every later summon rather than a tiled surprise.
            let deadline = Instant::now() + Duration::from_secs(5);
            loop {
                std::thread::sleep(Duration::from_millis(150));
                let workspaces = client.query_workspaces()?;
                if let Action::Banish { window_id } = decide(&workspaces, &cfg.process) {
                    float_and_focus(client, &window_id, cfg)?;
                    return Ok(format!("launched {}", cfg.launch));
                }
                if Instant::now() >= deadline {
                    return Ok(format!(
                        "launched {} (window not seen within 5s — leaving it as it landed)",
                        cfg.launch
                    ));
                }
            }
        }
    }
}

fn summon(
    client: &Client,
    workspaces: &[Workspace],
    window_id: &str,
    cfg: &ScratchpadConfig,
) -> io::Result<()> {
    // The focused workspace is where the summon lands; fall back to any
    // displayed one (summon.ps1's exact fallback).
    let here = workspaces
        .iter()
        .find(|w| w.has_focus)
        .or_else(|| workspaces.iter().find(|w| w.is_displayed))
        .map(|w| w.name.clone())
        .ok_or_else(|| io::Error::other("no displayed workspace to summon onto"))?;
    client.command_for(window_id, &["move", "--workspace", &here])?;
    float_and_focus(client, window_id, cfg)
}

fn float_and_focus(client: &Client, window_id: &str, cfg: &ScratchpadConfig) -> io::Result<()> {
    let mut float_args: Vec<&str> = vec!["set-floating", "--centered", "--shown-on-top"];
    if let Some(w) = cfg.width.as_deref() {
        float_args.extend_from_slice(&["--width", w]);
    }
    if let Some(h) = cfg.height.as_deref() {
        float_args.extend_from_slice(&["--height", h]);
    }
    client.command_for(window_id, &float_args)?;
    client.command(&["focus", "--container-id", window_id])
}

fn launch(cfg: &ScratchpadConfig) -> io::Result<()> {
    use std::os::windows::process::CommandExt;
    // Detached, with its own console: a console-subsystem launch target must
    // not inherit whatever console this CLI ran from (live finding — a `cmd`
    // pad wrote its prompt into the invoking shell and never made a window).
    // GUI-subsystem targets ignore the console flag.
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
    std::process::Command::new(&cfg.launch)
        .args(&cfg.launch_args)
        .creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NEW_CONSOLE)
        .spawn()?;
    Ok(())
}

// -- the anonymous summon key (rwin+s) ---------------------------------------

/// What the focused window's Win32 probes said. Separated from the decision
/// so the decision stays pure and testable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FocusProbe {
    pub cloaked: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SummonAction {
    /// The focused window is not presentable — materialize it.
    Rescue {
        window_id: String,
        /// A displayed workspace on the same monitor to pull it onto, when it
        /// sits on a non-displayed workspace.
        pull_to: Option<String>,
    },
    /// Pocket recall: the most recently focused window parked in scratch.
    Summon {
        window_id: String,
        to: String,
    },
    Nothing,
}

pub fn decide_summon(
    workspaces: &[Workspace],
    focused: Option<&Node>,
    probe: Option<FocusProbe>,
) -> SummonAction {
    // Rescue pass, exactly summon.ps1's: Alt+Tab to a ghost makes it the
    // focused window; this key then materializes it.
    if let Some(f) = focused
        && f.is_window()
        && f.state_type() != Some("minimized")
    {
        let own_ws = workspaces
            .iter()
            .find(|w| windows_under(&w.children).iter().any(|c| c.id == f.id));
        let parked_off_screen = own_ws.is_some_and(|w| !w.is_displayed);
        let ghostly = probe.is_some_and(|p| p.cloaked || !p.visible);
        if ghostly || parked_off_screen {
            let pull_to = if parked_off_screen {
                let own = own_ws.expect("parked_off_screen implies own_ws");
                workspaces
                    .iter()
                    .find(|w| w.parent_id == own.parent_id && w.is_displayed)
                    .map(|w| w.name.clone())
            } else {
                None
            };
            return SummonAction::Rescue {
                window_id: f.id.clone(),
                pull_to,
            };
        }
    }

    let Some(scratch) = workspaces.iter().find(|w| w.name == SCRATCH_WORKSPACE) else {
        return SummonAction::Nothing;
    };
    let parked = windows_under(&scratch.children);
    if parked.is_empty() {
        return SummonAction::Nothing;
    }
    let parked_ids: Vec<&str> = parked.iter().map(|w| w.id.as_str()).collect();
    let target = scratch
        .child_focus_order
        .iter()
        .find(|id| parked_ids.contains(&id.as_str()))
        .map(String::as_str)
        .unwrap_or(parked_ids[0]);

    let Some(to) = workspaces
        .iter()
        .find(|w| w.has_focus)
        .or_else(|| workspaces.iter().find(|w| w.is_displayed))
        .map(|w| w.name.clone())
    else {
        return SummonAction::Nothing;
    };
    SummonAction::Summon {
        window_id: target.to_string(),
        to,
    }
}

/// The rwin+s behavior, end to end.
pub fn summon_key(client: &Client) -> io::Result<String> {
    let workspaces = client.query_workspaces()?;
    let focused = client.query_focused()?;
    let probe = focused.as_ref().and_then(|f| f.handle).map(probe_window);

    match decide_summon(&workspaces, focused.as_ref(), probe) {
        SummonAction::Rescue { window_id, pull_to } => {
            if let Some(ws) = pull_to {
                client.command_for(&window_id, &["move", "--workspace", &ws])?;
            }
            client.command_for(
                &window_id,
                &["set-floating", "--centered", "--shown-on-top"],
            )?;
            client.command(&["focus", "--container-id", &window_id])?;
            Ok("rescued the focused window".to_string())
        }
        SummonAction::Summon { window_id, to } => {
            client.command_for(&window_id, &["move", "--workspace", &to])?;
            client.command_for(
                &window_id,
                &["set-floating", "--centered", "--shown-on-top"],
            )?;
            client.command(&["focus", "--container-id", &window_id])?;
            Ok("summoned from scratch".to_string())
        }
        SummonAction::Nothing => Ok("nothing to summon".to_string()),
    }
}

/// Win32 probes the tree cannot answer: is the window DWM-cloaked (the
/// ApplicationFrameHost ghost) or Win32-invisible?
fn probe_window(handle: i64) -> FocusProbe {
    use windows_sys::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
    use windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible;

    let hwnd = handle as isize as windows_sys::Win32::Foundation::HWND;
    let mut cloaked: u32 = 0;
    unsafe {
        // A failed attribute read leaves cloaked = 0: unknown is not ghostly.
        let _ = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED as u32,
            (&raw mut cloaked).cast(),
            std::mem::size_of::<u32>() as u32,
        );
        FocusProbe {
            cloaked: cloaked != 0,
            visible: IsWindowVisible(hwnd) != 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspaces(json: &str) -> Vec<Workspace> {
        serde_json::from_str(json).unwrap()
    }

    const WITH_SCRATCH: &str = r#"[
        {"type":"workspace","id":"ws1","name":"1","isDisplayed":true,"hasFocus":true,
         "children":[{"type":"window","id":"c1","state":{"type":"tiling"},"processName":"chrome"}]},
        {"type":"workspace","id":"wss","name":"scratch",
         "childFocusOrder":["t2","t1"],
         "children":[
            {"type":"window","id":"t1","state":{"type":"tiling"},"processName":"WindowsTerminal"},
            {"type":"window","id":"t2","state":{"type":"tiling"},"processName":"WindowsTerminal"}
         ]}
    ]"#;

    #[test]
    fn a_parked_match_is_summoned_most_recent_first() {
        assert_eq!(
            decide(&workspaces(WITH_SCRATCH), "windowsterminal"),
            Action::Summon {
                window_id: "t2".into()
            },
            "childFocusOrder decides which one comes out"
        );
    }

    #[test]
    fn a_match_out_in_the_world_is_banished() {
        assert_eq!(
            decide(&workspaces(WITH_SCRATCH), "chrome"),
            Action::Banish {
                window_id: "c1".into()
            }
        );
    }

    #[test]
    fn no_match_anywhere_launches() {
        assert_eq!(decide(&workspaces(WITH_SCRATCH), "wezterm"), Action::Launch);
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert_eq!(
            decide(&workspaces(WITH_SCRATCH), "WINDOWSTERMINAL"),
            Action::Summon {
                window_id: "t2".into()
            }
        );
    }

    fn probe(cloaked: bool, visible: bool) -> Option<FocusProbe> {
        Some(FocusProbe { cloaked, visible })
    }

    fn window(id: &str) -> Node {
        serde_json::from_str(&format!(
            r#"{{"type":"window","id":"{id}","state":{{"type":"tiling"}}}}"#
        ))
        .unwrap()
    }

    #[test]
    fn a_cloaked_focused_window_is_rescued() {
        let ws = workspaces(WITH_SCRATCH);
        let f = window("c1");
        assert_eq!(
            decide_summon(&ws, Some(&f), probe(true, true)),
            SummonAction::Rescue {
                window_id: "c1".into(),
                pull_to: None
            }
        );
    }

    #[test]
    fn a_window_on_an_undisplayed_workspace_is_pulled_to_its_monitors_displayed_one() {
        let json = r#"[
            {"type":"workspace","id":"ws1","name":"1","parentId":"mon1","isDisplayed":true,"hasFocus":true,
             "children":[]},
            {"type":"workspace","id":"ws6","name":"6","parentId":"mon1","isDisplayed":false,
             "children":[{"type":"window","id":"g1","state":{"type":"tiling"}}]}
        ]"#;
        let ws = workspaces(json);
        let f = window("g1");
        assert_eq!(
            decide_summon(&ws, Some(&f), probe(false, true)),
            SummonAction::Rescue {
                window_id: "g1".into(),
                pull_to: Some("1".into())
            },
            "off-screen beats healthy probes — the workspace is the ghost"
        );
    }

    #[test]
    fn a_healthy_focus_falls_through_to_pocket_recall() {
        let ws = workspaces(WITH_SCRATCH);
        let f = window("c1");
        assert_eq!(
            decide_summon(&ws, Some(&f), probe(false, true)),
            SummonAction::Summon {
                window_id: "t2".into(),
                to: "1".into()
            },
            "childFocusOrder picks the newest; the focused workspace receives it"
        );
    }

    #[test]
    fn empty_scratch_and_healthy_focus_is_a_quiet_nothing() {
        let json = r#"[
            {"type":"workspace","id":"ws1","name":"1","isDisplayed":true,"hasFocus":true,
             "children":[{"type":"window","id":"c1","state":{"type":"tiling"}}]},
            {"type":"workspace","id":"wss","name":"scratch","children":[]}
        ]"#;
        let ws = workspaces(json);
        let f = window("c1");
        assert_eq!(
            decide_summon(&ws, Some(&f), probe(false, true)),
            SummonAction::Nothing
        );
    }

    #[test]
    fn a_minimized_focus_is_never_rescued() {
        let ws = workspaces(WITH_SCRATCH);
        let f: Node =
            serde_json::from_str(r#"{"type":"window","id":"m1","state":{"type":"minimized"}}"#)
                .unwrap();
        assert!(matches!(
            decide_summon(&ws, Some(&f), probe(true, false)),
            SummonAction::Summon { .. }
        ));
    }

    #[test]
    fn a_scratch_window_of_the_wrong_process_does_not_shadow_a_banishable_one() {
        let json = r#"[
            {"type":"workspace","id":"ws1","name":"1","isDisplayed":true,
             "children":[{"type":"window","id":"c1","state":{"type":"tiling"},"processName":"wt"}]},
            {"type":"workspace","id":"wss","name":"scratch",
             "children":[{"type":"window","id":"x1","state":{"type":"tiling"},"processName":"other"}]}
        ]"#;
        assert_eq!(
            decide(&workspaces(json), "wt"),
            Action::Banish {
                window_id: "c1".into()
            }
        );
    }
}
