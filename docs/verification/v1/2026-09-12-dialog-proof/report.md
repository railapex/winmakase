# D1 dialog policy proof fixtures

Status: source and pure-host proof complete; disposable guest UI not run by this worker.

## Baselines and worker

- Winmakase source: `bea1f00bd300dbcdac8ad8570dd613248b4e128e`, branch `p0-dialog-proof`, clean before this slice.
- Glaze source inspected read-only: `dd8fb7d515eb73c7dc1210076922e75c742bb702`, `D:/dev/glazewm-wt-v1-input`, clean at inspection.
- Requested worker: Sol builder. Actual runtime identity exposed to this worker: Codex based on GPT-6. No nested delegation or external provider run.
- Shared environment: not claimed or used. Lead session `winmakase-crashdown` owns `env:winmakase-uat`.

## Delivered proof contract

The accepted four action IDs and their exact checked-in argument arrays are unchanged. An accepted guest process now registers only its own active main plus requested owned/modeless, modal or utility HWND. Registration records the exact action tuple, PID, HWND/title and intended role facts. Duplicate modeless/tool windows are suppressed so each role has one deterministic active HWND.

`Capture-P0DialogProof.ps1` repeats the Sandbox user/root/marker/consent guards before loading native interop or invoking Glaze. It reads each registered HWND directly, verifies its PID/title, and records:

- intended classification, native owner role, modality, resizability, taskbar intent and WinForms border style;
- observed HWND, owner/root-owner HWND, PID/TID, title, class, raw style/ex-style, relevant style flags, visibility/enabled state and rectangle;
- fixture-filtered Glaze managed status, ID/parent ID, workspace, state/previous state, display/focus state, class/AppUserModelID and geometry.

The Glaze query is `query workspaces`; the raw recursive response remains in memory and is never persisted. Selection is by the registered HWND, followed by exact fixture title/process validation. Unrelated windows are omitted.

`Compare-P0DialogProof.ps1` compares two named phase captures. It reports native owner/resizability/handle/style continuity and Glaze state/workspace/geometry/parent/display/focus continuity. For an owned window represented by Glaze, it also checks that the window stays on its native owner's workspace. A registered window absent from either Glaze capture is reported as incomplete rather than silently passing; this matters for the fixed tool window, which current Glaze manageability code can exclude through `WS_EX_TOOLWINDOW`.

## Pure host verification

Commands run from the assigned worktree:

```powershell
pwsh -NoProfile -File ./spike/p0/Test-P0Fixtures.ps1
powershell.exe -NoProfile -ExecutionPolicy Bypass -File ./spike/p0/Test-P0Fixtures.ps1
```

Both passed:

```text
PASS: 51 assertions; no fixture UI launched.
```

Coverage includes the original Unicode/space-bearing shortcut and exact argv contract, inbox compilation of the WinForms fixture and HWND interop, non-Sandbox rejection before UI/data/Glaze access, fixture-only recursive Glaze filtering, owner-workspace comparison and a failing reload-state regression case. `git diff --check` passed. No process, window, desktop, Start Menu, task, config or live stack was changed.

## Guest run for the lead

