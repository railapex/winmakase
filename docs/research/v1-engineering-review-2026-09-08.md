# V1 engineering review — 2026-09-08

## Decision

Finish Winmakase on GlazeWM, Zebar, Kanata and PowerToys. Scope v1 to launching, window policy, controls, appearance and recovery. Omit the application catalog. Preserve Omarchy as the compatibility target; an unsupported chord is an explicit gap, not permission to substitute unrelated behavior.

This review follows the [initial assessment](v1-assessment-2026-09-08.md) and the user's accepted direction. Local code inspection and upstream source research inform the [implementation plans](../plans/v1/README.md). No implementation or live configuration was changed for this planning pass.

## Evidence quality and baseline

Local baseline: Winmakase main at e84de6c before this documentation pass; the prior float/tile fix is 2bd3f6b. Existing test evidence belongs to that implementation: 207 passed, two ignored; preexisting fmt/clippy failures in taskbar/supervisor were recorded. Those results do not prove the planned v1 behavior.

Fresh local process-file inspection found PowerToys PowerLauncher file version 0.100.2.0. Zebar's file version was 0.0.0 and does not prove its upstream version. API research below uses Zebar v3.3.1; P0 must reconcile exact deployed artifact provenance. The Glaze source baseline is 3.10.1 with a local AppUserModelID patch and an unsigned ui_access=false dogfood build; record exact commits/build hashes in R0 rather than relying on prose.

No immutable Muxel live binary was inspected, hashed or changed. Isolated profile probes and previous live findings remain dated evidence; P0/R1 define what must be repeated for a release candidate.

## Omarchy fidelity

Omarchy 4's release describes Quickshell bringing desktop controls together. This makes the useful target a coherent interaction across launcher, bar and controls, rather than a collection of copied key labels. [Omarchy 4 release](https://github.com/omacom/omarchy/releases/tag/v4.0.0)

