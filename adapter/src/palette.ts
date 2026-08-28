import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
// smol-toml: zero-dependency, actively maintained, spec-compliant (TOML 1.0.0) parser with
// native ESM output — a better fit than @iarna/toml (CJS-first, TOML spec gaps) for this
// project's `type: module` setup.
import { parse as parseToml } from 'smol-toml';

/**
 * Palette shape for an omarchy theme's colors.toml, as shipped by omarchy v4.0.1
 * (verified against tokyo-night, catppuccin, nord, gruvbox, rose-pine, kanagawa,
 * everforest — schema is uniform, no variants seen).
 *
 * NOTE: this is NOT the schema described in DESIGN.md (accent, cursor, foreground,
 * background, selection_foreground, selection_background, color0-15). That was the
 * omarchy v3.7.0 colors.toml shape. Omarchy redesigned the palette format before v4.0.0
 * shipped — cursor and the color0-15 ANSI slots are gone, selection_foreground/
 * selection_background collapsed to one `selection` key, and `mode` (dark|light) was
 * added directly to the file. DESIGN.md's "format verified identical across v3.7.0 and
 * v4.0.1" is stale. Parsing against the v3.7.0 shape here would throw "missing key" on
 * every real v4.0.1 theme, including both fixtures this adapter ships with — so this
 * module targets the real, current format instead. Flagged to team-lead; DESIGN.md
 * correction is out of this task's footprint (docs/ is off-limits).
 */
export interface Palette {
  mode: 'dark' | 'light';
  accent: string;
  selection: string;
  muted: string;
  background: string;
  darkBackground: string;
  darkerBackground: string;
  lighterBackground: string;
  foreground: string;
  darkForeground: string;
  lightForeground: string;
  brightForeground: string;
  red: string;
  yellow: string;
  orange: string;
  green: string;
  cyan: string;
  blue: string;
  magenta: string;
  brown: string;
  brightRed: string;
  brightYellow: string;
  brightGreen: string;
  brightCyan: string;
  brightBlue: string;
  brightMagenta: string;
}

// TypeScript field name -> colors.toml key. `mode` is handled separately (enum, not hex).
const HEX_FIELDS: Record<Exclude<keyof Palette, 'mode'>, string> = {
  accent: 'accent',
  selection: 'selection',
  muted: 'muted',
  background: 'background',
  darkBackground: 'dark_background',
  darkerBackground: 'darker_background',
  lighterBackground: 'lighter_background',
  foreground: 'foreground',
  darkForeground: 'dark_foreground',
  lightForeground: 'light_foreground',
  brightForeground: 'bright_foreground',
  red: 'red',
  yellow: 'yellow',
  orange: 'orange',
  green: 'green',
  cyan: 'cyan',
  blue: 'blue',
  magenta: 'magenta',
  brown: 'brown',
  brightRed: 'bright_red',
  brightYellow: 'bright_yellow',
  brightGreen: 'bright_green',
  brightCyan: 'bright_cyan',
  brightBlue: 'bright_blue',
  brightMagenta: 'bright_magenta',
};

// omarchy ships 6-digit hex only (#rrggbb). 3-digit shorthand is a CSS convention omarchy
// doesn't use — treat it as invalid input rather than silently expanding it.
const HEX_6_RE = /^#[0-9a-fA-F]{6}$/;

function fail(sourceLabel: string, detail: string): never {
  throw new Error(`${sourceLabel}: ${detail}`);
}

/**
 * Parse an omarchy colors.toml document into a validated Palette.
 * Pure — does no file I/O. Throws on any missing key, unparseable TOML, or non-hex value;
 * never returns a partial or defaulted palette.
 */
export function parsePalette(tomlText: string, sourceLabel: string): Palette {
  let raw: unknown;
  try {
    raw = parseToml(tomlText);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    fail(sourceLabel, `could not parse TOML: ${message}`);
  }

  if (raw === null || typeof raw !== 'object' || Array.isArray(raw)) {
    fail(sourceLabel, 'expected a flat TOML table, got something else');
  }
  const table = raw as Record<string, unknown>;

  const modeValue = table.mode;
  if (modeValue !== 'dark' && modeValue !== 'light') {
    fail(
      sourceLabel,
      `field "mode" must be "dark" or "light", got ${JSON.stringify(modeValue ?? null)}`,
    );
  }

  const palette = { mode: modeValue } as Palette;

  for (const [field, tomlKey] of Object.entries(HEX_FIELDS) as [
    Exclude<keyof Palette, 'mode'>,
    string,
  ][]) {
    const value = table[tomlKey];
    if (value === undefined) {
      fail(sourceLabel, `missing key "${tomlKey}"`);
    }
    if (typeof value !== 'string' || !HEX_6_RE.test(value)) {
      fail(
        sourceLabel,
        `field "${tomlKey}" is not a 6-digit hex color (#rrggbb): ${JSON.stringify(value)}`,
      );
    }
    palette[field] = value.toLowerCase();
  }

  return palette;
}

/** Read and parse `colors.toml` from an omarchy theme directory. */
export function loadPalette(themeDir: string): Palette {
  const colorsPath = join(themeDir, 'colors.toml');
  if (!existsSync(colorsPath)) {
    fail(themeDir, `no colors.toml found (expected ${colorsPath})`);
  }
  const text = readFileSync(colorsPath, 'utf-8');
  return parsePalette(text, colorsPath);
}
