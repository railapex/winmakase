#[path = "common/mod.rs"]
mod common;

use std::fs;

use winsome_theme::palette::{load_palette, Mode, Palette};
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
    let dir = common::make_temp_dir("mode-dark");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap(); // mode: dark
    assert!(!is_light_theme(&dir, &p));
    common::cleanup_temp_dir(&dir);
}

#[test]
fn is_true_for_a_theme_whose_mode_is_light_even_with_a_dark_background() {
    let dir = common::make_temp_dir("mode-light");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap();
    let mut light_mode = p.clone();
    light_mode.mode = Some(Mode::Light);
    assert!(is_light_theme(&dir, &light_mode));
    common::cleanup_temp_dir(&dir);
}

#[test]
fn palette_mode_wins_over_a_light_mode_marker_file() {
    let dir = common::make_temp_dir("mode-wins");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap(); // mode: dark
    fs::write(dir.join("light.mode"), "").unwrap();
    assert!(!is_light_theme(&dir, &p));
    common::cleanup_temp_dir(&dir);
}

#[test]
fn falls_back_to_background_luminance_when_palette_mode_is_absent() {
    let dir = common::make_temp_dir("fallback-luminance");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap();

    let modeless: Palette = Palette { mode: None, ..p.clone() };
    assert!(!is_light_theme(&dir, &modeless));

    let light_background: Palette =
        Palette { mode: None, background: "#ffffff".to_string(), ..p.clone() };
    assert!(is_light_theme(&dir, &light_background));

    common::cleanup_temp_dir(&dir);
}

#[test]
fn a_light_mode_marker_file_wins_over_luminance_when_mode_is_absent() {
    let dir = common::make_temp_dir("marker-wins");
    let p = load_palette(&common::fixtures_dir().join("tokyo-night")).unwrap(); // dark background
    let modeless: Palette = Palette { mode: None, ..p.clone() };
    fs::write(dir.join("light.mode"), "").unwrap();
    assert!(dir.join("light.mode").exists());
    assert!(is_light_theme(&dir, &modeless));
    common::cleanup_temp_dir(&dir);
}
