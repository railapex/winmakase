# Direct Caps candidate — reopened P0 gate

Observed 2026-09-08. No keyboard cutover performed. Independent reviewer examined existing source; textual compatibility is not runtime acceptance.

The [archived spike](../../../plans/archive/2026-09-08/glaze-caps-ownership.md) records automated reducer/hook tests and an injected live A/B, followed by restoration of the old stack. Current tasks/processes/config confirm that Kanata remains deployed. The active v1 plan initially failed to carry this remaining adoption gate forward.

## Candidate

- Upstream base: Glaze v3.10.1 `56a8bdee81e9d1a11d6ad840b83b183bb1b3195f`.
- Deployed AppID source: `6d62d6616e6acdfd1c1d35ad42ba468f9bb7972e`.
- Direct leader: `ba1c7e4061454a109d9891733f97a51d59982fa9`.
- Separate cleanup fix on that branch: `f17c552` (retain independent identification).
- Worktree: `D:/dev/glazewm-wt-caps-leader`; clean at review. A read-only three-way merge preview against the AppID branch had no conflict markers. No combined build has passed yet.

Kanata is not necessary for Glaze's window commands. Its current job is Caps→right Win and Scroll Lock→Caps. The candidate directly consumes physical Caps and dispatches matching chords. It already supports no tap action: declare the leader, retarget chords to `caps_lock`, and omit a bare Caps binding. Add a test for that exact configuration. Removing Kanata is conditional on acceptance, not on the number of mappings.

## Findings to resolve

1. **Explicit launcher required.** Caps+Space currently depends on synthetic Win+Space reaching PowerToys; the keymap's native launcher entry emits no Glaze command. Direct Caps needs an explicit working Run activation action plus renderer support. Tap launching can remain deferred.
2. **Bounded hook work.** The hook callback blocks on the state mutex while reload constructs its binding map under the same mutex. Prepare replacement data outside the lock, define contention behavior and test held-key reload/pause transitions. A failed acquisition must not create a partial press or silently toggle Caps.
3. **Injected-key policy.** The candidate deliberately treats injected Caps as a leader. PowerToys Scroll Lock→Caps may therefore become another leader rather than raw Caps Lock. Decide and verify coexistence; do not promise the old hatch survives automatically.
4. **Alt/release behavior.** Exercise Caps+Alt families and reversed releases for menu activation/focus side effects. This is a source-derived risk, not an observed failure.
5. **Remaining physical gate.** Literal Caps, elevated foreground, launcher/Keyboard Manager startup order, sleep/wake/reboot, ignored/fullscreen game behavior, and graceful/crash teardown remain open. Sandbox cannot replace physical keyboard/game/display evidence.

Direct Caps would also remove the synthetic Win+L collision. It does not supply Glaze's missing scrolling-layout feature. Reclassify reserved chords for the chosen input target after acceptance.

Next slice: combine the existing patches in a separate worktree, address the bounded-hook/transition contract, prove no-tap behavior and explicit launcher activation, then run the remaining controlled acceptance. Keep the currently working input stack until the candidate and rollback are ready for the cutover checkpoint.
