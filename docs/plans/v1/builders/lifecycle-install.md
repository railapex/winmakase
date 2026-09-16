# Lifecycle, shared mutations and installation

Reader: Sol builders assigned J0, L0/L1 or INST0/INST1 and their Astra reviewer. Use [C5–C7](../build-contracts.md), [P4](../04-supervisor.md), [P5](../05-package.md) and the full [R4/R5 matrix](../verification.md). Existing tested adoption/backoff/process handles are retained.

## L0 — Singleton and request completion

Requires F0 types. Own `src/control.rs`, new `src/session_owner.rs`, request tests and the necessary `src/paths.rs` additions, all beneath `crates/winmakase`. CLI/daemon/supervisor wiring is serialized and explicitly assigned at integration; do not race a window-action writer of `main.rs`.

Replace the PID-file ownership check with the current-user/session named mutex in C5, including abandoned-owner reconciliation before a new generation starts. Replace the overwriteable `state/control` and single reload-result slot with atomically published/claimed request files and per-request outcomes. A second invocation discovers and routes to the owner or fails clearly; no duplicate supervisor. Stop completion means termination/restoration outcome, not merely deletion of a request file.

Cases: simultaneous starts from interactive/logon paths; stale PID/reused PID; same SID/session with a different home; two reloads; reload/down race; requester timeout during claim; owner death before claim and after applying; stale generation/result; malformed/oversized requests; bounded queue/retention; old client migration diagnostics. A live failed kill stays tracked. Record request ordering and cancellation semantics from C5 in tests.

Allowed here: pure filesystem/stand-in tests in isolated temporary paths. Task registration and actual stack singleton tests wait for the guest. Stop after reviewed ownership/request implementation and integration evidence; no native taskbar change.

## J0 — Mutation journal and single composition

Requires F0, accepted P0.D1 dialog/reload compatibility and C6. Own new `src/mutation.rs` implementation, `src/composition.rs`, associated pure tests; exclusively own `crates/winmakase-keymap/src/lib.rs` when extracting the existing renderer adapter. Public config/CLI integration occurs after W1 or under an explicit file transfer. T1/INST0 cannot each invent another journal.

Implement one mutation mutex and the durable intent/apply/verify/recover protocol. Keep typed target IDs, prior/expected values, owned-key restoration, generation validation and conflicts. Add the one base/roles/keymap/theme composition path; it renders before mutation and rejects contradictory inputs. Preserve the current validated app-rule semantics and local overrides.

Tests must kill/simulate interruption before and after each journal flush, external write, verify and active-generation publication. An intent-only write must recover by inspecting live state. Cover absent versus empty values, repeated recovery, later edits, locked/disk-full/erroring targets, stale rollback, owner death, concurrent theme/role/key changes and emergency-lease revocation while the mutation lock is held. Prove accepted inputs are retained and emergency native access cannot wait behind the journal owner.

Stop after usable primitives and pure/file-adapter behavior. Actual registry/task/Terminal acceptance belongs to T1/INST1/L1 in a guest. Return exact schema fixtures and migration policy to those builders.

## INST0 — Protected payload and staged installation

Requires F0 and J0 journal schema/implementation; may draft manifest fixtures earlier. Own `installer/` and new `packaging/` manifest/notices fixtures. Lead assigns any Rust protected-manifest validator as an isolated new module and serializes Cargo/CLI changes. The existing `register-tasks.ps1` is a dogfood script: it currently points elevated actions into the repo and PATH-resolves pwsh. It is not the finished install boundary.

Implement C7 versioned protected payload layout, fixed privileged ownership manifest, hash verification, schema migration checks, safe generated paths and preview. Replace task creation with exact owned task definitions using protected absolute targets; registration must distinguish a foreign same-name task from Winmakase ownership. Keep user preferences separate. Validate retained Kanata config from the finite grammar; its elevated parser cannot accept arbitrary user-supplied execution directives.

Stage binaries/assets/config and notices without starting the desktop. Record app availability, current overlap with PowerToys Keyboard Manager/FancyZones, required changes and restoration entries. Preserve unrelated remaps/modules and never embed Chris's browser profiles/repo paths as public defaults. Define supported OS/dependency versions from actual provenance; do not download latest.

FancyZones preview identifies automatic placement, display/layout-change movement and overlapping hotkeys. Default integration gives Glaze sole placement ownership for managed windows; a disjoint ignored-app arrangement requires evidence, not an assumed per-window exclusion API. User-approved conflicting-setting changes use the ownership journal and restore later; a Workspaces launch is not treated as launch-only. Doctor reports coexistence conflicts without silently enabling/disabling whole modules.

