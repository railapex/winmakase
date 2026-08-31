//! Omarchy keybinding grammar as data, and the GlazeWM generator.
//!
//! `keymap/omarchy.toml` carries every binding upstream omarchy ships. Each
//! entry either `map`s to GlazeWM command(s) or explains itself with a
//! `status` + `note`. This crate parses that file, enforces the structural
//! rules, and renders the `keybindings:` section of a GlazeWM config. A
//! caller may also compose that section with a binding-free GlazeWM base so
//! the deployed YAML remains generated rather than becoming a second source.
//!
//! The generator is deliberately dumb: all judgment lives in the TOML. A new
//! upstream binding that isn't in the file is invisible here — the drift
//! contract is the re-fetch-and-diff procedure described at the top of the
//! TOML, not this code.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeymapFile {
    pub meta: Meta,
    #[serde(default)]
    pub apps: BTreeMap<String, String>,
    #[serde(rename = "bind", default)]
    pub binds: Vec<Bind>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    pub upstream: String,
    #[serde(rename = "ref")]
    pub upstream_ref: String,
    pub files: Vec<String>,
    pub fetched: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bind {
    pub id: String,
    pub chord: Option<String>,
    pub chords: Option<Vec<String>>,
    pub desc: String,
    pub src: String,
    pub map: Option<Vec<String>>,
    pub status: Option<Status>,
    pub note: Option<String>,
    pub deviation: Option<String>,
    pub foreach: Option<Foreach>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Gap,
    Native,
    App,
    Omitted,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Foreach {
    pub from: u32,
    pub to: u32,
}

/// One binding after `foreach` expansion and `{app}` substitution.
#[derive(Debug)]
pub struct Expanded {
    pub id: String,
    pub chords: Vec<String>,
    pub desc: String,
    pub commands: Option<Vec<String>>,
    pub status: Option<Status>,
}

/// What the file covers, by fate. `chords_*` count individual chords
/// (a `chords` list or `foreach` family is several).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Coverage {
    pub mapped: usize,
    pub gap: usize,
    pub native: usize,
    pub app: usize,
    pub omitted: usize,
    pub chords_mapped: usize,
    pub chords_total: usize,
}

pub fn parse(text: &str) -> Result<KeymapFile, String> {
    toml::from_str(text).map_err(|e| format!("keymap parse error: {e}"))
}

/// A user's local keymap overrides — `keymap/local.toml` under the Winmakase
/// home, never in the repo. Three abilities: set `[apps]` commands (override
/// a stock key or add a new one for local placeholders), add or replace whole
/// `[[bind]]` entries (matched by id — an existing id is replaced in place,
/// a new one appended), and `disable` stock ids.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverrideFile {
    #[serde(default)]
    pub apps: BTreeMap<String, String>,
    #[serde(rename = "bind", default)]
    pub binds: Vec<Bind>,
    #[serde(default)]
    pub disable: Vec<String>,
}

/// What `merge` did. The CLI prints this so a local file's effect on the
/// rendered grammar is visible, never silent.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MergeReport {
    pub apps_set: usize,
    pub binds_replaced: usize,
    pub binds_added: usize,
    pub binds_disabled: usize,
}

impl std::fmt::Display for MergeReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} apps set, {} binds replaced, {} added, {} disabled",
            self.apps_set, self.binds_replaced, self.binds_added, self.binds_disabled
        )
    }
}

pub fn parse_overrides(text: &str) -> Result<OverrideFile, String> {
    toml::from_str(text).map_err(|e| format!("keymap overrides parse error: {e}"))
}

/// Fold local overrides into the stock keymap. Merge is dumb like the
/// generator — the merged file still goes through `check`, which owns all
/// shape and conflict judgment. Merge itself only rejects references to
/// nothing: a `disable` naming no stock bind (a typo must fail loudly, not
/// quietly change nothing) or an id both disabled and re-bound.
pub fn merge(base: &mut KeymapFile, local: OverrideFile) -> Result<MergeReport, Vec<String>> {
    let mut errors = Vec::new();
    let mut report = MergeReport::default();

    for id in &local.disable {
        if local.binds.iter().any(|b| b.id == *id) {
            errors.push(format!(
                "{id}: both disabled and re-bound — drop the disable"
            ));
            continue;
        }
        match base.binds.iter().position(|b| b.id == *id) {
            Some(pos) => {
                base.binds.remove(pos);
                report.binds_disabled += 1;
            }
            None => errors.push(format!("{id}: disable names no stock bind")),
        }
    }

    for bind in local.binds {
        match base.binds.iter().position(|b| b.id == bind.id) {
            Some(pos) => {
                base.binds[pos] = bind;
                report.binds_replaced += 1;
            }
            None => {
                base.binds.push(bind);
                report.binds_added += 1;
            }
        }
    }

    report.apps_set = local.apps.len();
    base.apps.extend(local.apps);

    if errors.is_empty() {
        Ok(report)
    } else {
        Err(errors)
    }
}

