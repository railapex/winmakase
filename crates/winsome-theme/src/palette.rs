use std::path::Path;

use serde::{Deserialize, Serialize};
use toml::Value as TomlValue;

use crate::error::ThemeError;

/// The two values omarchy's `mode` key can hold. A closed enum (not a bare `String`) so an
/// invalid value like `"purple"` is unrepresentable once past `parse_palette` — the TS
/// original relies on its `'dark' | 'light'` union plus runtime validation for the same
/// guarantee; this makes it a type-system invariant instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Dark,
    Light,
}

/// Palette shape for an omarchy theme's colors.toml, as shipped by omarchy v4.0.1 (the
/// "Quattro" palette redesign) and documented in DESIGN.md's theme-adapter section.
/// Verified directly against the raw v4.0.1 colors.toml for tokyo-night, catppuccin, nord,
/// gruvbox, rose-pine, kanagawa, and everforest — schema is uniform, no variants seen. This
/// superseded the v3.7.0 shape (accent, cursor, foreground, background,
/// selection_foreground, selection_background, color0-15), which DESIGN.md briefly and
/// incorrectly described as identical to v4.0.1 before that was corrected.
///
/// `mode` is `Option<Mode>` rather than a required field: the normal path
/// (`parse_palette`/`load_palette`) always produces `Some(Mode::Dark)` or `Some(Mode::Light)`,
/// but the light/dark resolver's fallback branches (marker file, luminance) are exercised by
/// constructing a palette with `mode: None` directly — the TS tests do this by casting around
/// the type (`as unknown as Palette`); Rust has no such escape hatch, so the field itself has
/// to be optional to let tests build that state. `Mode` still stays a closed enum, so `None`
/// is the only invalid-relative-to-normal-parse state a `Palette` can hold — there's no way
/// to construct one with a mode value that isn't dark, light, or absent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette {
    pub mode: Option<Mode>,
    pub accent: String,
    pub selection: String,
    pub muted: String,
    pub background: String,
    pub dark_background: String,
    pub darker_background: String,
    pub lighter_background: String,
    pub foreground: String,
    pub dark_foreground: String,
    pub light_foreground: String,
    pub bright_foreground: String,
    pub red: String,
    pub yellow: String,
    pub orange: String,
    pub green: String,
    pub cyan: String,
    pub blue: String,
    pub magenta: String,
    pub brown: String,
    pub bright_red: String,
    pub bright_yellow: String,
    pub bright_green: String,
    pub bright_cyan: String,
    pub bright_blue: String,
    pub bright_magenta: String,
}

// omarchy ships 6-digit hex only (#rrggbb). 3-digit shorthand is a CSS convention omarchy
// doesn't use — treat it as invalid input rather than silently expanding it.
// pub(crate): also used by light_dark::relative_luminance, so the two validations can't drift.
pub(crate) fn is_hex6(s: &str) -> bool {
    let bytes = s.as_bytes();
    bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(|b| b.is_ascii_hexdigit())
}

fn fail(source_label: &str, detail: &str) -> ThemeError {
    ThemeError(format!("{source_label}: {detail}"))
}

/// Renders a TOML value the way JS's `JSON.stringify(value ?? null)` would for the simple
/// scalar cases the error messages actually need (a bad `mode` string, or `None` for a
/// missing key). Not a general JSON encoder — this crate has no reason to serialize
/// TOML tables/arrays into error text.
fn describe(value: Option<&TomlValue>) -> String {
    match value {
        None => "null".to_string(),
        Some(TomlValue::String(s)) => format!("{s:?}"),
        Some(TomlValue::Integer(i)) => i.to_string(),
        Some(TomlValue::Float(f)) => f.to_string(),
        Some(TomlValue::Boolean(b)) => b.to_string(),
        Some(TomlValue::Datetime(d)) => format!("{d:?}"),
        Some(TomlValue::Array(_)) => "[...]".to_string(),
        Some(TomlValue::Table(_)) => "{...}".to_string(),
    }
}

