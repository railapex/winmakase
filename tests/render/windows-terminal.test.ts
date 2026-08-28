import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { loadPalette } from '../../adapter/src/palette.js';
import { renderWindowsTerminalScheme } from '../../adapter/src/render/windows-terminal.js';

const FIXTURES = join(__dirname, '..', 'fixtures');

describe('renderWindowsTerminalScheme — golden files', () => {
  it('tokyo-night matches its committed snapshot', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night'));
    const scheme = renderWindowsTerminalScheme('tokyo-night', palette);
    expect(scheme).toMatchSnapshot();
  });

  it('catppuccin matches its committed snapshot', () => {
    const palette = loadPalette(join(FIXTURES, 'catppuccin'));
    const scheme = renderWindowsTerminalScheme('catppuccin', palette);
    expect(scheme).toMatchSnapshot();
  });
});

describe('renderWindowsTerminalScheme — mapping sanity', () => {
  it('uses WT naming (purple, not magenta) and direct-source fields', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night'));
    const scheme = renderWindowsTerminalScheme('tokyo-night', palette);
    expect(scheme.name).toBe('tokyo-night');
    expect(scheme.background).toBe(palette.background);
    expect(scheme.foreground).toBe(palette.foreground);
    expect(scheme.selectionBackground).toBe(palette.selection);
    expect(scheme.cursorColor).toBe(palette.brightForeground);
    expect(scheme.purple).toBe(palette.magenta);
    expect(scheme.brightPurple).toBe(palette.brightMagenta);
    expect(scheme).not.toHaveProperty('magenta');
  });

  it('maps color0/7/8/15 per omarchy-theme-color\'s ansi_alias table', () => {
    const palette = loadPalette(join(FIXTURES, 'tokyo-night'));
    const scheme = renderWindowsTerminalScheme('tokyo-night', palette);
    expect(scheme.black).toBe(palette.background); // color0
    expect(scheme.white).toBe(palette.foreground); // color7
    expect(scheme.brightBlack).toBe(palette.muted); // color8
    expect(scheme.brightWhite).toBe(palette.brightForeground); // color15
  });
});
