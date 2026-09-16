# Winmakase build handoff

- **Audience:** Astra lead, Sol builders
- **State date:** 2026-09-16
- **Winmakase checkpoint:** `8ec83dc` on `main`
- **Workspace version:** `0.1.8` (development version; no release tag)
- **Paired Glaze checkpoint:** `6bc83d1` on `winmakase-v1-input`
- **Daily deployed Glaze checkpoint:** `9c8e93b` (older than the paired candidate)
- **State authority:** [`docs/plans/v1/NEXT.md`](docs/plans/v1/NEXT.md) and [`docs/plans/v1/execution-state.json`](docs/plans/v1/execution-state.json)

This file is the build and delegation brief. It explains the product arc, the four cuts to daily dogfood, the Glaze patch strategy, and the release horizon. It does not replace the execution ledger. Every accepted slice still closes through `NEXT.md`, `execution-state.json`, evidence, design/TODO updates where contracts changed, and the changelog.

## Position

Winmakase already has a credible spine: a supervised GlazeWM/Kanata/Zebar stack, generated keymap, scratchpads, reflow, app homes, native window facts, guarded PowerToys Run activation, recovery machinery, a theme adapter core, and a staged verification model. The remaining job is integration and product discipline. The project does not need another architecture search before dogfood.

The target is a dependable Windows desktop for one real daily machine. It should preserve native Windows escape hatches, make the high-frequency window actions feel intentional, recover without drama, and install or roll back without archaeology. Omarchy is the interaction reference. Literal feature parity is not the release test.

The shortest sound path is four cuts:

1. close the input decision;
2. complete the daily window loop;
3. make the visible shell work offline;
4. prove lifecycle, install, and rollback.

After those cuts, begin daily dogfood. Smoothing, quality of life, themes, broader verification, packaging, and release communication happen against observed use rather than guesses.

## Non-negotiable boundaries

- Keep the native Windows taskbar available as the fallback until the replacement controls and recovery path pass their gates.
- Treat Winmakase and its patched Glaze binary as a paired deployment. New native window facts change the IPC DTO; this is not backward-compatible with an older Glaze process.
- Do not test disruptive shell, failure, or recovery paths on the daily machine when the disposable Windows lane can prove them.
- A direct Caps leader must pass with a literal physical Caps press. Synthetic Caps cannot satisfy the protected-input path by design.
- Kanata is an acceptable v1 component. Removing it is useful only if direct Caps is clean and dependable; it must not hold daily dogfood hostage.
- Keep implementation ownership clear: Winmakase owns policy, composition, supervision, user configuration, packaging, and evidence. Glaze owns primitives that require compositor/window-manager internals.
- No platform rewrite during v1. A replacement window manager or shell needs measured failure, a bounded proof, and a rollback plan.

## Four cuts to daily dogfood

### Cut 1: Close input and launcher activation

**Question:** Can the paired Glaze build accept physical Caps as the leader across ordinary and protected contexts without stealing taps, sticking modifiers, or losing recovery?

Work:

- Run the remaining P0 input acceptance with literal physical Caps.
- Exercise normal apps, elevated/protected surfaces, rapid chord sequences, reload, suspend/resume, and emergency recovery.
- Confirm the guarded PowerToys Run path from both keymap and bar button.
- Record whether direct Caps wins or Kanata remains the v1 input layer.
- Preserve Scroll Lock or the documented escape path in either outcome.

Exit:

- One input architecture is selected and documented.
- Every protected/context test has evidence.
- The daily machine can escape a broken mapping without editing files first.
- P0.I2 and its dependent P0 input rows are accepted or explicitly closed with Kanata retained.

This is an Astra-led acceptance decision. A Sol builder can extend automation and fixtures, but automation cannot substitute for the physical-key proof.

### Cut 2: Complete the daily window loop

**Question:** Can the user launch, find, arrange, recover, and dismiss daily work without dropping into scripts or Glaze internals?

Work:

- Finish typed app roles and stable identity refinement.
- Implement focus-or-launch and launch-new as separate actions.
- Finish float recovery and the terminal toggle.
- Add the missing split-next interaction.
- Ship the first daily layout.
- Bound agent-window placement so transient tool windows do not poison app homes or workspace policy.
- Exercise dialog placement and reload preservation on the paired `71b77d3`/`6bc83d1` implementation.

