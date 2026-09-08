# Desktop interactions: tray, splits and floats

Reviewed 2026-09-08. Source-verified behavior below is distinguished from live verification. This follow-up changes plans/reporting only.

## Tray and date

V1 will keep chosen icons in the bar and put the rest in an undocked dropdown. Customize tray controls visibility/order without taking over an app's right-click menu. Pinned Zebar exposes native tray actions, so presentation can change without rebuilding tray handling. [Tray API](https://raw.githubusercontent.com/glzr-io/zebar/v3.3.1/packages/client-api/src/providers/systray/systray-provider-types.ts)

The popup must not own the tray provider. Pinned Zebar shares providers by configuration but stopping one consumer stops the backend; tray construction also uses a process-wide one-time initialization. Keep one persistent owner, exchange snapshots/fixed action events and reject stale generations. [Provider manager](https://raw.githubusercontent.com/glzr-io/zebar/v3.3.1/packages/desktop/src/providers/provider_manager.rs), [commands](https://raw.githubusercontent.com/glzr-io/zebar/v3.3.1/packages/desktop/src/commands.rs), [tray construction](https://raw.githubusercontent.com/glzr-io/zebar/v3.3.1/crates/systray-util/src/tray_spy.rs)

Persistent preferences need identity work: IDs may be GUIDs or HWND-plus-UID, and tooltips/images change. Prefer verified GUID identity, otherwise app executable plus icon UID; expose ambiguity or session-only fallback. Keyboard native menus also need correct icon anchoring rather than the parked pointer position. These are P0 proofs, not completed features. [Tray identity/actions](https://raw.githubusercontent.com/glzr-io/zebar/v3.3.1/crates/systray-util/src/systray.rs)

Calendar means an actual month grid with Today, month navigation, selected date, locale and first-weekday settings. It uses bundled date functionality; the provider supplies time, not appointments or a calendar UI. Account/event sync is out of v1. [Date API](https://raw.githubusercontent.com/glzr-io/zebar/v3.3.1/packages/client-api/src/providers/date/date-provider-types.ts)

Tray, volume, calendar and launcher complete the main replacement surfaces. Taskbar suppression still waits for the full P2 access matrix and P4 recovery proof.

## Building a nested split today

Desired layout:

~~~text
+-------------+-------------+
|             |      B      |
|      A      +-------------+
|             |      C      |
+-------------+-------------+
~~~

Current-key route, **derived from the source and not live-tested in this follow-up**:

1. Put A/B/C in one simple vertical column. If they are in a flat row, Caps+J changes that row to a column.
2. Focus A.
3. Caps+Shift+Left moves A out to the left and leaves B/C stacked on the right.

Glaze's movement operation wraps the remaining windows into the opposite split when moving out of a vertical workspace horizontally. This creates the required nested tree. [Movement implementation](file:///D:/dev/glazewm-wt-app-id/packages/wm/src/commands/window/move_window_in_direction.rs#L270)

Afterward J on B/C flips only their pair. J on A refuses because the adjacent sibling is a multi-window split; that is the Winmakase reflow helper's current boundary. [Reflow](file:///D:/dev/winmakase/crates/winmakase/src/reflow.rs#L72)

The more direct construction is also supported: with A/B side by side, focus B, set its next insertion direction to vertical, then open C. That operation is currently unbound. A command issued from another terminal must target B's container ID, or the terminal becomes the subject. P1 exposes this as Split next: stacked / side by side and verifies the sequence. [Split operation](file:///D:/dev/glazewm-wt-app-id/packages/wm/src/commands/container/toggle_tiling_direction.rs#L39), [window insertion](file:///D:/dev/glazewm-wt-app-id/packages/wm/src/commands/window/manage_window.rs#L312)

Omarchy uses dwindle's automatic splitting of the active tile and J to flip a split. Its stock map does not supply a separate direction-preselection key family. Match the pattern without pretending a new Winmakase convenience key is an existing Omarchy binding. [Pinned tiling bindings](https://github.com/omacom/omarchy/blob/v4.0.2/default/hypr/bindings/tiling.lua), [dwindle behavior](https://wiki.hypr.land/0.56.0/Configuring/Layouts/Dwindle-Layout/)

## Float, scratch, quake and pin are different

| Role | Current behavior / v1 disposition |
|---|---|
| Main app | Tiled workspace home; P1 fixes dialog-safe home rules and role actions |
| Dialog | Target: float on parent's workspace, never global scratch; classification fix pending |
| Manual float | Caps+T floats on the current workspace, centered; T returns it to tiling. No default global/topmost behavior |
| Pop/topmost | Caps+O currently sets centered floating and topmost, one-way. It does not follow workspace changes |
| Anonymous scratch | Caps+Alt+S parks a window; Caps+S rescues or retrieves one. Pressing S again is not a hide toggle |
| Named terminal pad | Existing scratchpad toggle term launches/hides/summons a dedicated centered terminal; no generated shortcut invokes it yet. P1 makes it discoverable |
| True pinned tool | Workspace-independent PiP/tool visibility is deferred to a Glaze core primitive |
| Special workspace | Whole multi-window overlay toggled over the active monitor is deferred to a Glaze core primitive |

Dialog lifecycle stays separate from intentional scratch tools. Manual floats do not get moved to a pseudo workspace. [Current bindings](file:///D:/dev/winmakase/keymap/omarchy.toml#L86), [scratch implementation](file:///D:/dev/winmakase/crates/winmakase/src/scratchpad.rs#L119), [Omarchy system rules](https://github.com/omacom/omarchy/blob/v4.0.2/default/hypr/apps/system.lua)

Windows Terminal has a separate native quake mode: a window named _quake occupies the top half of a monitor and can hide/summon. It is not what the current named pad launches. Its Windows virtual-desktop behavior does not automatically integrate with Glaze workspaces; using it requires a narrow ignore/activation test. V1 defaults to the existing dedicated scratch terminal; classic quake remains an optional bounded proof. [Terminal quake mode](https://learn.microsoft.com/en-us/windows/terminal/tips-and-tricks#quake-mode)

## Every former W cut accounted for

| Former cut | Current disposition |
|---|---|
| W0 generated config | Shipped; P1/P3 harden one composition owner and reload behavior |
| W1 typed app homes | Shipped foundation; P1 corrects classification, precedence and adoption |
| W2 focus-or-launch | P1, with explicit launch-new roles and deterministic selection rather than global MRU/cycling |
| W3 float recovery | Restored in P1 and P2: list/cycle loose floats, nonzero-only indicator, scratch separate |
| W4 first named layout | Restored as one bounded v1 composition after P1 identity; originally deferred, now explicitly included at this reconciliation |
| W5 switcher decision | Restored: after a week using app actions and float recovery, record need/no-need; build only from observed missing behavior |

W3/W4 and the named W5 decision were omitted in the prior rewrite. Archiving preserved their text but did not preserve active ownership; this mapping corrects that omission.