/// Expand and validate. All violations are collected, not just the first.
pub fn check(file: &KeymapFile) -> Result<(Vec<Expanded>, Coverage), Vec<String>> {
    let mut errors = Vec::new();
    let mut ids = HashSet::new();
    let mut expanded = Vec::new();

    for bind in &file.binds {
        if !ids.insert(bind.id.as_str()) {
            errors.push(format!("{}: duplicate id", bind.id));
        }
        if let Err(e) = validate_shape(bind) {
            errors.push(format!("{}: {e}", bind.id));
            continue;
        }
        match expand(bind, &file.apps) {
            Ok(mut items) => expanded.append(&mut items),
            Err(e) => errors.push(format!("{}: {e}", bind.id)),
        }
    }

    // Mapped chords must translate to GlazeWM syntax and not collide.
    let mut seen: HashMap<String, String> = HashMap::new();
    for item in &expanded {
        if item.commands.is_none() {
            continue;
        }
        for chord in &item.chords {
            match translate_chord(chord) {
                Ok(normalized) => {
                    if let Some(other) = seen.insert(normalized.clone(), item.id.clone()) {
                        errors.push(format!(
                            "{}: chord {normalized} already bound by {other}",
                            item.id
                        ));
                    }
                }
                Err(e) => errors.push(format!("{}: {e}", item.id)),
            }
        }
    }

    if errors.is_empty() {
        let coverage = coverage_of(&expanded);
        Ok((expanded, coverage))
    } else {
        Err(errors)
    }
}

/// Render the `keybindings:` section of a GlazeWM config from the mapped
/// entries, in file order.
pub fn render_glazewm(expanded: &[Expanded]) -> String {
    let mut out = String::new();
    out.push_str("# Generated by winmakase-keymap from keymap/omarchy.toml — do not edit.\n");
    out.push_str("keybindings:\n");
    for item in expanded {
        let Some(commands) = &item.commands else {
            continue;
        };
        for chord in &item.chords {
            let binding = translate_chord(chord).expect("checked by `check`");
            let _ = writeln!(out, "  # {} — {}", item.id, item.desc);
            let _ = writeln!(out, "  - commands: [{}]", yaml_list(commands));
            let _ = writeln!(out, "    bindings: ['{binding}']");
        }
    }
    out
}

/// One typed app policy supplied by the product config. The launch command is
/// deliberately absent: Glaze only needs the stable window identity and the
/// manage-time policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppWindowRule {
    pub name: String,
    pub process: String,
    pub class: Option<String>,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub workspace: Option<String>,
    pub state: WindowRuleState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowRuleState {
    Tiling,
    Floating,
    Ignored,
}

#[derive(Serialize)]
struct RenderedWindowRule<'a> {
    commands: Vec<String>,
    #[serde(rename = "match")]
    match_window: Vec<RenderedWindowMatch<'a>>,
    on: [&'static str; 1],
}

#[derive(Serialize)]
struct RenderedWindowMatch<'a> {
    window_process: EqualsMatch<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    window_class: Option<EqualsMatch<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    window_title: Option<EqualsMatch<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    window_app_id: Option<EqualsMatch<'a>>,
}

#[derive(Serialize)]
struct EqualsMatch<'a> {
    equals: &'a str,
}

#[derive(Debug)]
struct BaseWindowRule {
    commands: Vec<String>,
    matches: Vec<BaseWindowMatch>,
}

#[derive(Debug)]
struct BaseWindowMatch {
    process: Option<BaseSelector>,
    class: Option<BaseSelector>,
    title: Option<BaseSelector>,
    app_id: Option<BaseSelector>,
}

#[derive(Debug)]
enum BaseSelector {
    Equals(String),
    Includes(String),
    Regex(String),
    NotEquals(String),
    NotRegex(String),
}

