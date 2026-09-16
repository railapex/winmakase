# Daily window loop

Reader: Sol window builders and the Astra paired-integration reviewer. Follow [C1–C3](../build-contracts.md), every [P1](../01-window-policy.md) acceptance case and the [A1–A4 proof](../../../research/agent-window-ownership-2026-09-12.md). P0.D1 dialog/reload and P0.I1 launcher results are accepted foundations, not reasons to repeat or replace them.

## W0 — Role schema and actual identity

Requires F0. Own `crates/winmakase/src/app_roles.rs`, existing `config.rs`, associated role/config tests and a dedicated profile discovery fixture. Glaze DTO changes in `src/glazewm.rs` and the paired Glaze tree belong to one lead-assigned owner; do not share that file with W1/G0. CLI discovery wiring is serialized.

Extend existing `Config.apps`/`AppConfig` launch/argument/process/class/AppID/home/state data into C2. Preserve current defaults and explicit `preserve`/`auto` migration. Canonical aliases share a normalized identity and lock key; reject incompatible eligibility or launch definitions. Role schema includes separate launch-new capability, with no fallback for unsupported singleton apps.

Prove fresh per-window identity and main-window eligibility across work/personal Chrome, two same-profile windows, incognito, picker, installed app, owned popup and background process. The current Glaze query caches AppUserModelID; a repeated cached query is not a refresh. Implement the smallest native refresh/query primitive if the fixture proves it necessary, with a paired provenance/minimum-version change. A protected or unavailable identity yields unknown/unavailable, never a wrong-profile selection.

Extend the request-only Glaze client to subscribe before snapshot without dropping event frames; include owner/resizability/PID facts in the local DTO and verify actual process-generation identity. These are W1/W2 prerequisites, not already-shipped APIs. Test query/event interleaving and process restart, not merely JSON parsing.

Return a preview with observed identity and excluded window classes before mappings are written. Include remaining utility and cloaked-owner dialog classification in the proof return; route required Glaze corrections as paired sub-slices instead of hiding them in broad process rules. Accept the identity/eligibility matrix before W1 starts relying on it.

## W1 — Focus-or-launch and launch-new

Requires accepted W0 and F0 action types. Own `src/app_actions.rs` (new), `app_roles.rs`, action tests and explicitly transferred `main.rs`/`bin` GUI action wiring. Integrate catalog/keymap projection in a serialized step with `crates/winmakase-keymap/src/lib.rs`, keymap data and goldens; J0 must not edit that file concurrently. Keep guarded PowerToys activation in `launcher.rs`/`winmakase-run.rs` intact.

Implement the C2 selection order, per-identity mutex, persisted pre-spawn attempt, finite timeout/grace, late identity refresh, vanished-target re-evaluation and foreground-context check. Initial timeout/grace values are an explicit implementation choice validated with slow fixtures; do not silently turn an uncertain launch into an immediate second spawn. Reuse the process creation identity discipline already used by `launcher.rs` where applicable, without turning launcher-specific focus rules into a generic bus.

Expose explicit focus-or-launch versus launch-new actions and generate finite shortcuts/keymap references from typed definitions. Migrate duplicate launch strings with conflict diagnostics. Show distinct permission, missing-app/profile, startup-pending and unsupported-new-window outcomes. Generic Omarchy launch intent stays separate from explicit app-focus shortcuts.

Pure tests: selection order, aliases, identity-version invalidation, stale WM IDs, held-key repeat, deliberate separate new actions, attempt crash/expiry and context changes. Guest tests: 20 repeat focus calls without duplicate; two concurrent aliases; cold/slow/brokered startup; kill CLI after spawn; late AppID; current/hidden/minimized matches; scratch exclusion; target closes during focus; unrelated browser opens during wait; user switches app/workspace. Assert IDs/profile/workspace/state/focus, not just process counts.

Stop after role actions and their finite generated routes work. No new app catalog, installation or global MRU tracker.

## W2 — Floats, named terminal and split/fullscreen fidelity

Requires W1 shared launch primitives and the relevant P0 split/terminal proof. Own `src/scratchpad.rs`, `src/reflow.rs`, new `src/window_actions.rs`, their tests and exclusive Glaze adapter/keymap/CLI wiring assigned for this slice. No concurrent W3/G0 writer of those files.

Add loose-float current/all-workspace queries plus deterministic forward/reverse current-workspace recovery. Keep dialogs attached to parents, scratch separate and unmanaged/elevated windows in native switching. Return structured data for S2; no independent UI-owned enumeration.