macro_rules! hex_field {
    ($table:expr, $toml_key:literal, $source_label:expr) => {{
        match $table.get($toml_key) {
            None => {
                return Err(fail(
                    $source_label,
                    &format!("missing key \"{}\"", $toml_key),
                ));
            }
            Some(TomlValue::String(s)) if is_hex6(s) => s.to_lowercase(),
            other => {
                return Err(fail(
                    $source_label,
                    &format!(
                        "field \"{}\" is not a 6-digit hex color (#rrggbb): {}",
                        $toml_key,
                        describe(other)
                    ),
                ));
            }
        }
    }};
}

/// Parse an omarchy colors.toml document into a validated Palette.
/// Pure — does no file I/O. Errors on any missing key, unparseable TOML, or non-hex value;
/// never returns a partial or defaulted palette.
pub fn parse_palette(toml_text: &str, source_label: &str) -> Result<Palette, ThemeError> {
    // toml::from_str::<Table> parses a *document*; `str::parse::<Value>` was
    // redefined in toml 0.9 to parse a single value and rejects documents.
    let table: toml::Table = toml::from_str(toml_text)
        .map_err(|e: toml::de::Error| fail(source_label, &format!("could not parse TOML: {e}")))?;

    let mode = match table.get("mode") {
        Some(TomlValue::String(s)) if s == "dark" => Mode::Dark,
        Some(TomlValue::String(s)) if s == "light" => Mode::Light,
        other => {
            return Err(fail(
                source_label,
                &format!(
                    "field \"mode\" must be \"dark\" or \"light\", got {}",
                    describe(other)
                ),
            ));
        }
    };

    Ok(Palette {
        mode: Some(mode),
        accent: hex_field!(table, "accent", source_label),
        selection: hex_field!(table, "selection", source_label),
        muted: hex_field!(table, "muted", source_label),
        background: hex_field!(table, "background", source_label),
        dark_background: hex_field!(table, "dark_background", source_label),
        darker_background: hex_field!(table, "darker_background", source_label),
        lighter_background: hex_field!(table, "lighter_background", source_label),
        foreground: hex_field!(table, "foreground", source_label),
        dark_foreground: hex_field!(table, "dark_foreground", source_label),
        light_foreground: hex_field!(table, "light_foreground", source_label),
        bright_foreground: hex_field!(table, "bright_foreground", source_label),
        red: hex_field!(table, "red", source_label),
        yellow: hex_field!(table, "yellow", source_label),
        orange: hex_field!(table, "orange", source_label),
        green: hex_field!(table, "green", source_label),
        cyan: hex_field!(table, "cyan", source_label),
        blue: hex_field!(table, "blue", source_label),
        magenta: hex_field!(table, "magenta", source_label),
        brown: hex_field!(table, "brown", source_label),
        bright_red: hex_field!(table, "bright_red", source_label),
        bright_yellow: hex_field!(table, "bright_yellow", source_label),
        bright_green: hex_field!(table, "bright_green", source_label),
        bright_cyan: hex_field!(table, "bright_cyan", source_label),
        bright_blue: hex_field!(table, "bright_blue", source_label),
        bright_magenta: hex_field!(table, "bright_magenta", source_label),
    })
}

/// Read and parse `colors.toml` from an omarchy theme directory.
pub fn load_palette(theme_dir: &Path) -> Result<Palette, ThemeError> {
    let colors_path = theme_dir.join("colors.toml");
    if !colors_path.exists() {
        return Err(fail(
            &theme_dir.display().to_string(),
            &format!("no colors.toml found (expected {})", colors_path.display()),
        ));
    }
    let text = std::fs::read_to_string(&colors_path).map_err(|e| {
        fail(
            &colors_path.display().to_string(),
            &format!("could not read file: {e}"),
        )
    })?;
    parse_palette(&text, &colors_path.display().to_string())
}