/// Compose a complete GlazeWM config from a machine/portable base, typed app
/// manage rules, and generated bindings. The base owns narrow host/system
/// rules; app config owns app policy; this function is the sole owner of the
/// rendered `window_rules` list. A base `keybindings` owner remains forbidden.
pub fn render_glazewm_config(
    base: &str,
    expanded: &[Expanded],
    apps: &[AppWindowRule],
) -> Result<String, String> {
    let normalized = base
        .strip_prefix('\u{feff}')
        .unwrap_or(base)
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    if normalized.trim().is_empty() {
        return Err("GlazeWM base is empty".into());
    }
    let mut document: serde_yaml_ng::Value = serde_yaml_ng::from_str(&normalized)
        .map_err(|error| format!("GlazeWM base is not valid YAML: {error}"))?;
    let root = document
        .as_mapping()
        .ok_or_else(|| "GlazeWM base must be a top-level mapping".to_string())?;
    let keybindings_key = serde_yaml_ng::Value::String("keybindings".into());
    if root.contains_key(&keybindings_key) {
        let line = normalized
            .lines()
            .position(|text| text.starts_with("keybindings:"))
            .map_or_else(|| "an unknown".to_string(), |line| (line + 1).to_string());
        return Err(format!(
            "GlazeWM base line {line} contains keybindings:; remove that section because the keymap renderer owns it"
        ));
    }

    let workspaces = workspace_names(root)?;
    let base_rules = base_window_rules(root)?;
    validate_apps(apps, &workspaces, &base_rules)?;

    let bindings = render_glazewm(expanded);
    let (_, bindings_body) = bindings
        .split_once('\n')
        .expect("the generated keybindings header always ends with a newline");

    // Zero app definitions is the W0 compatibility contract: parse and
    // validate the base, but leave its comments and bytes untouched.
    let base_body = if apps.is_empty() {
        normalized.trim_end().to_string()
    } else {
        let mut ordered = apps.to_vec();
        ordered.sort_by(|left, right| {
            app_order(left)
                .cmp(&app_order(right))
                .then_with(|| left.name.cmp(&right.name))
        });
        let generated = ordered
            .iter()
            .map(rendered_window_rule)
            .map(serde_yaml_ng::to_value)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("could not serialize generated window rule: {error}"))?;

        let root = document
            .as_mapping_mut()
            .expect("top-level mapping was checked above");
        let rules_key = serde_yaml_ng::Value::String("window_rules".into());
        match root.get_mut(&rules_key) {
            Some(serde_yaml_ng::Value::Sequence(rules)) => rules.extend(generated),
            Some(_) => unreachable!("base_window_rules checked the value's shape"),
            None => {
                root.insert(rules_key, serde_yaml_ng::Value::Sequence(generated));
            }
        }
        serde_yaml_ng::to_string(&document)
            .map_err(|error| format!("could not serialize composed GlazeWM base: {error}"))?
            .trim_end()
            .to_string()
    };

    Ok(format!(
        "# Generated by `winmakase keymap render --base`; edit the base/keymap sources, not this file.\n{}\n\n{}",
        base_body, bindings_body
    ))
}

fn workspace_names(root: &serde_yaml_ng::Mapping) -> Result<HashSet<String>, String> {
    let key = serde_yaml_ng::Value::String("workspaces".into());
    let Some(value) = root.get(&key) else {
        return Ok(HashSet::new());
    };
    let sequence = value
        .as_sequence()
        .ok_or_else(|| "GlazeWM base workspaces must be a list".to_string())?;
    let mut names = HashSet::new();
    for (index, workspace) in sequence.iter().enumerate() {
        let mapping = workspace.as_mapping().ok_or_else(|| {
            format!(
                "GlazeWM base workspaces item {} must be a mapping",
                index + 1
            )
        })?;
        let name_key = serde_yaml_ng::Value::String("name".into());
        let name = mapping
            .get(&name_key)
            .and_then(serde_yaml_ng::Value::as_str)
            .ok_or_else(|| {
                format!(
                    "GlazeWM base workspaces item {} needs a string name",
                    index + 1
                )
            })?;
        if !names.insert(name.to_string()) {
            return Err(format!("GlazeWM base declares workspace {name:?} twice"));
        }
    }
    Ok(names)
}

fn base_window_rules(root: &serde_yaml_ng::Mapping) -> Result<Vec<BaseWindowRule>, String> {
    let key = serde_yaml_ng::Value::String("window_rules".into());
    let Some(value) = root.get(&key) else {
        return Ok(Vec::new());
    };
    let sequence = value
        .as_sequence()
        .ok_or_else(|| "GlazeWM base window_rules must be a list".to_string())?;
    sequence
        .iter()
        .enumerate()
        .map(|(index, value)| parse_base_window_rule(index, value))
        .collect()
}

