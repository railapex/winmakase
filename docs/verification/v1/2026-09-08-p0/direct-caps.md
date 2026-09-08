# Direct Caps candidate — reopened P0 gate

Observed 2026-09-08. No keyboard cutover performed. Independent reviewer examined existing source; textual compatibility is not runtime acceptance.

The [archived spike](../../../plans/archive/2026-09-08/glaze-caps-ownership.md) records automated reducer/hook tests and an injected live A/B, followed by restoration of the old stack. Current tasks/processes/config confirm that Kanata remains deployed. The active v1 plan initially failed to carry this remaining adoption gate forward.

## Candidate

- Upstream base: Glaze v3.10.1 `56a8bdee81e9d1a11d6ad840b83b183bb1b3195f`.
- Deployed AppID source: `6d62d6616e6acdfd1c1d35ad42ba468f9bb7972e`.
- Direct leader: `ba1c7e4061454a109d9891733f97a51d59982fa9`.
- Separate cleanup fix on that branch: `f17c552` (retain independent identification).
- Original worktree: `D:/dev/glazewm-wt-caps-leader`; clean at review. The patches combine in `D:/dev/glazewm-wt-v1-input`, branch `winmakase-v1-input`. Independent patch-ID comparison confirmed the original leader and cleanup patches at `1dd80ddb18f2bf60322e26d8549ab7fad2faa53f`; that commit adds one missing semicolon required by Clippy. The reviewed input-contract follow-up is `371a448c92dca053e0834d346d314d344aaa943f`, the current clean candidate.

Initial combined-source verification: 12 leader reducer tests and one matcher test passed at parent `f4d3348`; [captured output](direct-caps-tests.txt). Workspace check passed; formatting and strict workspace Clippy passed after the semicolon fix. Toolchain was `rustc 1.100.0-nightly (17fd5b8a3 2026-08-28)`, target `x86_64-pc-windows-msvc`, `VERSION_NUMBER=3.10.1`, default features with no `ui_access`. These selected tests install no hook. No combined application was launched or deployed.

Follow-up `371a448`: independent source review passed; lead reran 22 leader tests and one matcher test successfully with `--locked`. Builder formatting, workspace check and strict workspace Clippy passed. Lead built `glazewm`, `glazewm-cli` and `glazewm-watcher` in release mode for `x86_64-pc-windows-msvc`, using the same Rust version and `VERSION_NUMBER=3.10.1`. [Build identifiers, commands and artifact hashes](direct-caps-build.json) are recorded; the three executables and manifest are in the [guest kit](guest-kit.md). Building and hashing did not launch the candidate.

The builder's wider unfiltered `wm-platform` run had 30 passes and four failures: `test_key_conversion_roundtrip`, `dispatch_sync_with_nested`, `dispatch_sync_executes_in_order` and `dispatch_sync_from_different_threads`. A clean detached checkout at parent `1dd80dd` reproduced all four failures (20 passed, 4 failed), establishing that they predate this follow-up. [Raw parent output](direct-caps-parent-tests.txt). Both runs used `VERSION_NUMBER=3.10.1` and the same test target with one thread; the parent command also specified `+nightly` and `--locked`. The dispatcher tests create hidden process-local message windows but install no keyboard or mouse hooks. The full suite is still failing; this slice does not fix those failures. The build manifest retains the earlier, timestamped pending-baseline status from when the ZIP was staged.

Kanata is not necessary for Glaze's window commands. Its current job is Caps→right Win and Scroll Lock→Caps. The candidate directly consumes physical Caps and dispatches matching chords. It already supports no tap action: declare the leader, retarget chords to `caps_lock`, and omit a bare Caps binding. Add a test for that exact configuration. Removing Kanata is conditional on acceptance, not on the number of mappings.

## Findings and disposition

1. **Explicit launcher required.** Caps+Space currently depends on synthetic Win+Space reaching PowerToys; the keymap's native launcher entry emits no Glaze command. Direct Caps needs an explicit working Run activation action plus renderer support. Tap launching can remain deferred.
2. **Hook/reload contract fixed in source at `371a448`.** Mutable state now stays on the existing dispatcher thread through `ThreadBound<RefCell<_>>`; replacement maps are built before dispatch, swapped briefly and retired on the caller. Reload and pause transitions cancel pending actions while retaining owned repeat/release drainage. Physical-down tracking prevents adoption of a native press on autorepeat after a configuration change or re-enable. Pause preserves the existing filtered resume bindings, including Caps-based resume. The callback's borrow-failure branch is an invariant alarm, not a recovery mechanism: Windows removes the callback during invocation and update/enable make no native calls while borrowed. Runtime timing remains unmeasured.
3. **Injected-key policy fixed in source at `371a448`.** Injected configured-leader edges pass through; raw Windows key state cannot make injected Caps satisfy a leader chord. Selected tests cover the policy, including a physical leader held across injected edges. PowerToys hook ordering and Scroll Lock→Caps coexistence still need runtime acceptance.
4. **Alt/release behavior.** Exercise Caps+Alt families and reversed releases for menu activation/focus side effects. This is a source-derived risk, not an observed failure.
5. **Remaining physical gate.** Literal Caps, elevated foreground, launcher/Keyboard Manager startup order, sleep/wake/reboot, ignored/fullscreen game behavior, and graceful/crash teardown remain open. Sandbox cannot replace physical keyboard/game/display evidence.

Direct Caps would also remove the synthetic Win+L collision. It does not supply Glaze's missing scrolling-layout feature. Reclassify reserved chords for the chosen input target after acceptance.

Next slice: implement and visibly prove explicit launcher activation, then run controlled guest and physical acceptance. No-tap and held-key transitions now have selected test evidence. Keep the currently working input stack until the candidate and rollback are ready for the cutover checkpoint.
