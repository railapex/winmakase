//! Versioned finite actions shared by the CLI, bar, shortcuts and later
//! lifecycle request channel.
//!
//! These are data contracts, not a plugin bus. A caller selects one known
//! operation and supplies typed arguments; no shell command crosses the UI or
//! request boundary.

use serde::{Deserialize, Serialize};

pub const ACTION_SCHEMA: u32 = 1;
pub const POPUP_SNAPSHOT_SCHEMA: u32 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultRoleAction {
    #[default]
    FocusOrLaunch,
    LaunchNew,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActionId {
    FocusOrLaunch { role_id: String },
    LaunchNew { role_id: String },
    OpenLauncher,
    ToggleFloating,
    Reflow,
    Scratchpad { name: String },
    OpenControls,
    OpenKeyHelp,
    SetTheme { preset_id: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionOutcome {
    Focused,
    Launched,
    Pending,
    CompletedWithoutFocus,
    Unsupported,
    Unavailable,
    Conflict,
    Failed,
}

impl ActionOutcome {
    pub fn completed(self) -> bool {
        matches!(
            self,
            Self::Focused | Self::Launched | Self::CompletedWithoutFocus
        )
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryDisposition {
    #[default]
    NotApplicable,
    WaitForObservation,
    RetryAllowed,
    DoNotRetry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionResult {
    pub schema: u32,
    pub request_id: String,
    pub action: ActionId,
    pub outcome: ActionOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wm_generation: Option<String>,
    #[serde(default)]
    pub retry: RetryDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic_code: Option<String>,
    pub message: String,
}

impl ActionResult {
    pub fn validate(&self) -> Result<(), String> {
        require_schema("action result", self.schema, ACTION_SCHEMA)?;
        if self.request_id.trim().is_empty() {
            return Err("action result request_id must not be empty".into());
        }
        if self.message.trim().is_empty() {
            return Err("action result message must not be empty".into());
        }
        if self.outcome == ActionOutcome::Pending
            && self.retry != RetryDisposition::WaitForObservation
        {
            return Err("pending action result must wait for observation".into());
        }
        Ok(())
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        let value: Self = serde_json::from_str(text)
            .map_err(|error| format!("action result parse error: {error}"))?;
        value.validate()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionDescriptor {
    pub id: String,
    pub label: String,
    pub action: ActionId,
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_reason: Option<String>,
}

/// Immutable UI data from one owner generation. Later shell work can extend
/// the payload behind a schema bump; it must never smuggle executable text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopupSnapshot {
    pub schema: u32,
    pub owner_generation: String,
    pub sequence: u64,
    pub actions: Vec<ActionDescriptor>,
}

impl PopupSnapshot {
    pub fn validate(&self) -> Result<(), String> {
        require_schema("popup snapshot", self.schema, POPUP_SNAPSHOT_SCHEMA)?;
        if self.owner_generation.trim().is_empty() {
            return Err("popup snapshot owner_generation must not be empty".into());
        }
        for action in &self.actions {
            if action.id.trim().is_empty() || action.label.trim().is_empty() {
                return Err("popup action id and label must not be empty".into());
            }
            if action.available && action.unavailable_reason.is_some() {
                return Err(format!(
                    "popup action {} is available but has an unavailable reason",
                    action.id
                ));
            }
        }
        Ok(())
    }
}

pub(crate) fn require_schema(name: &str, actual: u32, expected: u32) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "unsupported {name} schema {actual}; expected {expected}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending() -> ActionResult {
        ActionResult {
            schema: ACTION_SCHEMA,
            request_id: "request-7".into(),
            action: ActionId::FocusOrLaunch {
                role_id: "browser.work".into(),
            },
            outcome: ActionOutcome::Pending,
            role_id: Some("browser.work".into()),
            window_id: None,
            wm_generation: Some("wm-2".into()),
            retry: RetryDisposition::WaitForObservation,
            diagnostic_code: Some("startup_grace".into()),
            message: "Browser is still starting".into(),
        }
    }

    #[test]
    fn action_result_round_trips_with_the_frozen_shape() {
        let value = pending();
        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(json["schema"], ACTION_SCHEMA);
        assert_eq!(json["action"]["kind"], "focus_or_launch");
        assert_eq!(json["outcome"], "pending");
        assert_eq!(json["retry"], "wait_for_observation");
        let text = serde_json::to_string(&value).unwrap();
        assert_eq!(ActionResult::from_json(&text).unwrap(), value);
    }

    #[test]
    fn action_result_rejects_unknown_versions_and_false_pending_success() {
        let mut value = pending();
        value.schema += 1;
        assert!(value.validate().unwrap_err().contains("unsupported"));

        value.schema = ACTION_SCHEMA;
        value.retry = RetryDisposition::RetryAllowed;
        assert!(value.validate().unwrap_err().contains("observation"));
    }

    #[test]
    fn popup_snapshot_round_trips_without_command_strings() {
        let snapshot = PopupSnapshot {
            schema: POPUP_SNAPSHOT_SCHEMA,
            owner_generation: "owner-4".into(),
            sequence: 9,
            actions: vec![ActionDescriptor {
                id: "launcher".into(),
                label: "Launcher".into(),
                action: ActionId::OpenLauncher,
                available: true,
                unavailable_reason: None,
            }],
        };
        snapshot.validate().unwrap();
        let text = serde_json::to_string(&snapshot).unwrap();
        let decoded: PopupSnapshot = serde_json::from_str(&text).unwrap();
        assert_eq!(decoded, snapshot);
        assert!(!text.contains("command"));
    }
}
