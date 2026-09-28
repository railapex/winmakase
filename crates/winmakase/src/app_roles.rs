//! Stable application-role and Glaze composition contracts.
//!
//! The user configuration remains the source of roles. This module normalizes
//! that existing shape for window/action consumers and owns the sole adapter
//! into `winmakase-keymap`; it does not discover windows or launch processes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::actions::{DefaultRoleAction, require_schema};
use crate::config::{AppConfig, AppState, InputMode};

pub const ROLE_SCHEMA: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaunchSpec {
    pub executable: String,
    #[serde(default)]
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalIdentity {
    pub schema: u32,
    pub process: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
}

impl CanonicalIdentity {
    pub fn validate(&self) -> Result<(), String> {
        require_schema("role identity", self.schema, ROLE_SCHEMA)?;
        if self.process.trim().is_empty() {
            return Err("role identity process must not be empty".into());
        }
        for (name, value) in [("class", &self.class), ("app_id", &self.app_id)] {
            if value
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                return Err(format!("role identity {name} must not be empty"));
            }
        }
        Ok(())
    }

    pub fn normalized(&self) -> Result<Self, String> {
        self.validate()?;
        Ok(Self {
            schema: self.schema,
            process: normalize(&self.process),
            class: self.class.as_deref().map(normalize),
            app_id: self.app_id.as_deref().map(normalize),
        })
    }

    pub fn from_json(text: &str) -> Result<Self, String> {
        let value: Self = serde_json::from_str(text)
            .map_err(|error| format!("role identity parse error: {error}"))?;
        value.validate()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MainWindowEligibility {
    pub has_owner: bool,
    pub resizable: bool,
}

impl Default for MainWindowEligibility {
    fn default() -> Self {
        Self {
            has_owner: false,
            resizable: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleWindowState {
    Preserve,
    Tiling,
    Floating,
    Ignored,
}

impl From<AppState> for RoleWindowState {
    fn from(value: AppState) -> Self {
        match value {
            AppState::Preserve => Self::Preserve,
            AppState::Tiling => Self::Tiling,
            AppState::Floating => Self::Floating,
            AppState::Ignored => Self::Ignored,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleDefinition {
    pub schema: u32,
    pub id: String,
    pub label: String,
    pub launch: LaunchSpec,
    pub identity: CanonicalIdentity,
    pub eligibility: MainWindowEligibility,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    pub state: RoleWindowState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manage_title: Option<String>,
    pub default_action: DefaultRoleAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_window: Option<LaunchSpec>,
}

impl RoleDefinition {
    pub fn validate(&self) -> Result<(), String> {
        require_schema("role definition", self.schema, ROLE_SCHEMA)?;
        self.identity.validate()?;
        for (name, value) in [
            ("id", self.id.as_str()),
            ("label", self.label.as_str()),
            ("launch.executable", self.launch.executable.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(format!("role definition {name} must not be empty"));
            }
        }
        if self.default_action == DefaultRoleAction::LaunchNew && self.new_window.is_none() {
            return Err(format!(
                "role {} defaults to launch_new but has no new-window launch specification",
                self.id
            ));
        }
        Ok(())
    }
}

/// Normalize the existing `[apps]` map into the versioned role interface.
pub fn from_config(
    source: &str,
    apps: &BTreeMap<String, AppConfig>,
) -> Result<Vec<RoleDefinition>, String> {
    let mut roles = Vec::with_capacity(apps.len());
    let mut identities: BTreeMap<CanonicalIdentity, (&str, LaunchSpec)> = BTreeMap::new();

    for (id, app) in apps {
        let identity = CanonicalIdentity {
            schema: ROLE_SCHEMA,
            process: app.process.clone(),
            class: app.class.clone(),
            app_id: app.app_id.clone(),
        }
        .normalized()
        .map_err(|error| format!("{source}: apps.{id}.identity: {error}"))?;
        let launch = LaunchSpec {
            executable: app.launch.clone(),
            arguments: app.launch_args.clone(),
        };
        if let Some((other_id, other_launch)) = identities.get(&identity)
            && other_launch != &launch
        {
            return Err(format!(
                "{source}: apps.{id}.identity conflicts with apps.{other_id}.identity: aliases with the same canonical identity must use the same launch executable and arguments"
            ));
        }
        identities
            .entry(identity.clone())
            .or_insert((id.as_str(), launch.clone()));

        let role = RoleDefinition {
            schema: ROLE_SCHEMA,
            id: id.clone(),
            label: app.label.clone().unwrap_or_else(|| id.clone()),
            launch,
            identity,
            eligibility: MainWindowEligibility {
                has_owner: app.has_owner.unwrap_or(false),
                resizable: app.resizable.unwrap_or(true),
            },
            workspace: app.workspace.clone(),
            state: app.state.into(),
            manage_title: app.title.clone(),
            default_action: app.default_action,
            new_window: app.new_window_args.as_ref().map(|arguments| LaunchSpec {
                executable: app.launch.clone(),
                arguments: arguments.clone(),
            }),
        };
        role.validate()
            .map_err(|error| format!("{source}: apps.{id}: {error}"))?;
        roles.push(role);
    }
    Ok(roles)
}

/// A theme renderer may supply a changed binding-free base. Role rules and
/// keybindings are always applied afterward by this one composition path.
#[derive(Debug, Clone, Copy)]
pub struct ThemeBase<'a> {
    pub id: &'a str,
    pub rendered_base_yaml: &'a str,
}

pub struct CompositionInput<'a> {
    pub base_yaml: &'a str,
    pub theme: Option<ThemeBase<'a>>,
    pub keymap: &'a [winmakase_keymap::Expanded],
    pub roles: &'a [RoleDefinition],
    pub input_mode: InputMode,
}

/// Pure adapter into `winmakase-keymap`. It performs no filesystem or desktop
/// mutation and is safe to run as a preview before J0 owns the write protocol.
pub fn compose_glazewm(input: CompositionInput<'_>) -> Result<String, String> {
    let base = match input.theme {
        Some(theme) => {
            if theme.id.trim().is_empty() {
                return Err("theme composition id must not be empty".into());
            }
            theme.rendered_base_yaml
        }
        None => input.base_yaml,
    };
    let rules = input.roles.iter().map(to_window_rule).collect::<Vec<_>>();
    winmakase_keymap::render_glazewm_config_with_super(
        base,
        input.keymap,
        &rules,
        input.input_mode.super_key(),
    )
}

fn to_window_rule(role: &RoleDefinition) -> winmakase_keymap::AppWindowRule {
    winmakase_keymap::AppWindowRule {
        name: role.id.clone(),
        process: role.identity.process.clone(),
        class: role.identity.class.clone(),
        title: role.manage_title.clone(),
        app_id: role.identity.app_id.clone(),
        workspace: role.workspace.clone(),
        has_owner: Some(role.eligibility.has_owner),
        resizable: Some(role.eligibility.resizable),
        state: match role.state {
            RoleWindowState::Preserve => winmakase_keymap::WindowRuleState::Preserve,
            RoleWindowState::Tiling => winmakase_keymap::WindowRuleState::Tiling,
            RoleWindowState::Floating => winmakase_keymap::WindowRuleState::Floating,
            RoleWindowState::Ignored => winmakase_keymap::WindowRuleState::Ignored,
        },
    }
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn existing_app_config_becomes_a_versioned_role() {
        let cfg = Config::parse(
            r#"
            [kanata]
            command = "k.exe"
            [glazewm]
            command = "g.exe"
            [apps.browser]
            label = "Work browser"
            launch = "chrome.exe"
            launch_args = ["--profile-directory=Default"]
            new_window_args = ["--profile-directory=Default", "--new-window"]
            process = "Chrome"
            class = "Chrome_WidgetWin_1"
            app_id = "Chrome"
            workspace = "2"
            state = "preserve"
            "#,
        )
        .unwrap();
        let roles = from_config("fixture.toml", &cfg.apps).unwrap();
        assert_eq!(roles.len(), 1);
        let role = &roles[0];
        assert_eq!(role.schema, ROLE_SCHEMA);
        assert_eq!(role.id, "browser");
        assert_eq!(role.label, "Work browser");
        assert_eq!(role.identity.process, "chrome");
        assert_eq!(role.identity.app_id.as_deref(), Some("chrome"));
        assert_eq!(role.new_window.as_ref().unwrap().arguments.len(), 2);
    }

    #[test]
    fn identity_round_trip_rejects_unknown_versions() {
        let identity = CanonicalIdentity::from_json(include_str!(
            "../tests/fixtures/contracts/role-identity.json"
        ))
        .unwrap();
        let text = serde_json::to_string(&identity).unwrap();
        assert_eq!(CanonicalIdentity::from_json(&text).unwrap(), identity);
        let toml = include_str!("../tests/fixtures/contracts/role-identity.toml");
        assert!(toml.contains("schema = 1"));
        assert_eq!(toml::from_str::<CanonicalIdentity>(toml).unwrap(), identity);

        let text = text.replace("\"schema\":1", "\"schema\":2");
        assert!(
            CanonicalIdentity::from_json(&text)
                .unwrap_err()
                .contains("unsupported")
        );
    }

    #[test]
    fn composition_keeps_roles_and_keys_when_theme_changes_the_base() {
        let expanded = vec![winmakase_keymap::Expanded {
            id: "focus-left".into(),
            chords: vec!["SUPER+H".into()],
            desc: "Focus left".into(),
            commands: Some(vec!["focus --direction left".into()]),
            status: None,
        }];
        let roles = vec![RoleDefinition {
            schema: ROLE_SCHEMA,
            id: "browser".into(),
            label: "Browser".into(),
            launch: LaunchSpec {
                executable: "chrome".into(),
                arguments: Vec::new(),
            },
            identity: CanonicalIdentity {
                schema: ROLE_SCHEMA,
                process: "chrome".into(),
                class: None,
                app_id: None,
            },
            eligibility: MainWindowEligibility::default(),
            workspace: Some("2".into()),
            state: RoleWindowState::Preserve,
            manage_title: None,
            default_action: DefaultRoleAction::FocusOrLaunch,
            new_window: None,
        }];
        let base = "general: {}\nworkspaces:\n  - name: '2'\n";
        let themed = "general: {}\ngaps:\n  inner_gap: 8px\nworkspaces:\n  - name: '2'\n";
        let output = compose_glazewm(CompositionInput {
            base_yaml: base,
            theme: Some(ThemeBase {
                id: "tokyo-night",
                rendered_base_yaml: themed,
            }),
            keymap: &expanded,
            roles: &roles,
            input_mode: InputMode::Kanata,
        })
        .unwrap();
        assert!(output.contains("inner_gap: 8px"));
        assert!(output.contains("window_process"));
        assert!(output.contains("chrome"));
        assert!(output.contains("rwin+h"));
    }
}
