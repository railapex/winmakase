# Resume Winmakase v1

Read this file first in every continuation. It is a stable entry point, not a second task ledger.

## Current checkpoint

- Planning/reporting baseline: commit 7b83482; later commits may refine these instructions. Verify current HEAD and diff before continuing.
- Active build: P0 baseline/proofs, explicitly started by Chris. No v1 implementation round has passed. Lead owns Wire `env:winmakase-uat`; check the roster before continuing.
- Current slice, decisions and checkpoint history: [execution-state.json](execution-state.json).
- Package/round completion: [TODO](../../TODO.md).
- Procedure and authority boundaries: [execution.md](execution.md).
- Next action: P0.1, finish exact artifact provenance and guest-only fixtures. Chris is setting up Sandbox. Reconcile the existing direct Caps Glaze spike with the deployed AppID patch before assuming Kanata remains necessary. Bounded ordinary behavior checks can use the daily desktop; shell/failure injection requires disposable Windows.
- Next user checkpoint: after two accepted slices, a package boundary or 60–90 active minutes. R0 must present concrete interaction results and unresolved choices.

## Resume contract

Read repository instructions, verify branch/HEAD/worktree state and the Wire roster/claims, and check whether another lead or worker still owns this slice. Existing logs or a clean tree do not prove the environment is free. Inspect relevant live state before any authorized machine action.

Use the state file's current slice and latest evidence, not an old chat's next-step suggestion. Read only its package and dependencies. Continue through implementation, tests, fresh review and integration within the authorized batch; keep product and test-environment boundaries intact.

Before handing off, update current state, record the exact accepted commit/evidence and running ownership, regenerate the report, and refresh these checkpoint pointers when they change. Preserve old checkpoints. Never mark an unexecuted case passed or lose a deferred requirement during rewriting.

## Reusable continuation prompt

> Continue the Winmakase build from D:/dev/winmakase/docs/plans/v1/NEXT.md.

The user opens a fresh top-level session only when wanted or needed. Worker dispatch and return are the lead's job. Goals are started explicitly and do not replace this durable checkpoint.