Exit:

- Browser, terminal, editor, file manager, settings/dialog, and agent-window flows have repeatable acceptance cases.
- Reload preserves live workspace, tiling/floating state, geometry, and focus.
- Focus-or-launch never creates a duplicate when a matching usable window exists; launch-new always creates a new instance when the application supports it.
- Failure paths return a useful error and leave the current desktop intact.

Suggested parallel lanes after Astra freezes the role/action contracts:

- **Sol A:** app-role schema, identity, focus-or-launch, launch-new.
- **Sol B:** float recovery, terminal toggle, split-next, first layout.
- **Astra:** agent-window admission policy, shared interface review, paired integration.

### Cut 3: Make the visible shell work offline

**Question:** If the network is absent and a component restarts, does the desktop still expose the controls needed to work and recover?

Work:

- Vendor every runtime bar asset; no CDN, web-font, icon, or remote-script dependency.
- Finish the launcher/control popup.
- Make tray overflow, user-chosen persistent tray order, and calendar behavior deliberate.
- Add the native settings and recovery entry points required by the controls plan.
- Verify provider lifetime, popup focus, menu anchoring, multi-monitor docking, DPI changes, and bar restart.
- Keep the native taskbar fallback discoverable and tested.

Exit:

- Airplane-mode startup and restart render all required controls.
- Tray, calendar, launcher, settings, and recovery actions work from the visible shell.
- A failed or missing bar does not strand the session.
- Multi-monitor and DPI changes converge without manual cleanup.

Suggested lanes after Astra fixes the popup/focus/provider contracts:

- **Sol A:** asset vendoring, packaging, offline verification.
- **Sol B:** popup, tray order, calendar, and native actions.
- **Astra:** focus/provider lifetime, monitor composition, integration review.

### Cut 4: Prove lifecycle, installation, and rollback

**Question:** Can Winmakase own the session cleanly from logon through failure and removal?

Work:

- Finish singleton ownership, request routing, readiness, and component state reporting.
- Define start, adopt, reload, degraded, restart, shutdown, panic, and uninstall behavior for every component.
- Complete staged install/update/uninstall with a journal and deterministic rollback.
- Pin and verify the paired Glaze payload and every redistributed artifact.
- Exercise crash loops, permission denial, display churn, low commit headroom, interrupted install, and partial removal in the disposable lane.

Exit:

- A second Winmakase invocation routes to the owner or fails clearly; it never creates competing supervisors.
- Startup reaches a truthful ready/degraded state.
- Panic and uninstall restore taskbar/input state and remove owned tasks/files without touching user-owned configuration unless explicitly requested.
- An interrupted install can resume or roll back from its journal.
- The release candidate can be installed and removed on a clean disposable Windows instance.

Suggested lanes after Astra freezes lifecycle states and the journal schema:

- **Sol A:** lifecycle owner, request routing, readiness, recovery.
- **Sol B:** installer, artifact verification, journal, rollback/uninstall.
- **Astra:** failure semantics, security review, destructive-case review, integration.

## Orchestration rules

Use Astra as the lead and reviewer for shared contracts, platform-sensitive behavior, and the final composition. Use Sol builders for bounded implementation slices whose inputs and exit tests are already clear.

- Run at most two implementation lanes at once. Preserve the fourth slot for review or a targeted investigation.
- Give each builder its own branch/worktree and a disjoint write set.
- Freeze shared schemas and public interfaces before parallel work starts.
- Require each handoff to name base commit, changed files, exact checks run, evidence produced, remaining risk, and the next dependency.
- Accept behavior into `main` only after the execution ledger, evidence, changelog, and contract docs agree.
- Keep implementation commits about one behavior. A docs-only correction may stand alone when it changes acceptance truth rather than product behavior.
- Review Glaze and Winmakase as a pair whenever an IPC DTO, match field, rule lifecycle, or generated config changes.

The detailed delegation mechanics and handoff fields are in [`docs/plans/v1/execution.md`](docs/plans/v1/execution.md) and [`docs/plans/v1/handoff-template.md`](docs/plans/v1/handoff-template.md).

## Glaze patch ledger

