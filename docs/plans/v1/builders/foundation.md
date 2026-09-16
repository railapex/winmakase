# Foundation and remaining proofs

Reader: Astra lead and the Sol assigned a foundation/proof slice. Follow [dispatch](README.md) and [contracts C1–C8](../build-contracts.md). Source baseline is recorded in dispatch; active handoffs supply the accepted remote base and paired artifacts.

## F0 — Freeze and expose shared types

Outcome: later builders can import named, tested contracts without independently rewriting CLI/config/ownership code. This is a narrow source foundation, not implementation of all features.

Own existing `crates/winmakase/src/{lib.rs,main.rs,config.rs,paths.rs,health.rs}` and new bounded modules `app_roles.rs`, `actions.rs`, `mutation.rs`, `lifecycle.rs`; the lead explicitly assigns applicable Cargo manifests. Do not move all current code or introduce another crate/service solely for separation. Preserve current behavior while introducing versioned typed input mode, role/action result, health generation and journal target types from C1/C2/C5/C6. Use migration fixtures from the current default config. No nonfunctional CLI routes advertised as working.

Define the pure composition entry point over base/role/keymap/theme inputs, and ownership of its adapter to `winmakase-keymap`. Freeze serialized JSON/TOML examples for role identity, action results, health, popup snapshots and journal intent/result. Record version rejection and migration behavior. Export only the interfaces the next two slices consume; subsequent internal types stay local.

Checks: current config parses/migrates without changing launch/home behavior; unknown/conflicting role keys identify their source; older health readers show unsupported rather than green; mode defaults to Kanata; all generated examples round-trip; no test spawns real hooks/Glaze or registers tasks. Run targeted Rust tests/check and validate generated YAML structurally. Review any change to shared public type names before dispatching importers.

Stop: pure tested foundation and updated handoff interface references. No desktop mutation. If the planned module boundary requires broad churn, keep the existing entry point and serialize the two consumers instead of refactoring for hypothetical parallelism.

## I2 — Input decision and Run context acceptance

Lead-owned acceptance, with a Sol fixture helper if needed. Reuse [I1 evidence](../../../verification/v1/2026-09-13-p0-i1/README.md), the generated direct-Caps candidate and independent panic path. Own only a new case-specific evidence/fixture directory unless a failed case justifies a separately reviewed correction.

Prepare a concrete packet before asking for daily hardware: exact paired binaries/config hashes, current deployed-state verification, staged backups, input-mode switch, independent native panic, restoration command, expected observations and stop conditions. Show it to Chris; this planning session does not authorize that cutover. Synthetic Caps is diagnostic only. Use the guest for ordinary/elevated/fullscreen/Run/focus/reload cases it can represent, record hardware limitations and reserve literal key/hardware sleep tests for authorized hardware.

Run normal apps, elevated/protected contexts, rapid/repeated chords, bare Caps behavior as specified by the selected grammar, launcher from chord and bar, fullscreen, focused monitor, reload, suspend/resume and emergency escape. Stop the candidate on stuck modifiers, swallowed native escape or unrecoverable input. Record the exact failing sequence and restore, rather than continuing a broken desktop test.

Return one disposition: direct Caps accepted for stated environments; or Kanata retained for v1 with retained-mode acceptance and direct-Caps cases explicitly unaccepted/deferred. The latter unblocks input architecture consumers. It does not excuse launcher context failures or mark physical direct Caps passed. Update component/readiness/package selection together under C1.

## P0 remainder — Proof queue, not one giant blocker

Each row is a bounded lead-selected slice under the existing P0 case family. A failed proof may close its experiment with a documented alternative; required behavior stays open until that alternative passes. No arbitrary number of retries or new platform search.

| Proof | Builder/lead deliverable | Unblocks / stop condition |
|---|---|---|
| Provenance and environment | Exact source/artifact/pair table, Windows/display matrix, guest ownership/recovery; reconcile historical versus freshly observed live facts | Every runtime handoff; no unknown payload or second live stack |
| Profile identity/classification | W0 late-AppID refresh plus Chrome/main/utility/hidden-owner cases; exact minimum Glaze changes | W1 and remaining P1 policy; no generic Chrome fallback |
| Popup and tray | S1 transport/provider lifetime/focus/anchoring proof against pinned Zebar, including owner handover | S2/S3; no UI atop assumed provider API |
| Agent A1–A4 | Existing sequence observer with provider roots, cadence, same-admission routing, hidden-workspace capture/input; cooperation only for measured gaps | G0; unknown/shared brokers retain current policy |
| Split/terminal | Native insertion persistence, main-left/right-stacked sequence, named-pad identity/ordinary-terminal exclusion | W2/W3; no claim of one-shot direction or quake parity |
| Appearance primitives | Terminal fragment/selection behavior, wallpaper/Windows modes and pinned Run response; arranged-window reload | T1 supported target matrix |
| Suppression and synthetic trays | Disposable comparison of autohide/minimal suppression; Explorer identity, Start/flyouts/fullscreen/DPI and synthetic TaskbarCreated | L1 final takeover; native fallback remains if proof fails |
| Guard and pressure | L0/L1 bounded lease/restoration under blocked calls; pinned Glaze fresh-start/adoption/wake under measured commit availability | L1; no permanent 50% wait or PID-only readiness |
| Performance | 30 warm/5 cold input-ready samples; matched one/three-monitor CPU/private-memory/process/dock baseline | Relevant R0/R2 targets; preserve raw samples and cold/warm separation |

The 104-row pinned Omarchy comparison and accepted P0.D1/P0.I1/ancestry evidence are reused. Re-run affected parts only when their source/config contract changes. The remaining P0 report accounts for every experiment in [00-baseline](../00-baseline.md); R0 stays pending until all required dispositions and evidence exist.

## Planning decisions and authority

Chris confirmed detailed execution through v1 and roadmap only afterward. The daily layout question concerns a personal recipe, not public app defaults: ship the optional main/side/lower shape unassigned, with local assignment/preview only when selected. Work browser/editor/terminal is an example, not a prerequisite decision Chris owes this planning session. Do not install machine-specific identities from documentation.

Later required decisions are physical input/cutover, dogfood default session, external Glaze publication timing/authorship and public release. Do not ask those approvals before the concrete artifacts and rollback are ready. Publication and live deployment remain outside this planning pass.
