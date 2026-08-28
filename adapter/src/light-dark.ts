import { existsSync } from 'node:fs';
import { join } from 'node:path';
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
 * Precedence mirrors omarchy's own resolver exactly — `omarchy-theme-color`'s
 * `resolve_theme_mode`: `palette.mode` (present on every omarchy v4.0.1 theme, required by
 * parsePalette) is authoritative and short-circuits before the marker file is even
 * checked; only when it's absent does a `light.mode` marker file in the theme directory
 * apply; relative-luminance-of-background is the last resort, falling back to "dark" if
 * even that can't be read. All three tiers exist in the canonical resolver for themes
 * that aren't v4-shaped — that's the legacy-fallback story this mirrors, not invention.
 * In practice, since parsePalette requires `mode`, the marker-file and luminance branches
 * only fire for a palette built outside the normal load path (older schema, hand-built
 * test fixtures).
 */
export function isLightTheme(themeDir: string, palette: Palette): boolean {
  if (palette.mode === 'light' || palette.mode === 'dark') {
    return palette.mode === 'light';
  }
  if (existsSync(join(themeDir, 'light.mode'))) {
    return true;
  }
  return relativeLuminance(palette.background) >= LUMINANCE_LIGHT_THRESHOLD;
}
