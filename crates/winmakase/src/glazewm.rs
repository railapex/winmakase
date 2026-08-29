//! GlazeWM IPC, over its WebSocket directly (localhost, default port 6123).
//!
//! One connection per CLI invocation, then round trips per command — about
//! 20ms end to end for a keybind-driven verb. Shelling to the `glazewm` CLI
//! was the first cut and paid a process spawn per command; three or four
//! commands per keypress put it back in the pwsh-latency territory the Rust
//! port exists to escape. The resident warm-socket relay (R5) is only needed
//! for the launcher tap, where single-digit milliseconds matter.
//!
//! Command strings are exactly the CLI's: `query workspaces`,
//! `command --id <uuid> move --workspace 2`. The spike-proven set
//! (spike/summon.ps1, source-verified 2026-08-28): `--id` targets a container
//! without focusing it; `focus --container-id <uuid>` focuses one.

use std::cell::RefCell;
use std::io;
use std::net::TcpStream;
use std::time::Duration;

use serde::Deserialize;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Message, WebSocket, connect};

use crate::config::Config;

/// GlazeWM's IPC server port (its default; not currently configurable here).
const PORT: u16 = 6123;

/// A reply must arrive within this or the WM is wedged, not busy.
const READ_TIMEOUT: Duration = Duration::from_secs(5);

pub struct Client {
    socket: RefCell<WebSocket<MaybeTlsStream<TcpStream>>>,
}

/// One frame from the server.
#[derive(Deserialize)]
struct Reply {
    #[serde(rename = "messageType")]
    message_type: Option<String>,
    data: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
    #[serde(default)]
    success: bool,
}

#[derive(Deserialize)]
struct WorkspacesData {
    workspaces: Vec<Workspace>,
}

#[derive(Deserialize)]
struct FocusedData {
    focused: Option<Node>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    /// The monitor's container id — what "same monitor" means for a pull.
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub has_focus: bool,
    #[serde(default)]
    pub is_displayed: bool,
    #[serde(default)]
    pub tiling_direction: Option<String>,
    #[serde(default)]
    pub children: Vec<Node>,
    #[serde(default)]
    pub child_focus_order: Vec<String>,
}

/// One container in the tree. A single struct with optional fields rather
/// than a tagged enum: the queries this crate makes only care about a few
/// fields, and an unknown container type must degrade to "not a window", not
/// to a parse failure of the whole tree.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    #[serde(rename = "type")]
    pub kind: String,
    pub id: String,
    #[serde(default)]
    pub has_focus: bool,
    #[serde(default)]
    pub tiling_direction: Option<String>,
    #[serde(default)]
    pub children: Vec<Node>,
    #[serde(default)]
    pub state: Option<serde_json::Value>,
    #[serde(default)]
    pub process_name: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    /// Win32 HWND, for probes the tree cannot answer (cloak, visibility).
    #[serde(default)]
    pub handle: Option<i64>,
    // Screen-space geometry, when the container has any.
    #[serde(default)]
    pub x: Option<i64>,
    #[serde(default)]
    pub y: Option<i64>,
    #[serde(default)]
    pub width: Option<i64>,
    #[serde(default)]
    pub height: Option<i64>,
}

impl Node {
    pub fn is_window(&self) -> bool {
        self.kind == "window"
    }

    /// The window's state type ("tiling", "floating", "fullscreen",
    /// "minimized"), when present.
    pub fn state_type(&self) -> Option<&str> {
        self.state.as_ref()?.get("type")?.as_str()
    }
}

/// Every window under a container, depth first — the traversal
/// spike/summon.ps1 proved out.
pub fn windows_under(children: &[Node]) -> Vec<&Node> {
    let mut out = Vec::new();
    for child in children {
        if child.is_window() {
            out.push(child);
        } else {
            out.extend(windows_under(&child.children));
        }
    }
    out
}

impl Client {
    /// Connect to the running GlazeWM. The config parameter reserves the seam
    /// for a configurable port; today the port is GlazeWM's default.
    pub fn from_config(_cfg: &Config) -> io::Result<Self> {
        let url = format!("ws://127.0.0.1:{PORT}");
        let (socket, _response) = connect(&url).map_err(|e| {
            io::Error::new(
                io::ErrorKind::ConnectionRefused,
                format!("cannot reach GlazeWM's IPC at {url} — is it running? ({e})"),
            )
        })?;
        if let MaybeTlsStream::Plain(stream) = socket.get_ref() {
            let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
        }
        Ok(Self {
            socket: RefCell::new(socket),
        })
    }