fn parse_base_window_rule(
    index: usize,
    value: &serde_yaml_ng::Value,
) -> Result<BaseWindowRule, String> {
    let prefix = format!("GlazeWM base window_rules item {}", index + 1);
    let mapping = value
        .as_mapping()
        .ok_or_else(|| format!("{prefix} must be a mapping"))?;
    let commands_key = serde_yaml_ng::Value::String("commands".into());
    let commands = mapping
        .get(&commands_key)
        .and_then(serde_yaml_ng::Value::as_sequence)
        .ok_or_else(|| format!("{prefix}.commands must be a list"))?
        .iter()
        .map(|command| {
            command
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{prefix}.commands must contain only strings"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if commands.is_empty() {
        return Err(format!("{prefix}.commands must not be empty"));
    }

    let match_key = serde_yaml_ng::Value::String("match".into());
    let matches = mapping
        .get(&match_key)
        .and_then(serde_yaml_ng::Value::as_sequence)
        .ok_or_else(|| format!("{prefix}.match must be a list"))?
        .iter()
        .enumerate()
        .map(|(match_index, value)| parse_base_window_match(&prefix, match_index, value))
        .collect::<Result<Vec<_>, _>>()?;
    if matches.is_empty() {
        return Err(format!("{prefix}.match must not be empty"));
    }
    Ok(BaseWindowRule { commands, matches })
}

fn parse_base_window_match(
    prefix: &str,
    index: usize,
    value: &serde_yaml_ng::Value,
) -> Result<BaseWindowMatch, String> {
    let prefix = format!("{prefix}.match item {}", index + 1);
    let mapping = value
        .as_mapping()
        .ok_or_else(|| format!("{prefix} must be a mapping"))?;
    for key in mapping.keys() {
        let key = key
            .as_str()
            .ok_or_else(|| format!("{prefix} field names must be strings"))?;
        if ![
            "window_process",
            "window_class",
            "window_title",
            "window_app_id",
        ]
        .contains(&key)
        {
            return Err(format!("{prefix} contains unknown match field {key:?}"));
        }
    }
    let process = base_selector(mapping, "window_process", &prefix)?;
    let class = base_selector(mapping, "window_class", &prefix)?;
    let title = base_selector(mapping, "window_title", &prefix)?;
    let app_id = base_selector(mapping, "window_app_id", &prefix)?;
    if process.is_none() && class.is_none() && title.is_none() && app_id.is_none() {
        return Err(format!(
            "{prefix} needs a process, class, title, or app ID matcher"
        ));
    }
    Ok(BaseWindowMatch {
        process,
        class,
        title,
        app_id,
    })
}

fn base_selector(
    mapping: &serde_yaml_ng::Mapping,
    field: &str,
    prefix: &str,
) -> Result<Option<BaseSelector>, String> {
    let key = serde_yaml_ng::Value::String(field.into());
    let Some(value) = mapping.get(&key) else {
        return Ok(None);
    };
    let selector = value
        .as_mapping()
        .ok_or_else(|| format!("{prefix}.{field} must be a matcher mapping"))?;
    if selector.len() != 1 {
        return Err(format!(
            "{prefix}.{field} must contain exactly one GlazeWM matcher"
        ));
    }
    let (kind, value) = selector
        .iter()
        .next()
        .expect("a one-item mapping has a first item");
    let kind = kind
        .as_str()
        .ok_or_else(|| format!("{prefix}.{field} matcher name must be a string"))?;
    let value = value
        .as_str()
        .ok_or_else(|| format!("{prefix}.{field}.{kind} must be a string"))?
        .to_string();
    if value.trim().is_empty() {
        return Err(format!("{prefix}.{field}.{kind} must not be empty"));
    }
    let selector = match kind {
        "equals" => BaseSelector::Equals(value),
        "includes" => BaseSelector::Includes(value),
        "regex" => BaseSelector::Regex(value),
        "not_equals" => BaseSelector::NotEquals(value),
        "not_regex" => BaseSelector::NotRegex(value),
        _ => return Err(format!("{prefix}.{field} uses unknown matcher {kind:?}")),
    };
    Ok(Some(selector))
}

fn validate_apps(
    apps: &[AppWindowRule],
    workspaces: &HashSet<String>,
    base_rules: &[BaseWindowRule],
) -> Result<(), String> {
    let mut names = HashSet::new();
    for app in apps {
        if app.name.trim().is_empty() {
            return Err("app rule name must not be empty".into());
        }
        if !names.insert(app.name.as_str()) {
            return Err(format!("app rule name {:?} is duplicated", app.name));
        }
        if app.process.trim().is_empty() {
            return Err(format!("app {:?} process must not be empty", app.name));
        }
        for (field, value) in [
            ("class", &app.class),
            ("title", &app.title),
            ("app ID", &app.app_id),
        ] {
            if value
                .as_deref()
                .is_some_and(|value| value.trim().is_empty())
            {
                return Err(format!("app {:?} {field} must not be empty", app.name));
            }
        }
        if let Some(workspace) = &app.workspace
            && !workspaces.contains(workspace)
        {
            return Err(format!(
                "app {:?} uses workspace {:?}, which the GlazeWM base does not declare",
                app.name, workspace
            ));
        }
        if app.state == WindowRuleState::Ignored && app.workspace.is_some() {
            return Err(format!(
                "app {:?} is ignored and cannot have a workspace home",
                app.name
            ));
        }
    }

    for (index, left) in apps.iter().enumerate() {
        for right in &apps[index + 1..] {
            validate_app_overlap(left, right)?;
        }
    }
    for app in apps {
        validate_base_overlap(app, base_rules)?;
    }
    Ok(())
}

fn validate_app_overlap(left: &AppWindowRule, right: &AppWindowRule) -> Result<(), String> {
    if !app_matches_overlap(left, right) {
        return Ok(());
    }
    if app_matches_equal(left, right) {
        return Err(format!(
            "apps {:?} and {:?} own the same GlazeWM match",
            left.name, right.name
        ));
    }
    if left.state == right.state && left.workspace == right.workspace {
        return Ok(());
    }
    if ignore_exception_precedes(left, right) || ignore_exception_precedes(right, left) {
        return Ok(());
    }
    Err(format!(
        "apps {:?} and {:?} have overlapping matches with contradictory state/workspace policy",
        left.name, right.name
    ))
}

fn validate_base_overlap(app: &AppWindowRule, rules: &[BaseWindowRule]) -> Result<(), String> {
    for rule in rules {
        for base_match in &rule.matches {
            if !base_match_overlaps_app(base_match, app) {
                continue;
            }
            let base_is_narrow_ignore = rule.commands == ["ignore"]
                && base_match_specificity(base_match) > app_specificity(app);
            if base_is_narrow_ignore {
                continue;
            }
            return Err(format!(
                "app {:?} overlaps a base window_rules match; keep that policy in one source",
                app.name
            ));
        }
    }
    Ok(())
}

fn rendered_window_rule(app: &AppWindowRule) -> RenderedWindowRule<'_> {
    let mut commands = Vec::new();
    if let Some(workspace) = &app.workspace {
        commands.push(format!("move --workspace {workspace}"));
    }
    commands.push(
        match app.state {
            WindowRuleState::Tiling => "set-tiling",
            WindowRuleState::Floating => "set-floating",
            WindowRuleState::Ignored => "ignore",
        }
        .to_string(),
    );
    RenderedWindowRule {
        commands,
        match_window: vec![RenderedWindowMatch {
            window_process: EqualsMatch {
                equals: &app.process,
            },
            window_class: app.class.as_deref().map(|equals| EqualsMatch { equals }),
            window_title: app.title.as_deref().map(|equals| EqualsMatch { equals }),
            window_app_id: app.app_id.as_deref().map(|equals| EqualsMatch { equals }),
        }],
        // Homes are manage-time policy. Glaze defaults rules to manage plus
        // title changes, which can visibly move a window after its provisional
        // title settles. Never inherit that default here.
        on: ["manage"],
    }
}

