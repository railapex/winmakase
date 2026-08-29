#[path = "common/mod.rs"]
mod common;

use std::fs;

use winsome_theme::palette::{Mode, Palette, load_palette};
use winsome_theme::{is_light_theme, relative_luminance};

// --- relativeLuminance ---

#[test]
fn tokyo_night_background_is_dark_below_the_light_threshold() {
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap();
    assert!(relative_luminance(&p.background).unwrap() < 0.5);
}

#[test]
fn a_light_hex_is_above_the_light_threshold() {
    assert!(relative_luminance("#ffffff").unwrap() >= 0.5);
}

#[test]
fn rejects_a_non_hex_string() {
    let err = relative_luminance("not-a-color").unwrap_err();
    assert!(err.to_string().contains("not a 6-digit hex color"), "{err}");
}

// --- isLightTheme ---

#[test]
fn is_false_for_a_dark_theme_with_no_marker_file_mode_driven() {
    let dir = common::TempDir::new("mode-dark");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap(); // mode: dark
    assert!(!is_light_theme(dir.path(), &p).unwrap());
}

#[test]
fn is_true_for_a_theme_whose_mode_is_light_even_with_a_dark_background() {
    let dir = common::TempDir::new("mode-light");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap();
    let mut light_mode = p.clone();
    light_mode.mode = Some(Mode::Light);
    assert!(is_light_theme(dir.path(), &light_mode).unwrap());
}

#[test]
fn palette_mode_wins_over_a_light_mode_marker_file() {
    let dir = common::TempDir::new("mode-wins");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap(); // mode: dark
    fs::write(dir.path().join("light.mode"), "").unwrap();
    assert!(!is_light_theme(dir.path(), &p).unwrap());
}

#[test]
fn falls_back_to_background_luminance_when_palette_mode_is_absent() {
    let dir = common::TempDir::new("fallback-luminance");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap();

    let modeless: Palette = Palette {
        mode: None,
        ..p.clone()
    };
    assert!(!is_light_theme(dir.path(), &modeless).unwrap());

    let light_background: Palette = Palette {
        mode: None,
        background: "#ffffff".to_string(),
        ..p.clone()
    };
    assert!(is_light_theme(dir.path(), &light_background).unwrap());
}

#[test]
fn a_light_mode_marker_file_wins_over_luminance_when_mode_is_absent() {
    let dir = common::TempDir::new("marker-wins");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap(); // dark background
    let modeless: Palette = Palette {
        mode: None,
        ..p.clone()
    };
    fs::write(dir.path().join("light.mode"), "").unwrap();
    assert!(dir.path().join("light.mode").exists());
    assert!(is_light_theme(dir.path(), &modeless).unwrap());
}

#[test]
fn errors_when_mode_is_absent_no_marker_file_and_background_is_invalid_hex() {
    // Mirrors the TS original: relativeLuminance's throw isn't caught inside isLightTheme,
    // so an invalid background hex propagates out as a loud failure, not a silent "dark".
    let dir = common::TempDir::new("invalid-background");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap();
    let bad_background: Palette = Palette {
        mode: None,
        background: "not-a-color".to_string(),
        ..p.clone()
    };
    let err = is_light_theme(dir.path(), &bad_background).unwrap_err();
    assert!(err.to_string().contains("not a 6-digit hex color"), "{err}");
}
