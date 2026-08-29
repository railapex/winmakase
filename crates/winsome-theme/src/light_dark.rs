use std::path::Path;

use crate::error::ThemeError;
use crate::palette::{is_hex6, Mode, Palette};

/// WCAG relative luminance of a #rrggbb color, in [0, 1].
/// https://www.w3.org/TR/WCAG21/#dfn-relative-luminance
pub fn relative_luminance(hex: &str) -> Result<f64, ThemeError> {
    if !is_hex6(hex) {
        return Err(ThemeError(format!(
            "relativeLuminance: not a 6-digit hex color: {hex:?}"
        )));
    }

    let channel = |offset: usize| -> f64 {
        let component = &hex[offset..offset + 2];
        let raw = u8::from_str_radix(component, 16).expect("validated hex digit pair") as f64 / 255.0;
        if raw <= 0.03928 {
            raw / 12.92
        } else {
            ((raw + 0.055) / 1.055).powf(2.4)
        }
    };

    let r = channel(1);
    let g = channel(3);
    let b = channel(5);
    Ok(0.2126 * r + 0.7152 * g + 0.0722 * b)
}

const LUMINANCE_LIGHT_THRESHOLD: f64 = 0.5;

/// Decide whether a theme should drive Windows light mode.
///
/// Precedence mirrors omarchy's own resolver exactly — `omarchy-theme-color`'s
/// `resolve_theme_mode`: `palette.mode` (present on every omarchy v4.0.1 theme, required by
/// `parse_palette`) is authoritative and short-circuits before the marker file is even
/// checked; only when it's absent does a `light.mode` marker file in the theme directory
/// apply; relative-luminance-of-background is the last resort. All three tiers exist in the
/// canonical resolver for themes that aren't v4-shaped — that's the legacy-fallback story
/// this mirrors, not invention. In practice, since `parse_palette` requires `mode`, the
/// marker-file and luminance branches only fire for a palette built outside the normal load
/// path (older schema, hand-built test fixtures).
///
/// The luminance branch propagates `relative_luminance`'s error instead of defaulting to
/// dark: the TS original doesn't catch that throw either, so an invalid background hex here
/// is loud, not silently resolved — matches the repo's fail-loud rule.
pub fn is_light_theme(theme_dir: &Path, palette: &Palette) -> Result<bool, ThemeError> {
    match palette.mode {
        Some(Mode::Light) => return Ok(true),
        Some(Mode::Dark) => return Ok(false),
        None => {}
    }
    if theme_dir.join("light.mode").exists() {
        return Ok(true);
    }
    Ok(relative_luminance(&palette.background)? >= LUMINANCE_LIGHT_THRESHOLD)
}
