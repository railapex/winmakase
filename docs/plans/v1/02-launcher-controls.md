# P2 — Launcher and Zebar controls

Status: not started. Depends on P0 launcher/popup proofs; P1 actions for app wiring. Exit: R2.

## Changes

1. Define a finite shared command catalog: role actions, launcher, themes, keys, settings, restart bar, restore Windows and panic. Generate owned Start Menu shortcuts and key-help/bar entries from it. Use executable/argument arrays, quoted at the integration boundary; no arbitrary shell text from labels or theme data.
2. Keep PowerToys Run as the launcher. Caps+Space and bar launcher button invoke the same instance on the focused monitor. Escape returns to prior focus. Native left Win remains available. Record exact supported behavior in elevated/fullscreen contexts.
3. Bundle production JS/CSS/fonts and Zebar API locally with a lockfile and licenses. Remove CDN startup dependencies and runtime Babel. Generate executable paths, monitor policy, clock preferences and other host settings.
4. Use fresh Windows monitor identities and actual primary flags; reconcile after topology/DPI change. Exactly one tray on the current primary; one expected docked bar per configured monitor. Preserve work-area accounting when a popup opens.
5. Replace per-monitor status process polling. Use one primary/shared producer or existing file/event mechanism and fan out state. A stale health record cannot display healthy when its owner died. Do not introduce a daemon just for bar status.
6. Build one keyboard-accessible undocked popup host with controls, tray, calendar, theme and key-help views. Focus enters on explicit activation. Escape restores the original surviving target only while the popup still owns focus; click-away retains the clicked window; an action opening an app/settings window or native tray menu retains that destination. Persistent bars do not take focus.
7. Complete controls: volume/mute, media transport, native sound-output selection, network/Bluetooth/display/power settings, a real month calendar, health and recovery. Hide or label absent optional capabilities; provider failure must not blank the bar.
8. Restore W3's nonzero-only loose-float count and a list with forward/reverse current-workspace recovery, backed by P1 state. Distinguish scratch tools/dialogs from loose floats; keep unmanaged elevated windows in native Alt+Tab. Expose split-next actions, one named terminal toggle and the daily layout through the shared catalog/help after P1 verification.

## Tray dropdown and user pinning

Replace inline expansion with an undocked icon grid. User-pinned icons stay in the bar; remaining provider-emitted icons go in the dropdown. Keep bar width and dock reserve constant as it opens. Customize tray provides labeled Always show controls and ordering; app right-click continues to open the app's native menu.

One long-lived primary-bar module owns the tray provider. The popup consumes snapshots and sends fixed icon-action events to that owner; it never creates/stops the provider. Pinned Zebar shares backend providers without safe per-consumer stop ownership, and its tray backend is initialized once per process. Owner transfer on primary change needs its own proof, not a destroy/recreate assumption.

Use owner generation plus current runtime icon ID for actions; reject removed/stale icons. Serialize image data across webviews and manage popup-local image URLs by content identity; do not pass a blob URL whose owner can disappear. Native tray-menu activation hands off focus without restoring over the menu. Keyboard invocation must anchor to the focused icon; a provider patch may be required, not pointer warping.

Store user pin preferences separately from native Windows tray settings. Prefer a verified persistent GUID; otherwise resolve live owner executable plus icon UID where discriminating. Runtime HWND IDs, tooltip text and image hashes are not universally durable identities. Offer explicit app-wide matching or labeled fallback/session-only pinning when identity is ambiguous. P0 selects a bounded helper or metadata patch from observed coverage. New/unmatched icons default to the dropdown; missing apps retain preferences without dead UI entries.

## Date and calendar

The bar shows date/time according to user preferences. Clicking opens a month grid with today, selected date, previous/next month, Today and a localized full-date label. Support keyboard navigation, first weekday, locale and 12/24-hour choices. Optional Open calendar invokes a configured role; account connection and event sync are outside v1.

Use bundled date functionality and civil-date arithmetic in the configured zone. Refresh today after midnight, wake and timezone change. Selecting a date does not change the system clock. No calendar framework is needed.

## Taskbar replacement matrix

| User need | V1 route | Evidence required |
|---|---|---|
| Search/start apps and role actions | Run; generated entries | Offline indexing and correct arguments |
| Select running windows | Native Alt+Tab / Run window search; workspace controls | Hidden workspace and fullscreen behavior |
| Tray apps and context menus | Configurable pinned icons plus undocked tray grid | Persistent choices, native menus, keyboard anchoring, hotplug and Explorer restart |
| Volume, mute, media | Zebar provider controls | State reflects actual device/session |
| Select output/device; network/Bluetooth | Native settings links | Correct page opens with taskbar suppressed |
| Notifications / quick settings | Native Windows shortcuts/surfaces, exposed in help/controls | Visible and dismissible with taskbar suppression |
| Time and calendar | Bar date/time plus month calendar and optional external calendar action | Keyboard/mouse access, locale/first weekday, midnight/wake and 12/24-hour preference |
| Lock, sleep, restart/shutdown | Native action routes | Explicit activation; confirm disruptive power actions |
| Restore desktop / panic | Run and independent native shortcut/task | Works with remapper and bar unavailable |

No custom notification center or winget catalog/status subsystem. Optional weather/stats may remain, with freshness/error labels and no readiness dependency.

## Verification

R2 repeats launch/popup interaction from tiled, floated, Glaze fullscreen, application F11 fullscreen and elevated applications; mouse and keyboard; empty and occupied workspaces. Assert Escape restores the surviving origin only when appropriate, click-away keeps the clicked target, and completed actions keep their destination. Exercise fast toggles, long labels and screen edges.

Display matrix: one/three monitors, primary not at the upper-left of the virtual desktop, negative coordinates, secondary directly above/below at x=0, primary change, 100/150/200% DPI, occasional monitor unplug/replug. Popup bounds stay inside the target work area; it adds zero dock reserve. Exactly the expected bars and one tray survive each settled change.

Run offline from a cold profile/cache. Thirty-minute idle measurement includes Zebar backend, all widgets, WebView2 children and status producers. There must be no per-monitor status subprocess loop, console flashes, orphan popups, duplicate listeners or increasing child count.

Capture 30 warm / 5 cold activations; proposed warm p95 ≤200 ms to input-ready. Compare memory/CPU to R0 with identical monitors/providers. Investigate sustained growth over a multi-hour soak. Finish every row of the replacement matrix before P4 removes edge-peek behavior.

Tray acceptance: zero/some/all pinned; many icons; stable order; app/Explorer/Zebar/OS restart; late GUID/changing tooltip/image; duplicate/unresolvable identities; icon removal during focus/action; pointer on another monitor during keyboard menu activation; primary removal with popup open. Verify required tray apps remain reachable when the provider omits hidden/unconvertible icons. No taskbar retirement while a required app's access is lost.

Run 100 tray/calendar open-close cycles without provider restart, lost dock or growing listener/image/process counts. Test leap February/year boundary, multiple locales/first weekdays/DST zones, midnight/wake/timezone changes, both themes and all scales. Tray preference writes interleaved with theme/update/off/uninstall obey shared mutation ownership.