Make the existing named terminal pad a discoverable summon/hide action with dedicated identity, centered geometry and C2 launch serialization. It cannot capture the normal terminal. Anonymous S remains summon/rescue; no sticky or multi-window overlay claim. Classic quake behavior remains an optional proof outside the default.

Proposed dedicated terminal binding is Caps+grave, subject to the interaction checkpoint. The current keymap marks grave unsupported because its translation is absent; add parser/translation and collision evidence before presenting it as available. Do not steal Caps+S or group chords to conceal that missing key support.

Keep Caps+J reflow. Implement the planned Caps+Ctrl+J direction toggle after checking local collisions; show side-by-side/stacked insertion direction. Prove persistence across focus and insertion; no one-shot claim. Preserve reflow's refusal of unsupported nested sibling changes. Make F explicitly true fullscreen and Alt+F maximized; exit restores prior tile/float state, while T returns either to tile.

Cases: covered/off-workspace loose floats, empty lists, dialogs/scratch separation, terminal warm/cold/concurrent/held/closed/late-start with ordinary-terminal negative control; native and application fullscreen transition matrix; local binding collisions; empty/single/multi-window direction changes; main-left/right-stacked construction using new and existing windows. Capture exact state/tree/focus and help labels. Return accepted API fixtures for S2, no live binding deployment from a source-only handoff.

## W3 — One layout, preview/adoption and restart preservation

Requires W1/W2/J0 and paired P0.D1 preservation. Own new `src/layout.rs`, layout/restart fixtures and controlled-restart snapshot adapter; transfer `glazewm.rs`, `main.rs` or `supervisor.rs` only after other owners finish. The lead may split restart integration into a serial sub-slice of L1 to avoid conflicting supervisor edits.

Implement one optional main-left/right-stacked recipe using logical main/side/lower assignments, targeted Glaze commands and bounded waits. Ship unassigned roles and an opt-in local assignment/preview; work browser/editor/ordinary terminal is only an example. Ordinary dynamic tiling stays the default; the recipe never runs automatically at installation/login. Reuse C2 selection; show missing apps and affected workspaces, including neighbor resize. Never pull a scratch/dialog/unrelated window into the preset. Revalidate against actual state before each step and stop on user-context or WM-generation change. Handle occupied targets without claiming arbitrary tree reconstruction or transactionally undoing another actor's work.

Define failure behavior literally: preserve existing windows, retain newly launched usable windows, report completed and skipped steps, and only undo an owned mutation whose identity/context still matches. Do not close applications to fake atomic rollback. Repeat invocation converges on the same role windows without duplicates. Deliberate rehome and first-run adoption use a reviewable preview and default to preserving the existing session.

Controlled restart captures a bounded in-memory layout snapshot and restores surviving eligible identities. Unplanned death uses current observable state/policy and reports lost structure; no durable lost-tree reconstruction. Test parent/dialog relationships separately.

Cases: cold/warm/twice-run, slow/missing role, identical aliases, occupied targets, user movement during launch/apply, window closes, monitor removed, failed command halfway, hidden/scratch exclusions, manually arranged reload, controlled restart and unsnapshotted failure. R1 must verify the specified preservation result rather than simply count commands sent.

## G0 — Supported agent admission

Requires completed relevant A1–A3 evidence; A4 cooperation is conditional on a measured coverage gap. Own `src/agent_ancestry.rs`, new `src/agent_policy.rs`, policy tests and a dedicated evidence directory. Existing Glaze generation/admission integration is lead-owned paired work with an explicit file/branch transfer. No separate resident attribution service.

Reuse the accepted sequence-feed observer. Recognize validated native agent roots, preserve process creation/sequence identity and bounded observation history, reject unsafe retrospective timestamp ancestry. Human terminal shells and Winmakase components are negative controls. Exact conversation identity and Muxel participation are optional. Unknown ownership and shared browser brokers keep their existing placement.

For supported output classes, route before the first mutation of the human tree/focus; test the existing Manage event before adding a deeper insertion primitive. Native application activation and Glaze queued focus are separate assertions. Prove inactive-workspace capture/input under the actual hiding mode. No late external move presented as prevention, browser-group relocation, process kill authority or universal browser ownership claim.

R1 evidence includes 20 creation/destruction cycles for each supported class, concurrent agents, human/unrelated negative controls, late windows, auth handoff, deliberate inspection/return, observer restart/expiry and hidden-owner dialogs. Record unsupported classes by name and behavior. A failure affecting the required daily flow remains a gate; a clearly unsupported shared-browser class follows the existing scope limitation rather than a fabricated success.
