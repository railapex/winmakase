# Resume Winmakase v1

Read this file first in every continuation. It is a stable entry point, not a second task ledger.

## Current checkpoint

- **2026-09-27 desk-first recut (supersedes the next-step bullets below).** Chris decided on these after the Parsec remote week: Caps→F13 instead of right-Win, retire Kanata, park the direct-Caps leader, keep Glaze pinned and thin, land every slice on the desk first, move the launcher toward Command Palette, and make a coarse Agents workspace the first window slice. Stock Glaze passed F13 chords (9/9 injected; Chris's physical checks; Parsec from the laptop). Admin windows fail because our Glaze build has `uiAccess=false`, a separate signing decision deferred past VRMA. Decisions, evidence and the next five slices: [input/remote recut](../../research/2026-09-27-input-remote-recut.md). Live machine: standalone Glaze on the F13 test config with KBM Caps→F13, no Kanata and no supervisor (see [NOTES](../../../spike/NOTES.md) LIVE MACHINE STATE). **Next source slice:** `SuperKey::F13` render target, then a supervisor input mode without Kanata.

- F0/S0 source checkpoint, 2026-09-16: reviewed baseline and planning are published at `5969051` on `origin/main`. Published review branch `build/f0-s0` contains F0 contracts (`74586c6`, fixtures `a97d24d`) and reviewed S0 assets (`2e2bc2f`, corrections `eee678b`/`cb2555b`), with Winmakase bumped to 0.1.9. F0 passes 159 library tests, 33 keymap tests, all-target check and scoped strict Clippy. S0 passes clean dependency install, deterministic build, TypeScript and remote-runtime-reference checks. [Evidence and exact limits](../../verification/v1/2026-09-16-f0-s0/README.md). GitHub rejected Friday's PR creation because the token lacks `createPullRequest`; the branch itself is published. No desktop state or daily deployment changed; disconnected Zebar runtime proof and all P/R gates remain open.

- Previous planning checkpoint: inspected source `1f47d497069b69a627f9b8c8598d21b4b55f4aa9`, when remote main was `ff91369b11e8ce6f51e913954613ce93d1a94d55`. [Shared contracts](build-contracts.md) and [Sol work orders](builders/README.md) cover remaining v1. The accepted plans are now published. P0.I2 remains the next input acceptance slice and does not block independent source work.

- Planning review: two Sol audits and fresh Astra review; [findings/resolutions](../../verification/v1/2026-09-16-planning/README.md). Chris confirmed detailed v1/later roadmap only. Dynamic tiling remains default, optional recipe roles ship unassigned. [Current PowerToys/Omarchy/Whim research](../../research/layouts-powertoys-2026-09-16.md) preserves later Command Palette/Dock and layout interoperability opportunities without changing the v1 backend.

- Targeted 2026-09-10 hotfix: Glaze `9c8e93b` on deployed-source base `6d62d66` is active after Chris authorized restart. A live elevated fixture was automatically excluded from tiling. Supervisor adoption now bypasses the fixed commit-pressure guard; all three components are supervised. The fresh-launch 50% cutoff remains open work. Also carried into the future input branch as `dd8fb7d`. [Evidence and delivery status](../../verification/v1/2026-09-10-placement-denied.md). No P0/P1 gate is closed by this patch.

- Accepted source baseline: `dfb3e7d`; fixtures `7161a6d`; binding comparison `1b0ceca`; Run payload/kit `7cef5e6`; resource baseline `9edc253`. Reviewed Glaze source/build: `371a448`. Verify current HEAD and diff before continuing. [P0 evidence](../../verification/v1/2026-09-08-p0/README.md).
- Build scope: P0 baseline/proofs, explicitly started by Chris. P0.D1 and P0.I1 are accepted in source and disposable guests; no verification round has passed. Check Wire and claim `env:winmakase-uat` before guest/environment changes. [I1 evidence](../../verification/v1/2026-09-13-p0-i1/README.md) records the latest environment disposition.
- Current slice, decisions and checkpoint history: [execution-state.json](execution-state.json).
- 2026-09-12 research: two Sol passes selected an initial native ancestry candidate. The subsequent prototype review rejected retrospective creation-time ordering and accepted a bounded unique-sequence observation feed. [A1–A4 plan and results](../../research/agent-window-ownership-2026-09-12.md) retains provider/admission/background-input gates; Muxel and exact conversation attribution are optional. Browser-group movement is deferred. This does not replace input/Run or dialog/popup proofs.
- Package/round completion: [TODO](../../TODO.md).
- Procedure and authority boundaries: [execution.md](execution.md).
- Latest accepted integration: Winmakase 0.1.8 I1 source in this checkpoint; paired D1 Glaze `6bc83d1a9682e885c507ac489ca0208f20f985e0`. The guarded Run adapter passes hidden, repeated, concurrent, GUI-helper, stopped-process and real rendered-bar-click cases in a fresh guest. Direct-Caps config generation passes. The bar click needed networking because the current bar still loads frontend dependencies from CDNs; offline packaging remains P2. D1 dialog/reload results remain accepted. [I1 runtime evidence](../../verification/v1/2026-09-13-p0-i1/README.md).
- Latest source integration: Winmakase 0.1.9 on `build/f0-s0`; F0 interfaces and S0 local assets are implementation-complete in source. The latest accepted *runtime* integration remains Winmakase 0.1.8 I1 with paired D1 Glaze `6bc83d1a9682e885c507ac489ca0208f20f985e0`; the new bar has not yet passed disconnected startup or rendered interaction checks.
- Next runtime action: P0.I2 direct-Caps and Run context acceptance. Prepare exact candidate/backups/rollback before asking for daily-machine authorization. Exercise literal physical Caps through the patched Glaze candidate on authorized hardware, and use the disposable guest for the bounded fullscreen, elevated, focus/monitor, reload and teardown cases it can represent. Synthetic Caps is diagnostic only: injected protected-leader edges are deliberately rejected and cannot satisfy physical acceptance. Then decide whether Kanata can retire. Keep offline Zebar packaging in P2. Provider discovery/cadence and admission/background input still need A1–A3, and utility/hidden-owner classification remains open. No host cutover is authorized by these proofs. Read-only planning auditors do not own a build or guest environment.
- Next source action: start W0 role identity proof/schema and L0 singleton/request mechanics from the frozen F0 contracts. Invoke Astra at the documented post-W1 high-risk contract gate, not for routine source work. Keep P0.I2 as the separately prepared physical-input lane.
- Next user checkpoint: F0 and S0 are the two accepted source slices. Present their evidence and the W0/L0 dispatch; R0 still needs concrete interaction results before any runtime gate moves.

## Resume contract

Read repository instructions, verify branch/HEAD/worktree state and the Wire roster/claims, and check whether another lead or worker still owns this slice. Existing logs or a clean tree do not prove the environment is free. Inspect relevant live state before any authorized machine action.

Use the state file's current slice and latest evidence, not an old chat's next-step suggestion. Read only its package and dependencies. Continue through implementation, tests, fresh review and integration within the authorized batch; keep product and test-environment boundaries intact.

Before handing off, update current state, record the exact accepted commit/evidence and running ownership, regenerate the report, and refresh these checkpoint pointers when they change. Preserve old checkpoints. Never mark an unexecuted case passed or lose a deferred requirement during rewriting.

## Reusable continuation prompt

> Continue the Winmakase build from D:/dev/winmakase/docs/plans/v1/NEXT.md.

The user opens a fresh top-level session only when wanted or needed. Worker dispatch and return are the lead's job. Goals are started explicitly and do not replace this durable checkpoint.
