# Winmakase design

Accepted v1 direction, 2026-09-08. **Target contracts below are not claims that current code satisfies them.** [TODO](TODO.md) records status; [plans](plans/v1/README.md) define implementation and evidence. Earlier decisions and completed spike history are preserved in the [design archive](plans/archive/2026-09-08/DESIGN.md).

## Product boundary

A coherent Windows desktop for launching, tiling, controls and appearance. Keep GlazeWM, Zebar, Kanata LLHOOK and PowerToys Run. Follow Omarchy's interaction patterns and exact bindings where the stack supports them; unsupported operations remain explicit gaps with future convergence routes.

V1 does not include a packaged app catalog, general webapp installer, replacement WM/shell, community-theme guarantee or broad editor configuration. Existing apps can be assigned personal roles. Future platforms are [parked](plans/future-desktop-platforms.md).

## Current implementation

| Area | Present today | Remaining v1 work |
|---|---|---|
| Input | Caps → right Win; Glaze native chords; left Win native; generated grammar/local overrides | Update comparison against pinned Omarchy 4; test native collisions and held/repeated chords |
| Windows | Current-state float/tile, reflow, scratchpads, generated typed app homes; patched window_app_id | Dialog-safe classification, deliberate reconciliation, reliable focus-or-launch |
| Launch | PowerToys Run and direct launch strings | One app-role source, shared command entries, explicit profile launch/focus |
| Bar | Per-monitor workspaces/stats/tray/health | Offline production assets, real monitor identity, controls and accessible popup |
| Themes | Palette parsing and Terminal scheme rendering; two fixture themes | Apply/rollback, core surfaces, dark/light visual verification |
| Lifecycle | User-level supervisor; elevated Kanata task; recovery/status/reload; taskbar hover controller | Durable restoration, symmetric pair failure contract, correct readiness and request ownership |
| Distribution | Dogfood task-registration scripts | Protected install, owned settings manifest, update/rollback/uninstall |

## Ownership

Kanata emits the modifier; Glaze owns tiling and keybindings. Winmakase owns generated policy, finite action commands, configuration and lifecycle. Zebar displays state and invokes fixed actions. PowerToys owns app search. Explorer remains the Windows shell and recovery desktop.

Use ordinary bounded CLI calls for app actions. Do not put browser selection or general UI dispatch in the supervisor. A small shared command catalog supplies IDs, labels, availability and action arguments to shortcuts, key help and bar controls; it is metadata, not a plugin framework.

The existing Rust CLI and windowless supervisor entry point remain. Install scripts use PowerShell 7. Avoid additional runtimes for users; compiled Zebar assets ship locally with dependencies and licenses.

## Input and Omarchy fidelity

Keep Caps as a pure right-Win modifier, Caps+Space as launcher, physical left Win native and ScrLk as a raw Caps escape. Tap-Caps launching is deferred. Current Apps-key mode needs its own live validation before being advertised.

P0 reassessment, 2026-09-08: the existing direct-Caps Glaze spike passed automated/injected A/B tests but was not adopted. Complete its remaining input/launcher/reload acceptance before retaining Kanata as a v1 dependency. Direct Caps without tap launching is a supported candidate configuration. [Current evidence and migration findings](verification/v1/2026-09-08-p0/direct-caps.md) distinguish it from the deployed right-Win path. Lifecycle removal follows successful input acceptance; existing Kanata-dependent contracts still describe the current stack until then.

A versioned correspondence table distinguishes exact, same-pattern Windows equivalent, unavailable and deferred bindings. Preserve workspace/focus/move/resize/float/fullscreen/launcher families; do not substitute an unrelated action merely to fill a chord. Existing deviations, including the fullscreen focus escape, need explicit review rather than silently becoming the target.

Omarchy's generic browser/terminal bindings invoke launch commands; selected app shortcuts explicitly focus-or-launch. Preserve that distinction. A launch command does not universally guarantee a new window: Winmakase launch-new requires role-specific support/arguments and reports unsupported singleton apps. SUPER+G grouping can remain unavailable in v1. Native reserved chords get an honest explanation, not an unreliable remap.

Float/tile already uses current state: tiled becomes centered floating; floating/fullscreen/minimized returns to tiled. Fullscreen's separate toggle retains its own restore semantics.

Manual floats remain on the current workspace. Known dialogs belong with their parent, not in scratch. Current O is one-way centered topmost, not cross-workspace pinning. Current S summons/rescues one window; named pads provide a dedicated one-window toggle. A true pinned tool or multi-window special workspace needs a future Glaze core primitive. The [interaction guide](research/desktop-interactions-2026-09-08.md) separates current behavior from these targets.

