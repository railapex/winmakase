# Winsome — Design

Decisions with rationale. If code disagrees with this doc, one of them is wrong — fix whichever it is, in the same PR.

## Goals

1. **Omakase**: one command in, a finished desktop out. Curation over configuration.
2. **Muscle-memory portability**: omarchy's keybinding grammar, verbatim, on a mod key that exists in the same physical spot on a future omarchy machine (`caps:super`). Hands trained here work there.
3. **Omarchy theme compatibility**: real omarchy theme folders in, themed Windows desktop out. Stock and community themes.
4. **Stock Windows where Windows wins**: PowerToys, Win+V, built-in OCR/dictation stay. Winsome never rebuilds what the OS already ships.
5. **Reversible and honest**: manifest-driven uninstall, disable-without-uninstall, loud failure. Documented gaps stay documented, not papered over.
6. **Scriptable throughout**: every action a CLI verb, every config a file. Agents (human or AI) can drive the whole thing.

## Architecture

```
kanata (elevated, LLHOOK)         — mod key: tap-hold, emits hyper chord
  └─> GlazeWM (user-level)        — tiling, omarchy grammar bound in YAML
        └─> Zebar                 — per-monitor top bar (taskbar hidden)
winsome-helper                    — wallpaper/monitor, light-dark, taskbar, audio
winsome supervisor                — start order, restart-on-crash, monitor-replug bounce, health
theme adapter                     — omarchy theme folder -> rendered Windows configs
winsome CLI                       — theme|bg|update|render|toggle|keys|doctor
```

### Key decisions and why

**GlazeWM as base tiler, not komorebi.** komorebi is technically richer (deeper CLI/IPC), but its license (Komorebi 2.0.0, a PolyForm Strict fork) prohibits redistribution and requires a paid license for workplace use — undistributable as the core of an open-source project. GlazeWM: GPL-3.0, 12k+ stars, V3 Rust rewrite, YAML config, `focus_follows_cursor`, border effects, per-monitor workspaces. komorebi returns later as a bring-your-own-license power option (`winsome-wm-switch`, the omacosy AeroSpace/OmniWM pattern).

**kanata runs elevated with the LLHOOK backend; the Interception driver is never installed.** Two problems, one move: (a) UIPI starves a user-level keyboard hook when an admin window has focus — an elevated hook process sees those keys; (b) the Interception kernel driver is on anti-cheat blocklists (FACEIT names it) — LLHOOK is the same user-mode API PowerToys uses, uncontroversial. Elevated kanata reads its config from an ACL-protected path (an elevated process must not execute user-writable config). Residual: kernel anti-cheat games can ignore synthetic input in-game — a game-foreground auto-suspend toggle covers it.

**Mod key is configurable, two modes.** `caps` mode: Caps Lock is the WM mod (tap = launcher hotkey, hold = the WM chord); omarchy's Caps emoji layer is sacrificed. `apps` mode: the context-menu key (VK_APPS) is the mod — same physical position as Super on a standard board — and Caps keeps omarchy's compose layer. The WM chord is **Ctrl+Alt+Win — deliberately not Ctrl+Alt+Shift+Win**: (a) that four-modifier chord is the Office key, which Windows binds to app launches (Office+W = Word) when Microsoft 365 is installed; (b) the grammar requires Shift as a free second modifier so `mod+X` and `mod+Shift+X` stay distinct chords. Bare Win+letter is unusable because Windows reserves too many combos (Win+L locks the machine, unrebindable).

**The keymap is omarchy's, verbatim, as a tiler-agnostic mapping file.** mod+W close, F fullscreen, T float, G group, S scratchpad-equivalent, J/L layout, arrows focus, +Shift swap, +Shift+1-4 move to workspace, Shift+Return browser, Return terminal, etc. The mapping file tracks upstream omarchy; grammar drift shows up as a diff, not a surprise.

**Render, don't symlink.** Windows symlinks want admin/dev-mode and tools clobber them. The repo holds source-of-truth defaults plus git-ignored `*.local` overrides (omacosy's `apps.conf` pattern); `winsome render` writes each tool's real config to its expected path, backing up anything pre-existing.

