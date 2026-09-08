# P1 — Window policy and app actions

Status: not started. Depends on P0 classification/profile evidence. Exit: R1.

## Problem

Generated app homes currently include an explicit state, defaulting to tiling. That can override Glaze's initial floating decision. Base exceptions appearing first do not make them authoritative when subsequent matching rules also execute. Glaze replays manage rules on reload, so configuration changes can rehome/retile windows the user arranged.

Existing typed launch definitions also coexist with direct keymap launch strings. Profile-aware homes do not yet provide profile-aware focus-or-launch.

## Changes

### 1. Classification and safe homes

- Add preserve/auto initial state and separate main-window home identity from dialog/utility classification. Define precedence explicitly: ignored shell/tool surfaces; owned/modal/non-resizable dialogs; explicit utility policy; main-window app policy; normal tile default.
- Keep dialog ownership and placement with the parent. A dialog must not jump to an app's home workspace while its parent is elsewhere.
- Use actual owner/style/class facts plus precise app exceptions. Avoid blanket process-wide float rules.
- Inventory Glaze DTO/rule support. Add only the bounded facts or lifecycle change necessary in the chosen patched base; include its regression coverage and provenance.
- Separate initial management from explicit rehome/reconcile. Preserve the live tree on key/theme reload. For controlled restart, capture a bounded in-memory snapshot and restore surviving eligible windows. For unplanned death without a valid snapshot, reconstruct from observable state and declared policy, preserve parent/dialog relationships, and report lost layout information. Durable recovery of a lost Glaze tree is out of v1 scope. Provide a preview for deliberate rehome and first-run adoption.
- Migrate existing config with clear diagnostics and local overrides preserved. Test source conflicts rather than relying on rule order.
- Keep one Glaze composition path for base, app roles, keymap and theme under the shared mutation owner. Concurrent role/keymap/theme edits must preserve each accepted input, not replace each other's generated sections.

### 2. One app-role source

Typed definitions own executable plus argument array, canonical identity, eligibility, optional home/state and default action mode. Define work browser, personal browser, terminal, editor and chosen utilities as configurable roles, without installing apps.

Discover Chrome profile IDs locally from observed window properties. Show a preview of role/profile identity before applying the mapping. No title markers, machine-specific public defaults, preference-file hacks or profile-wide color policy. The role definition must specify how incognito/picker/popups/installed apps are excluded or separately addressed.

Migrate keymap launch overrides to role references; reject contradictory duplicate definitions. Preserve explicitly configured launch-new chords. Missing apps/profiles produce actionable errors; no wrong-profile fallback.

### 3. Bounded launch behavior

Implement focus-or-launch by canonical identity:
1. Acquire a per-user identity mutex; requery after lock. Subscribe before the initial snapshot.
2. Choose eligible focused match; otherwise current-workspace match (last role-selected if eligible, stable order otherwise); otherwise last role-selected elsewhere; otherwise deterministic eligible match.
3. Reveal a hidden ordinary workspace without rehoming its window; restore minimized matches deliberately. Exclude scratch-owned windows from ordinary selection; summon them through scratch actions. Focus by current Glaze ID and verify. If it closes, re-evaluate once. Record selection only on success.
4. If absent, persist an in-flight attempt before spawn, then observe/requery for a bounded configurable startup timeout. Other invocations wait/observe the same attempt.
5. A timeout or dead launcher command leaves a bounded grace record so the next keypress does not immediately duplicate a slow opening. At expiry, requery and report/retry explicitly.
6. First prove an identity refresh/query mechanism: current Glaze caches AppUserModelID at management, and repeating that cached IPC query cannot discover a later value. Add a bounded refresh/query path in the chosen base if required, then re-evaluate late identity. Never focus an unrelated intermediate popup.
7. Clear stale selection on WM generation or identity-definition changes. Stored IDs are hints validated against fresh state.
8. If the user changes focus/workspace during startup, report completion without taking focus back.

This is last selected by this role, not global MRU. No resident focus-history tracker.