Glaze supports nested splits today. J reflows existing siblings; split-next direction is a distinct operation. P1 adds discoverable split controls, restores loose-float recovery, exposes a named terminal toggle and implements one idempotent daily layout using logical roles. It does not become a general layout engine. After a week with app actions and float recovery, W5 decides whether another switcher is needed.

## Window policy and application roles — target

Main application windows tile by default. Owned dialogs, modal surfaces and non-resizable utilities float by default; explicit utility rules can float or ignore. Process name alone cannot distinguish a main window from its dialogs. Keep splash windows, launchers, Zebar, shell surfaces and elevation boundaries out of main-window matching.

Separate three decisions: classification, main-window home, and requested initial state. Home rules must not override dialog classification or move a dialog away from its parent. Introduce preserve/auto state rather than generating unconditional set-tiling. All matching Glaze rules may execute, so list ordering alone is not precedence.

Automatic homes apply at initial management. Routine key/theme reload preserves current workspace, float state, geometry and focus. Deliberate rehome/reconcile is an explicit command with preview; first adoption of an existing session uses a reviewed preview with a preserve-existing default. Controlled restart can capture and restore a bounded in-memory layout snapshot. After an unplanned Glaze death without a valid snapshot, reconstruct from observable state and declared policy, report lost layout information and keep dialogs with parents. V1 does not promise durable recovery of a lost Glaze tree.

Typed app definitions own launch executable/argument arrays, process and optional class/AppUserModelID refinements, main-window eligibility, home and launch mode. Migrate legacy keymap launch overrides with conflicts reported. No machine profile path or identity belongs in the public defaults.

Work/personal Chrome roles use explicit profile arguments and observed window AppUserModelIDs. Those IDs are profile grouping identities, not unique window IDs; discovery is machine-local and may be late. Normal, incognito, popup, profile-picker and installed-app windows require distinct eligibility tests. External URL routing stays with Windows/Chrome defaults.

Focus-or-launch is a bounded command:
- Lock the canonical app/profile identity; subscribe then query fresh state.
- Prefer an eligible focused match, then the current workspace, then the last window selected by this role elsewhere, then a deterministic remaining match.
- Within multiple current-workspace candidates prefer the last role-selected candidate, otherwise a stable ordering.
- If absent, record an in-flight attempt before spawning; observe for a bounded timeout/grace period. A second invocation must not duplicate a slow launch.
- Validate remembered Glaze window ID and definition generation against current state; clear on WM/profile changes. Never trust a persisted HWND alone.
- Reveal a hidden ordinary workspace without rehoming the window; deliberately restore minimized matches. Exclude scratch-owned windows from ordinary selection and use scratch actions to summon them.
- Verify focus before recording selection; re-evaluate once if the target closes. Late startup must not steal focus after the user moves elsewhere. Late identity detection requires a proven native refresh/query path because current Glaze caches AppUserModelID at management.

Launch-new is separate and intentionally ignores existing candidates when the role supports creating another window. Specify keyboard autorepeat suppression. Ordinary user focus history is not tracked or promised. P1 also restores distinct true fullscreen (F) and maximized (Alt+F) behavior, with prior tile/float restoration on fullscreen exit.

## Launcher and controls — target

Retain PowerToys Run. Generate owned Start Menu shortcuts for role actions and finite Winmakase commands. First prove distinct argument-bearing shortcuts are independently indexed. If this fails, compare a bounded PowerToys-native alternative before adding a plugin.

One undocked Zebar popup hosts controls, theme selection and key help. Escape restores the original surviving target only while the popup still owns focus; click-away keeps the clicked window focused; an action that opens another surface keeps its destination focused. Prove these transitions, keyboard navigation, fullscreen behavior, mixed DPI and zero added work-area reserve before building on it. Audio volume/mute and media use the pinned provider APIs; default-output selection and Bluetooth/network/display/power settings can open native Windows surfaces.

Taskbar independence means app search, running-window access, tray/context menus, audio/device settings, notifications, time/calendar and recovery are reachable by keyboard and mouse. Shutdown/restart actions retain confirmation. Persistent bars must never steal focus.

Tray overflow is an undocked dropdown, not inline bar expansion. Users choose and order always-visible icons. One persistent tray provider owner serves both bar and popup; transient views cannot stop/recreate it. Preferences use verified GUID/app identity rather than transient HWNDs, with explicit ambiguity handling. Calendar is a real month grid with date navigation, locale/weekday choices and midnight/wake updates; external calendar launch is optional and event sync is deferred.

Use fresh monitor identity after topology changes; x-coordinate is not a primary-monitor test. Replace per-monitor status process polling with one shared source using existing files/events or one primary producer. No new daemon solely for status. Offline startup, absent optional providers and useful failure labels are mandatory.

## Appearance — target

