# Winsome — Work Breakdown

Source of truth for work state. Check items in the same commit as the work. A cold session starts here: find the first unchecked item in the active milestone, read its notes, go. When a milestone completes, note the date and move anything cut to Deferred.

Conventions: `[v]` needs verification on the real rig before checking · items reference DESIGN.md sections rather than restating rationale.

## M0 — Spike (prove the risky parts before building on them)

Everything console-mode / temporary. No scheduled tasks, no installs to Startup, Chris's live PowerToys config untouched.

- [ ] kanata console-mode: caps tap-hold config — tap emits PowerToys Run hotkey, hold emits hyper chord. Verify tap latency feels right (tune tap-timeout/hold-timeout)
- [ ] kanata `apps` mode variant: VK_APPS tap = context menu, hold = hyper
- [ ] kanata elevated (manual `Run as administrator` console): confirm hyper chord works while an **admin** Windows Terminal has focus `[v]`
- [ ] GlazeWM installed (pinned release): minimal config, hyper-chord bindings for a grammar subset (W/F/T, arrows, workspaces 1-4) — confirm hyper chords bind cleanly in GlazeWM YAML `[v]`
- [ ] `hide_title_bar` window effect (Win11): enable on a test app, note stability/glitches — decides default-on vs per-app opt-in `[v]`
- [ ] GlazeWM IPC capability pass: enumerate query/command/subscribe surface; document what komorebic has that it lacks (matters for agent features + UAT harness design)
- [ ] Chrome `BrowserThemeColor` policy: HKCU registry write → does Chrome tint? Document result either way
- [ ] Triple-monitor behavior: workspaces per monitor, focus crossing monitor boundaries, DPI sanity check across the three displays `[v]`
- [ ] Monitor-set change via the real workflow (teleprompter display toggled on/off): confirm glazewm#1233 symptom on re-add, GlazeWM restart heals, daily three don't reshuffle (supervisor requirement + occasional-display exclusion validated) `[v]`
- [ ] Spike retro → update DESIGN.md with findings; anything invalidated gets redesigned before M1

## M1 — Core stack

- [ ] Full omarchy grammar → keymap mapping file (tiler-agnostic data) + GlazeWM YAML generator
- [ ] kanata config templates rendered from mode setting (`caps` | `apps`), game-foreground auto-suspend included
- [ ] winsome-helper: taskbar hide/restore, per-monitor wallpaper (IDesktopWallpaper), Windows light/dark flip, audio output switch
- [ ] Supervisor: start order kanata→GlazeWM→Zebar, restart-on-exit with backoff, display-change event → GlazeWM bounce, state file for health
- [ ] Zebar config: workspaces, clock, health dot (reads supervisor state), per-monitor
- [ ] Scheduled-task registration scripts (kanata elevated + supervisor at logon) with removal counterparts
- [ ] kanata config lands in ACL-protected path; verify a non-admin write fails `[v]`

## M2 — Theming

- [ ] Adapter core: colors.toml palette extraction (22 flat keys; loud failure on missing/unknown shape; format stable v3.7↔v4.0)
- [ ] Renderers: WT scheme · Zebar stylesheet template · GlazeWM border colors · wallpaper · light/dark (marker + luminance fallback) · nvim colorscheme pass-through · vscode.json pass-through
- [ ] Per-app compact-chrome settings pack (WT hide-titlebar, VS Code, Chrome flags) applied on render
- [ ] Golden-file tests: 19 stock omarchy themes → snapshotted outputs, CI green
- [ ] `winsome theme set|list`, `winsome bg next`
- [ ] Theme visual pass harness: cycle themes, capture per-theme screenshots (doubles as gallery assets)
- [ ] Chromium tint renderer (only if M0 spike verified the policy)
- [ ] muxel target: blocked on muxel user-theme-dir + watched-apply feature (tracked in muxel repo, not here)

## M3 — Install / update / uninstall

- [ ] winsome scoop bucket, exact versions pinned for every tool
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

legacy pre-v3 community themes (alacritty.toml fallback parser) · komorebi power option via winsome-wm-switch (BYO license; check whkd license) · per-project accent colors (WT tab / border tint keyed to repo) · agent layouts (omarchy `tdl`/`tsl` equivalent: editor+agent+terminal workspace via IPC) · workspace overview with live previews · trackpad gestures · community-theme guarantee · VS theme VSIX pack · menu (Command Palette extension or TUI) · notification daemon · website/manual
