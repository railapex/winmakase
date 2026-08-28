# M0 Spike Notes — 2026-08-28

Machine: Chris's triple-monitor desktop, Windows 11 Pro, PowerToys 0.100.2, winget 1.29, no scoop.

## Pinned versions

- kanata **v1.12.0** — `windows-binaries-x64.zip`; spike binary: `kanata_windows_tty_winIOv2_x64.exe` (console, no Interception driver, no cmd). Variants observed: tty/gui × winIOv2/wintercept × cmd_allowed — wintercept builds are the Interception-driver ones we never ship.
- GlazeWM **v3.10.1** — no portable zip exists; assets are bundled installer `.exe` (includes Zebar) and `standalone-*.msi`. Spike installs via winget (recorded for uninstall).

## Displays (teleprompter OFF at time of enumeration)

| Screen | Monitor | Resolution | Notes |
|---|---|---|---|
| DISPLAY1 (primary) | LG HDR WQHD+ (GSM774D, serial 103NTYTG7818) | 3840x1600 | ultrawide, main |
| DISPLAY2 | DELL U2415 (DELA0B8, serial CFV9N8A51MCS) | 1200x1920 | **portrait** |
| DISPLAY3 | LG SDQHD (GSM5BF4, serial 204NTPC5G888) | 2560x2880 | LG DualUp |
| (no screen) | HDMI (TXD0000, UID24627) | — | **teleprompter — connected, display off** |

Workspace rule input: daily three above get workspaces; TXD0000 gets none. Mixed resolutions AND orientations — watch tiling behavior on the portrait Dell and the DualUp.

## Chris's PowerToys KBM state (to restore at session end)

- remapKeys: ScrollLock(145) → Caps(20); Caps(20) → Alt+Space(164;32)
- remapShortcuts: 260;192 → 17;192 (global)
- KBM disabled during kanata testing; re-enable unless Chris says otherwise.

## Findings

