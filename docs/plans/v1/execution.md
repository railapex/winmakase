# V1 build procedure

Status: in use for P0. This is the Winmakase procedure, not a new general orchestrator.

## Roles and boundaries

Recommended allocation: **Astra lead, Sol builders, fresh Astra review for window/lifecycle/security contracts**. Sol can review ordinary UI/rendering changes independently. This is a task-specific allocation, not a measured claim that one model always produces better code. Record requested and actual model/effort for each slice. [Astra documentation](https://developers.openai.com/api/docs/models/gpt-6-astra), [Sol documentation](https://developers.openai.com/api/docs/models/gpt-5.6-sol)

Use native bounded subagents for ordinary work. At most two independent implementation scopes at once, leaving a slot for review. Give each writer an isolated worktree and one explicit file/interface scope; no nested delegation. Shared Glaze composition, app identity, mutation journal and recovery ownership stay lead-owned until their contracts are frozen. Lead reads diffs, integrates and owns acceptance.

No installed build-orchestrate skill was found in the inspected user/Friday/plugin locations. Do not invent a command or adopt OwnerRez-specific workflow/model defaults. The existing local precedent allows native bounded delegation and reserves managed launchers for external provider runs. Any such run must use its verified workflow configuration and budget/process controls; do not start raw unattended provider fleets. [Local handoff precedent](file:///D:/dev/orez-desk/handoffs/REVIEW-CYCLE-BUILD-2026-09-05.md#L166)

## First batch

The original P0 batch below records the procedure's starting point. Current remaining work is split in [Sol dispatch](builders/README.md); its [contract annex](build-contracts.md) freezes shared behavior. Instantiate F0/S0 from a published accepted baseline before wider fan-out. P0.I2 physical acceptance continues separately and does not block independent source work. Do not repeat accepted D1/I1 proofs without a changed dependency.

Start with P0 proofs, not six simultaneous packages:

1. Record exact provenance and define a disposable Windows test environment, ownership and recovery.
2. In independent scopes, prove tray/popup/Run activation and dialog/profile/split behavior.
3. Review results together. Resolve the smallest needed component patches before P1/P2 depend on them.
4. Return a concrete interaction checkpoint: working tray/calendar proof, split sequence, role identity preview, proposed missing-key assignment and named terminal behavior.

This batch can finish with failed proofs plus clear dispositions; it cannot mark R0 passed while unresolved prerequisite cases remain. No product takeover or daily-machine deployment is implied by planning/report preparation.

## Slice loop

1. Lead chooses one runnable slice from P0–P5 and writes a handoff with contract, base commit, owned paths, dependencies, authority, expected evidence and stopping point.
2. Builder inspects current instructions/state, implements only that slice and writes meaningful behavior/regression tests.
3. Run required checks for affected code in the correct isolated environment. Record actual execution, failures and unverified cases. Static inspection is not runtime acceptance.
4. Fresh reviewer sees requirement, diff and evidence without the builder's suggested verdict. Review the whole slice, including deleted requirements and shared-contract effects.
5. Fix findings; rerun affected checks. After two unsuccessful fix/review cycles or a failed premise, the lead re-derives the approach before further work. A new commit does not reset this count.
6. Lead integrates, checks shared behavior, commits the accepted slice and updates TODO/evidence/checkpoint/report. An agent saying done is not acceptance.

Use one reviewable result per slice, usually a few focused commits. Parallelize independent code, not shared live desktop experiments. Claim the environment before changing it. No taskbar/crash/Explorer test on the daily machine as a first experiment.

## Goals and check-ins

Use a bounded goal per integration batch, with a concrete outcome and exit checkpoint. Begin with P0's proven integration contract, then P1/P2 interactions, P3/P4 appearance/recovery and P5 distribution. A goal is persistence across turns, not a replacement for plans/evidence. Create it only when the user explicitly starts it; no goal was created by this planning update.

Recenter with Chris after two accepted slices, at a package boundary, or after about 60–90 minutes of active build work, whichever comes first. A check-in states what now works, shows the evidence, names the next slice and presents only actual decisions. Continue independent authorized work while optional feedback is pending. Pause dependent work for a required product/authority decision; elapsed time is not consent.

Required checkpoints:
- R0 interaction choices and bounded patch scope.
- P1/P2 daily-use demonstration, role layout and terminal behavior.
- Before first daily-machine takeover, show isolated R4 recovery results and exact rollback.
- R3 appearance comparison and R5 install/update restoration.
- R6 review after the real elapsed dogfood period; no accelerated “one week” simulation.

Check-ins happen in this session by default. Do not send email/chat/phone notifications without an explicit instruction.

## Handoff and resumption

The stable entry point is [NEXT.md](NEXT.md). Continue in the current lead session by default; the lead dispatches and collects worker handoffs and chooses the next authorized slice. Chris does not relay prompts between workers. At a batch checkpoint the lead reports, records the continuation and continues work already authorized. Wait only for required decisions or an actual authority boundary, not routine approval of every slice. The lead does not silently create a new top-level session.

If Chris opens a fresh top-level session, the same single prompt works each time: "Continue the Winmakase build from D:/dev/winmakase/docs/plans/v1/NEXT.md." That session verifies the current lead/claims and recorded commit, reads the current slice/decision state, and follows the next action. Update NEXT.md's checkpoint pointers at each accepted slice; preserve earlier evidence. If the former lead still owns the work, coordinate or wait instead of running a second coordinator. No transcript copying or manually selected next handoff is required.

Use [handoff template](handoff-template.md). Store instantiated slice handoffs and evidence under docs/verification/v1/<build>/; sanitize machine data before committing. A fresh session checks actual branch/HEAD, worktree cleanliness and claims before trusting the checkpoint.

At every accepted slice or session boundary record: exact commit and integration base, closed requirement/case IDs, evidence paths, running processes and owners, unresolved defects/decisions, next command and authority limits. Preserve prior checkpoints. Never promise that a new session automatically inherits runtime state or unlimited budget.

## Progress report

[progress.html](progress.html) is a read-only snapshot. TODO owns package/round completion; [execution-state.json](execution-state.json) owns current slice, decisions, evidence and checkpoint history. Regenerate the report using [render-progress.ps1](render-progress.ps1) after each checkpoint. It reads these sources; do not manually mark progress in HTML.

Show last update, tested commit, current/next slice, verified rounds, blockers and actual decisions. No invented completion percentages. The page marks its snapshot stale after a day; decisions are answered in the active session and recorded by the lead, not through a fake submit button.

The report and handoffs make work inspectable. They do not become a second task system or an autonomous deployment service.
