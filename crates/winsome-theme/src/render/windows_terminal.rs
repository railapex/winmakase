use serde::Serialize;

use crate::palette::Palette;

/// Windows Terminal color scheme, as consumed by WT's `schemes` array
/// (https://learn.microsoft.com/en-us/windows/terminal/customize-settings/color-schemes).
/// WT calls ANSI magenta "purple" — field names below follow WT's convention, not ANSI's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsTerminalScheme {
    pub name: String,
    pub background: String,
    pub foreground: String,
    pub cursor_color: String,
    pub selection_background: String,
    pub black: String,
    pub red: String,
    pub green: String,
    pub yellow: String,
    pub blue: String,
    pub purple: String,
    pub cyan: String,
    pub white: String,
    pub bright_black: String,
    pub bright_red: String,
    pub bright_green: String,
    pub bright_yellow: String,
    pub bright_blue: String,
    pub bright_purple: String,
    pub bright_cyan: String,
    pub bright_white: String,
}

/// Render an omarchy Palette (v4.0.1 schema — see palette.rs) to a Windows Terminal scheme.
///
/// The v4.0.1 palette has no color0-15 ANSI slots to map 1:1 (that was the v3.7.0 shape).
/// The mapping below mirrors omarchy's own terminal template exactly, not an invented
/// convention — verbatim from `default/themed/ghostty.conf.tpl` at the v4.0.1 tag
/// (https://raw.githubusercontent.com/basecamp/omarchy/v4.0.1/default/themed/ghostty.conf.tpl):
///   palette 0=background, 1=red, 2=green, 3=yellow, 4=blue, 5=magenta, 6=cyan, 7=foreground,
///   8=muted, 9=bright_red, 10=bright_green, 11=bright_yellow, 12=bright_blue,
///   13=bright_magenta, 14=bright_cyan, 15=bright_foreground; cursor-color=bright_foreground.
/// (`bin/omarchy-theme-color`'s `ansi_alias` table — the resolver that template is generated
/// from — carries the identical mapping, confirming it's not a ghostty-only quirk.)
///
/// `orange` and `brown` are unused here by design — they don't feed the terminal palette,
/// only later non-terminal render targets.
///
/// selectionBackground: the tpl also references `{{ selection_background }}` /
/// `{{ selection_foreground }}`, neither a colors.toml key. Per `omarchy-theme-color`'s
/// resolver (called from `bin/omarchy-theme-set-templates`, which is what actually expands
/// these templates): `selection_background` falls back to `selection` when unset, and every
/// v4.0.1 theme leaves it unset — so `selection` is the real value, not a provisional guess.
/// (`selection_foreground` likewise falls back to `bright_foreground`; WT's scheme format has
/// no selectionForeground field, so that one has nowhere to go yet.)
pub fn render_windows_terminal_scheme(name: &str, palette: &Palette) -> WindowsTerminalScheme {
    WindowsTerminalScheme {
        name: name.to_string(),
        background: palette.background.clone(),
        foreground: palette.foreground.clone(),
        cursor_color: palette.bright_foreground.clone(),
        selection_background: palette.selection.clone(),
        black: palette.background.clone(),
        red: palette.red.clone(),
        green: palette.green.clone(),
        yellow: palette.yellow.clone(),
        blue: palette.blue.clone(),
        purple: palette.magenta.clone(),
        cyan: palette.cyan.clone(),
        white: palette.foreground.clone(),
        bright_black: palette.muted.clone(),
        bright_red: palette.bright_red.clone(),
        bright_green: palette.bright_green.clone(),
        bright_yellow: palette.bright_yellow.clone(),
        bright_blue: palette.bright_blue.clone(),
        bright_purple: palette.bright_magenta.clone(),
        bright_cyan: palette.bright_cyan.clone(),
        bright_white: palette.bright_foreground.clone(),
    }
}