Pinned v4.0.2 application bindings distinguish launch commands from explicit focus-or-launch actions. The browser launcher delegates to the selected browser without universally forcing a new window. A global “all app keys focus existing windows” rule would misrepresent that behavior. Preserve the distinction and use role-specific new-window support where promised. [Application bindings](https://github.com/omacom/omarchy/blob/v4.0.2/default/hypr/bindings/applications.lua), [browser launcher](https://github.com/omacom/omarchy/blob/v4.0.2/bin/omarchy-launch-browser), [terminal launcher](https://github.com/omacom/omarchy/blob/v4.0.2/bin/omarchy-launch-terminal), [focus helper](https://github.com/omacom/omarchy/blob/v4.0.2/bin/omarchy-launch-or-focus)

The current keymap remains valuable coverage data. Exact-key parity is constrained by Windows-reserved chords and Glaze capabilities. Keep unavailable grouping recorded and plan future support; no requirement to build native grouping before v1.

F versus Alt+F needs an explicit correction: the bare toggle uses Glaze's configured fullscreen default, and the dogfood source base sets maximized=true. Both current chords therefore select maximized mode. P1 assigns distinct true fullscreen/maximized behavior and verifies restoration. [Glaze dispatch](file:///D:/dev/glazewm-wt-app-id/packages/wm/src/wm.rs#L678), [machine base](file:///C:/Users/chris/.winmakase/glazewm/base.yaml#L41), [keymap](file:///D:/dev/winmakase/keymap/omarchy.toml#L93)

## Windows and Chrome roles

Local source review found an unconditional generated state choice in app homes, with tiling as the default. Glaze's automatic floating decision can therefore be overridden. All matching rules execute, so placing base exceptions first is insufficient. Reload also replays manage rules against existing windows. These are source-level findings; R1 must reproduce the user-visible cases and fix classification/reconciliation together. [Keymap renderer](file:///D:/dev/winmakase/crates/winmakase-keymap/src/lib.rs), [archived app-home contract](../plans/archive/2026-09-08/DESIGN.md)

Chrome assigns Windows AppUserModelIDs for taskbar grouping using installation/profile-related information. A profile identity can group several windows, and incognito is not a separate profile identity guarantee. Explicit per-window identity is useful but is not a unique window handle or a complete main-window classifier. Read observed local IDs, keep machine-specific values local and test late publication. [Chromium Windows shortcut/taskbar design](https://chromium.googlesource.com/chromium/src/+/main/docs/windows_shortcut_and_taskbar_handling.md), [AppUserModelID semantics](https://learn.microsoft.com/en-us/windows/win32/shell/appids)

The existing patch exposes window_app_id/appUserModelId; it does not settle dialog/popup eligibility or update timing. Work/personal actions need explicit launch arguments plus matching rules. External URL routing stays unchanged. The least complex reliable focus policy is deterministic plus last selected by the role, with per-identity serialization and a bounded in-flight launch record. A resident global MRU tracker is unnecessary.

## Launcher and controls

PowerToys Run indexes Start Menu program/shortcut entries and includes window search, system commands and settings access. Generated shortcuts can expose a finite Winmakase command set through the existing launcher. Whether distinct arguments to one binary remain distinct indexed results is a required experiment, not an established result. [Run documentation](https://learn.microsoft.com/en-us/windows/powertoys/run)

Command Palette remains a bounded alternative within PowerToys if needed. Its current capabilities should be tested rather than judged from early releases. A full extension carries a separate packaged .NET/COM integration, so it is not the default for a finite command set. [Command Palette](https://learn.microsoft.com/en-us/windows/powertoys/command-palette/overview), [extension architecture](https://learn.microsoft.com/en-us/windows/powertoys/command-palette/extensibility-overview)

Pinned Zebar audio types include volume/mute and device enumeration, but no default-output switching operation. Use its native provider for volume/mute and Windows Settings for output selection. Native Settings links also cover Bluetooth, network and display. [Audio API](https://raw.githubusercontent.com/glzr-io/zebar/v3.3.1/packages/client-api/src/providers/audio/audio-provider-types.ts), [Settings URI reference](https://learn.microsoft.com/en-us/windows/apps/develop/launch/launch-settings)

Zebar's widget API makes an undocked popup plausible. It does not prove correct focus, dismissal, fullscreen z-order or DPI placement; P0 tests those. Its monitor helper caches primary/all monitors with a refresh TODO, so merely replacing the x=0 heuristic with that helper does not prove correct topology handling. [Widget API](https://raw.githubusercontent.com/glzr-io/zebar/v3.3.1/packages/client-api/src/desktop/widgets.ts), [monitor implementation](https://raw.githubusercontent.com/glzr-io/zebar/v3.3.1/packages/client-api/src/desktop/monitors.ts)

Local bar findings: development React/Babel/API/font downloads at runtime; embedded paths/preferences; primary tray inferred from screenX=0; each bar launches a status child every five seconds. Three bars produce 36 child launches per minute by arithmetic. Runtime cost beyond that count has not been measured. Bundle assets and use one shared status producer without adding another daemon. [Bar source](file:///D:/dev/winmakase/zebar/bar.html), [bar styles](file:///D:/dev/winmakase/zebar/styles.css)

## Theme boundary

The current Rust adapter parses the required semantic palette fields, tolerates extra fields and renders Terminal schemes. It has two fixture themes and two scheme golden files. Older claims of complete nineteen-theme coverage or a finished theme apply path were wrong. [Theme crate](file:///D:/dev/winmakase/crates/winmakase-theme)

The v4.0.2 source tree has 22 stock colors.toml palettes. Generate a pinned manifest for parser coverage; ship two fully verified visual presets first. Keep the existing ANSI mapping aligned with the upstream template. [Pinned theme tree](https://github.com/omacom/omarchy/tree/v4.0.2/themes), [Ghostty template](https://github.com/omacom/omarchy/blob/v4.0.2/default/themed/ghostty.conf.tpl)

Omarchy's repository-theme staging filters executable/configuration material and serializes theme application. Winmakase should import palette/image data, not execute theme Lua or copy arbitrary editor settings. [Theme staging implementation](https://github.com/omacom/omarchy/blob/v4.0.2/bin/omarchy-theme-set)

Windows Terminal fragments can add schemes, while selection and arbitrary user-profile behavior need narrower settings handling and verification. Use owned fragments plus backed-up owned-key selection edits where needed. [Terminal fragment documentation](https://learn.microsoft.com/en-us/windows/terminal/json-fragment-extensions)

Core appearance can cover bar/popup, borders, Terminal, wallpaper and optional system modes. Run's exact appearance response remains a P0 probe. Preserve Chrome's useful work/personal profile colors; global Chromium tint stays off. Applying several independent targets is staged with rollback, not one atomic transaction.

## Supervisor audit

| Finding from current source | V1 disposition |
|---|---|
| Glaze death stops Kanata; Kanata death restarts alone | Define and implement the symmetric pair failure contract |
| Startup includes bar in the rollback set; runtime recovery separates it | Keep healthy pair through initial bar failure |
| kill_and_record can mark stopped after failed termination | Retain tracking and report cleanup failure |
| PID/health singleton is a check rather than exclusive ownership | Use a real per-session ownership primitive |
| Single control file can be overwritten/deleted by another request | Atomic claims, token-owned results and concurrency tests |
| Taskbar baseline exists only in memory and suppression precedes readiness | Durable baseline and explicit takeover gate |
| Worker cannot restore after hard death; task restart is delayed | Minimal independent guard and native panic fallback |
| Dock health accepts any positive top reserve | Verify actual expected widget ownership and geometry |
| Commit wait can indefinitely block startup/adoption | Reproduce, remove or bound; distinguish adoption |
| Edge peek/flyout exceptions implement temporary taskbar UX | Delete after replacement and suppression gates |

These are local source findings, not a claim that every failure occurred on the daily desktop. [Supervisor](file:///D:/dev/winmakase/crates/winmakase/src/supervisor.rs), [control transport](file:///D:/dev/winmakase/crates/winmakase/src/control.rs), [taskbar controller](file:///D:/dev/winmakase/crates/winmakase/src/taskbar.rs), [pressure gate](file:///D:/dev/winmakase/crates/winmakase/src/commit_pressure.rs)

Windows documents ABM_SETSTATE as setting autohide/always-on-top; it is not a permanent taskbar-disable API, and its TRUE return does not prove a visible result. Test native autohide against minimal suppression and retain Explorer. [ABM_SETSTATE](https://learn.microsoft.com/en-us/windows/win32/shell/abm-setstate)

Appbars negotiate their position and respond to position-change notifications. Favor a correct dock lifecycle over permanent restart-based repair. TaskbarCreated signals recreation and can also occur after primary DPI changes; reconcile current state rather than assuming a cause. [Application desktop toolbars](https://learn.microsoft.com/en-us/windows/win32/shell/application-desktop-toolbars), [ABN_POSCHANGED](https://learn.microsoft.com/en-us/windows/win32/shell/abn-poschanged), [Taskbar creation notification](https://learn.microsoft.com/en-us/windows/win32/shell/taskbar#taskbar-creation-notification)

Use actual rcMonitor/rcWork geometry and MONITORINFOF_PRIMARY for monitor identity. The memory gate's ullAvailPageFile value concerns commit available to the calling process, not an exact system-wide pressure percentage. [MONITORINFO](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-monitorinfo), [MEMORYSTATUSEX](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/ns-sysinfoapi-memorystatusex)

Privileged task registration currently reaches user-writable repository paths. Protect the entire execution chain: binary, config, parent replacement, elevated panic code and update path. Config ACLs alone are insufficient. [Task registration](file:///D:/dev/winmakase/installer/register-tasks.ps1), [panic implementation](file:///D:/dev/winmakase/spike/panic.ps1)

## Decisions carried into plans

Keep the supervisor, fix its ownership and failure contract, then delete temporary hover behavior. Keep the application model out of lifecycle supervision. Keep Run unless a bounded proof disproves the shortcut route. Use a small Zebar popup if its focus/dock tests pass. Ship two complete themes with restoration. Finish packaging last against a protected layout designed early.

P0 resolves uncertain primitives; R1–R6 test actual behavior and failure recovery. A thorough plan does not convert those unknowns into passed gates.

## Planning review completed

Three independent domain reviews covered input/apps, launcher/appearance and supervisor/recovery. Round 1 narrowed upstream launch claims, separated reload/controlled restart/crash reconstruction, defined hidden/minimized/scratch selection and late identity refresh, assigned the fullscreen correction, corrected the display matrix, bounded preview behavior and made restoration independent of blocking cleanup.

Round 2 reviewed shared behavior across packages. It added one generated-config composition path, mutation ownership across themes/update/off/uninstall, stale-generation rejection, privileged validation of journal input and conditional popup focus restoration. Required findings were incorporated into the design and relevant verification cases.

Document verification checks current local links, compares all seven archived originals against the prior commit, runs prose lint and checks the final diff. These reviews close the planning deliverable only. R0–R6 remain pending.