Guest checks: spaces/Unicode, non-admin attempts to replace every protected parent/file/task target, reparse replacement/containment, tampered journal, wrong hash/pair, existing same-name task, failed elevation, missing optional app, repeat staging and interrupted ownership creation. Before L1 privileged recovery tests, deliver the installed protected panic/ownership primitives and prove exact task-instance cleanup authority.

Stop at verified staging and protected recovery primitives, without enabling taskbar takeover. This breaks the apparent installer/lifecycle dependency cycle: L1 consumes safe payload ownership, then INST1 verifies full activation/update/removal with L1.

## L1 — State machine, restore host and readiness

Requires L0/F0. Pure transition work can begin early; native acceptance requires J0 baseline storage, INST0 protected recovery, S1 provider contract and later S3 replacement evidence. Own `src/supervisor.rs`, `taskbar.rs`, `process.rs`, `procs.rs`, `commit_pressure.rs`, `component.rs`, `display_watch.rs`, `monitors.rs`, `bin/winmakased.rs`, new `restore.rs`/`lifecycle.rs` and their tests. Health/config/installer integration files are transferred explicitly. All paths are within `crates/winmakase` unless named otherwise.

Implement the C5 transition table with an injectable clock/process/task abstraction. Separate restoration/lease observation from potentially blocking discovery/start/reload/stop calls. Keep a single scheduled host launching one worker; prove exact ownership before adopting/cleaning. Neither actor recursively restarts the other. Derive Kanata pairing from input mode. Initial/bar-only failure keeps healthy input/tiler while native access is restored.

Capture baseline before any taskbar mutation, including emergency showing; adoption reuses the baseline. Validate Explorer ownership, excluding the tray provider's synthetic taskbar. Re-observe debounced topology/TaskbarCreated events without provider suppression or restart loops. Actual widget IDs/bounds/reserves gate readiness; a PID or positive top reserve alone does not. Retain bounded dock restart until direct reconciliation passes. Remove the unused display-pair bounce option only with its no-need evidence.

Reproduce the pinned Glaze pressure failure. Remove the fixed 50% guard if unneeded; otherwise cap fresh-start wait/retries using measured available commit and report degraded native fallback. Never re-block healthy adoption behind allocation pressure.

Pure tests: every startup failure; either linked member fails; simultaneous pair/bar failure; failed termination; stale host/worker generation; invalid reload retains working config; baseline-write interruption; retry exhaustion; shutdown reports partial failure. Guest tests: worker/host killed and suspended separately during blocking operations and every takeover transition, concurrent logon/start, absent Explorer, replaced/unavailable panic helper, logoff, pressure, DPI/primary/hotplug, synthetic and real TaskbarCreated, originally visible/autohidden taskbar and later edits. Separate fault detection from restoration latency; test simultaneous actor death using independent panic/next-login.

Add a deterministic lease race: pause the native writer after its permission check but before hide; revoke its generation and restore; resume it; verify the delayed stale write cannot leave takeover effective. Independent restoration remains active until exact old-writer cleanup is confirmed; no new generation is granted meanwhile. Measure final native visibility and resumed-write recovery latency, including denied cleanup. A second cooperative lease check is not the sole enforcement mechanism.

Taskbar retirement is the final sub-slice after S2/S3 replacement matrix, INST1 recovery integration and the suppression proof pass. Until then keep native fallback and hover implementation available, with takeover disabled for the new candidate. Remove hover/flyout/delayed-hide code only in the accepted replacement commit. A failed suppression proof leaves that v1 gate open; it does not block continued safe source work or a separately approved native-fallback dogfood configuration.

## INST1 — Activate, update, repair, rollback and uninstall

Requires INST0/J0/L0 and integrated L1 start/stop readiness. Own installer/packaging scripts and tests exclusively; L1 transfers any shared task-registration changes first. Consume C5 readiness/stop outcomes and C6 mutation ownership.

Complete staged activation, idempotent same-version repair, supported-version migration/update, last-working rollback, off/on and manifest-driven uninstall. A new generation is active only after verified readiness. On failed readiness restore the previous compatible generation or native desktop, with conflicts/cleanup failures preserved in the report. Local overrides and unrelated tasks/files/settings survive. Do not erase user data by default or remove foreign same-name tasks. Retain backups until restoration no longer references them.

Run every P5/R5 row: clean and existing stack, repeat install, locked binaries, disk full, interrupted step-by-step update, failed elevation, missing artifacts, uninstall after partial install/failed update/missing files, later user edits, theme/update/off/uninstall races and tampered privileged metadata. Compare before/after manifests, ACLs, tasks, processes, appbars and settings. Offline runtime is verified from a cold cache; label network bootstrap separately.

Doctor must report provenance, owner/lease/component health, input mode, config/role/profile availability, docks and recovery readiness, sanitizing personal details. Return exact install/update/rollback/uninstall commands and evidence for the final cutover packet; never run them on the daily host under a planning authorization.
