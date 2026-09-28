# V1 builder contracts

Reader: the Astra integration lead and Sol builders. Design baseline: Winmakase `1f47d497069b69a627f9b8c8598d21b4b55f4aa9`, paired Glaze `6bc83d1a9682e885c507ac489ca0208f20f985e0`.

This is the implementation-contract annex to [DESIGN](../../DESIGN.md), dated 2026-09-16. It specifies targets, not shipped interfaces. [Work orders](builders/README.md) identify which slice implements each contract. Existing P0–P5 requirements and R0–R6 cases remain binding. A builder may choose internal types and algorithms; changing an observable rule below returns to the lead before dependent work proceeds.

## C1. Input and paired component compatibility

Represent the selected input mode explicitly as `kanata`, `direct_caps` or `f13` (added 2026-09-27 by the [input recut](../../research/2026-09-27-input-remote-recut.md), which makes `f13` the desk target and parks `direct_caps`). The existing configuration migrates to `kanata`. The generated keymap, required components, readiness checks, restart group and release manifest must derive from that same selection. Never infer the mode from whether a Kanata process happens to be alive.

In Kanata mode the linked failure group is Kanata plus Glaze. In direct-Caps and F13 modes it is Glaze alone, and no Kanata task starts; `[kanata]` configuration is required only in Kanata mode. When Kanata's own config takes Caps (`[keyboard] mode = "caps"`), Kanata never starts, or is adopted, while the registry or PowerToys Keyboard Manager already remaps Caps; an adoptable Kanata already running then is stopped, never left unwatched. F13 mode reports a degraded reason when no Caps→F13 remap is visible, and does not block on it, because a firmware remap cannot be observed. Glaze and its CLI/watcher remain a hash-pinned artifact set. Validate the required native facts/commands against that exact set before applying generated configuration. An unsupported pair fails before replacing working configuration. A new DTO or rule change produces a new pair record and invalidates affected evidence.

Direct Caps remains a candidate until literal physical Caps, ordinary/elevated/fullscreen contexts, repeats, stuck-key recovery, reload and hardware sleep/resume pass. Retaining Kanata is a valid v1 disposition; record that decision and retained-mode evidence instead of marking direct-Caps cases passed. Physical left Win and the independent panic route remain native/reachable. No tap-Caps or new remapper in this work.

## C2. App roles and finite actions

Keep role definitions in the existing user configuration; migrate the existing typed app definitions rather than create a second catalog. Each definition has: stable role ID; display label; executable plus argument array; canonical identity; main-window eligibility; optional home/state; default action; optional separate new-window launch arguments. Aliases reference one identity and must agree on matching/launch semantics. Conflicting definitions fail validation with both source locations.

Canonical identity is versioned normalized matching data, not a title, PID or role label. Match process/image and applicable class/AppUserModelID precisely; observed Chrome grouping IDs are machine-local. Eligibility excludes owned dialogs, scratch-owned windows, profile pickers, installed apps and incognito unless a role deliberately includes them. A normal profile role never falls back to any Chrome window. Display identity discovery as a preview before writing local mappings.

Keep existing title matchers only as explicit manage-time exceptions or dedicated scratch markers; do not promote them to canonical main-window identity. Define WM generation from the actual Glaze process creation identity; keep accepted identity/configuration revision alongside it. The current `glazewm.rs` client discards non-response frames and its local Node omits native owner/resizability/PID. W0 must supply subscription-before-snapshot and the missing facts before consumers depend on them.

The finite command catalog projects these roles plus a fixed set of controls into key help, bar entries and owned Start Menu shortcuts. It is not a plugin/action bus. Machine executable paths are resolved at installation/configuration time. All launch boundaries receive executable and argument arrays; labels and theme values cannot become shell text.

Action results use one versioned envelope: request ID, action, outcome, optional role/window/WM-generation IDs, retry disposition, diagnostic code and human message. Outcomes distinguish `focused`, `launched`, `pending`, `completed_without_focus`, `unsupported`, `unavailable`, `conflict` and `failed`. A timeout is `pending` while a live attempt can still resolve; it is never a successful focus claim. GUI callers display useful errors without spawning a console. CLI exit success means the declared operation completed; machine-readable output carries the exact outcome.

### Focus-or-launch

Subscribe before snapshot, acquire one current-user canonical-identity mutex, then requery. Choose: eligible focused match; eligible current-workspace match, preferring the last selection for this role; last eligible selection elsewhere; deterministic remaining match. Stable ordering uses current window identity, never a remembered HWND alone. Hidden-workspace matches are revealed without rehoming; minimized matches are restored. Revalidate target identity/generation before focus. One vanished-target re-evaluation is allowed.

