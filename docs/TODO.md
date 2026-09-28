# Winmakase work state

Updated 2026-09-27. **Active milestone: coherent v1 on GlazeWM, Zebar and PowerToys, landed desk-first.** The [2026-09-27 input recut](research/2026-09-27-input-remote-recut.md) retires Kanata for a Caps→F13 remap and defers installer, taskbar retirement, the second theme and the R0–R6 ceremony. Plans are accepted direction; implementation gates below remain open. The previous M0–M4 ledger is [archived](plans/archive/2026-09-08/TODO.md); its completed evidence remains useful, its unchecked sequence is superseded. [Builder contracts and work orders](plans/v1/builders/README.md) now define the remaining execution; planning closes no product gate.

## Already present

- [x] Rust CLI/supervisor; task hosting, adoption, logs/status/reload and recovery.
- [x] Pure Caps → right-Win mapping, generated Omarchy grammar and local overrides.
- [x] Current-state float/tile fix; reflow and scratchpads.
- [x] Typed app homes plus Glaze AppUserModelID patch; profile routing dogfood verification.
- [x] Per-monitor Zebar pack with tray and health; bounded dock-restart workaround.
- [x] Quattro palette parser and Terminal renderer with two fixture themes.
- [x] V1 source review, updated scope, implementation packages and verification design.

These checks do not imply clean installation, dialog safety, focus-or-launch, complete theming or hardened recovery.

Source checkpoint `build/f0-s0`: the v1 shared interfaces and committed schema fixtures are implemented, and the Zebar runtime is locally bundled with deterministic offline-asset checks. [Evidence](verification/v1/2026-09-16-f0-s0/README.md). No runtime round closes: disconnected Zebar startup/rendering, physical Caps and the remaining window/popup/lifecycle proofs are still required.

Targeted recovery: [2026-09-10 placement-denied hotfix](verification/v1/2026-09-10-placement-denied.md) implements permission-denied tile release on the deployed Glaze source base. Source review, seven recovery tests, four error tests and elevated/ordinary native component checks pass. Delivery status is recorded in the evidence; P0/P1 gates remain open.

## Desk-first recut (target: slices 1, 2 and 4 live before Fri 2026-10-02)

Each slice lands on the desk within a day, with rollback.

- [ ] 1 — F13 keymap render target, native rows bound where a real action exists. Source done in 0.1.10 (`input_mode = "f13"`, `--super-key f13`; lock, capture and pause chords); live render pending.
- [ ] 2 — Supervisor input mode without Kanata; never start Kanata over a Caps remap. Source done in 0.1.10; supervisor not yet re-enabled on the desk.
- [ ] 3 — Registry `Scancode Map` (Caps→F13, ScrLk→Caps) on desk and laptop, replacing PowerToys KBM. Admin plus reboot; Chris runs it.
- [ ] 4 — Supervisor back on the F13 config, Glaze redraw after the bar re-docks, pause chord. Redraw and chord source done in 0.1.10; live pending.
- [ ] 5 — Agents workspace first cut, then Command Palette. (`build/f0-s0` merge done 09-27.)
- [ ] Before signing anything: test official GlazeWM 3.10.1 (`uiAccess=true`) for admin-window chords and elevated placement.

## Active packages

- [ ] [P0 — Baseline and integration proofs](plans/v1/00-baseline.md): active; exact provenance, direct-Caps physical adoption, popup/focus/dock capability, classification, agent ancestry/admission/background-input and suppression probes. [I1 guest evidence](verification/v1/2026-09-13-p0-i1/README.md): guarded Run activation passes for hidden, repeated, concurrent, GUI-helper and real Zebar-click paths; direct-Caps config generation passes. Physical Caps, fullscreen/elevated/focused-monitor placement and offline bar startup remain open. [D1 guest evidence](verification/v1/2026-09-13-p0-d1/README.md): main/owned/modal classification and arranged-window reload preservation pass; utility and hidden-owner admission remain open. Earlier launcher argv/indexing and the unwired sequence-feed prototype remain accepted in the [2026-09-12 checkpoint](verification/v1/2026-09-12-p0/README.md). Exit: R0.
- [ ] [P1 — Window policy and app actions](plans/v1/01-window-policy.md): dialog-safe defaults, supported agent output placement, homes/profiles, launch modes, safe adoption/reload, loose-float recovery, practical splits, named terminal and one daily layout. Exit: R1.
- [ ] [P2 — Launcher and controls](plans/v1/02-launcher-controls.md): shared entries, offline bar, monitor identity, dropdown tray with user pinning, month calendar, float indicator and accessible controls/help. Exit: R2.
- [ ] [P3 — Appearance](plans/v1/03-appearance.md): two complete presets, staged apply/rollback, owned settings. Exit: R3.
- [ ] [P4 — Supervisor and taskbar retirement](plans/v1/04-supervisor.md): lifecycle correctness, crash restoration, privileged-path boundary, delete hover controller after replacement proof. Exit: R4.
- [ ] [P5 — Distribution and release](plans/v1/05-package.md): install/update/uninstall, provenance/licenses, clean Windows verification and dogfood. Exit: R5–R6.

P1 and offline-bar work can proceed after their P0 proofs. P4 correctness and privileged-layout work can start early; taskbar retirement waits for P1/P2 controls and recovery. P3 reload behavior depends on P1. See the [dependency order](plans/v1/README.md).

## Verification ledger

- [ ] R0 — baseline and bounded experiments.
- [ ] R1 — window/dialog/profile behavior.
- [ ] R2 — launcher, controls and monitor behavior.
- [ ] R3 — appearance, reload and rollback.
- [ ] R4 — failure injection and taskbar takeover.
- [ ] R5 — clean install, update, uninstall and offline operation.
- [ ] R6 — integrated use and release review.

Record build, environment, observations and artifacts under the [verification contract](plans/v1/verification.md). No public v1 release tag until all blocking cases pass. A separately authorized dogfood prerelease records its limited scope and outstanding gates under the [dispatch contract](plans/v1/builders/README.md). Future WM/shell/grouping work lives in [future platforms](plans/future-desktop-platforms.md).

Use the [execution procedure](plans/v1/execution.md) and [generated progress report](plans/v1/progress.html) for session handoffs and check-ins. Package/round checkboxes here remain the completion source; execution-state.json records only current slice, decisions and evidence. W5 gets a written need/no-need verdict after at least one week with app actions and float recovery.