Ship Tokyo Night dark and Catppuccin Latte light as the first two complete presets. Import stock palette data from a pinned Omarchy revision. A manifest defines parser coverage; palette acceptance does not claim visual support for every upstream theme.

Core surfaces: bar and popup, Glaze borders/geometry, Windows Terminal scheme, per-monitor wallpaper, and optional Windows app/system light-dark mode. PowerToys appearance follows capabilities proved in the pinned build. Preserve Chrome profile colors; machine-wide tint is off. Muxel integration waits for its supported theme interface in its own repository.

Validate and render a new generation before changing live state. Serialize application, journal previous values, atomically replace owned files, apply native settings, reload only affected surfaces and verify. This is staged apply with rollback, not a globally atomic transaction. Report partial restoration failure explicitly.

Shared settings use owned-key edits. Restore only if the current value equals Winmakase's last write; preserve later user edits and report conflicts. Prefer a Terminal scheme fragment plus a small backed-up selection edit. No theme-provided code or arbitrary editor config runs.

One per-install mutation owner serializes normal configuration, theme, update, off and uninstall writes. Update/off/uninstall quiesces or rejects in-flight applies; commits and rollbacks reject obsolete generations. Emergency native restoration never waits behind that lock. This is bounded synchronization in the existing commands, not another resident service.

One composition path produces Glaze configuration from base, roles, keymap and theme inputs under that ownership rule. Separate valid writers must not overwrite each other's sections.

## Supervisor and native recovery — target

Keep adoption, process handles, bounded restart/backoff, logs, reload and display/session-end observation. Correct the state contract before removing workarounds:

| State/event | Required behavior |
|---|---|
| Native / starting | Native taskbar available; capture its original state durably before any takeover |
| Ready | Keyboard/tiler ready, launcher reachable, all expected bars/controls/tray usable; taskbar takeover allowed |
| Kanata or Glaze unavailable | Immediately restore native access independently of survivor cleanup; block pair restart until termination is confirmed |
| Zebar or launcher unavailable | Restore taskbar; keep healthy keyboard/tiler pair; recover failed surface independently |
| Terminal retry failure | Visible degraded state and native recovery; no endless hidden startup |
| Stopping / panic | Stop owned processes/tasks, verify termination, restore baseline; report failures honestly |
| Supervisor hard death | Independent minimal guard restores native access and handles owned input state |
| Guard death | Worker detects loss and restores native access; takeover prohibited until guard is ready |

Use a small user-level native restore host, preferably a mode of the existing executable, as the single logon owner. It launches the worker and owns only restoration and exact process/task cleanup. Task Scheduler restarts the host, not an independently competing worker. A generation/lease and cleanup acknowledgement prevent stale owners touching a replacement generation; retained handles/creation identity establish process ownership. Restoration runs independently of blocking lifecycle calls. Neither actor recursively restarts the other. Simultaneous host/worker death is a separate failure case: provide an independent native panic route and test next-login recovery without claiming guaranteed two-second automatic repair in that case.

Persist baseline and ownership generation before any taskbar mutation, including temporary emergency visibility; adoption must not capture our state as the user's original preference. Normal off/uninstall restores only still-owned values and preserves later user edits. Emergency visibility is a temporary recovery action, distinct from preference restoration. Reconcile Explorer/taskbar recreation. Do not kill Explorer or replace the shell.

Delete edge-hover peek, flyout allowlists and delayed hide once replacement controls and suppression experiments pass. Native autohide alone versus minimal suppression must be measured; Windows provides no permanent-disable promise here. If takeover cannot meet the gate, keep the native taskbar available and the v1 taskbar gate open.

Replace PID-file singleton checks with owned synchronization. Repair control request ownership and per-request completion without building a generic bus. A failed kill remains tracked, not reported stopped.

Dock health must identify actual expected Zebar widgets, bounds and reserved geometry. Keep bounded repair until direct dock reconciliation is proven. Do not restart healthy tiling on routine monitor changes. The fixed 50% commit-pressure gate needs a reproducible removal/replacement experiment, a bounded wait and visible failure; adoption should not wait for permission to allocate a new process.

## Distribution and verification

Protect the entire privileged execution chain, including parent directories and update replacement paths. Supervisor/Glaze stay user-level; elevated Kanata uses its registered task. Keep LLHOOK; no driver migration.

User-writable journal fields cannot authorize arbitrary elevated commands, paths, task names or process termination. Privileged recovery validates against protected installed ownership and the actual task instance.

Install/update/uninstall own a manifest and restoration journal, support local overrides and preserve unrelated user settings. Pin versions, patches, hashes and licenses. No bare git-pull update as the finished distribution contract.

The [verification plan](plans/v1/verification.md) defines R0–R6, evidence formats, destructive-test isolation, measurable proposed gates and the final release decision. Documentation approval closes planning only.
