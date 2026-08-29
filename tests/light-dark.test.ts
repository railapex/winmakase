import { existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { isLightTheme, relativeLuminance } from '../adapter/src/light-dark.js';
import { loadPalette, type Palette } from '../adapter/src/palette.js';

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
  let tempDir: string;

  beforeEach(() => {
    tempDir = mkdtempSync(join(tmpdir(), 'winmakase-theme-'));
  });

  afterEach(() => {
    rmSync(tempDir, { recursive: true, force: true });
  });

  it('is false for a dark theme with no marker file (mode-driven)', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night')); // mode: dark
    expect(isLightTheme(tempDir, palette)).toBe(false);
  });

  it('is true for a theme whose mode is light, even with a dark background', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night'));
    const lightMode = { ...palette, mode: 'light' } as Palette;
    expect(isLightTheme(tempDir, lightMode)).toBe(true);
  });

  it('palette.mode wins over a light.mode marker file (mode short-circuits first)', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night')); // mode: dark
    writeFileSync(join(tempDir, 'light.mode'), '');
    expect(isLightTheme(tempDir, palette)).toBe(false);
  });

  it('falls back to background luminance when palette.mode is absent', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night'));
    const modeless = { ...palette, mode: undefined } as unknown as Palette;
    expect(isLightTheme(tempDir, modeless)).toBe(false);

    const lightBackground = { ...palette, mode: undefined, background: '#ffffff' } as unknown as Palette;
    expect(isLightTheme(tempDir, lightBackground)).toBe(true);
  });

  it('a light.mode marker file wins over luminance when mode is absent', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night')); // dark background
    const modeless = { ...palette, mode: undefined } as unknown as Palette;
    writeFileSync(join(tempDir, 'light.mode'), '');
    expect(existsSync(join(tempDir, 'light.mode'))).toBe(true);
    expect(isLightTheme(tempDir, modeless)).toBe(true);
  });
});
