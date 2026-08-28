import type { Palette } from '../palette.js';

/**
 * Windows Terminal color scheme, as consumed by WT's `schemes` array
 * (https://learn.microsoft.com/en-us/windows/terminal/customize-settings/color-schemes).
 * WT calls ANSI magenta "purple" — field names below follow WT's convention, not ANSI's.
 */
export interface WindowsTerminalScheme {
  name: string;
  background: string;
  foreground: string;
  cursorColor: string;
  selectionBackground: string;
  black: string;
  red: string;
  green: string;
  yellow: string;
  blue: string;
  purple: string;
  cyan: string;
  white: string;
  brightBlack: string;
  brightRed: string;
  brightGreen: string;
  brightYellow: string;
  brightBlue: string;
  brightPurple: string;
  brightCyan: string;
  brightWhite: string;
}

/**
 * Render an omarchy Palette (v4.0.1 schema — see palette.ts) to a Windows Terminal scheme.
 *
 * The v4.0.1 palette has no color0-15 ANSI slots to map 1:1 (that was the v3.7.0 shape).
 * The mapping below mirrors omarchy's own terminal template exactly, not an invented
 * convention — verbatim from `default/themed/ghostty.conf.tpl` at the v4.0.1 tag
 * (https://raw.githubusercontent.com/basecamp/omarchy/v4.0.1/default/themed/ghostty.conf.tpl):
 *   palette 0=background, 1=red, 2=green, 3=yellow, 4=blue, 5=magenta, 6=cyan, 7=foreground,
 *   8=muted, 9=bright_red, 10=bright_green, 11=bright_yellow, 12=bright_blue,
 *   13=bright_magenta, 14=bright_cyan, 15=bright_foreground; cursor-color=bright_foreground.
 * (`bin/omarchy-theme-color`'s `ansi_alias` table — the resolver that template is generated
 * from — carries the identical mapping, confirming it's not a ghostty-only quirk.)
 *
 * `orange` and `brown` are unused here by design — they don't feed the terminal palette,
 * only later non-terminal render targets.
 *
 * selectionBackground: the tpl also references `{{ selection_background }}` /
 * `{{ selection_foreground }}`, neither a colors.toml key. Per `omarchy-theme-color`'s
 * resolver (called from `bin/omarchy-theme-set-templates`, which is what actually expands
 * these templates): `selection_background` falls back to `selection` when unset, and every
 * v4.0.1 theme leaves it unset — so `selection` is the real value, not a provisional guess.
 * (`selection_foreground` likewise falls back to `bright_foreground`; WT's scheme format has
 * no selectionForeground field, so that one has nowhere to go yet.)
 */
export function renderWindowsTerminalScheme(name: string, palette: Palette): WindowsTerminalScheme {
  return {
    name,
    background: palette.background,
    foreground: palette.foreground,
    cursorColor: palette.brightForeground,
    selectionBackground: palette.selection,
    black: palette.background,
    red: palette.red,
    green: palette.green,
    yellow: palette.yellow,
    blue: palette.blue,
    purple: palette.magenta,
    cyan: palette.cyan,
    white: palette.foreground,
    brightBlack: palette.muted,
    brightRed: palette.brightRed,
    brightGreen: palette.brightGreen,
    brightYellow: palette.brightYellow,
    brightBlue: palette.brightBlue,
    brightPurple: palette.brightMagenta,
    brightCyan: palette.brightCyan,
    brightWhite: palette.brightForeground,
  };
}
