# Handoff — WinMakase window policy W0

Closed: 2026-08-30
Next repo: `D:/dev/winmakase`
Next cut: W0 only

## Direction

WinMakase W0–W4 is the next product arc, in order and as separate bounded sessions:

1. W0 — adopt the generated Glaze config and make the source/live boundary honest.
2. W1 — app definitions and generated workspace-home rules.
3. W2 — `focus-or-launch` and app-key conversion.
4. W3 — float inventory, recovery commands and a nonzero-only Zebar indicator.
5. W4 — one idempotent named layout from real daily use.

W5, the visual switcher, stays behind its one-week need gate. Glazemakaze is parked in `docs/plans/glazemakaze-core.md`; do not pull it into this arc.

## Cold start

Read in this order:

1. `CLAUDE.md`
2. `docs/TODO.md`
3. `spike/NOTES.md` § LIVE MACHINE STATE
4. `docs/plans/winmakase-window-policy.md`
5. This file

The local `main` branch is ahead of `origin/main`; pushing `main` is Chris's button. The planning split is commit `c9d29b1`.

## Proven live state

- WinMakase v0.1.1 supervisor owns kanata, stock GlazeWM 3.10.1 and Zebar.
- Reboot hardening is deployed: Glaze starts wait out high commit pressure; linked-pair and Zebar recoveries are independent.
- Zebar starts under the supervisor and reserves 40px on all three monitors; the dock healer treats partial and all-zero reserves as failure.
- Daily workspace mapping is `1/4 -> Dell`, `2/5 -> ultrawide`, `3/6 -> DualUp`. Workspace 7 is the occasional-display fallback; `scratch` stays last and unbound.
- Reflow and scratchpad verbs are live. The grouping chord shows a gap notice.
- The rig is back on stock Glaze. KhangHLe and the official animation branch are not live candidates in this cut.
- `target/debug/muxel-live.exe` is Chris's primary session. Never stop, replace, overwrite or use it as a disposable WM fixture.

## The inconsistency W0 closes

The generated Omarchy keymap is marked built, but `C:/Users/chris/.glzr/glazewm/config.yaml` is still the hand-written spike config. Generated Caps+Escape → `wm-cycle-focus` therefore is not live, which leaves no Glaze-native recovery across tiling/floating/fullscreen layers.

## W0 procedure

1. Render the Glaze keybindings/config to a staging file. Do not write the live config first.
2. Diff staging against the live config and classify every difference.
3. Preserve these machine facts explicitly:
   - workspace declaration order and monitor pairs;
   - workspace 7 before inactive pair-partners;
   - `scratch` last and unbound;
   - Zebar ignore rule;
   - Chrome Default and Profile 1 launch commands from local overrides;
   - WinMakase panic, reload and exit bindings;
   - reflow and scratchpad bindings;
   - animation settings absent;
   - Alt+Tab and physical left-Win bindings untouched.
4. Fix the renderer/local override source for any missing fact. Do not preserve it with a post-render hand edit.
5. Back up the live Glaze config, render into place, and use the existing reload path. Keep the rollback command in the session notes before reload.
6. Verify through Glaze IPC and physical keys:
   - Caps+Tab / Caps+Shift+Tab / Caps+Ctrl+Tab;
   - Caps+Escape reaches a deliberately separated floating/fullscreen layer;
   - Caps+J, Caps+S, Caps+Alt+S;
   - one workspace move and one silent move;
   - Alt+Tab remains native;
   - left-Win+Tab remains Task View;
   - Zebar remains ignored and all three 40px reserves remain present.
7. Update the grouping notice from “upstream dormant” to the accurate missing-core-primitive / Glazemakaze wording.
8. Check the W0 TODO item only in the same commit as the verified live adoption.

## W0 acceptance

- The live config is reproducible from WinMakase sources plus machine-local overrides.
- No fact exists only in the deployed YAML.
- Caps+Escape recovers the focus-layer trap.
- Native Windows recovery keys still work.
- Workspaces, scratchpad, panic and Zebar geometry survive reload.
- `git diff --check`, relevant tests and the real-rig verification are green.

## Guardrails for W1–W4

- App homes are creation rules, not persisted pixels or fixed tiling slots.
- Numeric homes live in machine-local config; portable defaults do not assume Chris's monitors.
- Chrome profiles are not distinguishable by `chrome.exe`; observe a stable identity or leave profile homes unsupported.
- Normal windows tile. Repeated overlays become scratchpads. Loose floats are exceptions.
- Do not intercept Alt+Tab or spend Caps+Alt+Tab; the latter remains reserved for future native groups.
- Do not build Groupy integration, a fake grouping layer, a switcher, or Glazemakaze work during W0–W4.

## Other open work, deliberately outside this handoff

The TODO still contains kanata apps-mode UAT, game foreground suspend, the warm relay/tap-caps launcher, helper verbs, theming, install and release work. None blocks W0. Do not absorb them because they are nearby.

## Fresh-session prompt

> Work in `D:/dev/winmakase`. Read `CLAUDE.md`, `docs/TODO.md`, `spike/NOTES.md` § LIVE MACHINE STATE, `docs/plans/winmakase-window-policy.md`, and `docs/plans/handoff-w0-window-policy.md`. Execute W0 only. Stage and diff the generated Glaze config before touching the live file; preserve every machine-local rule in source, deploy with rollback ready, verify the full W0 acceptance list on the real rig, then commit. Do not start W1, Glazemakaze, Groupy, or a switcher.
