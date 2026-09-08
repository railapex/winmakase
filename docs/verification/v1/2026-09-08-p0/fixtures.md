# P0 disposable fixture evidence — 2026-09-08

Status: source fixture complete; disposable Windows UI gates not run.

Integration base: `b73f48f52c50036673a8c77baaa705653e7a58e9`

Builder model: requested `gpt-5.6-sol` at high effort. The runtime did not independently expose the actual model/effort; no substitution was reported.

## Delivered contract

- One C# WinForms executable provides a resizable main window, an owned modeless fixed dialog, an owned modal fixed dialog and an owned fixed tool window. Titles are fixed and exported in the action manifest.
- Four shortcuts target that same executable with distinct checked-in argument arrays. The cases include `Work Profile`, `Tools Pop-up`, and Unicode `Café 東京`.
- Accepted launches write one local JSON file per action under the disposable fixture root. Captures contain only checked-in fixture fields and the exact checked-in arguments.
- The executable checks the Windows Sandbox account, exact `C:\WinmakaseP0` root, fixture marker and explicit consent token before any data write or WinForms initialization.
- Shortcut generation uses `IShellLinkW` for Unicode-safe arguments. It has mandatory root/executable parameters, requires an owned marker, rejects desktop/system roots and out-of-root executables, and writes only `shortcuts/*.lnk` plus `expectations.json` beneath that root.
- Start Menu publication is a separate explicit guest-only step restricted to the Sandbox current-user `Winmakase P0 Fixtures` folder.
- The `.wsb` template maps only `spike/p0` read-only. Network, clipboard and printer redirection are disabled. It does not enable Windows features.

## Host verification

Command:

```powershell
pwsh -NoProfile -File ./spike/p0/Test-P0Fixtures.ps1
```

The check is designed to remain GUI-free. It verifies Windows argument quoting, manifest invariants, `.wsb` isolation settings, C# compilation with the inbox .NET Framework compiler, `.lnk` target/argument/working-directory round trips, out-of-root rejection and exit-before-UI behavior on a non-Sandbox host. The temporary marked fixture directory is removed in `finally`.

Results:

```text
PowerShell 7.6.5: PASS: 38 assertions; no fixture UI launched.
Windows PowerShell 5.1: PASS: 38 assertions; no fixture UI launched.
```

The Windows PowerShell run covers the inbox shell used by the `.wsb` guest bootstrap. The Unicode `.lnk` argument round trip passed through `IShellLinkW` in both runs.

## Disposable UI gates

| Gate | State | Required evidence |
|---|---|---|
| Four window shapes and ownership/classification facts | NOT RUN | Guest screenshots plus sanitized Win32/Glaze facts |
| Shortcut discovery and activation in PowerToys Run | NOT RUN | Search/invoke captures and resulting JSON for all four actions |
| Exact spaces/Unicode argv round trip | NOT RUN | Guest `expectations.json` compared with four capture JSON files |
| Rename/remove/restart Run indexing | NOT RUN | Guest transcript and before/after search captures |
| No console flash | NOT RUN | Guest screen recording or repeated observed activations |
| Focused-monitor key/bar activation | NOT RUN | Integrated chosen-stack guest evidence |
| Fullscreen/elevated activation contexts | NOT RUN | Disposable guest evidence; no daily desktop test |

Windows Sandbox and Hyper-V were unavailable at fixture-slice start. Chris enabled the Sandbox feature during the slice; a host reboot is pending, so guest UI verification remains unrun. This fixture slice did not enable a feature or install a VM tool. It launched no window and changed no desktop, Start Menu or PowerToys state.
