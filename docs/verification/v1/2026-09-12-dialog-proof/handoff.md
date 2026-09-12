# D1 dialog policy proof fixture handoff

## Contract

- Slice ID / parent package / requirement IDs: D1 / P0 proof supporting P1 / main-dialog-utility classification, parent placement and reload evidence.
- Outcome and acceptance cases: extend the accepted four-action Sandbox fixture with deterministic native HWND facts and fixture-only Glaze-state captures at named phases; provide host-only tests for guards and evidence schemas.
- Explicit exclusions: no Glaze policy implementation, live stack or task/config changes, UI/provider launch, shared desktop/guest use, global window inventory, channel-selected Muxel access, or edits to global plan/design/work-state/changelog files.
- Base repository, branch, commit and integration contract: `railapex/winmakase`, `p0-dialog-proof`, `bea1f00bd300dbcdac8ad8570dd613248b4e128e`; preserve the four action IDs and exact checked-in argv tuples.
- Worker worktree / owned paths / shared interfaces: `D:/dev/winmakase-wt-dialog-proof`; `spike/p0/` fixture source/scripts/tests and `docs/verification/v1/2026-09-12-dialog-proof/` only. Lead owns shared Glaze composition/policy and the disposable guest.
- Requested model and effort; observed actual model: requested `gpt-5.6-sol` at high effort; the runtime did not independently expose the actual model/effort, and no substitution was reported. No nested delegation.
- Authorized actions and test environment; live-change boundary: source edits, pure host compilation/tests and read-only source review are authorized. No desktop mutation or guest claim.
- Dependencies verified against: local `CLAUDE.md`, `docs/plans/v1/execution.md`, `docs/plans/v1/01-window-policy.md`, `docs/research/agent-window-ownership-2026-09-12.md` D1, accepted P0 fixture evidence and current `spike/p0` source.

## Build return

- Commit(s) and actual changed behavior: the return commits contain this handoff; hashes are reported by the worker after commit. The four exact launch tuples are unchanged. Accepted guest processes register exact active fixture HWNDs with process creation and per-window lifetime identity; guarded scripts bracket the fixture-filtered Glaze query with native identity validation and compare process/window plus Glaze container identity across named phases.
- Required commands, runtime/build/config, executed counts and evidence paths: `pwsh -NoProfile -File ./spike/p0/Test-P0Fixtures.ps1` and inbox `powershell.exe` both pass 70 assertions; full commands, guest invocations and source findings are in `report.md`.
- Regression failed on prior behavior where applicable: synthetic comparisons now fail for changed state, changed PID, reused PID/HWND/title with new process/window lifetimes, changed Glaze management and a changed Glaze container ID.
- Unverified cases / failed checks / unresolved defects: the first lead guest run exposed a single-role PowerShell scalar bug and an unobserved GUI-subsystem loader result; both fixture defects are covered by host regressions. That guest also lacked pinned `VCRUNTIME140.dll`, so actual Glaze query/reload/placement cases remain lead-owned and pending after the runtime prerequisite is restored.
- Independent reviewer and disposition: lead-owned.
- Integration checks and accepting owner: root lead.

## Checkpoint

- Actual HEAD / clean status / claims: started clean at `bea1f00bd300dbcdac8ad8570dd613248b4e128e`; source is review-ready and pending its single return commit; worker registered as `sol-dialog-proof`; `env:winmakase-uat` is claimed by lead session `winmakase-crashdown`.
- Running processes/tasks and exact owner; cleanup needed: no fixture or Glaze processes launched by this worker; no cleanup expected.
- Decisions for Chris: none in this bounded source slice.
- Next executable action and its prerequisites: lead re-runs guest initialization from the mapped updated source, then runs named captures around its fixture-only placement/reload actions using the commands in `report.md`.
- Next recenter trigger: fixture commit returned to lead.
- Report regenerated from source: no; worker may not edit shared execution state/report.
- Prior checkpoint reference: `docs/verification/v1/2026-09-08-p0/fixtures.md`.
