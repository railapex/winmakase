//! `winmakase reflow` — omarchy Super+J parity (in-place split re-orientation).
//!
//! GlazeWM's `toggle-tiling-direction` changes where the *next* window
//! inserts; it never re-arranges an existing split (spike bench, 2026-08-28:
//! IPC rect comparison, zero movement). Upstream's release/review lane is
//! stalled, so the verb is
//! the permanent route (docs/research/gap-features.md): compute a plan of
//! `move --direction` commands that rebuilds the focused row in the other
//! orientation, then execute it via the CLI.

use std::fmt;

use crate::glazewm::{Node, Workspace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

impl Direction {
    fn parse(s: &str) -> Option<Self> {
        match s {
            "horizontal" => Some(Direction::Horizontal),
            "vertical" => Some(Direction::Vertical),
            _ => None,
        }
    }

    fn flipped(self) -> Self {
        match self {
            Direction::Horizontal => Direction::Vertical,
            Direction::Vertical => Direction::Horizontal,
        }
    }

    /// The `move --direction` that pushes a window out of a row of this
    /// orientation and into the flipped one.
    fn move_arg(self) -> &'static str {
        match self {
            Direction::Horizontal => "down",
            Direction::Vertical => "right",
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Direction::Horizontal => "horizontal",
            Direction::Vertical => "vertical",
        })
    }
}

#[derive(Debug, Clone)]
pub struct PlannedMove {
    pub window_id: String,
    pub title: String,
    pub direction: &'static str,
}

#[derive(Debug, Clone)]
pub struct Plan {
    pub from: Direction,
    pub to: Direction,
    /// In execution order.
    pub moves: Vec<PlannedMove>,
}

/// Compute the reflow plan for the focused window's row.
///
/// The row is the deepest ancestor of the focused window holding two or more
/// participants — GlazeWM leaves single-child split wrappers behind after
/// moves (live finding, 2026-08-28), so the focused window's *immediate*
/// parent is often a one-window shell that means nothing to a human. Those
/// wrappers are seen through; a genuine multi-window nested split among the
/// siblings still refuses, because guessing a choreography for it would be
/// guessing with someone's windows.
pub fn plan(workspaces: &[Workspace]) -> Result<Plan, String> {
    let path = path_to_focus(workspaces)?;

    for (direction, children) in path.iter().rev() {
        let mut participants: Vec<&Node> = Vec::new();
        let mut nested: Option<&Node> = None;
        let mut units = 0usize;
        for child in *children {
            match unwrap_single(child) {
                Some(w) if w.state_type() == Some("tiling") => {
                    units += 1;
                    participants.push(w);
                }
                // Floating/fullscreen/minimized: present, not part of the row.
                Some(_) => {}
                None => {
                    units += 1;
                    nested = Some(child);
                }
            }
        }
        // Fewer than two layout units means this level is a shell; the row a
        // human sees is further up.
        if units < 2 {
            continue;
        }
        if let Some(n) = nested {
            return Err(format!(
                "the focused row contains a nested multi-window {} — reflow handles flat rows only (for now)",
                n.kind
            ));
        }

        // Every window after the first is pushed out of the row — in REVERSE
        // reading order (last first). Verified live on the rig 2026-08-28:
        // each move lands the window at the top of the already-moved tail, so
        // moving in reading order reverses the tail (a [1,3,2] row came out
        // [1,2,3]); moving last-to-second preserves the row's order.
        let moves = participants[1..]
            .iter()
            .rev()
            .map(|w| PlannedMove {
                window_id: w.id.clone(),
                title: w.title.clone().unwrap_or_default(),
                direction: direction.move_arg(),
            })
            .collect();

        return Ok(Plan {
            from: *direction,
            to: direction.flipped(),
            moves,
        });
    }

    Err("nothing to reflow — no ancestor row of the focused window has two tiled windows".into())
}

/// A node that is, for layout purposes, exactly one window: a window itself,
/// or a chain of single-child splits ending in one.
fn unwrap_single(node: &Node) -> Option<&Node> {
    if node.is_window() {
        return Some(node);
    }
    match node.children.as_slice() {
        [only] => unwrap_single(only),
        _ => None,
    }
}

/// Every (direction, children) row on the way from the workspace down to the
/// focused window, outermost first.
fn path_to_focus(workspaces: &[Workspace]) -> Result<Vec<(Direction, &[Node])>, String> {
    for ws in workspaces {
        let ws_direction = ws
            .tiling_direction
            .as_deref()
            .and_then(Direction::parse)
            .ok_or_else(|| format!("workspace {} has no tiling direction", ws.name))?;
        let mut path = Vec::new();
        if descend(&ws.children, ws_direction, &mut path) {
            return Ok(path);
        }
    }
    Err("no focused window — focus a tiled window and try again".into())
}

