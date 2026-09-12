# Replaying the guest probes

These are the guarded scripts used by the lead, saved as experiment evidence. They hard-code disposable guest paths and pinned hashes. They are not an installer or production action API. Read the batch handoff and verify the environment claim before replay.

Map `spike/p0` to `C:/WinmakaseSource`, this probe directory to `C:/WinmakaseProbe`, and the pinned assets to `C:/WinmakaseAssets`, all read-only. Map an isolated output directory to `C:/WinmakaseEvidence`, writable. Use software rendering and disable networking/clipboard/printer/audio input as recorded in the handoff. Initialize `C:/WinmakaseP0` with the checked-in `Initialize-P0Guest.ps1` before any probe.

In the guest, invoke scripts through inbox PowerShell with `-NoProfile -WindowStyle Hidden -ExecutionPolicy Bypass -File`. `wsb exec` returns an inner JSON `ExitCode`; its own shell exit code alone is insufficient. It does not return script stdout, so inspect the named evidence files and installer logs. An observer timeout does not mean the installer stopped; inspect before retrying.

1. `Install-PinnedPowerToys.ps1`, `Start-GuestPowerToys.ps1`, then `Enable-GuestRun.ps1`. The last stops only guest PowerToys processes under its install path, enables only Run and writes BOM-free UTF-8 settings.
2. `Publish-GuestFixtures.ps1`, then restart Run via `Enable-GuestRun.ps1`. `Probe-RunEvent.ps1 -Action Signal -Label <label>` toggles Run. Use Observe first to know its state.
3. `Query-GuestRun.ps1 -Query "Winmakase P0 - Main Spaces" -Label <label>` captures Run's UIA result list. Inspect the exact application result before `Invoke-GuestRunFixture.ps1 -ActionId main-spaces`. Repeat the exact manifest labels/IDs for the other three actions. Invocation refuses an existing per-action capture; use a fresh guest for a full replay.
4. `Benchmark-GuestRun.ps1` records 30 warm show/input-ready samples and leaves Run hidden. `Benchmark-ColdGuestRun.ps1` records five runner/Run process restarts; those are warm-cache process-start samples, not cold boots.
5. `Change-GuestShortcuts.ps1 -Action RenameRemove`, restart Run, and inspect renamed/old/removed search results. Restore with `-Action Restore` and restart Run. No web-search result was invoked.
6. Install the exact runtime with `Install-GuestVCRuntime.ps1`. Its original 90-second observer deadline was too short for this offline guest; the install finished after about 244 seconds. Inspect the final log and runtime file before continuing. Start Glaze with `Start-GuestGlaze.ps1`; only the fixture is managed by `glaze-fixture.yaml`.
7. `Capture-GuestDialogs.ps1 -Phase initial`, then `Arrange-GuestDialogProof.ps1 -Action Arrange` and `-Action Reload`. Capture screenshots on each side. The saved comparisons report failure; that is the regression being reproduced.
8. Optional hidden-parent follow-up: `Probe-GuestOwnedDialog.ps1 -Action Inspect`, then `CloseMove`, then `Open`. It closes only its registered modeless dialog and moves that main window. The fallback uses a previously inspected native child HWND with fresh PID/parent/text checks; it is intentionally narrower than general background CUA support.

`Refresh-GuestProbeScripts.ps1` updates only capture/comparison scripts without recompiling an active fixture. `Snapshot-GuestFinal.ps1` exports only the known guest processes and fixture hashes. Save evidence before closing the client and discarding the Sandbox. Do not reuse historical guest IDs, PIDs or HWNDs.
