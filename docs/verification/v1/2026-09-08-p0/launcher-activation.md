# PowerToys Run activation seam — P0

Observed 2026-09-08. **Source and event-existence proof only; visible activation is not yet tested.**

PowerToys v0.100.2 resolves to `1d11b732b7ba7dbb265d1151531655fd8d83c76d`. Its [shared constants](https://github.com/microsoft/PowerToys/blob/1d11b732b7ba7dbb265d1151531655fd8d83c76d/src/common/interop/shared_constants.h#L18-L21) define an invocation event named `Local\PowerToysRunInvokeEvent-30f26ad7-d36d-4c0e-ab02-68bb5ff3c4ab`. This is an internal integration seam, not a promised public API; keep its use pinned and capability-checked.

[MainViewModel](https://github.com/microsoft/PowerToys/blob/1d11b732b7ba7dbb265d1151531655fd8d83c76d/src/modules/launcher/PowerLauncher/ViewModel/MainViewModel.cs#L100-L109) routes that event to `OnHotkey`. The handler checks the fullscreen-ignore preference, applies last-query behavior and toggles Run's visibility. Signaling does not guarantee opening, readiness, foreground focus or fullscreen access. A command must distinguish already visible from hidden and prove its intended behavior.

The host call below succeeded and disposed its handle. It did **not** signal or consume the event:

```powershell
$runEvent = [Threading.EventWaitHandle]::OpenExisting('Local\PowerToysRunInvokeEvent-30f26ad7-d36d-4c0e-ab02-68bb5ff3c4ab')
try { 'event opened; not signaled' } finally { $runEvent.Dispose() }
```

The chosen Windows observation tool initialized, but `sky.list_apps()` twice failed with `Computer Use native pipe is unavailable ... (os error 2)`. No UI input, launcher activation, screenshot, timing claim or keyboard change followed. Resolve observation capability after the Sandbox/host restart; this error does not prove that the invocation seam fails.

Next proof: normal/elevated/fullscreen activation, already-open semantics, input focus, focused-monitor placement, Escape/click-away restoration and warm/cold timing. Preserve later user focus. An explicit command would replace Caps+Space's current reliance on a synthesized Windows modifier; a tap-Caps action remains optional.

Shortcut source check: [Win32Program](https://github.com/microsoft/PowerToys/blob/1d11b732b7ba7dbb265d1151531655fd8d83c76d/src/modules/launcher/Plugins/Microsoft.Plugin.Program/Programs/Win32Program.cs#L889-L911) deduplicates by shortcut-derived name, executable-name field and resolved path; arguments are not part of that equality key. Distinct owned shortcut filenames are therefore a prerequisite for the four-action proof. The launch path retains the `.lnk`, allowing its arguments to apply. This supports testing the current approach; only runtime search/invocation can establish indexing and correct argv.

The pinned PowerToys user installer is staged locally for guest preparation: `PowerToysUserSetup-0.100.2-x64.exe`, 284,920,432 bytes, SHA-256 `945FDF327E4D38E4CED61B0727B7AB8A1222958782982052989DDF7CB7096F62`. It matches the release asset digest and was not executed. Data extraction of its attached CAB, MSI and `cab1.cab` yielded `BaseApplicationsFiles_File_PowerToys.PowerLauncher.exe`; its 192,312 bytes and SHA-256 `062ED25AA8F343C83641277111BCCAC366B25EBF80021B1E4D783EEF25D2E688` match both the captured baseline and a fresh hash of the installed Run executable. The MSI SHA-512 matches the bundle manifest. [Machine-readable chain](upstream.json). This verifies the Run executable, not every installed dependency.
