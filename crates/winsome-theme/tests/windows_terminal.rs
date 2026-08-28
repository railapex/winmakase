#[path = "common/mod.rs"]
mod common;

use std::fs;

use winsome_theme::palette::load_palette;
use winsome_theme::render::windows_terminal::render_windows_terminal_scheme;

fn golden(name: &str) -> serde_json::Value {
    let text = fs::read_to_string(common::golden_dir().join(format!("{name}.json"))).unwrap();
    serde_json::from_str(&text).unwrap()
}

// --- golden files ---
// Ports the vitest `toMatchSnapshot()` cases against `tests/render/__snapshots__/windows-
// terminal.test.ts.snap`; the golden JSON files here hold the same scheme-object values,
// extracted from that committed snapshot rather than re-derived from the mapping under test.

#[test]
fn tokyo_night_matches_its_committed_golden_file() {
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap();
    let scheme = render_windows_terminal_scheme("tokyo-night", &p);
    let actual = serde_json::to_value(&scheme).unwrap();
    assert_eq!(actual, golden("tokyo-night"));
}

#[test]
fn catppuccin_matches_its_committed_golden_file() {
    let p = load_palette(&common::fixtures_dir().join("catppuccin")).unwrap();
    let scheme = render_windows_terminal_scheme("catppuccin", &p);
    let actual = serde_json::to_value(&scheme).unwrap();
    assert_eq!(actual, golden("catppuccin"));
}

// --- mapping sanity ---

#[test]
fn uses_wt_naming_purple_not_magenta_and_direct_source_fields() {
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap();
    let scheme = render_windows_terminal_scheme("tokyo-night", &p);
    assert_eq!(scheme.name, "tokyo-night");
    assert_eq!(scheme.background, p.background);
    assert_eq!(scheme.foreground, p.foreground);
    assert_eq!(scheme.selection_background, p.selection);
    assert_eq!(scheme.cursor_color, p.bright_foreground);
    assert_eq!(scheme.purple, p.magenta);
    assert_eq!(scheme.bright_purple, p.bright_magenta);
    // TS asserts `expect(scheme).not.toHaveProperty('magenta')` at runtime. In Rust,
    // `WindowsTerminalScheme` simply has no `magenta` field — a compile-time guarantee
    // strictly stronger than the runtime check it replaces, so there's nothing to assert.
}

#[test]
fn maps_color0_7_8_15_per_omarchy_theme_colors_ansi_alias_table() {
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap();
    let scheme = render_windows_terminal_scheme("tokyo-night", &p);
    assert_eq!(scheme.black, p.background); // color0
    assert_eq!(scheme.white, p.foreground); // color7
    assert_eq!(scheme.bright_black, p.muted); // color8
    assert_eq!(scheme.bright_white, p.bright_foreground); // color15
}
