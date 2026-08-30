# WinMakase — window policy, bindings and app homes

Status: active design, session-cut implementation
Created: 2026-08-30
Owner boundary: user-facing policy compiled onto GlazeWM primitives

## Outcome

Make windows predictable without teaching WinMakase to be a second window manager:

- Ordinary windows tile.
- Apps may have persistent workspace homes.
- App keys focus the existing window or launch it; they do not produce duplicates by accident.
- Temporary overlays are named scratchpads, not forgotten floats.
- Exact multi-app arrangements are explicit layout launches, not brittle per-app pixel rules.
- Native Windows remains the recovery surface: left-Win shortcuts and Alt+Tab continue to work.

## Boundary

This track owns:

- Generated GlazeWM keybindings and `window_rules`.
- Machine-local app definitions, workspace homes and launch commands.
- `focus-or-launch`, named layouts, float discovery and any later switcher UI.
- The portable WinMakase defaults for those policies.

This track does not own:

- Monitor hardware identity, container grouping/tab semantics, atomic repositioning, cloak cleanup or animation internals. Those belong to `glazemakaze-core.md`.
- A fake tabbed container made from overlapping floats.
- Stardock Groupy integration. Current Glaze reports show lost HWND tracking, broken focus and bar overlap when Groupy switches tabs.

## State at extraction

Already shipped:

- All newly managed windows default to tiling.
- `1/4`, `2/5`, `3/6` workspace pairs are bound to the three daily monitors; teleprompter behavior is stable enough.
- `winmakase reflow` gives Super+J true in-place split reorientation.
- Named scratchpads and anonymous summon/banish are live.
- The keymap generator contains Omarchy's 226-chord grammar.
- Caps emits right-Win; physical left-Win remains native Windows.
- Caps+Tab is next workspace in both Omarchy and generated WinMakase policy.
- Alt+Tab is deliberately native Windows, not intercepted.

Outstanding or inconsistent:

- The live Glaze config is still the hand-written spike config. The generated keymap is not adopted live.
- Therefore generated Caps+Escape → `wm-cycle-focus` exists in source but is absent from the live config. Floats/fullscreen layers can feel lost.
- There is no first-class app schema or generated app-home rule yet.
- `focus-or-launch` is an unchecked M2 item.
- The install/adoption policy for pre-existing, unknown windows is unresolved.
- Exact-role layouts (`dev`, editor+agent+terminal) and workspace overview remain deferred.
- There is no float inventory/indicator. Native Alt+Tab is the only general recovery today.
- Chrome profiles share process/class identity; a process-only rule cannot distinguish Default from `Profile 1` reliably.

## Decisions already made

### Bindings

- Caps+Tab / Caps+Shift+Tab: next/previous workspace.
- Caps+Ctrl+Tab: former workspace.
- Caps+Escape: cycle Glaze focus across tiling, floating and fullscreen layers. This is the no-UI recovery path.
- Alt+Tab: native Windows switcher. Do not shadow it.
- Left-Win+Tab: native Windows Task View.
- Caps+Alt+Tab: reserved for future native groups, matching Omarchy. Do not spend it on a temporary float command.

Omarchy itself uses no graphical custom Alt+Tab switcher: Alt+Tab cycles windows on the active workspace and raises the winner. WinMakase keeps native Alt+Tab because Windows is also the escape hatch when Glaze state is wrong.

### App homes

- A home is a creation rule, not saved pixel geometry: match an app, move the new window to workspace N, set tiling/floating/ignored state.
- Portable WinMakase defaults do not assign common apps to numbered workspaces. Numbering is personal and monitor-dependent.
- Machine-local config may assign homes such as browser → 2. The renderer compiles them into Glaze `window_rules` so placement happens at manage time.
- Match with the strongest available identity: process + class, then title only where stable. Profile-specific Chromium homes need an explicit marker/correlation design; do not pretend `chrome.exe` identifies a profile.

Proposed minimum shape; W1 freezes names after round-trip tests:

```toml
[apps.browser]
launch = 'chrome --profile-directory=Default'
process = 'chrome'
workspace = '2'
state = 'tiling'
```

Optional `class` and `title` refine the match. Multiple-window policy is part of `focus-or-launch`, not the placement rule.

### Slots and layouts

- Do not add `slot = 2` or persisted pixel rectangles to app rules. A tiling leaf is transient: another window, dialog or delayed launch invalidates it.
- The focused split and launch order determine normal insertion, as in Omarchy/Hyprland.
- Repeatable compositions belong to `winmakase layout <name>`: launch/focus named apps, wait for matching HWNDs, move them to a workspace, then arrange via targeted Glaze commands and `reflow`.
- Build the first layout only after app homes and focus-or-launch are stable.

### Floating

- Tile by default. Float only dialogs, pickers, PiP, small tools, intentional pop-outs and unmanaged elevated windows.
- A repeatedly used temporary window is a scratchpad, not a loose float.
- Unknown apps on a mature install tile. A first-run adoption mode may initially float pre-existing windows so installation does not mosaic an active desktop.
- No always-on-top default for loose floats.