fn app_order(app: &AppWindowRule) -> (u8, std::cmp::Reverse<usize>) {
    (
        u8::from(app.state != WindowRuleState::Ignored),
        std::cmp::Reverse(app_specificity(app)),
    )
}

fn app_specificity(app: &AppWindowRule) -> usize {
    1 + usize::from(app.class.is_some())
        + usize::from(app.title.is_some())
        + usize::from(app.app_id.is_some())
}

fn base_match_specificity(base_match: &BaseWindowMatch) -> usize {
    usize::from(base_match.process.is_some())
        + usize::from(base_match.class.is_some())
        + usize::from(base_match.title.is_some())
        + usize::from(base_match.app_id.is_some())
}

fn app_matches_overlap(left: &AppWindowRule, right: &AppWindowRule) -> bool {
    left.process == right.process
        && selectors_overlap(left.class.as_deref(), right.class.as_deref())
        && selectors_overlap(left.title.as_deref(), right.title.as_deref())
        && selectors_overlap(left.app_id.as_deref(), right.app_id.as_deref())
}

fn base_match_overlaps_app(base_match: &BaseWindowMatch, app: &AppWindowRule) -> bool {
    base_selector_overlaps_exact(base_match.process.as_ref(), Some(&app.process))
        && base_selector_overlaps_exact(base_match.class.as_ref(), app.class.as_deref())
        && base_selector_overlaps_exact(base_match.title.as_ref(), app.title.as_deref())
        && base_selector_overlaps_exact(base_match.app_id.as_ref(), app.app_id.as_deref())
}

fn base_selector_overlaps_exact(base: Option<&BaseSelector>, app: Option<&str>) -> bool {
    let Some(base) = base else {
        return true;
    };
    let Some(app) = app else {
        // The generated rule leaves this property unconstrained. Any supported
        // base matcher might therefore intersect it; reject unless a decisive
        // narrow ignore owns the overlap.
        return true;
    };
    match base {
        BaseSelector::Equals(value) => value == app,
        BaseSelector::Includes(value) => app.contains(value),
        // Proving arbitrary regex intersection is the wrong problem here.
        // Generated selectors are exact, so unknown regex behavior is treated
        // as overlap and the two policy sources must be reconciled explicitly.
        BaseSelector::Regex(value) | BaseSelector::NotRegex(value) => {
            let _ = value;
            true
        }
        BaseSelector::NotEquals(value) => value != app,
    }
}

fn selectors_overlap(left: Option<&str>, right: Option<&str>) -> bool {
    left.is_none() || right.is_none() || left == right
}

fn app_matches_equal(left: &AppWindowRule, right: &AppWindowRule) -> bool {
    left.process == right.process
        && left.class == right.class
        && left.title == right.title
        && left.app_id == right.app_id
}

fn ignore_exception_precedes(exception: &AppWindowRule, broad: &AppWindowRule) -> bool {
    exception.state == WindowRuleState::Ignored
        && app_specificity(exception) > app_specificity(broad)
}

fn yaml_list(items: &[String]) -> String {
    let quoted: Vec<String> = items
        .iter()
        .map(|s| format!("'{}'", s.replace('\'', "''")))
        .collect();
    quoted.join(", ")
}

fn validate_shape(bind: &Bind) -> Result<(), String> {
    match (&bind.chord, &bind.chords) {
        (Some(_), Some(_)) => return Err("has both chord and chords".into()),
        (None, None) => return Err("needs chord or chords".into()),
        _ => {}
    }
    match (&bind.map, &bind.status) {
        (Some(_), Some(_)) => return Err("has both map and status".into()),
        (None, None) => return Err("needs map or status".into()),
        _ => {}
    }
    if bind.status.is_some() && bind.note.is_none() {
        return Err("status entries need a note".into());
    }
    if bind.chords.is_some() && bind.map.is_some() {
        return Err(
            "chords lists are for non-mapped entries; mapped entries bind one chord".into(),
        );
    }
    if bind.foreach.is_some() && bind.chords.is_some() {
        return Err("foreach expands a single chord template, not a chords list".into());
    }
    if let Some(f) = bind.foreach
        && f.from > f.to
    {
        return Err(format!("foreach range {}..{} is empty", f.from, f.to));
    }
    Ok(())
}