fn descend<'a>(
    children: &'a [Node],
    direction: Direction,
    path: &mut Vec<(Direction, &'a [Node])>,
) -> bool {
    path.push((direction, children));
    if children.iter().any(|c| c.is_window() && c.has_focus) {
        return true;
    }
    for child in children {
        if !child.is_window() {
            let child_direction = child
                .tiling_direction
                .as_deref()
                .and_then(Direction::parse)
                .unwrap_or(direction);
            if descend(&child.children, child_direction, path) {
                return true;
            }
        }
    }
    path.pop();
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspaces(json: &str) -> Vec<Workspace> {
        serde_json::from_str(json).unwrap()
    }

    const TWO_WINDOW_ROW: &str = r#"[{
        "type":"workspace","id":"ws2","name":"2","tilingDirection":"horizontal",
        "isDisplayed":true,"hasFocus":true,
        "children":[
            {"type":"window","id":"w1","hasFocus":true,"state":{"type":"tiling"},"title":"left"},
            {"type":"window","id":"w2","state":{"type":"tiling"},"title":"right"}
        ]
    }]"#;

    #[test]
    fn a_two_window_horizontal_row_plans_one_move_down() {
        let p = plan(&workspaces(TWO_WINDOW_ROW)).unwrap();
        assert_eq!(p.from, Direction::Horizontal);
        assert_eq!(p.to, Direction::Vertical);
        assert_eq!(p.moves.len(), 1);
        assert_eq!(p.moves[0].window_id, "w2");
        assert_eq!(p.moves[0].direction, "down");
    }

    #[test]
    fn a_three_window_vertical_stack_plans_two_moves_right_in_order() {
        let json = r#"[{
            "type":"workspace","id":"ws3","name":"3","tilingDirection":"vertical",
            "isDisplayed":true,
            "children":[
                {"type":"window","id":"w1","state":{"type":"tiling"}},
                {"type":"window","id":"w2","hasFocus":true,"state":{"type":"tiling"}},
                {"type":"window","id":"w3","state":{"type":"tiling"}}
            ]
        }]"#;
        let p = plan(&workspaces(json)).unwrap();
        assert_eq!(p.to, Direction::Horizontal);
        let ids: Vec<&str> = p.moves.iter().map(|m| m.window_id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["w3", "w2"],
            "reverse reading order — each move lands atop the moved tail (live finding)"
        );
        assert!(p.moves.iter().all(|m| m.direction == "right"));
    }

    #[test]
    fn a_floating_sibling_is_left_out_of_the_plan() {
        let json = r#"[{
            "type":"workspace","id":"ws","name":"1","tilingDirection":"horizontal",
            "children":[
                {"type":"window","id":"w1","hasFocus":true,"state":{"type":"tiling"}},
                {"type":"window","id":"f1","state":{"type":"floating"}},
                {"type":"window","id":"w2","state":{"type":"tiling"}}
            ]
        }]"#;
        let p = plan(&workspaces(json)).unwrap();
        let ids: Vec<&str> = p.moves.iter().map(|m| m.window_id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["w2"],
            "the floating window neither moves nor counts"
        );
    }

    #[test]
    fn a_single_window_row_has_nothing_to_reflow() {
        let json = r#"[{
            "type":"workspace","id":"ws","name":"1","tilingDirection":"vertical",
            "children":[{"type":"window","id":"w1","hasFocus":true,"state":{"type":"tiling"}}]
        }]"#;
        let err = plan(&workspaces(json)).unwrap_err();
        assert!(err.contains("nothing to reflow"), "{err}");
    }

    #[test]
    fn a_multi_window_nested_split_in_the_row_is_refused_loudly() {
        let json = r#"[{
            "type":"workspace","id":"ws","name":"1","tilingDirection":"horizontal",
            "children":[
                {"type":"window","id":"w1","hasFocus":true,"state":{"type":"tiling"}},
                {"type":"split","id":"s1","tilingDirection":"vertical","children":[
                    {"type":"window","id":"w2","state":{"type":"tiling"}},
                    {"type":"window","id":"w3","state":{"type":"tiling"}}
                ]}
            ]
        }]"#;
        let err = plan(&workspaces(json)).unwrap_err();
        assert!(err.contains("nested"), "{err}");
    }

    #[test]
    fn a_single_child_split_wrapper_is_seen_through() {
        // The live regression (2026-08-28): after one reflow, GlazeWM left the
        // first window inside a one-child split shell. Focus sits on the
        // wrapped window; the meaningful row is the workspace's.
        let json = r#"[{
            "type":"workspace","id":"ws","name":"7","tilingDirection":"vertical",
            "children":[
                {"type":"split","id":"s1","tilingDirection":"horizontal","children":[
                    {"type":"window","id":"w1","hasFocus":true,"state":{"type":"tiling"}}
                ]},
                {"type":"window","id":"w2","state":{"type":"tiling"}},
                {"type":"window","id":"w3","state":{"type":"tiling"}}
            ]
        }]"#;
        let p = plan(&workspaces(json)).unwrap();
        assert_eq!(
            p.from,
            Direction::Vertical,
            "the workspace row, not the shell"
        );
        let ids: Vec<&str> = p.moves.iter().map(|m| m.window_id.as_str()).collect();
        assert_eq!(ids, vec!["w3", "w2"]);
    }

    #[test]
    fn focus_inside_a_nested_split_reflows_that_split_not_the_workspace() {
        let json = r#"[{
            "type":"workspace","id":"ws","name":"1","tilingDirection":"horizontal",
            "children":[
                {"type":"window","id":"w1","state":{"type":"tiling"}},
                {"type":"split","id":"s1","tilingDirection":"vertical","children":[
                    {"type":"window","id":"w2","hasFocus":true,"state":{"type":"tiling"}},
                    {"type":"window","id":"w3","state":{"type":"tiling"}}
                ]}
            ]
        }]"#;
        let p = plan(&workspaces(json)).unwrap();
        assert_eq!(
            p.from,
            Direction::Vertical,
            "the split's direction, not the workspace's"
        );
        let ids: Vec<&str> = p.moves.iter().map(|m| m.window_id.as_str()).collect();
        assert_eq!(ids, vec!["w3"]);
    }

    #[test]
    fn no_focused_window_is_a_clear_error() {
        let json = r#"[{
            "type":"workspace","id":"ws","name":"1","tilingDirection":"vertical",
            "children":[{"type":"window","id":"w1","state":{"type":"tiling"}}]
        }]"#;
        let err = plan(&workspaces(json)).unwrap_err();
        assert!(err.contains("no focused window"), "{err}");
    }
}
