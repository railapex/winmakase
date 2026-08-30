# Glazemakaze — core patch train

Status: parked, cold-start ready
Created: 2026-08-30
Owner boundary: GlazeWM engine behavior only

## Outcome

Keep stock GlazeWM 3.10.1 on the live rig. Build Glazemakaze as a thin, rebased patch train with upstream-sized commits. Do not publish or adopt a permanent hard fork until upstream rejection, release paralysis, or a concrete production need makes that cost real.

The first work is recovery correctness. Speed comes next. Animations come last and stay off by default.

## Boundary

This track owns:

- GlazeWM container, event, platform-sync, monitor-identity and cleanup behavior.
- The official `animations-test` branch evaluation and any zero-cost disabled path.
- Patch extraction, benchmarks, invariant tests and upstream PRs.
- Optional local build/sign/package mechanics if a patched binary becomes necessary.

This track does not own:

- WinMakase supervision, task registration, keybindings, app homes, scratchpads, bars, themes or user-facing profiles.
- Direct Caps leader semantics and kanata retirement; that is the separate `glaze-caps-ownership.md` patch track.
- A fake tab UI outside the WM.
- Replacing GlazeWM with komorebi, Seelen UI or Groupy.

Those boundaries are deliberate. Product policy stays in WinMakase; window-manager primitives stay here.

## State at extraction

- Live rig: stock GlazeWM 3.10.1, supervised by WinMakase. The KhangHLe build was fully removed after A/B testing.
- Current workspace ownership is good enough: `1/4 -> monitor 0`, `2/5 -> 1`, `3/6 -> 2`; the occasional teleprompter stays virtually far right. This is index-stable, not hardware-identity persistence.
- KhangHLe A/B: animations on measured 163–225ms relayout; animations off measured 20–35ms typical against stock's 41–58ms. Its exit cloak leak reproduced twice. This proves the speed opportunity and cleanup danger in that fork only; it is not a benchmark of the official branch.
- Upstream correction: `main` and releases are stalled, but the maintainer did not simply vanish. The official `animations-test` branch is 93 commits ahead / 9 behind `main`, with work through 2026-06-20. Treat this as concentrated, unpublished development plus absent review bandwidth—not abandonment.
- Official animation config is already opt-in: `window_move` and `window_open` are optional. The disabled path still constructs animation state and runs decision logic in `platform_sync`.
- Official cleanup still calls `show()` and resets alpha without an explicit `set_cloaked(false)` in both `WmState::drop` and the watcher.
- The branch rewrites the same hot files needed by atomic positioning and hardening: `platform_sync`, `native_window`, `wm_state`, `main`, `manage_window` and event handling. Expect reimplementation, not clean cherry-picks.

Primary evidence:

- Official branch: https://github.com/glzr-io/glazewm/tree/animations-test
- Branch comparison: https://github.com/glzr-io/glazewm/compare/main...animations-test
- Atomic reposition PR: https://github.com/glzr-io/glazewm/pull/1409
- Monitor persistence PR: https://github.com/glzr-io/glazewm/pull/1284
- Focus-order fix: https://github.com/glzr-io/glazewm/pull/1404
- Local A/B record: `docs/research/gap-features.md` § KhangHLe A/B

## Decisions already made

1. **Thin patch train, not a hard fork.** Preserve upstream crate names, config schema and module boundaries. One concern per commit.
2. **No animation by default.** A later WinMakase `motion` profile may opt in; Glaze itself keeps the existing optional category config.
3. **No redundant `animations.enabled`.** Make the existing omitted-category state a literal zero-work path.
4. **No blind uncloak sweep.** UWP/tab-host windows can be legitimately cloaked. Cleanup acts only on HWNDs Glaze recorded as managed and visible.
5. **No animation branch on the live desktop before recovery passes.** Overlay windows, transparency and cloak widen the failure surface.
6. **No structural deletes.** Keep macOS modules, IPC, CLI, watcher and platform abstractions. Conditional compilation already removes runtime cost; deleting them only buys merge pain.
7. **No self-signing by default.** Build with `uiAccess=false` first. Add a local trusted certificate only if a focused test proves UIAccess behavior is required and worth the machine-trust change.

## Session cuts

Each cut is independently reviewable and should end with a commit or a written no-go verdict.

### G0 — reproduce and baseline the official branch