### Switcher and grouping

- Do not build a visual switcher yet. First ship Caps+Escape, focus-or-launch and float visibility; measure whether anything remains hard to find.
- If needed later, the switcher belongs in WinMakase as a resident native overlay backed by cached Glaze IPC state. No process spawn and no WebView in the hot path; target first paint within one frame.
- Grouping/tabs require a Glaze container primitive. WinMakase may render bindings and a group bar after that primitive exists, but must not simulate it with floating windows.
- Groupy remains an optional personal experiment only for apps Glaze ignores. That cannot deliver “tabs inside a tiled Glaze slot,” so it is not a product route.

## Session cuts

### W0 — adopt the generated keymap on the dogfood rig

- Render to a staging file and diff against the live spike config.
- Preserve machine-local Chrome launch commands, monitor/workspace order, scratch workspace, Zebar ignore rule and panic bindings.
- Verify Caps+Tab, Caps+Shift+Tab, Caps+Ctrl+Tab and Caps+Escape through IPC state.
- Verify Alt+Tab and physical left-Win+Tab remain native.
- Update the grouping-gap notice to describe the missing core primitive and Glazemakaze route; remove the stale claim that all upstream work is dormant.
- Deploy through the existing render/reload path with backup and rollback. Do not hand-edit the generated copy after adoption.

Exit: source and live config agree; Caps+Escape recovers a deliberately hidden float/fullscreen focus case.

### W1 — app schema and window-rule renderer

- Add parsed app definitions with `launch`, process/class/title match, optional workspace and state.
- Validate unique app names, valid workspaces, at least one match field and supported states.
- Generate deterministic, ordered Glaze rules. `ignore` rules and system/dialog exceptions precede broad process rules.
- Keep numeric homes in machine-local config; ship portable examples, not Chris's monitor map.
- Add golden tests for tiled home, floating dialog, ignored bar, overlapping rules and invalid config.
- Decide Chromium-profile identity from an observed window source; if no stable marker exists, explicitly leave profile homes unsupported.

Exit: a test app opens directly on its configured workspace and state without visible move-after-open.

### W2 — focus-or-launch

- `winmakase focus-or-launch <app>` reads the app definition and current Glaze state over the existing IPC client.
- Selection order: matching window on current workspace, otherwise most-recent matching managed window, otherwise launch.
- When selecting a hidden workspace window, focus its workspace then the window. Do not drag it to the caller's workspace; its home remains meaningful.
- Define multi-instance behavior explicitly. Default cycles matching windows; an app may opt into singleton behavior.
- Measure warm focus and absent-app launch dispatch separately. Warm focus target: under 50ms p95 on the dogfood rig.
- Retarget app bindings to this verb after direct command behavior passes.

Exit: repeated browser/editor bindings focus rather than duplicate, and a missing app launches once into its home.

### W3 — float visibility and recovery

- Add `winmakase windows floats [--workspace current|all]` using Glaze state, including process/title/workspace and focused status.
- Add forward/reverse focus of current-workspace floats without consuming Omarchy's group chords.
- Expose a small Zebar float count only when nonzero. Use cached/event-driven state once the resident relay exists; do not spawn a CLI poll per frame.
- Verify scratchpads are identified separately from loose floats.
- Document elevated unmanaged windows as native Alt+Tab territory.

Exit: every managed float is listable and focusable without searching beneath tiled windows.

### W4 — first named layout

- Implement one real composition, likely `layout dev`, from observed daily use.
- Use app identities and targeted IPC; wait with a bounded deadline for launched windows.
- Arrange logical roles (`main`, `side`, `lower`) rather than persisted pixels.
- Make the command idempotent: running it twice focuses/reconciles the same windows instead of duplicating them.
- Keep layout definitions machine-local until two layouts reveal a portable schema.

Exit: a cold and warm invocation produce the same useful composition without manual cleanup.

### W5 — switcher gate

After at least a week with W0–W3, inspect actual failure cases.

- If Caps+Escape, app keys and the float indicator solve discovery, close this cut without code.
- If not, build a resident native overlay showing only the current Glaze workspace, grouped by tiled/floating/fullscreen state.
- Precompute labels/icons on window events; keypress should reveal existing state, not enumerate the desktop.
- Preserve native Alt+Tab unchanged.

Exit: either a written “not needed” verdict or a measured overlay with one-frame first paint and deterministic focus.

## Existing TODO ownership

- M1: generated-keymap live adoption and Caps+Escape — W0.
- M2: app layer, app homes and `focus-or-launch` — W1/W2.
- M3: safe first-run handling of pre-existing windows — W3 policy plus installer work.
- Deferred: agent layouts and workspace overview — W4/W5, promoted only when their gate fires.
- Grouping/tabs and monitor hardware identity: Glazemakaze plan, not this one.

## Completion condition

This plan is complete when normal apps open in deterministic homes, app keys focus-or-launch, every managed float is recoverable, live bindings come from the renderer, and no switcher or layout work remains open without a measured need.
