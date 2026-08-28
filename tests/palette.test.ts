import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { loadPalette, parsePalette, type Palette } from '../adapter/src/palette.js';

const FIXTURES = join(__dirname, 'fixtures');

const REQUIRED_HEX_FIELDS: (keyof Palette)[] = [
  'accent',
  'selection',
  'muted',
  'background',
  'darkBackground',
  'darkerBackground',
  'lighterBackground',
  'foreground',
  'darkForeground',
  'lightForeground',
  'brightForeground',
  'red',
  'yellow',
  'orange',
  'green',
  'cyan',
  'blue',
  'magenta',
  'brown',
  'brightRed',
  'brightYellow',
  'brightGreen',
  'brightCyan',
  'brightBlue',
  'brightMagenta',
];

const HEX_RE = /^#[0-9a-f]{6}$/;

describe('parsePalette / loadPalette — fixtures', () => {
  it('parses tokyo-night to a complete palette', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night'));
    expect(palette.mode).toBe('dark');
    expect(palette.background).toBe('#1a1b26');
    expect(palette.accent).toBe('#7aa2f7');
    for (const field of REQUIRED_HEX_FIELDS) {
      expect(palette[field], `field ${field}`).toMatch(HEX_RE);
    }
  });

  it('parses catppuccin to a complete palette', () => {
    const palette = loadPalette(join(FIXTURES, 'catppuccin'));
    expect(palette.mode).toBe('dark');
    expect(palette.background).toBe('#1e1e2e');
    expect(palette.accent).toBe('#89b4fa');
    for (const field of REQUIRED_HEX_FIELDS) {
      expect(palette[field], `field ${field}`).toMatch(HEX_RE);
    }
  });

  it('normalizes uppercase hex to lowercase', () => {
    const text = readFileSync(join(FIXTURES, 'tokyo-night', 'colors.toml'), 'utf-8').replace(
      '#1a1b26',
      '#1A1B26',
    );
    const palette = parsePalette(text, 'uppercase-fixture');
    expect(palette.background).toBe('#1a1b26');
  });
});

describe('parsePalette — fail-loud cases', () => {
  const tokyoNightText = readFileSync(join(FIXTURES, 'tokyo-night', 'colors.toml'), 'utf-8');

  it('throws on a missing key, naming the key and the source label', () => {
    const withoutRed = tokyoNightText
      .split('\n')
      .filter((line) => !line.startsWith('red ='))
      .join('\n');
    expect(() => parsePalette(withoutRed, 'broken-theme colors.toml')).toThrowError(
      /broken-theme colors\.toml.*missing key "red"/s,
    );
  });

  it('throws on a non-hex value, naming the field and the bad value', () => {
    const badHex = tokyoNightText.replace('#7aa2f7', 'not-a-color');
    expect(() => parsePalette(badHex, 'broken-theme colors.toml')).toThrowError(
      /broken-theme colors\.toml.*not a 6-digit hex color/s,
    );
  });

  it('throws on a 3-digit hex shorthand (omarchy ships 6-digit only)', () => {
    const shortHex = tokyoNightText.replace('#7aa2f7', '#7af');
    expect(() => parsePalette(shortHex, 'broken-theme colors.toml')).toThrowError(
      /not a 6-digit hex color/,
    );
  });

  it('throws on unparseable TOML', () => {
    expect(() => parsePalette('this = [is not, valid toml', 'garbled colors.toml')).toThrowError(
      /garbled colors\.toml.*could not parse TOML/s,
    );
  });

  it('throws on an empty file', () => {
    expect(() => parsePalette('', 'empty colors.toml')).toThrowError(/empty colors\.toml.*mode/s);
  });

  it('throws on a missing/invalid mode value', () => {
    const badMode = tokyoNightText.replace('mode = "dark"', 'mode = "midnight"');
    expect(() => parsePalette(badMode, 'broken-theme colors.toml')).toThrowError(
      /mode" must be "dark" or "light"/,
    );
  });
});

describe('loadPalette — fail-loud cases', () => {
  it('throws naming the theme path when colors.toml is missing', () => {
    const missingDir = join(FIXTURES, 'does-not-exist');
    expect(() => loadPalette(missingDir)).toThrowError(new RegExp(`no colors\\.toml found`));
  });
});
