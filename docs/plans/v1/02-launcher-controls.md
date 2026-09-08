# P2 — Launcher and Zebar controls

Status: not started. Depends on P0 launcher/popup proofs; P1 actions for app wiring. Exit: R2.

## Changes

1. Define a finite shared command catalog: role actions, launcher, themes, keys, settings, restart bar, restore Windows and panic. Generate owned Start Menu shortcuts and key-help/bar entries from it. Use executable/argument arrays, quoted at the integration boundary; no arbitrary shell text from labels or theme data.
2. Keep PowerToys Run as the launcher. Caps+Space and bar launcher button invoke the same instance on the focused monitor. Escape returns to prior focus. Native left Win remains available. Record exact supported behavior in elevated/fullscreen contexts.
3. Bundle production JS/CSS/fonts and Zebar API locally with a lockfile and licenses. Remove CDN startup dependencies and runtime Babel. Generate executable paths, monitor policy, clock preferences and other host settings.
4. Use fresh Windows monitor identities and actual primary flags; reconcile after topology/DPI change. Exactly one tray on the current primary; one expected docked bar per configured monitor. Preserve work-area accounting when a popup opens.
5. Replace per-monitor status process polling. Use one primary/shared producer or existing file/event mechanism and fan out state. A stale health record cannot display healthy when its owner died. Do not introduce a daemon just for bar status.
6. Build one keyboard-accessible undocked popup for controls, theme choice and key help. Focus enters on explicit activation. Escape restores the original surviving target only while the popup still owns focus; click-away retains the clicked window; an action opening an app/settings window retains that destination. Persistent bars do not take focus.
7. Complete controls: volume/mute, media transport, native sound-output selection, network/Bluetooth/display/power settings, clock/calendar action, health and recovery. Hide or label absent optional capabilities; provider failure must not blank the bar.

## Taskbar replacement matrix

| User need | V1 route | Evidence required |
|---|---|---|
| Search/start apps and role actions | Run; generated entries | Offline indexing and correct arguments |
| Select running windows | Native Alt+Tab / Run window search; workspace controls | Hidden workspace and fullscreen behavior |
| Tray apps and context menus | Primary Zebar tray | Click/right-click, overflow, hotplug and Explorer restart |
| Volume, mute, media | Zebar provider controls | State reflects actual device/session |
| Select output/device; network/Bluetooth | Native settings links | Correct page opens with taskbar suppressed |
| Notifications / quick settings | Native Windows shortcuts/surfaces, exposed in help/controls | Visible and dismissible with taskbar suppression |
| Time and calendar | Bar clock plus configured calendar action | Keyboard/mouse access, locale and 24-hour preference |
| Lock, sleep, restart/shutdown | Native action routes | Explicit activation; confirm disruptive power actions |
| Restore desktop / panic | Run and independent native shortcut/task | Works with remapper and bar unavailable |

No custom notification center or winget catalog/status subsystem. Optional weather/stats may remain, with freshness/error labels and no readiness dependency.

## Verification

R2 repeats launch/popup interaction from tiled, floated, Glaze fullscreen, application F11 fullscreen and elevated applications; mouse and keyboard; empty and occupied workspaces. Assert Escape restores the surviving origin only when appropriate, click-away keeps the clicked target, and completed actions keep their destination. Exercise fast toggles, long labels and screen edges.

Display matrix: one/three monitors, primary not at the upper-left of the virtual desktop, negative coordinates, secondary directly above/below at x=0, primary change, 100/150/200% DPI, occasional monitor unplug/replug. Popup bounds stay inside the target work area; it adds zero dock reserve. Exactly the expected bars and one tray survive each settled change.

Run offline from a cold profile/cache. Thirty-minute idle measurement includes Zebar backend, all widgets, WebView2 children and status producers. There must be no per-monitor status subprocess loop, console flashes, orphan popups, duplicate listeners or increasing child count.

Capture 30 warm / 5 cold activations; proposed warm p95 ≤200 ms to input-ready. Compare memory/CPU to R0 with identical monitors/providers. Investigate sustained growth over a multi-hour soak. Finish every row of the replacement matrix before P4 removes edge-peek behavior.