fn expand(bind: &Bind, apps: &BTreeMap<String, String>) -> Result<Vec<Expanded>, String> {
    let instances: Vec<(String, Option<u32>)> = match bind.foreach {
        Some(f) => (f.from..=f.to)
            .map(|n| (format!("{}-{n}", bind.id), Some(n)))
            .collect(),
        None => vec![(bind.id.clone(), None)],
    };

    let mut out = Vec::new();
    for (id, n) in instances {
        let subst = |s: &str| -> Result<String, String> {
            let mut s = s.to_string();
            if let Some(n) = n {
                s = s.replace("{n}", &n.to_string());
                s = s.replace("{digit}", &(n % 10).to_string());
            }
            for (name, cmd) in apps {
                s = s.replace(&format!("{{{name}}}"), cmd);
            }
            if let Some(pos) = s.find('{') {
                return Err(format!("unresolved placeholder in '{}'", &s[pos..]));
            }
            Ok(s)
        };

        let chords = match (&bind.chord, &bind.chords) {
            (Some(c), None) => vec![subst(c)?],
            (None, Some(cs)) => cs.clone(),
            _ => unreachable!("validated by validate_shape"),
        };
        let commands = match &bind.map {
            Some(cmds) => Some(cmds.iter().map(|c| subst(c)).collect::<Result<_, _>>()?),
            None => None,
        };
        out.push(Expanded {
            id,
            chords,
            desc: subst(&bind.desc)?,
            commands,
            status: bind.status,
        });
    }
    Ok(out)
}

/// Omarchy chord ("SUPER + SHIFT + LEFT") to GlazeWM binding ("rwin+shift+left").
///
/// SUPER becomes rwin — the v3 architecture: kanata maps Caps to right-Win and
/// GlazeWM binds rwin chords, leaving physical left Win fully native. Key
/// spellings verified against GlazeWM's own parser (wm-platform/src/models/key.rs).
/// Only mapped entries are translated, so the table covers exactly the keys the
/// mapped grammar uses; an unknown key is an error, never a guess.
pub fn translate_chord(chord: &str) -> Result<String, String> {
    let parts: Vec<&str> = chord.split('+').map(str::trim).collect();
    let (key_part, mod_parts) = parts
        .split_last()
        .ok_or_else(|| format!("empty chord '{chord}'"))?;

    let mut has = [false; 4]; // rwin, ctrl, alt, shift
    for m in mod_parts {
        let slot = match *m {
            "SUPER" => 0,
            "CTRL" => 1,
            "ALT" => 2,
            "SHIFT" => 3,
            other => return Err(format!("unknown modifier '{other}' in '{chord}'")),
        };
        if has[slot] {
            return Err(format!("repeated modifier '{m}' in '{chord}'"));
        }
        has[slot] = true;
    }

    let key = match *key_part {
        "LEFT" => "left",
        "RIGHT" => "right",
        "UP" => "up",
        "DOWN" => "down",
        "TAB" => "tab",
        "RETURN" => "enter",
        "SPACE" => "space",
        "ESCAPE" => "escape",
        "BACKSPACE" => "backspace",
        "code:20" => "oem_minus",
        "code:21" => "oem_plus",
        single if single.len() == 1 => {
            let c = single.chars().next().expect("len checked");
            if c.is_ascii_uppercase() || c.is_ascii_digit() {
                return Ok(assemble(&has, &c.to_ascii_lowercase().to_string()));
            }
            return Err(format!("unknown key '{single}' in '{chord}'"));
        }
        other => return Err(format!("unknown key '{other}' in '{chord}'")),
    };
    Ok(assemble(&has, key))
}

fn assemble(has: &[bool; 4], key: &str) -> String {
    const NAMES: [&str; 4] = ["rwin", "ctrl", "alt", "shift"];
    let mut parts: Vec<&str> = NAMES
        .iter()
        .zip(has)
        .filter_map(|(name, present)| present.then_some(*name))
        .collect();
    parts.push(key);
    parts.join("+")
}

fn coverage_of(expanded: &[Expanded]) -> Coverage {
    let mut cov = Coverage::default();
    for item in expanded {
        let chords = item.chords.len();
        cov.chords_total += chords;
        match item.status {
            None => {
                cov.mapped += 1;
                cov.chords_mapped += chords;
            }
            Some(Status::Gap) => cov.gap += 1,
            Some(Status::Native) => cov.native += 1,
            Some(Status::App) => cov.app += 1,
            Some(Status::Omitted) => cov.omitted += 1,
        }
    }
    cov
}

