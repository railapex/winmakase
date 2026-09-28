# Input, remote work and the desk-first recut

Reader: the next Winmakase build session and Chris. Decided with Chris on 2026-09-27, after the stack was off for a Parsec remote week. Facts here were observed on Diana on 2026-09-27 unless marked otherwise.

## Decisions

1. **Caps becomes F13, not right-Win.** The keymap's abstract `SUPER` renders as `f13+…` in Glaze. Physical left Win stays native.
2. **Kanata retires, and the direct-Caps Glaze leader is parked.** The leader patches (`fc8b20f` and `371a448` on `winmakase-v1-input`, 1,550 of the fork's 2,839 lines over v3.10.1) stop growing. The branch is kept.
3. **Where the remap lives.** On Chris's desk and laptop, a registry `Scancode Map` (Caps→F13, ScrLk→Caps) sits below every hook. For other people, PowerToys Keyboard Manager is the no-admin default and the registry map is the robust option. Firmware remaps also work.
4. **Glaze stays**: pinned, with a thin fork. No WM switch. Upstream check below.
5. **v1 recut to the desk first.** Every slice lands on the desk within a day, with rollback. Deferred: installer/update/uninstall (P5, then proven on the laptop), taskbar retirement with a restore host (P4 takeover), a second theme and staged theme apply, and the R0–R6 ceremony. NEXT plus TODO carry the status.
6. **The launcher moves toward Command Palette** instead of hardening PowerToys Run further. PowerToys is replacing Run ([#41696](https://github.com/microsoft/PowerToys/issues/41696)), and 0.101 shipped three times in September, mostly Command Palette work. The path to "a Winmakase PowerToy" is a separately distributed Command Palette extension (actions, keybinding help, later Dock bands). A first-party module isn't realistic: PowerToys is MIT and Glaze is GPL-3, and auto-tiling ([#2694](https://github.com/microsoft/PowerToys/issues/2694)) has been open since 2020-05-05.
7. **The Agents workspace stays in v1 as a coarse first cut**, directly after input: recognize agent root processes, route their new windows to one workspace, never steal focus. `DESIGN.md` and `01-window-policy.md` already carry it. **Presence modes** (desk / remote / recording) are a later idea: one switch changes layout, bar and notifications together. Remote would trigger from the Parsec log, recording from the teleprompter display.

## Why F13

With Caps as right-Win, Glaze swallows the chords it binds, but:

- Win+L is handled at the winlogon level, whatever the extra modifiers (keymap note at `omarchy.toml:814`; PowerToys 0.101.2652 added a Keyboard Manager check for it). Caps+L locks the PC.
- The `status = "native"` rows are unbound, so Omarchy muscle memory reaches the wrong Windows feature. Caps+Ctrl+V opens the sound-output flyout, and Caps+PrtScn saves a screenshot to file.
- Every unbound Caps chord leaks to Windows. Tapping Caps opens Start, and with the WM down Caps is a Win key.

No Windows shortcut uses F13. Stock Glaze matches any held key as a chord prefix: `keybinding_listener.rs` checks every non-trigger key with `is_key_down` (GetKeyState), and `Key` includes F13–F24. It has no injected-input filter, so remote input works. Costs: F13 is not a modifier, so apps receive F13 presses, and an unbound Caps+letter types the letter. Avoid F23, which the Copilot key sends as Win+Shift+F23.

## Test evidence (2026-09-27)

Setup: PowerToys KBM Caps(20)→F13(124), with PowerToys running elevated. Deployed Glaze `9c8e93b` ran standalone on the live config with all 73 `rwin+` bindings replaced by `f13+`. Test copy: `~/.winmakase/glazewm/config.f13-test.yaml`, SHA-256 prefix `6c211f0bc0c3974`. Kanata and the supervisor were off.

- Injected harness `spike/f13-harness.ps1`, with probe `spike/f13-keylog.ps1`: 9/9 passed. Move+follow, float then tile, silent move, workspace focus, no Start menu after chords, unbound F13+L types `l` without locking, and bare F13 reaches the app as a key with no character.
- Chris, physical: no F13 junk in muxel; daily chords work; no Alt-menu activation after Caps+Shift+Alt chords; a bare Caps tap is inert; the ScrLk hatch works.
- **Admin windows fail.** Caps+digit types the digit. Our self-built Glaze has `uiAccess=false`, so its user-level hook is deaf while an elevated window has focus. Official GlazeWM ships `uiAccess=true` (verified 2026-08-28, `gap-features.md`). The capability was lost at the 2026-08-31 W1.1 deploy, not by F13. Fix: build with `--features ui_access`, Authenticode-sign it, install to a secure location, and have the supervisor launch it through ShellExecute (plain CreateProcess fails with 740, and Glaze stdio capture is lost). The signing route is Chris's call: a self-signed cert trusted on this machine, or a real signing service, which public distribution needs anyway. Deferred until after VRMA. Chris, same evening: self-signed first, real signing once other people install it.
- **Test official GlazeWM before signing anything.** With the leader parked, the deployed fork is two patches on upstream `56a8bde` (797 lines): AppUserModelID matching (`6d62d66`) and placement-denied recovery (`9c8e93b`). The live config has no AppUserModelID rules; Winmakase only parses the field as optional (`glazewm.rs`). The placement-denied patch exists because our build runs with `uiAccess=0` against elevated windows ([evidence](../verification/v1/2026-09-10-placement-denied.md)). Official GlazeWM 3.10.1 is signed and ships `uiAccess=true`, so it may fix admin-window chords and elevated placement together, with no fork build and no cert (inferred, unverified). Test: install official 3.10.1, run the F13 config, try Caps chords over an elevated window and tile an elevated window. If both pass, run stock Glaze and offer the AppUserModelID patch upstream without waiting on it (upstream has merged nothing since 2026-04-08). The supervisor still has to launch a `uiAccess` executable through ShellExecute (CreateProcess returns 740).
- **Parsec:** with the laptop KBM also mapping Caps→F13, Caps chords work over Parsec. Kanata ignores all injected input (hard-coded in `llhook.rs` `hook_proc`, no config option), and the parked leader rejects injected leader edges by policy (`keybinding_listener.rs:222`), so neither could ever work remotely.

## Parsec display churn

On connect, Parsec switched the ultrawide from 3840×1600 to 2560×1600 and added its virtual display (`PSCCDD0`, 1920×1080 at x=5120). Glaze saw four monitors. After disconnect, all three Zebar appbars had lost their reservation (0 instead of 40px), so tiles sat under the bar. Restarting Zebar plus `glazewm-cli command wm-redraw` restored 40/40/40 and y=48. **Remote requirements:**

- After any settled display change, re-dock Zebar and then redraw Glaze. The supervisor's dock re-check covers the first half; add the redraw.
- Bind Glaze's existing `wm-toggle-pause` to a chord.
- Automatic remote mode from `C:/ProgramData/Parsec/log_cl.txt` (`<user>#<id> connected.` / `disconnected.`) comes only if the manual path proves insufficient.

Parsec resolution matching is a separate handoff: `D:/vaults/stuff/Friday/desk/handoffs/2026-09-27-parsec-resolution.md`.

## Upstream check (2026-09-27)

- GlazeWM: main's last commit was 2026-04-08; last release 3.10.1 on 2026-03-21. September community PRs sit unmerged. The maintainer's visible work is Zebar (merges on 09-20) and a macOS/animations branch (last push 06-20). Our patches have no merge path, and nothing needs rebasing.
- Whim: no non-bot commit since v0.8.6-alpha on 2025-11-29.
- komorebi is active, but its license allows only personal, non-commercial use and forbids redistribution. Seelen UI (AGPL, v2.8.6 on 09-18) is a whole desktop environment, as assessed on 08-28.

## Next build slices

1. Add F13 as a keymap render target: `SuperKey::F13` in `winmakase-keymap`, with a renderer test. Also bind the native rows (clipboard, emoji, capture) to real Windows actions where possible.
2. Supervisor input mode with no kanata: the pair logic must not require kanata, and nothing may start kanata while a Caps remap is present.
3. Registry `Scancode Map` on the desk (admin plus reboot), replacing the KBM mapping. The laptop gets the same.
4. Re-enable the supervisor on the new config, plus the Zebar re-dock / Glaze redraw after display changes, plus a pause chord.
5. ~~Merge `build/f0-s0`~~ (done 09-27: `main` fast-forwarded, work lands on `main`). Then do the Agents workspace first cut, then Command Palette.
