# Winsome — Work Breakdown

Source of truth for work state. Check items in the same commit as the work. A cold session starts here: find the first unchecked item in the active milestone, read its notes, go. When a milestone completes, note the date and move anything cut to Deferred.

Conventions: `[v]` needs verification on the real rig before checking · items reference DESIGN.md sections rather than restating rationale.

## M0 — Spike (prove the risky parts before building on them)

Everything console-mode / temporary. No scheduled tasks, no installs to Startup, Chris's live PowerToys config untouched.

- [x] kanata console-mode caps tap-hold: works at 200/200; latency "a smidge," livable — tuning stays open by daily use (fallback: drop tap, launcher on chord+Space). NOTE: WM chord corrected to **Ctrl+Alt+Win** — Ctrl+Alt+Shift+Win is the Office key and Shift must stay free for the grammar
- [ ] kanata `apps` mode variant: VK_APPS tap = context menu, hold = WM chord — config written (`spike/apps.kbd`, validates), not yet live-tested
- [x] kanata elevated: chords verified landing while an admin window has focus (LLHOOK, no driver) `[v]`
- [x] GlazeWM 3.10.1: grammar subset bound on lwin+ctrl+alt chords, full 60-second run clean across all three monitors `[v]`
- [x] `hide_title_bar`: works — classic-chrome charmap shaved clean, modern apps unaffected (own chrome). Default-on with per-app opt-out `[v]`
- [x] GlazeWM IPC capability pass: query/command/sub all present (16 event types incl. monitor_*; built-in pause) — UAT harness + agent surface fully covered; see spike/NOTES.md
- [x] Chrome `BrowserThemeColor`: HKCU Policies ACL-locked on this box → HKLM works, applies on Chrome's lazy policy refresh (minutes, not at launch), and BEATS user-installed themes across ALL profiles — kills per-profile color-coding, so Winsome ships it opt-in. Per-profile generated themes = arc
- [x] Triple-monitor behavior: per-monitor workspaces, focus crosses boundaries, uniform dpi 96 / scale 1.0 across all three (mixed-DPI risk absent on this rig) `[v]`
- [x] Monitor-set change via the real workflow: full teleprompter add→remove→re-add cycle CLEAN on 3.10.1 — workspace 4 auto-activates on it, daily three never reshuffle, **#1233 did not reproduce**. Supervisor display-watch demoted to insurance; crash-restart remains core `[v]`
- [x] Spike retro → DESIGN.md updated (titlebars solved, Chrome tint partial, #1233 not reproduced, WM chord = Ctrl+Alt+Win). Remaining M0 stragglers: apps-mode live test, timing tune by daily use

## M1 — Core stack

- [ ] Full omarchy grammar → keymap mapping file (tiler-agnostic data) + GlazeWM YAML generator
- [ ] kanata config templates rendered from mode setting (`caps` | `apps`), game-foreground auto-suspend included
- [x] caps→rwin is the DEFAULT (Sol review, source-verified in both codebases; live as v4.1): caps emits right Win, GlazeWM binds `rwin+X`, physical left Win fully native
- [ ] **R5 fast-follow card**: tap-caps launcher via resident relay — winsome.exe holds warm WebSocket to GlazeWM IPC (persistent tokio server, no spawn) + TCP to kanata; requires small kanata patch adding a "notify TCP clients" action (scoped, upstreamable — its TCP server exists for exactly this). Single-digit ms. Blocked on supervisor landing; do NOT block M1 on it
- [ ] Upstream freebie: CREATE_NO_WINDOW for kanata `cmd` spawns (kills console flash for low-frequency cmd actions like panic)
- [ ] winsome-helper: taskbar hide/restore, per-monitor wallpaper (IDesktopWallpaper), Windows light/dark flip, audio output switch
- [ ] Supervisor: start order kanata→GlazeWM→Zebar, restart-on-exit with backoff, display-change event → GlazeWM bounce, state file for health. **Linked-pair rule (spike incident): kanata+GlazeWM both-healthy-or-both-down — GlazeWM death reverts caps to stock instantly (prevents raw Win-shortcut soup), kanata death auto-restarts**
- [ ] Logging (foundational — DESIGN § Logging): supervisor captures all component output to rolling `~/.winsome/logs/`, components headless (no kanata log window), `winsome logs <component>` tail, debug-mode flag flip
- [ ] Zebar config: workspaces, clock, health dot (reads supervisor state), per-monitor
- [ ] Scheduled-task registration scripts (kanata elevated + supervisor at logon) with removal counterparts
- [ ] kanata config lands in ACL-protected path; verify a non-admin write fails `[v]`

## M2 — Theming

- [x] Adapter core: colors.toml palette extraction (26 semantic keys incl. mode, omarchy 4 format; loud failure on missing/unknown shape) — plus WT renderer (omarchy `ansi_alias` mapping, cited) and light/dark (mode > marker > luminance, matching omarchy's `resolve_theme_mode`); 21 tests, 2 fixture themes
- [ ] Renderers: WT scheme (ANSI mapping = omarchy's ghostty.conf.tpl, verbatim) · Zebar stylesheet template · GlazeWM border colors · wallpaper · light/dark (mode key; luminance = legacy fallback only) · **Windows accent color from palette `accent`** (tints PT Run highlight, borders, start — native launcher theming) · nvim colorscheme pass-through · vscode.json pass-through
- [ ] Per-app compact-chrome settings pack (WT hide-titlebar, VS Code, Chrome flags) applied on render
- [ ] App layer (restored from scoping plan — dropped in TODO translation): apps.conf launch bindings, webapp shortcuts via `chrome --app=URL` gen (omarchy webapp trick). Chrome profile chords (Default / "Profile 1") already live in spike config
- [ ] **focus-or-launch verb** (`winsome focus-or-launch <app>`): app keys focus the running instance via GlazeWM IPC, spawn only if absent — the omarchy launch-key behavior (latency-insensitive, relay path fine). Pair with window_rules home-workspace assignment (browser→2 etc.): switch-to-app = place, not list
- [ ] Golden-file tests: 19 stock omarchy themes → snapshotted outputs, CI green
- [ ] `winsome theme set|list`, `winsome bg next`
- [ ] Theme visual pass harness: cycle themes, capture per-theme screenshots (doubles as gallery assets)
- [ ] Chromium tint renderer (only if M0 spike verified the policy)
- [ ] muxel target: blocked on muxel user-theme-dir + watched-apply feature (tracked in muxel repo, not here)

## M3 — Install / update / uninstall

- [ ] winsome scoop bucket, exact versions pinned for every tool
- [ ] Adoption story (spike finding): mid-day first start tiles EVERY open window into mosaics — installer ships default float rules for non-curated apps and/or first-run-at-login guidance
- [ ] install.ps1: prereqs → scoop → bucket installs → clone → render → single elevation prompt → tasks registered. Idempotent (second run = no-op), backs up files it replaces, writes install manifest
- [ ] Bootstrap disables PowerToys KBM module + FancyZones (and records prior state in manifest)
- [ ] `winsome update`: pull (refuse dirty), re-render, restart stack
- [ ] uninstall.ps1: manifest-driven full restore (taskbar, PowerToys modules, backed-up configs, tasks removed)
- [ ] `winsome toggle off|on`
- [ ] `winsome doctor`: versions vs pins, tasks present, hooks alive, config parse status
- [ ] Pester suite: manifest correctness, idempotency, uninstall restoration
- [ ] Windows Sandbox e2e: install → assert up → uninstall → assert restored

## M4 — UAT harness + release

- [ ] UAT harness: spawn windows, SendInput keystrokes, assert via GlazeWM IPC (grammar coverage: focus, move, workspaces, launcher tap)
- [ ] CI: windows-latest — lint, unit, config validation (`kanata --check`, YAML), golden themes
- [ ] README: hero GIF, install one-liner, full keybind table, theme gallery
- [ ] Keybind reference: `winsome keys`
- [ ] Issue templates, CONTRIBUTING
- [ ] Manual release gate on the real rig (DESIGN.md § Verification)
- [ ] Tag v0.1.0

## Deferred (arc — revisit after v0.1)

legacy pre-v3 community themes (alacritty.toml fallback parser) · komorebi power option via winsome-wm-switch (BYO license; check whkd license) · per-project accent colors (WT tab / border tint keyed to repo) · per-profile accents — one base theme, an accent color per Chrome profile via generated theme extensions (spike finding: BrowserThemeColor is machine-wide and flattens profile color-coding) · agent layouts (omarchy `tdl`/`tsl` equivalent: editor+agent+terminal workspace via IPC) · workspace overview with live previews · trackpad gestures · community-theme guarantee · VS theme VSIX pack · menu (Command Palette extension or TUI) · notification daemon · website/manual