Persist an attempt before spawning. Concurrent aliases observe that attempt rather than spawn. Use a finite configurable startup timeout and finite late-arrival grace; record concrete defaults and their slow-app evidence in the implementation. A dead launching process alone does not prove that a brokered application failed. At grace expiry, requery and require an explicit retry disposition. A fresh query of Glaze's cached AppUserModelID cannot prove late identity; the identity proof must provide a refresh mechanism first.

Capture the user's foreground/workspace context with the action. If it changes while waiting, leave the completed window available without taking focus back. Selection hints are saved only after successful selection and invalidated when WM generation or identity definition changes. This records last selection by this role, not global focus history.

Launch-new bypasses existing-window selection and uses declared new-window capability. It reports completion only after observing a distinct newly eligible window; process spawn is insufficient for a brokered app. Unsupported singleton apps return `unsupported`; no silent focus fallback. Suppress held-key autorepeat at the activation boundary, while distinct deliberate invocations remain distinct. Reuse the identity mutex/attempt primitive for cold terminal-pad launch, without conflating its identity with the ordinary terminal role.

## C3. Window policy and composition

Policy precedence remains: explicitly ignored shell/tool surfaces; owned/modal/fixed-size dialogs; precise utility policy; eligible app-home policy; normal main-window default. An owner relationship takes precedence over a process home. Routine key/theme reload preserves workspace, tile/float/fullscreen state, geometry and focus. Controlled WM restart may use an in-memory snapshot; unplanned loss cannot promise reconstruction of a lost tree.

One pure composition function accepts base configuration, validated roles, keymap and theme inputs and renders candidate Glaze output. Normal writes use C6. Preview deliberate adoption/rehome; initial adoption preserves existing windows by default. Every accepted input survives a later theme/key/role change. Do not let separate writers replace each other's generated sections.

Loose-float queries return current WM generation, window ID, workspace, display label, focus and eligibility. Dialogs and scratch tools are separate categories; unmanaged/elevated windows remain reachable through native switching. Current-workspace forward/reverse cycling follows depth-first tree order with wrap; focus outside that ring selects first/last, and an empty ring is a clean no-op. The bar only shows a count when nonzero.

Keep Caps+J as sibling reflow; Caps+Ctrl+J is the planned insertion-direction toggle, subject to collision and native persistence proof. Label the result side-by-side/stacked. Do not promise a one-shot reset. F means true fullscreen, Alt+F maximized, and T returns either mode directly to tiling.

The daily layout is one optional main-left/right-stacked recipe with configurable main/side/lower roles. Ordinary dynamic tiling remains the default interaction. Ship the recipe shape with unassigned roles, not Chris's application/profile choices; opting into it prompts a local assignment/preview and it never auto-runs at login or installation. Work browser/editor/terminal is an example for a personal setup, not a public default. Preview exact chosen windows, missing roles and affected workspaces. Applying it does not select, move or change the explicit state of unrelated windows; ordinary tiling may resize neighbors, which the preview must disclose. Reuse C2 selection/launch and stop on user-context change. Revalidate the preview before each mutation; do not keep operating against a changed WM generation. Repeat execution is idempotent; no saved HWNDs/pixels or general layout language.

Agent admission remains proof-gated: validated agent roots plus observed sequence ancestry, unknown/shared-broker ownership unchanged, no kill authority from classification. Only A1–A3-supported classes may be routed before their first human-tree/focus mutation. A later move is not prevention. Preserve browser visibility/background input where proved; browser-group relocation stays deferred.

Glaze is the sole geometry owner for managed v1 windows. Do not back the optional recipe with PowerToys Workspaces: its current launch also arranges windows. Detect FancyZones placement/hotkey overlap and preview any required setting changes; do not assume profile-specific exclusion from an executable-name setting. Native PowerToys settings/editor links may be exposed through verified pinned routes, without claiming automatic layout interop. Later import/Command Palette/Dock experiments are documented in [the dated research](../../research/layouts-powertoys-2026-09-16.md).

## C4. Visible shell and provider ownership

One docked bar per configured current monitor; exactly one primary-bar tray owner; one undocked popup host. All assets needed at runtime are local production files with pinned dependencies and notices. Runtime Babel, remote fonts/scripts/icons and per-monitor status subprocess loops are removed. Optional weather/provider failure cannot block readiness or blank required controls.

The primary-bar owner publishes versioned snapshots with owner generation and increasing sequence. Popup requests contain that generation plus a current item/action ID. Reject stale generations, missing items and unknown actions. No command strings cross this boundary. View closure unsubscribes its listeners and releases its own images; it never destroys the long-lived tray provider. Serialize image content across views, not another view's object URL.

Primary-monitor handover must retire the old owner before the new owner activates providers; if clean disposal cannot be proved, use the bounded Zebar restart path. P0 must prove the pinned transport and provider-disposal behavior. This is a condition for implementation, not a claim that the present API supplies an ownership primitive.

