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

use serde::Deserialize;

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

/// Compose a complete GlazeWM config from a machine/portable base and the
/// generated bindings. The base must deliberately omit `keybindings:`: a
/// replace-in-place merge would make ownership ambiguous and could preserve
/// stale hand-written bindings.
pub fn render_glazewm_config(base: &str, expanded: &[Expanded]) -> Result<String, String> {
    let normalized = base
        .strip_prefix('\u{feff}')
        .unwrap_or(base)
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    if normalized.trim().is_empty() {
        return Err("GlazeWM base is empty".into());
    }
    if let Some((line, _)) = normalized
        .lines()
        .enumerate()
        .find(|(_, text)| text.starts_with("keybindings:"))
    {
        return Err(format!(
            "GlazeWM base line {} contains keybindings:; remove that section because the keymap renderer owns it",
            line + 1
        ));
    }

    let bindings = render_glazewm(expanded);
    let (_, bindings_body) = bindings
        .split_once('\n')
        .expect("the generated keybindings header always ends with a newline");
    Ok(format!(
        "# Generated by `winmakase keymap render --base`; edit the base/keymap sources, not this file.\n{}\n\n{}",
        normalized.trim_end(),
        bindings_body
    ))
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
        let error = render_glazewm_config("general: {}\nkeybindings:\n", &expanded)
            .expect_err("base bindings must be rejected");
        assert!(error.contains("line 2 contains keybindings:"), "{error}");

        let inline = render_glazewm_config("general: {}\nkeybindings: []\n", &expanded)
            .expect_err("inline base bindings must be rejected");
        assert!(inline.contains("line 2 contains keybindings:"), "{inline}");
    }
}