The Glaze work is tracked as seven commits on top of the shared stock/upstream commit `56a8bde`. The local branch's `origin` URL points at `KhangHLe/glazewm`, but none of the 66 later Khang-only commits are ancestors of our branch. We do not depend on Khang's animation/batching work.

| Commit | Change | v1 role | Upstream posture |
|---|---|---|---|
| `6d62d66` | AppUserModelID matching and IPC exposure | Stable app homes | Strong candidate. Generic Windows identity primitive, bounded and testable. Port to current official main. |
| `fc8b20f` | Dual-role keybinding leaders | Direct-Caps candidate | Weak as one PR. Large, hook-sensitive, and not yet physically accepted. Keep local until the input decision; then split or redesign for upstream. |
| `f4d3348` | Uncloak windows during cleanup | Clean recovery | Strong small bug-fix candidate if reproduced on current official main. |
| `1dd80dd` | Correct cleanup warning text | Companion cleanup | Fold into the cleanup port if still applicable. |
| `371a448` | Harden dual-role handling | Direct-Caps candidate | Same risk as `fc8b20f`; treat both as one local feature family until accepted. |
| `dd8fb7d` | Recover from permission-denied tile placement | Prevent empty tiles | Good bug-fix candidate, but the current patch is broad. Reproduce and reduce against official main before proposing. |
| `6bc83d1` | Native dialog facts, owner placement, and reload preservation | Correct dialogs and non-destructive reload | Valuable, but too many review decisions in one port. Split native facts/owner matching from rule-lifecycle preservation. |

Evidence and product state remain in the Winmakase verification ledger. For every Glaze port, add a compact delivery record: local commit, original base, official target base, test/evidence links, paired Winmakase minimum, upstream branch, and PR state.

### Why the checkout says KhangHLe

The remote came from an earlier A/B evaluation of Khang's development fork. Its animation/batching work was rejected as the production base after cloak leaks. Our branch instead starts at `56a8bde`, a commit shared with official upstream, and adds only the seven Winmakase commits above. We did not need Khang-only code for the accepted Winmakase work.

So the present state is less dramatic than the remote name suggests:

- **Dogfood base:** keep the exact tested `6bc83d1` artifact paired with Winmakase while v1 work proceeds.
- **Contribution base:** port each generally useful change onto current `glzr-io/glazewm` main in a clean worktree.
- **Do not submit:** the integration branch, Khang's main branch, or a 2,800-line omnibus PR.

The official repository still has focused PRs and code activity, but review latency is real. Upstream acceptance is therefore a maintenance objective, not a v1 gate. The GPL fork is a valid escape hatch as long as source, notices, patches, provenance, and reproducible artifact hashes stay exact.

### Likely acceptance, with the uncertainty stated

No one can promise maintainer acceptance. Based on scope, generality, and review surface:

1. **Best first PR:** cleanup uncloak, with a current reproduction and regression test.
2. **Next:** AppUserModelID, ported cleanly with Windows-only fact collection and matching tests.
3. **Then:** permission-denied placement recovery, reduced to the smallest fix that preserves allocation invariants.
4. **Then:** native dialog/owner/resizability facts and matching, separate from reload semantics.
5. **Then:** reload preservation, after agreeing on rule lifecycle semantics.
6. **Last, if retained:** dual-role leader, split into reviewable primitives after physical acceptance and an upstream design discussion.

This ordering gets the obvious bugs and reusable Windows primitives reviewed first. It also avoids asking maintainers to understand the entire Winmakase product policy in one sitting.

## Repository and release hygiene

The Winmakase commit history is generally sound: current work uses scoped subjects (`[p0]`, `[docs]`, `[supervisor]`, `[window-policy]`, `[keymap]`), accepted slices pair product and evidence, and the changelog records user-visible behavior. D1 and I1 are represented in `[Unreleased]`.

The weak point is publication and release bookkeeping, not granularity. Local `main` is 51 commits ahead of `origin/main`, there are no Winmakase PRs or local tags, and several development version bumps still live under one `[Unreleased]` section. That is too much unpublished history for the next phase. A Cargo version bump is not a release.

Before parallel building:

1. Publish the current Winmakase checkpoint as the baseline.
2. Create feature branches/worktrees from that remote baseline.
3. Merge each of the four cuts through reviewable Winmakase PRs instead of continuing to stack local `main`.
4. Keep Glaze ports in separate branches based on official upstream main.
5. Tag the first daily-dogfood cut and start a release section instead of letting `[Unreleased]` grow indefinitely.
6. Add a release manifest that pins the Winmakase version, Glaze base and patch series, dependency artifacts, hashes, licenses/notices, and accepted verification.