Popup opening captures origin identity and activation token. Escape restores a surviving origin only if the popup still owns foreground. Click-away retains the clicked destination. A dispatched action/native tray menu owns the subsequent focus; do not restore over it. The bar never takes foreground on refresh. Popup bounds use the invoking monitor's fresh work area/DPI and add zero reserve. Monitor removal closes/reanchors without using stale geometry.

Tray preferences store ordered stable identities: verified GUID first, then validated image identity plus icon UID when unambiguous. Duplicate/late identities offer an explicit app-wide or session-only choice; do not silently equate tooltip, image hash or HWND with persistence. Missing icons retain saved preference but render no dead control. New icons go to overflow. Native menu keyboard anchoring must be proved, not approximated by warping the pointer.

Calendar is a local civil-date month grid: locale, first weekday, zone and 12/24-hour preference; Today/month navigation; keyboard selection; midnight/wake/zone refresh. No event sync. Native settings/notifications/quick settings and confirmed power actions satisfy the existing replacement matrix. Every route uses a finite action and remains discoverable in help.

## C5. Session owner, requests and readiness

Use an OS-owned named mutex scoped to user SID and interactive session for the single restore host. A different `WINMAKASE_HOME` must not permit a second real stack in the same session. The host launches one worker generation; Task Scheduler restarts only the host. Worker loss causes host restoration; host loss causes worker restoration and prohibits further takeover. Neither recursively restarts the other.

Keep rare control requests as files. Each request has a collision-resistant ID, schema, expected owner generation, finite verb, creation/deadline and bounded arguments. Publish atomically into a requests directory, claim by atomic rename, and write a result under the same ID. Queue bounds, expiry and result retention are explicit configuration constants. Timeout only withdraws an unclaimed request with the same ID. Claimed work completes or reports owner loss; a client timeout does not authorize blind retry. Never delete another caller's request. Stale-generation requests receive an explicit rejection. Stop takes precedence over queued reload; each displaced request gets its own disposition.

The host/worker lease uses monotonic time and retained process creation identity/handles. Health snapshots include schema, owner and WM generation, selected input mode, lifecycle state, per-component observed status, takeover permission, cleanup failures and degraded reasons. Transition timestamps are not a heartbeat: `health.rs` currently writes only on state change. Publish responsiveness separately; stale/unsupported status displays unknown/degraded, never green.

| State/event | Transition contract |
|---|---|
| Native → starting | Native access visible; establish host and durable baseline before any suppression; validate pair/config |
| Starting → ready | Required input/tiler responsive, Run route reachable, actual expected widgets/docks/tray and recovery routes pass bounded probes |
| Any readiness loss | Revoke takeover and restore native access independently of cleanup; report reason |
| Bar/Run failure | Keep healthy input/tiler; recover only the failed surface |
| Required input/tiler failure | Restore first; clean linked survivors; no restart until tracked termination confirmed |
| Failed stop/kill | Retain live ownership and cleanup failure; never label stopped |
| Stop/panic | Revoke takeover, restore, terminate owned components, then report complete or partial failure |
| Host/worker hang | Lease expires independently of blocked lifecycle calls; restore and prevent a replacement generation until old cleanup resolves |

Start with the existing two-second single-actor-death target, measuring fault detection and restoration separately. Lease values are selected from P0 measurements, not invented to fit the target. Simultaneous host/worker death uses independent panic/next-login recovery and carries no two-second automatic guarantee.

Taskbar operations validate actual Explorer image/session/creation identity and current HWND ownership, excluding Zebar's synthetic `Shell_TrayWnd`. `TaskbarCreated` triggers debounced observation, not unconditional restart. Emergency restore never emits that broadcast. Preserve bounded dock repair until exact widget/bounds reconciliation is proved. Retire hover behavior only after the replacement matrix and restoration pass.

## C6. Shared mutations and restoration journal

One per-install OS-owned mutation mutex serializes configuration, theme, tray preferences, update, off and uninstall. The active session lease is independent. Normal operations acquire mutation ownership with a bounded wait and report busy rather than deadlock. Off/update/uninstall quiesce the current apply or reject it. Emergency restoration bypasses this mutex, revokes takeover through the independent session lease and cannot wait for settings writes. Recheck lease permission before every takeover mutation.

A pre-write lease check is insufficient: an actor can pause after checking, resume after restoration and issue a stale hide. Revocation must also retire the old writer's authority and keep independent native-state verification/restoration active until its exact process/task cleanup is confirmed. Never grant a replacement generation while that cleanup is unresolved. L1 must prove its bounded enforcement against delayed stale writes; cooperative checks alone do not satisfy the contract. Failure to terminate/reconcile stays visible as degraded recovery, not completed restoration.