    pub fn query_workspaces(&self) -> io::Result<Vec<Workspace>> {
        let data = self.request("query workspaces")?;
        let parsed: WorkspacesData = serde_json::from_value(data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("workspaces: {e}")))?;
        Ok(parsed.workspaces)
    }

    /// The focused container, when the WM reports one.
    pub fn query_focused(&self) -> io::Result<Option<Node>> {
        let data = self.request("query focused")?;
        let parsed: FocusedData = serde_json::from_value(data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("focused: {e}")))?;
        Ok(parsed.focused)
    }

    /// `command <args…>`, checking the reply's success flag.
    pub fn command(&self, args: &[&str]) -> io::Result<()> {
        let msg = format!("command {}", args.join(" "));
        self.request(&msg).map(|_| ())
    }

    /// `command --id <uuid> <verb …>` — act on a container without focusing it.
    pub fn command_for(&self, container_id: &str, args: &[&str]) -> io::Result<()> {
        let mut full = vec!["--id", container_id];
        full.extend_from_slice(args);
        self.command(&full)
    }

    /// One round trip. Skips non-response frames (a future subscription's
    /// events must not be misread as our answer).
    fn request(&self, msg: &str) -> io::Result<serde_json::Value> {
        let mut socket = self.socket.borrow_mut();
        socket
            .send(Message::text(msg))
            .map_err(|e| io::Error::other(format!("send {msg:?}: {e}")))?;
        loop {
            let frame = socket
                .read()
                .map_err(|e| io::Error::other(format!("no reply to {msg:?}: {e}")))?;
            let Message::Text(text) = frame else {
                continue;
            };
            let reply: Reply = serde_json::from_str(text.as_str()).map_err(|e| {
                io::Error::new(io::ErrorKind::InvalidData, format!("reply to {msg:?}: {e}"))
            })?;
            if reply.message_type.as_deref() != Some("client_response") {
                continue;
            }
            if !reply.success {
                return Err(io::Error::other(format!(
                    "glazewm rejected {msg:?}: {:?}",
                    reply.error
                )));
            }
            return reply.data.ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("empty reply to {msg:?}"),
                )
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../tests/fixtures/query-workspaces.json");

    #[derive(Deserialize)]
    struct FixtureEnvelope {
        data: WorkspacesData,
        success: bool,
    }

    #[test]
    fn parses_the_real_rig_workspaces_reply() {
        let env: FixtureEnvelope = serde_json::from_str(FIXTURE).unwrap();
        assert!(env.success);
        let workspaces = env.data.workspaces;
        assert!(workspaces.len() >= 3, "the rig runs at least three");
        let displayed: Vec<&Workspace> = workspaces.iter().filter(|w| w.is_displayed).collect();
        assert!(displayed.len() >= 3, "one per daily monitor");
        for w in &workspaces {
            assert!(w.tiling_direction.is_some(), "workspaces carry a direction");
            assert!(w.parent_id.is_some(), "workspaces hang off a monitor");
        }
        let all: usize = workspaces
            .iter()
            .map(|w| windows_under(&w.children).len())
            .sum();
        assert!(all >= 4, "the fixture was captured on a live desktop");
        assert!(
            windows_under(&workspaces[0].children)
                .iter()
                .all(|w| w.handle.is_some()),
            "windows carry their HWND"
        );
    }

    #[test]
    fn windows_under_recurses_into_splits() {
        let json = r#"[
            {"type":"split","id":"s1","children":[
                {"type":"window","id":"w1"},
                {"type":"split","id":"s2","children":[{"type":"window","id":"w2"}]}
            ]},
            {"type":"window","id":"w3"}
        ]"#;
        let nodes: Vec<Node> = serde_json::from_str(json).unwrap();
        let ids: Vec<&str> = windows_under(&nodes)
            .iter()
            .map(|w| w.id.as_str())
            .collect();
        assert_eq!(ids, vec!["w1", "w2", "w3"]);
    }

    #[test]
    fn an_unknown_container_kind_degrades_instead_of_failing_the_parse() {
        let json = r#"[{"type":"hologram","id":"h1","children":[{"type":"window","id":"w1"}]}]"#;
        let nodes: Vec<Node> = serde_json::from_str(json).unwrap();
        assert!(!nodes[0].is_window());
        assert_eq!(windows_under(&nodes).len(), 1);
    }
}
