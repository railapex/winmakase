# P0 checkpoint — 2026-09-08

Status: baseline and fixture preparation in progress. **R0 remains open.** This directory contains executable preparation and measured inventory, not completed desktop acceptance.

## Observed baseline

[Sanitized snapshot](baseline.json), collected by [read-only inventory](../../../../spike/p0-baseline.ps1), identifies allowlisted artifacts, config hashes, task-definition hashes, selected processes, Windows build, monitors and DACL entries. It reads no unrelated process images or window titles.

- Windows 11 Pro `10.0.26200.9278`, x64. Three active displays; each has a 40 px top work-area reserve. Primary is DISPLAY2 at `(0,0)`; the other two extend left/up and right/up. Effective DPI reported 96 on all three. Positive reserves do not prove widget ownership or controls readiness.
- Deployed Winmakase CLI and supervisor each match their respective local release file by SHA-256. Their latest source-touch commits differ (`2bd3f6b` for CLI, `53ef578` for supervisor/taskbar); this is consistent with the recorded selective deployments. Build-time source attestations were not retained, so these are candidate source associations, not reproducible-build proof.
- Deployed Glaze main/CLI/watcher each match the local AppID worktree's release files. The clean source HEAD is `6d62d6616e6acdfd1c1d35ad42ba468f9bb7972e`, based on upstream v3.10.1 `56a8bdee81e9d1a11d6ad840b83b183bb1b3195f`. The main executable contains an `asInvoker`, `uiAccess="false"` manifest snippet. The snapshot records raw XML snippets, not a parsed resource/security audit.
- Zebar's installed executable is **verified against the pinned v3.3.1 MSI payload**, despite its `0.0.0` file version. Kanata's configured task-path candidate likewise matches the v1.12.0 ZIP member. See [upstream evidence](upstream.json).
- PowerToys Run reports file version `0.100.2.0`; its executable hash is recorded. Matching to the release installer payload remains open.
- The elevated Kanata task points at the repo's `gui_winIOv2_cmd_allowed` binary and `spike/caps.kbd`. The process is running, but its elevated image path is unreadable from this session. The snapshot explicitly distinguishes the configured candidate file from process-image proof.
- Kanata and panic task inputs live under a repo whose DACL grants Authenticated Users Modify. These paths and their parents must be protected or retired before release. The current supervisor task retries three times at one-minute intervals; that does not meet the proposed two-second recovery gate.

License sources: Winmakase MIT; [Glaze GPL v3](https://github.com/glzr-io/glazewm/blob/56a8bdee81e9d1a11d6ad840b83b183bb1b3195f/LICENSE.md); [Zebar GPL v3](https://github.com/glzr-io/zebar/blob/8af3cbecff3c00a496b4b8e7e84b6a658e825537/LICENSE.md); [Kanata LGPL v3](https://github.com/jtroo/kanata/blob/02242ef761ef43b860eac30067ce86904be7d466/LICENSE); [PowerToys MIT](https://github.com/microsoft/PowerToys/blob/1d11b732b7ba7dbb265d1151531655fd8d83c76d/LICENSE). Transitive/bundled notice collection remains P5 work.

## Input decision reopened

The deployed config contains `rwin` bindings and no `keybinding_leader`; `caps.kbd` maps Caps to right Win. The existing direct-Caps Glaze candidate was tested and then replaced by the old stack. It was not adopted. [Direct-Caps review](direct-caps.md) records its remaining integration and acceptance work. Do not interpret the v1 plan's initial Kanata dependency as a decision to discard that tested candidate.

## Test environments

Ordinary bounded behavior checks can run here using disposable windows, captured state and verified restoration. WINMAKASE_HOME is file isolation only. A second stack, crash injection, Explorer restart and unproven taskbar suppression require disposable Windows.

Chris enabled `Containers-DisposableClientVM` after the initial inventory. The feature now reports enabled; CBS reports reboot pending and the Sandbox executable was not yet available. Last boot remained `2026-09-08T11:25:26.500073Z`. No restart or Sandbox launch was performed by this session. After reboot, verify the feature, launch Sandbox and apply only the reviewed guest fixture instructions. Sandbox does not establish physical keyboard, mixed-DPI/multi-monitor or real-game acceptance.

## Open evidence

No new launcher timings, popup/tray/calendar interactions, Chrome profile matrix, theme reload, suppression or crash-recovery cases have passed in this batch. Existing dated tests remain historical evidence for their exact builds. Full build flags/attestations, offline dependency packaging and relevant live cases stay open.

Independent source review found two inventory accuracy gaps: task argument changes were invisible, and the hardcoded Kanata candidate was named as if process-verified. The revised script hashes exported task definitions and arguments, labels file candidates, and was rerun successfully. No desktop process/configuration was changed by the inventory or archive extraction.
