import type { Palette } from './palette.js';

/**
 * WCAG relative luminance of a #rrggbb color, in [0, 1].
 * https://www.w3.org/TR/WCAG21/#dfn-relative-luminance
 */
export function relativeLuminance(hex: string): number {
  const match = /^#([0-9a-fA-F]{2})([0-9a-fA-F]{2})([0-9a-fA-F]{2})$/.exec(hex);
  if (!match) {
    throw new Error(`relativeLuminance: not a 6-digit hex color: ${JSON.stringify(hex)}`);
  }
  const [r, g, b] = [match[1], match[2], match[3]].map((component) => {
    const channel = parseInt(component, 16) / 255;
    return channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

const LUMINANCE_LIGHT_THRESHOLD = 0.5;

/**
 * Decide whether a theme should drive Windows light mode.
 *
 * `palette.mode` — present on every omarchy v4.0.1 theme, required by parsePalette — is
 * authoritative, per DESIGN.md and omarchy's own resolver (`omarchy-theme-color`'s
 * `resolve_theme_mode`: `mode` short-circuits before anything else is even checked).
 * Relative-luminance-of-background is a legacy-only fallback for a palette built without
 * a `mode` (older schema, hand-built test fixtures) — with parsePalette in place, that
 * path never fires for a real theme; it exists so this function never has to guess from
 * nothing. No `light.mode` marker-file check: DESIGN.md's parenthetical mentioning one
 * predates the schema correction — mode already carries that signal directly now.
 *
 * `themeDir` isn't used by this function; kept to match the deliverable's given signature
 * (`isLightTheme(themeDir, palette)`) for callers that may want it later.
 */
export function isLightTheme(_themeDir: string, palette: Palette): boolean {
  if (palette.mode === 'light' || palette.mode === 'dark') {
    return palette.mode === 'light';
  }
  return relativeLuminance(palette.background) >= LUMINANCE_LIGHT_THRESHOLD;
}
