# P0 disposable launcher and window fixtures

This directory is a source-only harness for a Windows Sandbox guest. Host-side tests compile the fixture, serialize shortcuts into a temporary owned directory and confirm that the executable exits before creating UI or data on a non-Sandbox host.

The executable exposes four deterministic actions through one binary:

| Action | Window fact | Argument edge case |
|---|---|---|
| `main-spaces` | resizable main window | profile label containing a space |
| `owned-dialog` | owned, modeless, non-resizable dialog | separate action against the same executable |
| `unicode-modal` | owned modal, non-resizable dialog | `Café 東京` |
| `utility-popup` | owned fixed tool window, absent from the taskbar | hyphenated/spaced profile label |

Each accepted invocation replaces `C:\WinmakaseP0\data\<action>.json`. The file contains only the checked-in fixture ID, window mode, profile label and exact checked-in argument array. The app rejects unknown action tuples. It does not inspect the environment, copy configuration or accept arbitrary capture fields.

While an accepted action is open, the app also writes `data/<action>-windows.json`. That registration contains only the action's exact checked-in arguments, PID plus process creation time, exact fixture HWNDs/titles, a monotonic per-process window generation and intended role facts. Each generation is stored on its live HWND as a native window property. It does not enumerate other windows. `Capture-P0DialogProof.ps1` re-reads only those registered HWNDs, validates process creation and the native property before and after the Glaze query, records native owner/style/geometry facts, and emits only matching fixture windows from the recursive workspace response. Reused PID/HWND/title values fail closed.

## Pure host check

```powershell
pwsh -NoProfile -File ./spike/p0/Test-P0Fixtures.ps1
```

This does not open fixture UI or alter the desktop, Start Menu, PowerToys or Windows features. Its temporary fixture directory is removed after the checks. Shortcut creation uses the Unicode `IShellLinkW` interface so non-ASCII arguments do not pass through the local ANSI code page.

## Disposable guest setup

1. Copy `WinmakaseP0.wsb.template` outside the repository and replace `__ABSOLUTE_PATH_TO_SPIKE_P0__` with the absolute path to this directory. Do not map the repository root or a secrets/config directory.
2. Once Windows Sandbox is available by an independently approved machine change, open that `.wsb` file. Its only host mapping is this source directory, read-only. Networking, clipboard and printer redirection are disabled.
3. Initialization compiles the C# WinForms source with the inbox .NET Framework compiler, creates `C:\WinmakaseP0`, generates four shortcuts under `C:\WinmakaseP0\shortcuts`, and writes `expectations.json`. It does not launch UI or publish to the Start Menu.
4. Direct-launch an action inside the guest when needed:

   ```powershell
   powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\WinmakaseP0\source\Start-P0Fixture.ps1 -FixtureRoot C:\WinmakaseP0 -ActionId unicode-modal -ConsentToken DISPOSABLE-WINDOWS-GUEST
   ```

5. For an explicit PowerToys Run indexing test, publish only to the disposable guest's current-user Start Menu:

   ```powershell
   powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\WinmakaseP0\source\Publish-P0GuestShortcuts.ps1 -FixtureRoot C:\WinmakaseP0 -DestinationDirectory "C:\Users\WDAGUtilityAccount\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Winmakase P0 Fixtures" -ConsentToken DISPOSABLE-WINDOWS-GUEST
   ```

6. Compare each `data/<action>.json` with the corresponding `expectedCapture` in `expectations.json`. Close Windows Sandbox to discard the guest and its Start Menu publication.

## D1 dialog and reload capture

Run one action at a time and leave its windows open. Capture the settled initial state with the guest's exact `glazewm.exe` path:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\WinmakaseP0\source\Capture-P0DialogProof.ps1 -FixtureRoot C:\WinmakaseP0 -ActionId owned-dialog -Phase initial -GlazeExecutablePath C:\path\to\glazewm.exe -ConsentToken DISPOSABLE-WINDOWS-GUEST
```

For reload preservation, arrange the main window in the test workspace/state, capture `before-reload`, invoke the lead-owned fixture stack's normal reload action, wait for it to settle, then capture `after-reload`. Compare the two fixture-only captures:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\WinmakaseP0\source\Compare-P0DialogProof.ps1 -FixtureRoot C:\WinmakaseP0 -ActionId owned-dialog -BeforePhase before-reload -AfterPhase after-reload -ConsentToken DISPOSABLE-WINDOWS-GUEST
```

Repeat for `unicode-modal` and `utility-popup`. The native record distinguishes main, owned modeless, owned modal and utility intent; observed facts include HWND, window generation, PID/process creation time/TID, owner/root-owner HWND, exact title/class, style/ex-style, resizability, visibility and rectangle. The Glaze subset records managed/unmanaged status, container ID/parent ID, workspace, state, display/focus state and geometry. The comparison reports replacement process/window lifetimes, Glaze remanagement, native ownership/style mismatches, parent-workspace separation and state/workspace/geometry changes across reload. A missing Glaze representation is `incomplete`, because an explicit utility policy may intentionally leave a tool window unmanaged.

The probe does not apply or reload config. It starts the accepted `glazewm.exe` with redirected in-memory output and applies one ten-second deadline to process exit plus stdout/stderr completion. At deadline it closes its readers and kills only the exact query process if still active; descendants that merely inherited pipe handles are not killed. It reports the actual exit error or timeout and never persists the unfiltered Glaze response. The operator owns fixture policy, parent moves and reload timing; named phases are evidence labels, not commands.

All mutating/UI entry points require the literal consent token, the Sandbox-only `WDAGUtilityAccount`, the exact guest root `C:\WinmakaseP0`, and the fixture marker. The raw executable repeats these checks before WinForms initialization. The unguarded shortcut generator can write only beneath an explicit marked fixture directory; it has no Start Menu default.