Launch-new skips selection and opens another requested profile/app window using role-specific arguments/capability. Singleton apps without that capability report new-window unsupported. Suppress keyboard autorepeat for the held activation; distinct intentional invocations remain distinct. Missing spawn/window/focus permission must return distinct outcomes.

### 4. Fullscreen fidelity

Make SUPER+F true fullscreen and SUPER+Alt+F maximized. The current bare Glaze toggle uses the configured default, which is maximized in the dogfood base, so the two chords currently converge. Use explicit supported state arguments and verify against pinned Glaze. Preserve previous tile/float state on F exit; T always returns either fullscreen mode directly to tile. Keep unsupported tiled-fullscreen semantics recorded as a gap.

### 5. Float recovery, splits and one daily layout

Restore W3: list managed loose floats for current/all workspaces, showing app/title/workspace/focus; focus current-workspace floats forward/reverse without borrowing Omarchy group chords. Dialogs stay attached to parents; scratch tools are a separate category. No always-on-top default for manual floats. P2 shows a count only when nonzero. Every eligible float must be recoverable even when covered by tiles.

Expose Glaze's split-next direction through catalog/help and a deliberate binding decision at the interaction checkpoint. Label choices by result (side by side / stacked), not ambiguous divider orientation. Keep J for reflow of existing siblings. Verify a main-left/right-stacked layout from both new insertion and existing windows; document the current J + directional move route and reflow's nested-sibling refusal.

Restore W4 as one bounded v1 daily composition after role identity is stable: logical main/side/lower roles, machine-local assignments, a preview and idempotent cold/warm invocation. Use targeted Glaze tree commands and bounded waits; no saved HWNDs/pixels or general layout engine. Never select, rehome or change the explicit state of unrelated windows. Ordinary tiling may resize neighbors on source/target workspaces; show those affected workspaces in the preview, rather than promising unchanged geometry. Verify twice-run no duplicates, slow/missing role, occupied targets and shared mutation/user-context rules. The original roadmap deferred W4; this reconciliation deliberately includes one preset, not arbitrary layout authoring.

Make the existing named terminal pad discoverable as one summon/hide toggle with a dedicated identity. It must never capture the normal terminal. Centered scratch is the default; classic top-edge Windows Terminal quake is an optional integration proof, not an alias for the current pad. Keep anonymous S behavior honestly labeled as summon/rescue until a real special-workspace primitive exists. Cross-workspace pinning and multi-window scratch overlay remain explicit future core work.

Reuse P1's per-identity serialization and in-flight launch handling for the named toggle. Concurrent/held cold activation creates one terminal; late startup must not take focus after the user changes context. Verify hide/summon repeatedly, closed-target recovery and ordinary-terminal exclusion. The existing scratch command does not yet prove these launch-race guarantees.

## Verification cases

R1 covers main/dialog windows, modal Open/Save/auth/settings, floating utilities, utility child windows, scratchpad and fullscreen history. Use at least one traditional Win32 and one modern application, plus ordinary disposable Muxel windows only if relevant.

Chrome matrix: work/personal closed/open, two same-profile windows on different workspaces, same profile on current workspace, multiple profiles open, incognito, picker, installed app, delayed identity, unavailable profile, unrelated browser launch during wait, target closes during focus, CLI killed mid-launch, two simultaneous aliases for one canonical identity and user context change during startup.

Assert window ID, profile identity, workspace, state and focused window. Repeat focus-or-launch twenty times without creating an extra window; race two invocations and hold the shortcut. Launch-new must still create a new window.

Reload keymap/theme with manually moved and floated windows; then test controlled restart with snapshot and unplanned restart without one separately. Verify each stated preservation/reconstruction contract, with no invented recovered tree. Explicit rehome moves only previewed eligible main windows. Cover hidden/minimized/scratch selection and the tiled/floating → true-fullscreen/maximized → restore/T transition matrix. R1 fails for a misplaced dialog or silent wrong-profile focus.
