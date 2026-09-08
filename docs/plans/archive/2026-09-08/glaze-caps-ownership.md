# Glaze — direct Caps leader ownership

Status: parked after successful C0/C1 and live C2 A/B; physical acceptance remains
Created: 2026-08-30
Owner boundary: keyboard-hook semantics in Glaze plus WinMakase dependency retirement

## Outcome

Make physical Caps the window-manager leader without synthesizing either Windows key:

- Tap Caps invokes the warm WinMakase launcher path.
- Hold Caps and press another key invokes the existing Omarchy/WinMakase chord.
- Physical left-Win and every native Windows shortcut remain untouched.
- A dead or stopped Glaze removes its hook and returns Caps to Windows; there is no stuck synthetic modifier.
- Retire kanata when the direct path passes the full live gate. Do not keep a privileged keyboard remapper for hypothetical future layers.

W0-W4 keep the live-proven Caps -> right-Win architecture. This track does not alter their acceptance gate or unpark the wider Glazemakaze core plan.

## Spike result — 2026-08-30

The narrow leader primitive works on stock GlazeWM 3.10.1 source. The clean worktree is `D:/dev/glazewm-wt-caps-leader`, branch `caps-leader-spike`:

- `ba1c7e4 feat: add dual-role keybinding leaders` — config schema, pure reducer, Windows/macOS event plumbing, matcher reuse, paused-mode passthrough and isolated hook/injection harnesses.
- `f17c552 fix: uncloak windows during cleanup` — separate cleanup fix exposed by the live A/B; graceful exit and watcher crash cleanup now uncloak only windows DWM reports cloaked.

The reducer's 12 transition tests pass. Formatting, clippy, workspace check and the targeted builds pass. A real `WH_KEYBOARD_LL` hook plus `SendInput` harness passed clean tap, long hold, reverse release, modifier chord, unbound chord, repeat, rapid taps and Caps-state preservation. The live patched-Glaze A/B passed Caps tap, Caps+1, Caps+Shift+Tab, Caps+Ctrl+Tab, pause passthrough and graceful hook teardown. Disposable-window probes passed graceful and watcher-crash uncloak cleanup. Stock WinMakase/Glaze/Kanata/Zebar was restored after the test.

That is not the adoption gate. The harness used injected input because the Windows-control bridge was unavailable. Physical feel, an elevated foreground window, reboot/hook ordering, PowerToys coexistence and a real fullscreen game remain open. C3 must not remove kanata until those literal tests pass on the patched build.

## Why stock Glaze cannot do this now

Stock GlazeWM 3.10.1 already parses `caps_lock` and can recognize a binding such as `caps_lock+w`. That is necessary but not sufficient.

The listener indexes bindings by their final trigger key. On Caps-down, no `caps_lock+w` candidate exists because `w` has not arrived, so the hook passes Caps to Windows and Caps Lock toggles. When `w` arrives, the listener can observe Caps held and consume `w`, but the damage is already done. The callback also rejects every key-up event before matching, so it has no release edge on which to resolve a tap. There is no pending-key state, timeout, interruption rule or replay/injection filter.

Binding Caps alone does not fix it: stock Glaze would fire the tap action immediately on key-down, before it can know whether the user intended a chord.

The hook itself is the right ownership point. `WH_KEYBOARD_LL` sees key-down and key-up before the target window receives them and can consume an event by returning nonzero. Microsoft says the callback runs in the installing process rather than being injected into the target process. Keep its work bounded: Windows silently removes a low-level hook that overruns the hook timeout.

## Decision

Add one narrow dual-role **leader** primitive to Glaze's existing keybinding listener. Do not turn Glaze into a general keyboard remapper.

Behavioral contract:

1. Leader down starts a pending press and is consumed. Auto-repeat cannot restart its clock.
2. Any other key-down interrupts the tap, even when no configured chord matches. This prevents `Caps+x` from typing `x` and then launching on release.
3. A matching leader chord dispatches immediately on the other key's down edge. It never waits for the tap timeout.
4. Leader up is consumed. If the press was uninterrupted and inside `tap_max_ms`, dispatch the leader's tap binding; otherwise do nothing.
5. A long solo hold does nothing. There is no synthetic modifier to press or release.
6. Disable, reload and shutdown clear pending state. Since no input was synthesized, cleanup cannot leave Windows holding a key.
7. Unbound second keys pass through normally. Only the owned leader and matched trigger are consumed.

Keep the state machine platform-neutral and pure where possible. The Windows hook must expose press/release, timestamp and injected-event metadata; the matcher owns policy. Do not sleep, spawn, call IPC or wait inside the hook callback. It may enqueue a resolved Glaze binding and return.

