#[path = "common/mod.rs"]
mod common;

use std::fs;

use winsome_theme::palette::{load_palette, parse_palette, Palette};
use winsome_theme::ThemeError;

fn assert_all_hex_lowercase(p: &Palette) {
    let fields: [(&str, &str); 25] = [
        ("accent", &p.accent),
        ("selection", &p.selection),
        ("muted", &p.muted),
        ("background", &p.background),
        ("darkBackground", &p.dark_background),
        ("darkerBackground", &p.darker_background),
        ("lighterBackground", &p.lighter_background),
        ("foreground", &p.foreground),
        ("darkForeground", &p.dark_foreground),
        ("lightForeground", &p.light_foreground),
        ("brightForeground", &p.bright_foreground),
        ("red", &p.red),
        ("yellow", &p.yellow),
        ("orange", &p.orange),
        ("green", &p.green),
        ("cyan", &p.cyan),
        ("blue", &p.blue),
        ("magenta", &p.magenta),
        ("brown", &p.brown),
        ("brightRed", &p.bright_red),
        ("brightYellow", &p.bright_yellow),
        ("brightGreen", &p.bright_green),
        ("brightCyan", &p.bright_cyan),
        ("brightBlue", &p.bright_blue),
        ("brightMagenta", &p.bright_magenta),
    ];
    for (name, value) in fields {
        let bytes = value.as_bytes();
        let ok = bytes.len() == 7
            && bytes[0] == b'#'
            && bytes[1..].iter().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase());
        assert!(ok, "field {name} = {value:?} is not a lowercase 6-digit hex color");
    }
}

fn err_string<T: std::fmt::Debug>(result: Result<T, ThemeError>) -> String {
    result.unwrap_err().to_string()
}

// --- parsePalette / loadPalette — fixtures ---

#[test]
fn parses_tokyo_night_to_a_complete_palette() {
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap();
    assert_eq!(p.mode.as_deref(), Some("dark"));
    assert_eq!(p.background, "#1a1b26");
    assert_eq!(p.accent, "#7aa2f7");
    assert_all_hex_lowercase(&p);
}

#[test]
fn parses_catppuccin_to_a_complete_palette() {
    let p = load_palette(&common::fixtures_dir().join("catppuccin")).unwrap();
    assert_eq!(p.mode.as_deref(), Some("dark"));
    assert_eq!(p.background, "#1e1e2e");
    assert_eq!(p.accent, "#89b4fa");
    assert_all_hex_lowercase(&p);
}

#[test]
fn normalizes_uppercase_hex_to_lowercase() {
    let text = fs::read_to_string(common::fixtures_dir().join("tokyo-night/colors.toml"))
        .unwrap()
        .replace("#1a1b26", "#1A1B26");
    let p = parse_palette(&text, "uppercase-fixture").unwrap();
    assert_eq!(p.background, "#1a1b26");
}

// --- parsePalette — fail-loud cases ---

fn tokyo_night_text() -> String {
    fs::read_to_string(common::fixtures_dir().join("tokyo-night/colors.toml")).unwrap()
}

#[test]
fn throws_on_a_missing_key_naming_the_key_and_the_source_label() {
    let without_red: String = tokyo_night_text()
        .lines()
        .filter(|line| !line.starts_with("red ="))
        .collect::<Vec<_>>()
        .join("\n");
    let msg = err_string(parse_palette(&without_red, "broken-theme colors.toml"));
    assert!(msg.contains("broken-theme colors.toml"), "{msg}");
    assert!(msg.contains("missing key \"red\""), "{msg}");
}

#[test]
fn throws_on_a_non_hex_value_naming_the_field_and_the_bad_value() {
    let bad_hex = tokyo_night_text().replace("#7aa2f7", "not-a-color");
    let msg = err_string(parse_palette(&bad_hex, "broken-theme colors.toml"));
    assert!(msg.contains("broken-theme colors.toml"), "{msg}");
    assert!(msg.contains("not a 6-digit hex color"), "{msg}");
}

#[test]
fn throws_on_a_3_digit_hex_shorthand() {
    let short_hex = tokyo_night_text().replace("#7aa2f7", "#7af");
    let msg = err_string(parse_palette(&short_hex, "broken-theme colors.toml"));
    assert!(msg.contains("not a 6-digit hex color"), "{msg}");
}

#[test]
fn throws_on_unparseable_toml() {
    let msg = err_string(parse_palette("this = [is not, valid toml", "garbled colors.toml"));
    assert!(msg.contains("garbled colors.toml"), "{msg}");
    assert!(msg.contains("could not parse TOML"), "{msg}");
}

#[test]
fn throws_on_an_empty_file() {
    let msg = err_string(parse_palette("", "empty colors.toml"));
    assert!(msg.contains("empty colors.toml"), "{msg}");
    assert!(msg.contains("mode"), "{msg}");
}

#[test]
fn throws_on_a_missing_invalid_mode_value() {
    let bad_mode = tokyo_night_text().replace("mode = \"dark\"", "mode = \"midnight\"");
    let msg = err_string(parse_palette(&bad_mode, "broken-theme colors.toml"));
    assert!(msg.contains("mode\" must be \"dark\" or \"light\""), "{msg}");
}

// --- loadPalette — fail-loud cases ---

#[test]
fn throws_naming_the_theme_path_when_colors_toml_is_missing() {
    let missing_dir = common::fixtures_dir().join("does-not-exist");
    let msg = err_string(load_palette(&missing_dir));
    assert!(msg.contains("no colors.toml found"), "{msg}");
}