The journal is schema-versioned data. Header: install ID, mutation ID, base generation, candidate generation, operation, owner creation identity and transaction status. Ordered entries: allowlisted target kind/ID, prior presence/value or backup hash, intended value/hash, last owned value/hash, intent/applied/verified/restore/conflict status and error. Use native types for registry/settings values. Privileged ownership is validated separately under C7; journal strings are never commands or authority.

Protocol: acquire owner; recover any incomplete prior transaction; verify active generation; validate and stage all inputs; durably flush a complete intent before the first external write; apply one entry; durably record its result; verify affected surfaces; publish the new active generation only after verification. Atomic replacement is per file, not a global transaction. Define and test Windows flush/rename failure handling. Reject commits and rollbacks from obsolete generations.

On restart, an intent-only entry whose live value equals the intended value is treated as possibly applied and reconciled; one matching its prior value is unapplied; any other value is a conflict. Restore applied entries in reverse order only if the live value still equals our last/intended write. Preserve later user edits and unrelated keys, including absent-versus-empty distinctions. Keep original takeover baseline separate from each operation's immediate rollback value; adoption must not recapture our modified state as the user's baseline.

File targets owned entirely by Winmakase may use immutable generations plus hashes/backups. Shared files use owned-key edits, including Terminal scheme selection. Task ownership includes actual definition/instance identity. Partial rollback produces a degraded report and retains recovery records. Do not garbage-collect the last working generation or backups still referenced by unfinished restoration.

## C7. Install and release boundary

Use a protected install root beneath Program Files for versioned release payloads, privileged configuration, fixed panic implementation and privileged ownership manifest. Administrators/System own replacement rights; ordinary-user replacement of every executable/config/parent/update staging path must fail. Per-user roles, overrides, preferences, logs and user-level generated state remain under the user's Winmakase home. No elevated code loads executable instructions from that home or a repo checkout. Keep the supervisor, Glaze and bar user-level.

Retained Kanata configuration is generated from the allowed keyboard grammar into the protected generation during an authorized installer/update operation. Never pass arbitrary user-configured command directives to an elevated Kanata build. Elevated tasks point to absolute protected executables/configuration with validated arguments; no PATH-resolved `pwsh`, arbitrary task names or user-journal-selected kill targets. Privileged cleanup checks the protected manifest and actual owned task instance. Validate containment/reparse behavior and access rights through the operation, not just a string prefix check.

Release manifest fields: release/schema ID; Winmakase commit/version; selected input mode; Glaze official base, ordered local patch revisions, artifact hashes/build flags and required pair capabilities; all runtime dependency versions/source URLs/hashes; bundled asset hashes and notices; configuration schema/migration range; supported Windows/display matrix; evidence references; previous compatible generation. A self-supplied hash does not authenticate a downloaded manifest; record the trusted distribution source and verification procedure.

Install previews existing apps/settings/tasks and defaults to preserving the running session. Stage and validate, record ownership, then activate through C6/C5. Updates retain local overrides and last working generation. Failed readiness restores the previous compatible generation or native access with a precise error. Same-version repair is idempotent. Uninstall removes only proved owned files/tasks/shortcuts and restores still-owned settings; foreign same-name tasks and later edits survive. Offline runtime is required; an online bootstrap is separately labeled.

## C8. Appearance and release acceptance

The two supported presets remain Tokyo Night dark and Catppuccin Latte light. Render bar/popup, Glaze borders/geometry, Terminal scheme and per-monitor wallpaper; optional Windows mode and pinned Run behavior are documented by observed capability. Preserve browser profile colors; no arbitrary theme code/editor configuration or Muxel integration.

Windows app/system light-dark changes default off and are explicit target opt-ins. The existing `catppuccin` fixture is dark, not Latte; retain its parser regression and add the actual light preset. Reconcile old v4.0.1 source comments against the accepted pinned Omarchy v4.0.2 manifest before claiming complete preset coverage.

Theme list/set use the same finite action from CLI and popup. Static palette/wallpaper previews mutate nothing; no trial-apply/cancel mode. Each declared target can be opted out. The UI reports the applied preset only after verification, and retains partial-failure/conflict details.

Palette/rendering stays in `winmakase-theme`; application uses C6 and C3 without restarting the input/tiler solely for colors. Record unsupported or restart-required targets truthfully. Ten alternating applies, concurrent/interrupted mutation tests, actual screenshot review at supported scales and later-edit preservation are required for R3.

Daily dogfood is a limited deployment milestone with an explicit native fallback, not R6. A named dogfood prerelease tag may be made only after its own cut evidence and Chris's deployment/publication decision. Public v1/release-candidate distribution claims require the full P0–P5/R0–R6 contract, five working days and the separate seven-elapsed-day W5 verdict. A tag never turns an untested requirement into a pass.