The spike froze the smallest YAML seam as an explicit leader declaration plus ordinary bindings, not magic behavior for every key that happens to appear both alone and as a prefix:

```yaml
general:
  keybinding_leader:
    key: caps_lock
    tap_max_ms: 180

keybindings:
  - commands: ['shell-exec winmakase launcher']
    bindings: ['caps_lock']
  - commands: ['close']
    bindings: ['caps_lock+w']
```

The single-key binding is resolved on release only because its key is the declared leader. Other single-key bindings keep existing key-down behavior.

## Tap/hold answer

Yes. Direct ownership permits tap versus hold because the Glaze hook already sees both physical edges; it simply discards releases today. The intended feel is `tap-hold-press`: another key-down commits to the hold/chord path immediately, while a quick clean release commits to tap.

Do not implement the tap by replaying Caps with `SendInput`. The product tap is a Glaze command to the resident WinMakase launcher relay. That avoids injection recursion and UIPI entirely. Microsoft documents that `SendInput` is blocked when injecting into a higher-integrity target; no input injection is needed here.

## Kanata retirement

Live caps mode gives kanata two mappings only:

- Caps -> right-Win.
- Scroll Lock -> raw Caps Lock as the escape hatch.

The untested apps mode and game-foreground suspension are planned possibilities, not current product duties. Direct Glaze ownership replaces the first duty. Hook teardown makes the second recovery duty unnecessary: when Glaze exits, Windows owns Caps again. PowerToys' independent Scroll Lock -> Caps Lock mapping may remain if raw Caps Lock is still wanted.

Once the live gate passes, remove kanata rather than leaving dead architecture:

- Remove the kanata config renderer, binary dependency and elevated scheduled task.
- Collapse the supervisor's kanata/Glaze linked-pair rule to ordinary Glaze supervision.
- Remove kanata-specific health, logs, installer, reload, panic and shutdown paths.
- Replace the game-suspend TODO with a narrower Glaze foreground-bypass question if real game testing shows a need.
- Keep native left-Win and the external `WinmakasePanic` recovery route.

Kanata remains the right tool if WinMakase later chooses arbitrary layers, home-row modifiers, macros or system-wide remapping outside Glaze. None is in the current product boundary. Re-adding a remapper for a concrete feature is cheaper than carrying an elevated dependency indefinitely.

## Rejected alternatives

### Keep Caps -> right-Win

Proven and suitable for W0-W4, but not the end state. If Glaze dies while the remapper survives, raw Windows shortcuts fire. That coupling created the supervisor's linked-pair rule and its shutdown/restart complexity.

### Caps -> F24 in kanata

Technically valid. Glaze treats F24 as an ordinary held key, so `f24+w` works and cannot collide with Windows shortcuts. Kanata could also emit one spare function key for tap and hold F24 for chords.

This is a decent emergency bridge, not an architecture: it retains two hooks, the privileged task, the linked lifecycle and the dependency. It also does not deliver the launcher tap without adding more kanata policy. Do not migrate W0 to F24 merely to migrate again.

### Glaze binding modes

Binding modes change the active keymap after a command. They do not defer one physical press, consume Caps-down, or resolve tap versus hold. They solve modal resize maps, not a dual-role leader.

## Risks to prove, not assume

- **Physical input:** the actual low-level hook passed injected-input automation, but Chris has not yet tested the patched build with literal Caps presses. Timing/feel remains an adoption gate.
- **Elevated foreground:** direct handling consumes physical input and dispatches a Glaze command; it does not inject output. Microsoft documents low-level hooks as running in the installer process, while UIPI blocks cross-integrity message/hook operations and `SendInput` injection. The exact physical behavior with an elevated terminal focused still needs a live test.
- **Hook ordering:** PowerToys and other low-level hooks can still run before or after Glaze. The acceptance test must cover PowerToys Run and Keyboard Manager enabled, then reboot once to expose order changes.
- **Injected events:** expose `KBDLLHOOKSTRUCT.flags` and tag/ignore any future Glaze-generated input. The first implementation should generate none.
- **Fast rollover:** test Caps+Shift+key, near-simultaneous Caps/key presses, autorepeat, unbound chords and release-order inversions. The leader state must not stick.
- **Games:** stock Glaze has manual `wm-toggle-pause`, not automatic foreground-game bypass. The spike fixed paused mode so keys actually pass through. Verify a real ignored/fullscreen game; if automatic policy is needed, WinMakase should watch foreground state and pause/unpause Glaze rather than resurrect synthetic Win.
- **Launcher latency:** tap dispatch must reach the resident WinMakase relay without a process spawn. Measure key-up to first paint; target one frame on the rig.