pub fn render_coverage(meta: &Meta, cov: &Coverage) -> String {
    format!(
        "keymap ok — {} {} (fetched {})\n\
         entries: {} mapped, {} gap, {} native, {} app, {} omitted\n\
         chords:  {} of {} bound in GlazeWM",
        meta.upstream,
        meta.upstream_ref,
        meta.fetched,
        cov.mapped,
        cov.gap,
        cov.native,
        cov.app,
        cov.omitted,
        cov.chords_mapped,
        cov.chords_total,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_bind(body: &str) -> KeymapFile {
        let text =
            format!("[meta]\nupstream = 'u'\nref = 'r'\nfiles = []\nfetched = 'd'\n\n{body}");
        parse(&text).expect("fixture parses")
    }

    #[test]
    fn translates_the_grammar_chords() {
        for (omarchy, glazewm) in [
            ("SUPER + W", "rwin+w"),
            ("SUPER + SHIFT + LEFT", "rwin+shift+left"),
            ("SUPER + SHIFT + ALT + 3", "rwin+alt+shift+3"),
            ("SUPER + CTRL + SHIFT + code:21", "rwin+ctrl+shift+oem_plus"),
            ("SUPER + RETURN", "rwin+enter"),
            ("SUPER + ESCAPE", "rwin+escape"),
        ] {
            assert_eq!(translate_chord(omarchy).unwrap(), glazewm);
        }
    }

    #[test]
    fn rejects_unknown_keys_and_mods() {
        assert!(translate_chord("SUPER + XF86AudioMute").is_err());
        assert!(translate_chord("HYPER + W").is_err());
        assert!(translate_chord("SUPER + mouse_down").is_err());
    }

    #[test]
    fn foreach_expands_with_digit_wrap() {
        let file = one_bind(
            "[[bind]]\nid = 'ws'\nchord = 'SUPER + {digit}'\ndesc = 'Workspace {n}'\nsrc = 't'\nmap = ['focus --workspace {n}']\nforeach = { from = 9, to = 10 }\n",
        );
        let (expanded, cov) = check(&file).expect("valid");
        assert_eq!(expanded.len(), 2);
        assert_eq!(expanded[0].chords, ["SUPER + 9"]);
        assert_eq!(expanded[1].chords, ["SUPER + 0"]);
        assert_eq!(
            expanded[1].commands.as_deref(),
            Some(&["focus --workspace 10".to_string()][..])
        );
        assert_eq!(cov.chords_mapped, 2);
    }

    #[test]
    fn app_placeholders_resolve_and_typos_fail() {
        let file = one_bind(
            "[apps]\nterminal = 'wt'\n\n[[bind]]\nid = 'term'\nchord = 'SUPER + RETURN'\ndesc = 'Terminal'\nsrc = 'a'\nmap = ['shell-exec {terminal}']\n",
        );
        let (expanded, _) = check(&file).expect("valid");
        assert_eq!(
            expanded[0].commands.as_deref(),
            Some(&["shell-exec wt".to_string()][..])
        );

        let bad = one_bind(
            "[[bind]]\nid = 'term'\nchord = 'SUPER + RETURN'\ndesc = 'Terminal'\nsrc = 'a'\nmap = ['shell-exec {terminl}']\n",
        );
        let errors = check(&bad).expect_err("typo caught");
        assert!(errors[0].contains("unresolved placeholder"));
    }

    #[test]
    fn duplicate_normalized_chords_conflict() {
        let file = one_bind(
            "[[bind]]\nid = 'a'\nchord = 'SUPER + W'\ndesc = 'x'\nsrc = 't'\nmap = ['close']\n\n[[bind]]\nid = 'b'\nchord = 'SUPER + W'\ndesc = 'y'\nsrc = 't'\nmap = ['close']\n",
        );
        let errors = check(&file).expect_err("conflict");
        assert!(
            errors.iter().any(|e| e.contains("already bound by a")),
            "{errors:?}"
        );
    }

    #[test]
    fn status_without_note_rejected() {
        let file = one_bind(
            "[[bind]]\nid = 'a'\nchord = 'SUPER + W'\ndesc = 'x'\nsrc = 't'\nstatus = 'gap'\n",
        );
        let errors = check(&file).expect_err("missing note");
        assert!(errors[0].contains("need a note"));
    }

    #[test]
    fn mapped_chords_list_rejected() {
        let file = one_bind(
            "[[bind]]\nid = 'a'\nchords = ['SUPER + W', 'SUPER + Q']\ndesc = 'x'\nsrc = 't'\nmap = ['close']\n",
        );
        assert!(check(&file).is_err());
    }

    #[test]
    fn merge_app_override_reaches_stock_binds() {
        // The Chrome-profile case: overriding {browser} retargets every stock
        // browser chord without touching a single bind entry.
        let mut base = one_bind(
            "[apps]\nbrowser = 'chrome'\n\n[[bind]]\nid = 'app-browser'\nchord = 'SUPER + SHIFT + RETURN'\ndesc = 'Browser'\nsrc = 'a'\nmap = ['shell-exec {browser}']\n",
        );
        let local = parse_overrides("[apps]\nbrowser = 'chrome --profile-directory=Default'\n")
            .expect("overrides parse");
        let report = merge(&mut base, local).expect("merges");
        assert_eq!(report.apps_set, 1);
        let (expanded, _) = check(&base).expect("valid");
        assert_eq!(
            expanded[0].commands.as_deref(),
            Some(&["shell-exec chrome --profile-directory=Default".to_string()][..])
        );
    }

    #[test]
    fn merge_adds_replaces_and_disables() {
        let mut base = one_bind(
            "[[bind]]\nid = 'a'\nchord = 'SUPER + W'\ndesc = 'x'\nsrc = 't'\nmap = ['close']\n\n[[bind]]\nid = 'b'\nchord = 'SUPER + Q'\ndesc = 'y'\nsrc = 't'\nmap = ['close']\n",
        );
        let local = parse_overrides(
            "disable = ['b']\n\n[[bind]]\nid = 'a'\nchord = 'SUPER + C'\ndesc = 'x2'\nsrc = 'local'\nmap = ['close']\n\n[[bind]]\nid = 'new'\nchord = 'SUPER + N'\ndesc = 'n'\nsrc = 'local'\nmap = ['close']\n",
        )
        .expect("overrides parse");
        let report = merge(&mut base, local).expect("merges");
        assert_eq!(
            (
                report.binds_replaced,
                report.binds_added,
                report.binds_disabled
            ),
            (1, 1, 1)
        );
        // Replacement holds the original position; the add is appended.
        assert_eq!(base.binds[0].id, "a");
        assert_eq!(base.binds[0].chord.as_deref(), Some("SUPER + C"));
        assert_eq!(base.binds[1].id, "new");
        check(&base).expect("merged file validates");
    }

    #[test]
    fn merge_rejects_dangling_and_ambiguous_disables() {
        let mut base = one_bind(
            "[[bind]]\nid = 'a'\nchord = 'SUPER + W'\ndesc = 'x'\nsrc = 't'\nmap = ['close']\n",
        );
        let local = parse_overrides("disable = ['typo']\n").expect("parses");
        let errors = merge(&mut base, local).expect_err("dangling disable");
        assert!(errors[0].contains("names no stock bind"), "{errors:?}");

        let local = parse_overrides(
            "disable = ['a']\n\n[[bind]]\nid = 'a'\nchord = 'SUPER + C'\ndesc = 'x'\nsrc = 'l'\nmap = ['close']\n",
        )
        .expect("parses");
        let errors = merge(&mut base, local).expect_err("disable + rebind");
        assert!(
            errors[0].contains("both disabled and re-bound"),
            "{errors:?}"
        );
    }

    #[test]
    fn merged_local_bind_on_taken_chord_conflicts_in_check() {
        let mut base = one_bind(
            "[[bind]]\nid = 'a'\nchord = 'SUPER + W'\ndesc = 'x'\nsrc = 't'\nmap = ['close']\n",
        );
        let local = parse_overrides(
            "[[bind]]\nid = 'mine'\nchord = 'SUPER + W'\ndesc = 'grab'\nsrc = 'local'\nmap = ['close']\n",
        )
        .expect("parses");
        merge(&mut base, local).expect("merge itself is dumb");
        let errors = check(&base).expect_err("check owns conflicts");
        assert!(
            errors.iter().any(|e| e.contains("already bound by a")),
            "{errors:?}"
        );
    }

    #[test]
    fn render_shape() {
        let file = one_bind(
            "[[bind]]\nid = 'close'\nchord = 'SUPER + W'\ndesc = 'Close window'\nsrc = 't'\nmap = ['close']\n",
        );
        let (expanded, _) = check(&file).expect("valid");
        let yaml = render_glazewm(&expanded);
        assert!(
            yaml.contains(
                "  # close — Close window\n  - commands: ['close']\n    bindings: ['rwin+w']\n"
            ),
            "{yaml}"
        );
    }

    #[test]
    fn full_config_composes_base_and_generated_bindings() {
        let file = one_bind(
            "[[bind]]\nid = 'close'\nchord = 'SUPER + W'\ndesc = 'Close window'\nsrc = 't'\nmap = ['close']\n",
        );
        let (expanded, _) = check(&file).expect("valid");
        let yaml = render_glazewm_config(
            "\u{feff}general:\r\n  focus_follows_cursor: false\r\n",
            &expanded,
            &[],
        )
        .expect("full config renders");
        assert_eq!(
            yaml,
            concat!(
                "# Generated by `winmakase keymap render --base`; edit the base/keymap sources, not this file.\n",
                "general:\n",
                "  focus_follows_cursor: false\n\n",
                "keybindings:\n",
                "  # close — Close window\n",
                "  - commands: ['close']\n",
                "    bindings: ['rwin+w']\n",
            )
        );
    }

    #[test]
    fn full_config_rejects_a_second_keybindings_owner() {
        let file = one_bind(
            "[[bind]]\nid = 'close'\nchord = 'SUPER + W'\ndesc = 'Close window'\nsrc = 't'\nmap = ['close']\n",
        );
        let (expanded, _) = check(&file).expect("valid");
        let error = render_glazewm_config("general: {}\nkeybindings:\n", &expanded, &[])
            .expect_err("base bindings must be rejected");
        assert!(error.contains("line 2 contains keybindings:"), "{error}");

        let inline = render_glazewm_config("general: {}\nkeybindings: []\n", &expanded, &[])
            .expect_err("inline base bindings must be rejected");
        assert!(inline.contains("line 2 contains keybindings:"), "{inline}");
    }
}
