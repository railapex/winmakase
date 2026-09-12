# P0.3 resumed build batch — 2026-09-12

## Contract

- User authorized continuing the bounded build batch with fresh Sol builders. Lead owns integration and shared desktop experiments.
- Winmakase base: `bea1f00`; Glaze input candidate: `dd8fb7d515eb73c7dc1210076922e75c742bb702`, clean before build.
- Lead slice: refresh guest artifacts, verify Sandbox first launch and explicit PowerToys Run activation/indexing. Existing physical Caps, elevation and recovery gates remain separate.
- No daily-stack replacement or channel-selected Muxel inspection/operation. Guest mappings expose only fixture sources and pinned assets, read-only; guest networking and clipboard remain disabled.
- Lead held `env:winmakase-uat` during the batch. CUA session: `winmakase-p0-20260912-build`. Both are released/ended at closeout.
- Sol/high builder scopes: `p0-agent-proof` in `D:/dev/winmakase-wt-agent-proof` (native ancestry prototype); `p0-dialog-proof` in `D:/dev/winmakase-wt-dialog-proof` (guest dialog fixtures). Each writes its own contract/evidence. No nested delegation or concurrent desktop tests.
- Fresh review precedes integration. Recenter after two accepted slices or 60–90 active minutes.

## Current evidence

- Main integration base after this batch's code: `b236794`. Dialog worker `16af7d6` integrated through `d21584f`; ancestry worker `460575e` plus test-only `5cd8699` integrated through `b236794`. Both received fresh review after fixes. Requested builders Sol/high; reviewers Astra/high; runtime-specific actual-model metadata was not exposed.
- Lead integration checks: 72 fixture assertions; 140 Rust library tests passed, 1 explicit real-taskbar test ignored. Baseline workspace fmt/Clippy and commit-pressure integration limitations remain recorded in the ancestry report; no blocked suite was rerun to manufacture a pass.
- Release build of all three Glaze executables passed in 25.63 seconds. Guest Glaze `dd8fb7d` started and answered CLI after installing the pinned VC runtime. Hashes and flags are in [direct-caps-build.json](direct-caps-build.json).
- [Launcher evidence](README.md): four exact argv cases, distinct same-executable indexing, rename/remove after restart, event show/toggle and focused Query. Thirty warm activations p95 54.78 ms; five process starts 2.28–2.83 seconds. No key/bar or physical/display/elevation pass.
- [D1 failures](dialog-runtime.md): broad home rules tile fixed owned/modal dialogs; reload changes the same main window from floating on workspace 2 to tiled and hidden on workspace 1. A new dialog from a hidden parent remains absent from Glaze. These failures define the next policy correction; no P1/R0 gate closes.
- [Ancestry result](../2026-09-12-agent-proof/README.md): unique-sequence adjacent-observation proof accepted as an unwired prototype. Fresh complete snapshot-to-answer median 0.6611 ms; provider coverage, event cadence, HWND placement and background operation remain open.

## Environment disposition

The completed guest was `7c7480ca-3450-4280-b7a5-88e20e0d1113`, build 26100, software rendering, one RDP display. Its Run/Glaze/fixture inventory and script hashes were captured in `evidence/guest-final.json` before shutdown. The lead closed the client first, verified client exit, then stopped the still-registered guest. Final checks show no registered Sandbox and no `vmmemWindowsSandbox` process. No guest remains running, and the environment claim is released.

An earlier default-vGPU guest failed with `udwm.dll`/`0xc00001ad` before login. Another stale-client shutdown sequence crashed the host service and briefly orphaned a VM. Windows released that orphan before administrator cleanup was needed; Chris's later approval did not result in an administrator action. Close the client before CLI stop in this installed Sandbox build to avoid double termination. The Sandbox package observed after its automatic first-start update is `0.8.107.0`.

Host fixture cleanup is complete: no ancestry fixture processes remain; the lead inspected and removed 17 named files without recursion, then seven empty directories, after the worker's broad recursive request was rejected by the local Friday hook.

Local recreation inputs remain at `D:/temp/Winmakase-P0-software.wsb`, `D:/temp/winmakase-p0-guest-assets`, `D:/temp/winmakase-p0-20260912-input` and `D:/temp/winmakase-p0-20260912-output`. The config maps fixture source/assets/probes read-only and only the isolated output directory writable; no repo/config/secret root is writable. `wsb` path arguments require backslashes. Portable probes are saved alongside this report. Historical guest IDs and PIDs are evidence, never valid targets without fresh discovery.

## Next slice

Read NEXT and the D1 source report, verify clean source/worktree/Wire state, then create isolated implementation worktrees for the bounded Glaze classification/reload and Winmakase preserve/auto renderer changes. Freeze their shared match/DTO contract before parallel edits. Fresh review, build and the same disposable guest captures precede acceptance. Keep provider discovery/cadence and A2/A3 as separate proof work; unknown ownership stays unchanged.

No new permission is needed for those authorized source/proof steps. First daily-stack cutover still requires concrete isolated recovery evidence and Chris's approval. The remaining R0 controls/physical-input/product choices are not silently waived.

## Prior checkpoint

[Resume contract](../../../plans/v1/NEXT.md) and [earlier staged guest kit](../2026-09-08-p0/guest-kit.md).