- [x] **kanata + PowerToys coexist**: KBM disabled, kanata console non-elevated, tap(Alt+Space) reaches the launcher — works. Tap latency "a smidge" vs KBM (tap-hold decides on key-UP vs KBM's key-down; inherent). Mitigations if it stays annoying: shorter timings, or drop tap and use chord+Space (omarchy-pure — caps:super has no tap on Linux either).
- [x] **WM chord corrected to Ctrl+Alt+Win (no Shift)**: Ctrl+Alt+Shift+Win IS the Office key (Office+W=Word with M365), and Shift must stay free for the mod+Shift+X grammar. Caught before any binding shipped.
- [x] **Uniform DPI across all three daily monitors** (dpi 96, scale 1.0 everywhere) — the mixed-DPI risk class is absent on this rig.
- [x] **Mid-session adoption problem (new, real)**: starting GlazeWM tiled ALL existing windows — 25 on the ultrawide, 26 on the DualUp, muxel-live squeezed to 159px. Omarchy never hits this (fresh login). Winsome needs an adoption story: default float rules for non-curated apps, or first-run-at-login guidance. glazewm-watcher.exe restores original positions on exit — confirmed present in install.
- [x] caps mode: works at default 200/200; tap latency noticeable-but-fine ("a smidge" — tap-hold decides on key-up). Tune by living with it; fallback = drop tap, launcher on chord+Space (omarchy-pure).
- [x] **THE TWO-HOOKS FINDING (spike's biggest): kanata synthesized chords + GlazeWM keybindings are incompatible.** v1 (multi hold) and v2 (per-press output chords) both desync: GlazeWM's hook CONSUMES bound chords, kanata's key-state tracking then disagrees with Windows → `llhook:215 Unexpected keycode is pressed in kanata but not in Windows. Clearing kanata states` on nearly every chord, dropped keys, and during one state-clear Chris's physical Caps press leaked raw and LATCHED CAPSLOCK with no escape (his ScrLk→Caps hatch was disabled with KBM). Ended the dogfood; stock restored (GlazeWM wm-exit + watcher restore, KBM re-enabled).
  **RESOLVED by v3 (real-Super architecture), live-verified**: caps hold = plain `lmet` (kanata synthesizes ONLY the modifier), `process-unmapped-keys no` (letters pass raw), GlazeWM binds genuine `lwin+X` chords. Repeat test passes, no llhook errors — kanata never tracks anything GlazeWM consumes. TRUE omarchy Super parity as a bonus. CLI-direct-drive experiment unnecessary; v3 is the M1 template. **Every config ships the ScrLk escape hatch** (scrlk → real CapsLock) — a busted remap must always leave one raw exit. Cost: L reserved by OS lock (omarchy's Super+L = scrolling layout, which GlazeWM lacks anyway — no functional loss).
- [x] **Cascade incident (the supervisor's spec, live)**: closing the kanata gui window EXITS it (no hide-to-tray — earlier claim wrong). Death cascade: kanata down → caps reverts to CapsLock → all chords dead while GlazeWM tiles on, uncontrollable; after dropping GlazeWM, raw Win+letter presses fired OS shortcuts (Feedback Hub etc.). muxel needed a restart after the tiling exit. **Requirement: kanata + GlazeWM are a LINKED PAIR — supervisor enforces both-healthy-or-both-down in both directions** (GlazeWM dies → revert caps to stock immediately; kanata dies → restart in seconds). No unsupervised dogfood again.
- [x] Fullscreen (Win+F) buried the window bottom under the taskbar — GlazeWM true-fullscreen uses the full monitor rect and the always-on-top taskbar overlaps it. Fix while taskbar visible: `state_defaults.fullscreen.maximized: true` (respects working area). End-state Winsome hides the taskbar (Zebar replaces it), where true fullscreen becomes correct — installer must set these consistently.
- [ ] apps mode (menu key): NOT tested this session — open item.
- [x] GlazeWM WM-chords: full 60-second grammar run clean — focus/swap/workspaces/float/fullscreen across all three monitors. Chris quote: "oh holy shit i could get used to this."
- [x] hide_title_bar: WORKS — charmap (classic chrome) had its titlebar shaved; modern apps draw their own chrome so are unaffected. Effective coverage total. Default-on candidate with per-app opt-out.
- [x] Admin-window chords (elevated kanata, LLHOOK, no driver): verified live — chords land while an elevated window has focus. The design's load-bearing claim holds.
- [x] GlazeWM IPC capability surface: `glazewm query` (app-metadata, binding-modes, focused, tiling-direction, monitors, windows, workspaces, paused — JSON out), `glazewm sub -e` (16 event types incl. monitor_added/updated/removed, window_managed/unmanaged, focus_changed, workspace_*, pause_changed), `glazewm command <verb>`. UAT-harness state assertions fully covered; supervisor gets monitor events (still needs OS-level display watch since GlazeWM itself is the broken party in #1233); built-in pause = game-mode/teleprompter lever. Agent-surface parity with komorebic for everything v1 needs.
- [x] Chrome BrowserThemeColor: HKCU Policies key is ACL-locked on this box (write denied even unsandboxed) — HKLM write via elevated batch succeeded. First look after bounce: no change; MINUTES LATER both profiles updated — Chrome applies registry policy on a lazy refresh, not at launch. Verdict: the policy BEATS user-installed profile themes and is machine-wide — all profiles get the same frame color, which destroys per-profile color-coding (Chris runs work/personal themes). Winsome: opt-in, documented tradeoff; per-profile generated themes remain the surgical answer (arc).
- [x] Teleprompter full cycle (add → remove → re-add): ALL CLEAN on GlazeWM 3.10.1. Workspace 4 auto-activates on the teleprompter, daily three never reshuffle, re-add restores workspace 4 healthy. **glazewm#1233 did not reproduce** on this rig. Supervisor display-watch demoted from hard-requirement to insurance; crash-restart remains the core need.