The current guest must copy and compile the updated mapped source once:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\WinmakaseSource\Initialize-P0Guest.ps1 -SourceRoot C:\WinmakaseSource -GuestRoot C:\WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST
```

Required mapped source additions are `DialogFactInterop.cs`, `DialogProof.Common.psm1`, `Capture-P0DialogProof.ps1` and `Compare-P0DialogProof.ps1`; updated `FixtureApp.cs`, `Initialize-P0Guest.ps1` and `README.md` must travel with them. The initializer enforces this set and recompiles the executable.

Example owned-dialog sequence in the guest:

```powershell
$glazePath = (Get-Command glazewm.exe -ErrorAction Stop).Source
powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\WinmakaseP0\source\Start-P0Fixture.ps1 -FixtureRoot C:\WinmakaseP0 -ActionId owned-dialog -ConsentToken DISPOSABLE-WINDOWS-GUEST
powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\WinmakaseP0\source\Capture-P0DialogProof.ps1 -FixtureRoot C:\WinmakaseP0 -ActionId owned-dialog -Phase before-reload -GlazeExecutablePath $glazePath -ConsentToken DISPOSABLE-WINDOWS-GUEST
# Lead invokes the fixture stack's normal reload action after arranging the main window.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\WinmakaseP0\source\Capture-P0DialogProof.ps1 -FixtureRoot C:\WinmakaseP0 -ActionId owned-dialog -Phase after-reload -GlazeExecutablePath $glazePath -ConsentToken DISPOSABLE-WINDOWS-GUEST
powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\WinmakaseP0\source\Compare-P0DialogProof.ps1 -FixtureRoot C:\WinmakaseP0 -ActionId owned-dialog -BeforePhase before-reload -AfterPhase after-reload -ConsentToken DISPOSABLE-WINDOWS-GUEST
```

Repeat the capture around parent movement/reload for `unicode-modal` and `utility-popup`; run `main-spaces` as the main-only control. Evidence is written beneath `C:\WinmakaseP0\data\dialog-proof\` and disappears with the Sandbox.

## Source finding and minimum policy recommendation

At the inspected Glaze baseline, `packages/wm/src/commands/window/manage_window.rs:308` initializes non-resizable windows as floating. Winmakase then always emits a home move plus one state command in `crates/winmakase-keymap/src/lib.rs:674`. Glaze executes every matching rule in `packages/wm/src/commands/window/run_window_rules.rs:25`, so rule order cannot make an earlier dialog float authoritative. `packages/wm/src/commands/general/reload_config.rs:30` clears completed rules and replays the `Manage` event for every active window, reproducing both home moves and state changes on routine reload.

The current match DTO supports process/class/title/AppUserModelID only (`packages/wm-common/src/parsed_config.rs:329`, evaluated in `packages/wm/src/user_config.rs:222`). Native Glaze can detect ownership (`packages/wm-platform/src/platform_impl/windows/native_window.rs:407`) and already caches resizability (`packages/wm/src/models/native_window_properties.rs:6`), but neither fact is available to rules or IPC (`packages/wm-common/src/dtos/window_dto.rs:12`). Normal management also chooses the focused workspace when no target parent is supplied (`packages/wm/src/commands/window/manage_window.rs:160`); it does not resolve an owned window's native owner to the owner's managed workspace.

The smallest supportable correction is:

1. In Glaze, cache native owner HWND/has-owner alongside `is_resizable`; expose both booleans plus owner HWND in `WindowDto`; add exact boolean owner/resizable match fields to `WindowMatchConfig` and `UserConfig::pending_window_rules`. Implement the Windows getter beside `has_owner_window`; populate both tiling/non-tiling DTO paths. Add fixture regression coverage for owned resizable/non-resizable and unowned non-resizable windows.
2. During `create_window`, when a native owner maps to a managed window, choose that owner's workspace for the owned non-tiling window before normal focus-based insertion. Keep native ownership as a fact; do not model a floating dialog as a tiling child of the main window.
3. Stop replaying `Manage` rules during routine config reload at `reload_config.rs:30`. If upstream compatibility needs reload-time rules, add a distinct explicit event; Winmakase's deliberate rehome/reconcile remains a separate previewed command.
4. In Winmakase, add preserve/auto to `WindowRuleState` (`crates/winmakase-keymap/src/lib.rs:270`) so generated rules can omit `set-tiling`/`set-floating`. Generate main-home rules with owner/resizable constraints plus precise app exceptions. Do not rely on ordering; update overlap validation at `lib.rs:654` and golden tests to reject a broad main rule that can still match its dialogs.

This is bounded to the demonstrated facts. Ownership does not classify every unowned modern auth/settings surface, and resizability alone does not identify every popup. Those remain precise app/class exceptions proven by D1 cases. No shared Glaze or Winmakase policy file was changed in this slice.

## Remaining UI cases

- Four action shapes and native/Glaze facts inside the disposable guest.
- Main manually moved and floated, followed by routine reload, for before/after state/workspace/geometry/focus evidence.
- Owned modeless and modal dialog same-workspace placement when the main window is away from its generated home.
- Fixed utility managed-versus-unmanaged disposition under the chosen fixture policy.
- Real Google login/settings and Viscosity cases where available; fixture success does not substitute for them.
