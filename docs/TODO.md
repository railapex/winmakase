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

- [x] Full omarchy grammar → keymap mapping file + GlazeWM YAML generator: `keymap/omarchy.toml` (quattro, 226 chords — 73 mapped, rest explicit gap/native/app/omitted) + `crates/winsome-keymap` + `winsome keymap check|render`. Golden + pinned coverage counts as drift alarm. Deviations recorded in DESIGN.md (SUPER+ESCAPE = cycle-focus, Alt+Tab native, SUPER+TAB restored to next-workspace). NOT YET: rendered YAML live on the rig (spike config still hand-written) — adopt at supervisor/config-gen integration `[v]`
- [ ] kanata config templates rendered from mode setting (`caps` | `apps`), game-foreground auto-suspend included
- [x] caps→rwin is the DEFAULT (Sol review, source-verified in both codebases; live as v4.1): caps emits right Win, GlazeWM binds `rwin+X`, physical left Win fully native
- [ ] **R5 fast-follow card**: tap-caps launcher via resident relay — winsome.exe holds warm WebSocket to GlazeWM IPC (persistent tokio server, no spawn) + TCP to kanata; requires small kanata patch adding a "notify TCP clients" action (scoped, upstreamable — its TCP server exists for exactly this). Single-digit ms. Blocked on supervisor landing; do NOT block M1 on it
- [ ] Upstream freebie: CREATE_NO_WINDOW for kanata `cmd` spawns (kills console flash for low-frequency cmd actions like panic)
- [ ] Upstream candidates from spike: GlazeWM fullscreen focus trap (directional focus can't reach fullscreen layer; wm-cycle-focus is the workaround). Reflow resolved by gap-features decision below — winsome-side verb is permanent, upstream FR filed as courtesy only (repo dormant)
- [x] **RESEARCH CARD — gap features (scratchpad · grouping/tabs · in-place reflow)**: decided 2026-08-28 — decision doc at `docs/research/gap-features.md`. Verdicts: reflow = build (`winsome reflow` via IPC); scratchpad = build (all primitives source-verified: `--id` subject targeting, `focus --container-id`, targeted `move --workspace`, `set-floating` w/ geometry); grouping = documented gap in v1, komorebi power option is the only genuine route. Upstream PR path dead: **GlazeWM dormant since 2026-04-08** (whole glzr-io org dark, 40+ PRs queued, maintainers' own stacking branches abandoned 2025-03). Field re-swept same day: Seelen UI only live challenger, loses on grammar fidelity + DE-not-component; FancyWM PolyForm-licensed; Whim alpha-dormant. Follow-on items below
- [x] `winsome reflow` verb SHIPPED + live-verified both directions 2026-08-28 `[v]`: plan-based (`--dry-run` prints it), direct-WebSocket IPC (tungstenite, ~25ms warm). Two live findings baked in: move choreography must run in REVERSE reading order (each move lands atop the moved tail — a [1,3,2] row came out [1,2,3] until fixed), and GlazeWM leaves single-child split shells after moves, which the planner sees through (deepest ancestor row with ≥2 units). Multi-window nested splits refused loudly — flat rows only for now
- [x] `winsome scratchpad` SHIPPED 2026-08-28: `toggle <name>` (config `[scratchpad.<name>]`: launch + process match + width/height; summon/banish/launch decided purely, launch gets CREATE_NEW_CONSOLE — a console pad otherwise inherits the invoker's console, live finding) and `summon` (anonymous rwin+s: rescue-first port of spike/summon.ps1 — DWM cloak + visibility + undisplayed-workspace pull — then newest-from-scratch, silent no-op; 57ms cold / ~25ms warm vs ~300ms pwsh). Banish stays pure GlazeWM. Binding flip owned by the config stream. REFINEMENT OWED: process-level match is too coarse for a terminal pad — it would grab the daily WT; needs a title/class rule (omarchy uses a dedicated window class)
- [ ] mod+G grouping stub — keymap binds notification pointing at komorebi power option + docs (DESIGN.md § Documented gaps entry shipped with the decision doc)
- [ ] winsome-helper: taskbar hide/restore, per-monitor wallpaper (IDesktopWallpaper), Windows light/dark flip, audio output switch
- [x] Supervisor core BUILT + review-hardened (61 tests): `winsome supervise/status/logs/down`, linked-pair rule extended to both-or-neither-on-START (rollback semantics — partial start rolls back rather than leaving kanata chord-less), backoff with sanitized config floors, console-shutdown budget const-asserted ≤ GRACE, round-trip date validation. NOT YET: Zebar in the set
- [x] Logging BUILT: rolling 5MB×3 per component + supervisor log, timestamped [out]/[err] capture, `winsome logs` tails across rotations. NOT YET: debug-mode flag flip. Note: only direct children get stdio capture — task-hosted kanata, adopted, and shell-launched (UIAccess) components keep their own logging
- [x] Supervisor adoption SHIPPED + verified live 2026-08-28 `[v]`: ProcessHandle over spawned/external processes, adopt-first (full-image-path match — slid under the live stack with zero disruption), kanata task-hosted (`schtasks /run`/`/end` on elevated WinsomeKanata, both proven from user level), GlazeWM shell-spawn fallback (manifest is asInvoker + uiAccess=true → CreateProcess 740s; found via live crashloop), `winsomed.exe` GUI-subsystem twin hosts the WinsomeSupervisor logon task (windowless; WM_ENDSESSION → graceful shutdown), singleton guard, display watch (log always; `bounce_on_display_change` off by default — #1233 never reproduced), `winsome down` waits for shutdown COMPLETION (ack-race orphaned the pair when panic killed the supervisor mid-shutdown — fixed), panic.ps1 goes through `winsome down` first (panic vs supervisor fight caught before it bit for real). Cold start, crash-restart, graceful down, and panic all exercised on the rig same evening
- [x] Zebar integration SHIPPED + verified live 2026-08-28 `[v]`: Zebar 3.3.1 (bundled install, recorded as the pin) supervised as the optional third component — outside the linked pair, dies-restarts-alone (proven live), `[zebar]` section optional in config.toml, adopted on takeover like the others. glzr-io.starter `with-glazewm` widget on all three monitors (workspaces + clock + stats, per-monitor); `ignore` window rule for zebar added to the live GlazeWM config (starter bars were getting TILED without it). Component-set generalization deliberately NOT done: three compiled-in components with distinct hard-coded policies (linked pair / independent bar) is honest — a config-driven policy language for one rig is speculative generality; revisit when the set actually grows. Still open from the review finding: sleepless backoff tests (needs a clock abstraction)
- [x] Winsome bar pack SHIPPED + live-verified 2026-08-28 `[v]` (`zebar/` in repo, deployed to `~/.glzr/zebar/winsome`): workspaces/clock/stats per-monitor, **SYSTRAY on the primary bar** (Chris's ask — tray gated on primary-at-origin heuristic until the M2 render knows monitors), **health dot** polling `winsome status --json` via shellExec privilege (json now carries `supervisorProcessAlive` — the file alone reads running forever after a hard kill). Dock-race findings: systray-in-every-instance breaks one monitor's dockToEdge; two widget configs lose the second config's docks wholesale → one widget, all monitors, tray gated in-page. **Supervisor now heals lost docks**: reserve-consistency check ~12s after each zebar start, partial dock → bounce (3-cap). Still M2: theme the pack (stylesheet render), vendor the CDN deps (react/babel/zebar via esm.sh at bar startup), real monitor knowledge for the tray gate
- [x] Scheduled-task registration scripts: `installer/register-tasks.ps1` (WinsomeSupervisor at logon needs no elevation; elevated pair (re)registered only when run elevated; deploys binaries to ~/.winsome/bin so the build tree is never locked) + `installer/unregister-tasks.ps1`. spike/winsome-up.ps1 reduced to taskbar-hide + task trigger; spike start/stop/restart scripts superseded by the supervisor
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
