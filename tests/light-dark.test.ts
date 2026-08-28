import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { isLightTheme, relativeLuminance } from '../adapter/src/light-dark.js';
import { loadPalette, type Palette } from '../adapter/src/palette.js';

// isLightTheme's themeDir param is unused (no marker-file check — see light-dark.ts);
// this placeholder documents that these tests never touch the filesystem for it.
const UNUSED_THEME_DIR = '(unused)';

const FIXTURES = join(__dirname, 'fixtures');

describe('relativeLuminance', () => {
  it('tokyo-night background is dark (below the light threshold)', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night'));
    expect(relativeLuminance(palette.background)).toBeLessThan(0.5);
  });

  it('a light hex is above the light threshold', () => {
    expect(relativeLuminance('#ffffff')).toBeGreaterThanOrEqual(0.5);
  });

  it('rejects a non-hex string', () => {
    expect(() => relativeLuminance('not-a-color')).toThrowError(/not a 6-digit hex color/);
  });
});

describe('isLightTheme', () => {
  it('is false for a dark theme (mode-driven)', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night')); // mode: dark
    expect(isLightTheme(UNUSED_THEME_DIR, palette)).toBe(false);
  });

  it('is true for a theme whose mode is light, even with a dark background', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night'));
    const lightMode = { ...palette, mode: 'light' } as Palette;
    expect(isLightTheme(UNUSED_THEME_DIR, lightMode)).toBe(true);
  });

  it('falls back to background luminance when palette.mode is absent', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night'));
    const modeless = { ...palette, mode: undefined } as unknown as Palette;
    expect(isLightTheme(UNUSED_THEME_DIR, modeless)).toBe(false);

    const lightBackground = { ...palette, mode: undefined, background: '#ffffff' } as unknown as Palette;
    expect(isLightTheme(UNUSED_THEME_DIR, lightBackground)).toBe(true);
  });
});
