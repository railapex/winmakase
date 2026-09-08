# P5 — Distribution, restoration and release

Status: not started. Skeleton/ownership design may begin earlier; completion depends on P1–P4. Exit: R5–R6.

## Changes

1. Pin every dependency, source revision, local patch, build setting and distributable hash. Resolve the Glaze AppUserModelID patch's source/provenance and redistribution obligations; ship license/source notices as required. Verify Zebar from its artifact/source rather than its 0.0.0 file version.
2. Define user-owned data and protected privileged paths before installation. Protect Kanata executable/config, every replaceable parent, elevated panic code and update chain. Non-admin replacement must fail; user-level supervisor/Glaze/bar stay user-level.
3. Build an idempotent installer with a manifest and restoration journal shared with P3/P4. Record prior settings/tasks/files and ownership. Use one per-install owner for normal config/theme/update/off/uninstall mutations: quiesce or reject in-flight applies and reject stale-generation commit/rollback. Emergency native restoration bypasses that lock. Generate paths and app roles; never install Chris's profile IDs or repo paths.
4. Review first-run preview of app homes/roles and existing windows. Default to preserving current windows; apply initial management policy to future windows. Do not tile an entire existing session unexpectedly.
5. Detect PowerToys Keyboard Manager/FancyZones overlap; change only needed owned settings, with prior state recorded. Preserve unrelated remaps and module settings. Provide native recovery before input takeover.
6. Stage and validate updates before switching owned artifacts/config. Preserve local overrides; migrate schemas explicitly; keep last known working generation. A failed start restores the previous generation or native desktop, with a useful report.
7. Implement off/on and manifest-driven uninstall. Remove only owned shortcuts/tasks/generated files and restore settings still equal to our last writes. Report later-edit conflicts. Never erase an unrelated preexisting task with the same name.
8. Doctor reports versions/provenance, owner/process health, config errors, role/profile availability, docks and recovery readiness without exposing secrets/profile details unnecessarily.
9. Ship documentation for setup, keys, profiles, appearance, recovery, supported environments and known gaps. Add install media/checksums only once install verification passes.

## R5 cases

- Clean supported Windows VM/account, paths containing spaces/Unicode, no preexisting stack.
- Existing PowerToys/Terminal configuration, custom remaps, visible and autohidden taskbar; preserve unrelated values.
- Repeat install, same-version repair, supported-version update and rollback.
- Update interrupted after every artifact/config/task/ownership transition.
- Missing app/profile, disconnected network, missing optional provider, failed elevation, locked executable and disk-full simulation where safe.
- Non-admin cannot overwrite or replace privileged binaries/config/parents/panic scripts; authorized elevated update and rollback succeed.
- Tampered user journal cannot choose arbitrary elevated commands, paths, task names or processes; privileged recovery validates protected installed ownership and actual task instance.
- Theme apply interleaved with update/off/uninstall; mutation owner killed; interrupted apply followed by restart recovery. No old generation writes after ownership transfers, and emergency visibility still works during the operation.
- Uninstall after normal use, partial install, failed update, later user setting edits and missing owned files.
- Offline first bar/launcher use with assets already installed; define separately which bootstrap downloads require a network. Never imply an online bootstrap itself is offline.
- Compare before/after manifests, ACLs, tasks, startup entries and settings; no orphan owned processes or appbars.

## R6 and release

Use the integrated candidate for five working days, including three cold logins, five sleep/wakes, ten occasional-monitor cycles and both visual presets. Record every native taskbar access and its reason; each required workflow must have a verified replacement before hover retirement is accepted.

Run the restored W5 window-discovery decision after at least seven elapsed days with app actions and W3 float recovery. This can share the dogfood period. Record whether a current-workspace window switcher is needed; a need finding proposes a bounded follow-up, not automatic scope expansion.

Finish with independent review of user workflows, unresolved defects, recovery evidence, supported-build matrix, provenance/licenses and documentation. Run full required CI against the release commit. Existing baseline lint failures must be triaged and resolved or explicitly scoped with evidence; they are not a permanent excuse for a red release.

V1 is the product milestone. Choose the release number at release preparation; do not mechanically reuse the old v0.1.0 checklist after later dogfood versions already shipped. No tag or distribution claim until R0–R6 pass.