- Create an isolated worktree from official `animations-test`; do not reuse the disposable Khang clone as evidence.
- Rebase or merge `main` only in the worktree and record actual conflicts.
- Build `uiAccess=false` and run the existing relayout benchmark with animations omitted.
- Benchmark stock 3.10.1, official branch off, and official branch minimally on under the same window set and sample count.
- Trace allocations, DWM calls, capture, transparency and monitor-property reads on the disabled path.
- Do not swap the live WM. Use disposable test windows; never stop or replace `muxel-live`.

Exit: measured table, exact diff/conflict map, and a go/no-go for G1 against `main` versus `animations-test`.

### G1 — recovery invariants

- Add explicit uncloak + alpha reset to graceful cleanup and watcher cleanup.
- Track the managed/visible HWND set; restore only that set.
- Require watcher readiness before allowing cloak, or degrade to the safer hide method.
- Remove blocking system-modal dialogs from supervised runtime error paths; log and publish degraded health instead.
- Restore other presentation state Glaze changes: taskbar visibility, topmost, titlebar/corners and layered state where applicable.
- Test graceful exit and forced termination with tiled, floating, fullscreen, hidden and elevated-window cases.

Exit: no managed test window remains cloaked, transparent, topmost or absent after either exit path.

### G2 — event and monitor correctness

- Carry/fix #1404 before #1284.
- Coalesce display-change ingress into a capacity-one dirty signal; reconcile the latest snapshot after it stabilizes.
- Make display reconciliation discover → plan → apply → validate. Do not partially mutate the live tree and then return an error.
- Skip identical monitor snapshots and no-op redraws.
- Add tree invariants: parent/child agreement, unique focus order, one displayed workspace per monitor, finite positive tiling sizes, unique window handles/workspace names.
- Test disconnect/reconnect, reverse enumeration, identical monitors, duplicate events, work-area-only changes and transient query failures.

Exit: monitor tests pass without relying on WinMakase restart recovery. Hardware identity remains optional until #1284 proves insufficient on this rig.

### G3 — raw and perceived speed

- Extract atomic/batched positioning rather than importing the animation stack wholesale.
- Integrate #1409 semantically: animation absent uses the atomic/no-op path; animation present uses its compositor transaction.
- Remove redundant alpha, border, frame and z-order writes after profiling proves they are redundant.
- Replace blind delayed z-order retries with generation-checked work or remove them once atomic positioning is sufficient.
- Preserve a benchmark harness for close, open, swap, focus, move and workspace switch. Report median and tail, not one best run.

Exit: animation-off is no slower than stock on every case, faster on relayout, and produces no new visible flash.

### G4 — animation branch integration and upstream PRs

- Make omitted animation categories bypass manager construction, channels, decision logic, monitor refresh reads, capture and transparency.
- Add an operation-parity test: disabled animations produce the same platform operation sequence as the non-animation baseline.
- Keep workspace animations off. Evaluate only short open/move effects after the zero path is proven.
- Submit small upstream PRs in this order: focus-order correctness, cleanup/uncloak, disabled-animation fast path, event coalescing, atomic positioning. Do not submit a branded fork dump.
- With Chris's approval, read `#glazewm-dev` before posting to learn whether the branch is being rebased or replaced. Draft any message; do not auto-send.

Exit: each patch stands alone upstream and in the downstream queue.

### G5 — fork/package gate

Open this cut only when one trigger fires:

- A required patch sits unreviewed through two upstream release opportunities.
- Stock repeats a crash/recovery defect that WinMakase cannot safely contain.
- The animation branch lands without a genuinely cheap off path.
- Native grouping becomes important enough to justify owning the container model.

Then:

- Create the public Glazemakaze fork and a `winmakase` patch branch; upstream `main` remains a clean remote.
- Automate rebase, tests, benchmark comparison and binary provenance.
- Package only after the cleanup and monitor suites pass.
- Decide `uiAccess` from evidence. If signing is needed, document certificate creation, trust installation, rotation and removal; never silently install trust.

## Parked ideas

- Native grouping/tabbed containers: legitimate Glaze-core work, but after G1–G3. Do not fake it in WinMakase or adopt Groupy as infrastructure.
- Master-stack, scrolling/column layouts and mouse grab/resize: evaluate after the recovery/performance base is quiet.
- Compile-time animation feature: downstream option only if the runtime zero path still leaves material binary/startup cost. It is not the first upstream ask.

## Completion condition

This plan is complete when either:

- the selected patches land upstream and WinMakase returns to an official release, or
- Glazemakaze is a reproducible, tested thin fork with a documented rebase cost and no live recovery regression.