**Supervisor is a hard requirement, not defense.** A dead tiler on Windows leaves a normal-looking desktop with hotkeys silently gone — worse than a crash. Additionally, GlazeWM has a confirmed open bug (glzr-io/glazewm#1233) where a reconnected monitor requires a restart — and monitor-set change is a *routine workflow*, not an edge case (e.g. a display toggled on only for teleprompter use). The supervisor owns: start order (kanata → GlazeWM → Zebar), restart-on-exit, display-change-event → GlazeWM bounce, and a health indicator in the bar. Workspaces bind to the daily displays; an occasional display gets no workspace auto-assignment, so toggling it never reshuffles the primaries.

**Theme adapter: colors.toml is the sole palette source.** Omarchy 4 (Quattro) themes ship a flat `colors.toml` with 26 semantic keys, verified against the raw v4.0.1 files across multiple themes: `mode` (`"dark"|"light"`), `accent`, `selection`, `muted`; `background` + `dark_`/`darker_`/`lighter_background`; `foreground` + `dark_`/`light_`/`bright_foreground`; eight named colors (`red`, `yellow`, `orange`, `green`, `cyan`, `blue`, `magenta`, `brown`); six `bright_*` variants. **Terminal ANSI mapping mirrors omarchy's own `default/themed/ghostty.conf.tpl`** (slot 0 = background, 7 = foreground, 8 = muted, 15 = bright_foreground, cursor = bright_foreground) — adopted verbatim, never invented. `mode` is authoritative for light/dark; the luminance fallback exists only for legacy themes. A theme without a parseable 26-key colors.toml fails loud ("couldn't parse this theme"), never silently mis-themes; waybar.css is never parsed. Earlier formats (v3-era color0-15 files, pre-v3 alacritty.toml themes) are a deferred legacy fallback, not v1. Render targets: Windows Terminal scheme, Zebar stylesheet (our template), GlazeWM border colors, per-monitor wallpaper (IDesktopWallpaper), Windows light/dark (from `light.mode` marker, falling back to background luminance), Neovim (pass-through of the theme's own `neovim.lua` colorscheme — no generation), VS Code (pass-through of the theme's shipped `vscode.json`), Chromium frame tint (`BrowserThemeColor` policy — pending verification). Per-app compact-chrome settings for curated apps ride along with render.

**Install**: `irm …/install.ps1 | iex` → scoop (no admin) → pinned versions from the winsome scoop bucket → clone to `~/.winsome` → render → one elevation prompt registers two scheduled tasks (kanata elevated; supervisor at logon). Idempotent. No driver, no reboot. `winsome update` = git pull (refuses on dirty tree) + re-render + restart. Bootstrap disables PowerToys Keyboard Manager and FancyZones (both double up on kanata/GlazeWM territory; hook order is unstable across reboots).

## Documented gaps (permanent or deferred)

- **Titlebars**: GlazeWM exposes a `hide_title_bar` window effect (Windows 11 only) — spike-pending. If it proves stable it becomes the default with per-app opt-out; if glitchy, an opt-in per-app rule, with compact-chrome settings covering curated apps either way.
- **Notifications unthemed** — Windows toasts render in ShellExperienceHost; no lever exists.
- **Elevated windows float untiled** — GlazeWM runs user-level; UIPI blocks managing admin windows. Keys still work in them (kanata is elevated).

## Verification strategy

Layered; everything below "manual" runs in CI or a single script.

**Unit (CI, windows-latest):**
- Theme adapter is pure functions → golden-file tests: all 19 stock omarchy themes render to snapshotted outputs (WT scheme JSON, Zebar CSS, GlazeWM YAML fragment). Luminance-fallback cases, malformed/missing colors.toml fail-loud cases.
- Rendered GlazeWM config: YAML parses, required keys assert.
- kanata configs: `kanata --check` in CI; tap-hold behavior via kanata's simulated-input mode (grammar tests without hardware).
- Installer PowerShell: Pester — manifest correctness, idempotency (two runs, identical state), path/ACL assertions.

**Integration (scripted, local or sandbox):**
- Supervisor: kill each child, assert restart + health state; fake display-change event, assert GlazeWM bounce.
- Install e2e in Windows Sandbox: clean install → assert stack up → uninstall → assert pre-install state restored (manifest diff).

**Desktop UAT (scripted state assertions, not vision):**
- Harness spawns real windows (terminal instances), injects real keystrokes via SendInput, asserts outcomes through GlazeWM's IPC state queries: mod+2 → focused workspace is 2; mod+Shift+arrow → window swapped; launcher tap → process appeared. Real keys through the real kanata → real tiler — end-to-end without a human.
- Theme visual pass: script switches through all themes capturing per-theme screenshots; reviewed by eye (agent or human) and doubling as README/gallery assets.

**Manual (release gate):** one human hour on the real triple-monitor rig — sleep/wake, monitor replug, admin terminal, game launch, PowerToys coexistence.