## Session cuts

Each cut ends with a commit or a written no-go. Use disposable test windows. Never stop, replace, overwrite or smoke-test `muxel-live`.

### C0 — pure leader state machine — complete (`ba1c7e4`)

- Extract a pure event reducer: key, down/up, timestamp, injected flag -> pass/consume plus optional resolved binding.
- Add table tests for tap, long solo hold, chord, modifiers, unbound key, repeat, reverse release, disable, reload and two rapid presses.
- Prove the existing non-leader keybinding suite is byte-for-byte unchanged.
- Freeze the smallest config schema only after the event table reads cleanly.

Exit passed: 12 deterministic tests cover every transition; no Windows API or timer is required by the reducer.

### C1 — isolated Windows hook harness — automated pass (`ba1c7e4`)

- Wire press/release, timestamp and hook flags through `KeyEvent`.
- Run the reducer from the existing hook and enqueue resolved bindings; no work may block in the callback.
- Add a small disposable harness that records consumed/passed events and resolved commands without starting the window manager.
- Exercise Caps tap/hold, unbound keys, modifiers, repeat and rapid rollover through Windows-control automation.
- Record hook latency and require a wide margin under `LowLevelHooksTimeout`.

Exit passed under injected-input automation: the real hook owns Caps without toggling Caps Lock, fires tap only on clean release, and never emits a Windows key. Literal physical input remains in C2.

### C2 — patched Glaze A/B — automated/live pass; physical gate open

- Build a minimal patch on stock GlazeWM 3.10.1 source with animations absent; do not bundle the wider Glazemakaze patch train.
- Back up config and binary, stage a rollback command, then swap only after the isolated harness passes.
- Retarget a temporary copy of the rendered grammar from `rwin` to `caps_lock`.
- Test every W0 chord physically, native left-Win+Tab, native Alt+Tab, an elevated terminal, PowerToys Run, ignored/fullscreen game behavior, sleep/wake and one reboot.
- Verify Glaze crash/exit returns Caps to native Windows immediately. Verify no Win-key shell shortcut fires during failure.
- Restore stock after the A/B unless the entire gate is green and Chris explicitly adopts the patch.

Current evidence: live command routing, pause passthrough, graceful teardown and both graceful/crash uncloak recovery pass. Exit remains open until literal physical Caps, elevated focus, PowerToys/reboot ordering and a fullscreen game pass.

### C3 — WinMakase migration and kanata removal

- Add a keymap target for the Glaze leader grammar; do not search/replace golden YAML by hand.
- Remove kanata from config, supervision, task registration, health, logs, reload, panic and install flows.
- Delete the linked-pair recovery rule and its stale-failure commentary.
- Update DESIGN, TODO, README, templates, migration and uninstall restoration.
- Run the full workspace suite, installer task tests and a real restart/crash/reboot pass.

Exit: no kanata process, task, config, binary or product claim remains; Caps tap/hold and all W0 bindings pass physically.

### C4 — upstream-sized patch

- Split event plumbing, pure reducer and config schema into reviewable commits.
- Submit the generic leader behavior upstream only after the live A/B proves it. Keep WinMakase launcher policy downstream.
- If upstream stalls, carry the patch as one thin Glazemakaze commit with a rebase test.

Exit: upstream adoption or one isolated, reproducible downstream patch with an explicit rebase cost.

## Primary evidence

- GlazeWM 3.10.1 listener (final-trigger map; key-up rejected): https://github.com/glzr-io/glazewm/blob/v3.10.1/packages/wm-platform/src/keybinding_listener.rs
- GlazeWM Windows hook (`WH_KEYBOARD_LL`, press/release available at the platform edge): https://github.com/glzr-io/glazewm/blob/v3.10.1/packages/wm-platform/src/platform_impl/windows/keyboard_hook.rs
- GlazeWM configuration/key list and binding modes: https://github.com/glzr-io/glazewm
- Microsoft `LowLevelKeyboardProc`: https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
- Microsoft `KBDLLHOOKSTRUCT` flags: https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct
- Microsoft `SendInput` UIPI boundary: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
- Microsoft UIAccess requirements: https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/security-policy-settings/user-account-control-allow-uiaccess-applications-to-prompt-for-elevation-without-using-the-secure-desktop
- Kanata tap-hold behavior and scope: https://github.com/jtroo/kanata/wiki/Configuration-guide

## Completion condition

This plan is complete when Glaze directly owns physical Caps as a tested tap/hold leader, native Windows keys remain untouched, kanata is absent from the installed WinMakase stack, and crash/reboot/elevated/game acceptance shows no input trap or leaked OS shortcut.