For each Winmakase PR, include the trigger, old behavior, new behavior, paired component requirement, checks run, live/disposable evidence, rollback, and open gate. Do not bury a Glaze minimum-version change in a generic feature note.

## From daily dogfood to v1

The four cuts authorize daily use; they are not the public v1 release by themselves.

During dogfood:

- Fix data-loss, focus theft, stranded-window, input, recovery, and rollback defects immediately.
- Keep a short friction log keyed to commands/actions rather than impressions.
- Measure startup/recovery, launcher latency, component restarts, and recurring manual interventions.
- Make the window-switcher decision after at least seven real days, as already specified in the window plan.
- Add convenience only when it removes repeated friction or completes an existing interaction family.

For the v1 release candidate:

- Finish the remaining P2–P5 rows and R0–R6 gates in the authoritative ledger.
- Ship coherent default theming plus a documented theme path; extra themes are welcome after the adapter contract is stable.
- Complete clean-machine install/update/uninstall and offline verification.
- Freeze artifact provenance, licenses/notices, configuration migration, and rollback documentation.
- Run the five-day stability gate and the longer measured daily-use gates in [`docs/plans/v1/verification.md`](docs/plans/v1/verification.md).
- Prepare a release announcement around the actual product: a recoverable Windows desktop layer with intentional window policy and familiar shortcuts, not a claim that Windows became Linux.

## v1.x, v2, and the platform horizon

The later arc is known well enough to guide boundaries. It is not a promise of dates or a reason to pull future work into v1.

### v1.x: Polish from use

- Smooth focus, placement, launcher, tray, and multi-monitor edge cases found in daily use.
- Add quality-of-life actions where the friction log shows repetition.
- Expand themes and safe customization.
- Improve diagnostics, repair, update, and migration.
- Add providers or controls only when the daily workflow shows a real gap.

### v2: Deeper interaction primitives

- Native window grouping/tab stacks rather than simulated grouping.
- A true multi-window scratch overlay.
- Sticky windows that remain visible across workspaces.
- Missing chord families and possibly tap-Caps, if the v1 input evidence supports them.
- Richer role/app catalog behavior and web-app identity where daily use proves it necessary.
- A measured window-switcher improvement if Windows' native path fails the seven-day trial.

These features probably require bounded Glaze core/fork work. They should preserve the Winmakase policy/config surface so an upstream implementation can replace a local one later.

### v3/platform horizon: Choose only from evidence

- Reassess official Glaze convergence versus maintaining a thin fork.
- Evaluate Whim, Seelen, Quickshell on Windows, and PowerToys Command Palette only against measured v1/v2 limitations.
- Consider a plugin/provider architecture when there are at least two real implementations that need it.
- Require migration, rollback, and a bounded proof before changing the desktop platform.

The parked research and triggers are in [`docs/plans/future-desktop-platforms.md`](docs/plans/future-desktop-platforms.md) and [`docs/research/gap-features.md`](docs/research/gap-features.md).

## Astra's first pass

1. Verify this checkpoint against `NEXT.md`, `execution-state.json`, the current trees, and live deployment state. Fix contradictions before delegating.
2. Decide whether to publish the current Winmakase baseline immediately or after one small documentation-only reconciliation PR. Do not begin more local-main feature stacking.
3. Own Cut 1's physical input acceptance and make the Kanata/direct-Caps decision.
4. Freeze the Cut 2 role/action contracts, then dispatch the two disjoint Sol lanes.
5. Open a separate official-upstream Glaze port lane beginning with cleanup uncloak or AppUserModelID. This lane may proceed in parallel but cannot block dogfood.
6. End each cut with a paired integration review, updated evidence/ledger/changelog, and a named next gate.

## Decisions that still belong to Chris

- Authorization for a daily-machine input or shell cutover when the prepared acceptance step is ready.
- Publication timing and authorship for external Glaze PRs.
- The point at which the dogfood build becomes the default session.
- The public v1 release and announcement.

Everything else here is an engineering decision Astra can make from evidence and record in the authoritative ledger.
