# Winmakase v1 assessment

Date: 2026-09-08. Status: proposal for discussion, not an approved implementation plan.
Reader: Chris, deciding the product boundary and next work.
Baseline: Winmakase commit `2bd3f6b`, running dogfood configuration, Omarchy `v4.0.2` source, and the current official manual. Intended first audience: Chris plus other Windows developers who should not need a maintainer to configure their machine.

## Recommendation

Winmakase v1 should deliver a complete daily desktop: predictable window controls, a coordinated appearance, discoverable commands, and reversible installation. Keep GlazeWM, Zebar and the working Caps modifier. Their integration is the product; another window manager or desktop shell is not required to make the first release useful.

Finish floating/fullscreen, a small scratchpad, useful split layouts and application switching. Ship a real theme application path. Defer cross-application groups, scrolling layout and exact Hyprland dwindle behavior unless dogfood demonstrates a daily task that cannot work without them.

## What the Omarchy comparison actually implies

DHH's original rationale was to distribute the developer setup he used, including the many defaults a bare window manager leaves for users to assemble. Appearance and an immediately usable environment were part of that promise from the beginning. The relevant lesson is that default choices must work together and be worth keeping. [DHH: Omarchy is out](https://world.hey.com/dhh/omarchy-is-out-4666dd31).

Quattro consolidates its bar, menus, launcher, notifications and other desktop surfaces into Quickshell. It adds a unified app/command menu, theme/background previews and broader shared styling. Winmakase can pursue consistent behavior and appearance while retaining Windows services and its existing components. A Quickshell-sized rewrite is not a v1 prerequisite. [Omarchy 4 release](https://github.com/omacom/omarchy/releases/tag/v4.0.0).

Current bar controls open interactive panels, including calendar and audio. Our TODO's old description of a clock without calendar and floating-TUI drill-ins is no longer a valid Quattro comparison. [Current top-bar manual](https://omarchy.org/manual/the-top-bar/).

This assessment uses DHH's published product choices and shipped defaults. It does not infer his unobserved personal use of every shortcut.

## Verified inventory

| Area | Running or implemented | Remaining product gap |
|---|---|---|
| Tiling and keyboard | Caps maps to right Win; Glaze handles tiling, focus, move, resize and seven configured workspaces. Physical left Win stays native. Today's T fix exits non-tiled states directly to tiling. | State transitions, fullscreen modes and escape paths need a permanent physical-key acceptance suite. |
| Recovery | Windowless supervisor; linked keyboard/tiler recovery; independent bar restart; adoption, reload, rotating logs, monitor/dock checks, taskbar peek and restoration. | Recovery has been observed, not crash elimination. Other-machine install and restore remain unfinished. |
| Bar | Zebar pack with workspaces, clock, stats, network/weather, tray and health dot. | Hardcoded paths and monitor heuristic, CDN runtime dependencies, no audio/media controls or useful clock/stats actions. |
| Appearance | Fixed gaps, border colors and starter CSS. | No coordinated theme switch; titlebar hiding currently disabled. This is some styling, not the promised ricing system. |
| Theme engine | Palette parser, light/dark resolver and Windows Terminal scheme renderer. | Theme crate is not a dependency of the CLI. No theme or background command, no wired bar/border/wallpaper application path. Two theme fixtures, not the claimed broad compatibility. |
| Scratchpad | Named-window toggle plus anonymous summon/rescue and banish. Terminal definition is configured locally. | S is not a symmetric hide/show toggle for Omarchy's special workspace. |
| App policy | Typed definitions and generated manage rules, including profile-aware identity matching. | No typed app homes configured on the rig; focus-or-launch absent. Current launch keys run commands. |
| Groups/layouts | Flat-row horizontal/vertical reflow works. | Nested multi-window reflow is refused. G displays a missing-feature notice. No cross-app grouping, scrolling or exact dwindle layout. |
| Distribution | Task registration/unregistration; Windows CI and tests. | No complete installer, update flow, restoration manifest, uninstall, doctor or key-reference command. |

Implementation evidence: [CLI](file:///D:/dev/winmakase/crates/winmakase/src/main.rs#L38), [supervisor](file:///D:/dev/winmakase/crates/winmakase/src/supervisor.rs#L278), [bar](file:///D:/dev/winmakase/zebar/bar.html#L10), [theme render module](file:///D:/dev/winmakase/crates/winmakase-theme/src/render/mod.rs#L1), [reflow](file:///D:/dev/winmakase/crates/winmakase/src/reflow.rs#L77), [scratchpad](file:///D:/dev/winmakase/crates/winmakase/src/scratchpad.rs#L1), [task registration](file:///D:/dev/winmakase/installer/register-tasks.ps1#L1).

The README and TODO mix implemented work, proposals and stale statements. In particular, README advertises coordinated themes, tap-Caps launcher and manifest uninstall before those exist. The inventory should be corrected when the v1 scope is adopted; milestone checkbox totals are not a completion measure.

## Keybinding compatibility

The live config has 73 bound chords: 72 dispatch actions, one opens the grouping notice. These are not 72 independently verified features. The stock catalog records 226 chords; local workspace limits remove nine and local additions add six, producing a 223-chord catalog. Many catalog entries marked native refer to different Windows shortcuts, not identical Omarchy bindings.

| Action | Current Winmakase | Compatibility assessment |
|---|---|---|
| Focus/move, workspace numbers and Tab variants | Main chord families present; workspaces 1–7 locally. | Useful core grammar coverage. Tree movement semantics are not necessarily identical. |
| Caps+T | Tiled → floating; non-tiled → tiled. | Required behavior now implemented. |
| Caps+F / Caps+Alt+F | Both resolve to maximized fullscreen under the current Glaze defaults. | True fullscreen versus full-width distinction is currently collapsed. Ctrl+F mode is absent. |
| Caps+J | Reorient a flat split in place. | Useful subset; not arbitrary dwindle restructuring. |
| Caps+S / Caps+Alt+S | Retrieve/rescue one window / banish focused window. | Not Omarchy's scratch-workspace show/hide behavior. |
| Caps+O | Centered floating, shown on top. | Does not establish Omarchy's sticky-across-workspaces semantics. |
| Caps+G | Modal unsupported-feature notice. | No grouping. |
| Caps+Space | PowerToys Run. | App search exists; integrated Winmakase command/theme/settings menu does not. |
| Caps+K | Unbound. | Discoverability gap. |
| Caps+Shift+E | Exit GlazeWM. | Conflicts with Omarchy's email binding; recovery should have an explicitly documented home. |
| Native Windows shortcuts | Alt+Tab, left-Win+Tab, clipboard/capture/etc. | Useful deliberate Windows behavior, not exact Omarchy parity. |

Pinned upstream evidence: [tiling](https://github.com/omacom/omarchy/blob/v4.0.2/default/hypr/bindings/tiling.lua), [applications](https://github.com/omacom/omarchy/blob/v4.0.2/default/hypr/bindings/applications.lua), [utilities](https://github.com/omacom/omarchy/blob/v4.0.2/default/hypr/bindings/utilities.lua). Local evidence: [catalog](file:///D:/dev/winmakase/keymap/omarchy.toml#L86), [live bindings](file:///C:/Users/chris/.glzr/glazewm/config.yaml#L73).

Pin the comparison to an upstream tag/commit. The existing moving `quattro` ref plus a fetched date cannot explain exactly when a shortcut diverged. Preserve familiar core gestures; label deliberate Windows differences instead of claiming full parity.

## Feature decisions

**Floating: required.** Dialogs, calculators and temporary controls should coexist with tiles. One obvious action must return a window to tiling, and the launcher and recovery command must remain reachable from every state. Include separate, tested fullscreen/full-width behavior if both chords are exposed.

**Scratchpad: include a small version.** A summon/hide utility window is useful for brief terminal, notes or agent interactions without leaving the current work. Make the repeated-key behavior symmetric and visible. A named terminal plus optional app definition is enough for v1; an entire parallel scratch-workspace system is not required. Omarchy's implementation is a special workspace, so describe this as a deliberate subset. [Navigation](https://omarchy.org/manual/navigation/).

**Groups: defer.** Cross-app groups put several windows in one tile with tabs. They are distinct from the tabs already available inside terminals, browsers and editors. Glaze lacks the container primitive; faking it with overlapping floats would create a second window-management mechanism. Revisit native grouping only if specific daily work is materially blocked. A modal missing-feature notice should not appear as a working feature in the key reference.

**Dwindle: defer exact parity; require usable layouts.** Dwindle repeatedly splits available space as windows arrive. Omarchy also offers a scrolling workspace layout. For v1, require sensible two-to-four-window arrangements, a predictable insertion direction, reliable resizing, and reflow that either handles the current structure or explains its limit clearly. Do not make users diagnose a split tree. [Omarchy layout defaults](https://github.com/omacom/omarchy/blob/v4.0.2/default/hypr/looknfeel.lua).

**Ricing: required.** Start with one excellent default, then a small verified set including a light theme. Apply palette, typography, spacing, wallpaper and supported app styles together. A theme change must update bar, borders, terminal and Windows mode/accent without losing app/profile preferences. VS Code and Neovim are sensible opt-in targets. Broad community-theme compatibility and arbitrary Win32 restyling can wait. Omarchy demonstrates coordinated theme application across supported surfaces; copying its complete theme count is unnecessary. [Themes](https://omarchy.org/manual/themes/).

## Proposed v1 work order

1. **Finish daily control.** State transitions, reliable launcher access, Caps+K generated reference, focus-or-launch with explicit new-window behavior, symmetric scratch toggle, clear indication of float/fullscreen state. Keep app homes optional and profile-aware.
2. **Finish the visual result.** One complete theme path, then the small tested set. Style bar, borders and terminal together; keep useful native app chrome until per-app behavior is verified.
3. **Finish the bar and command surface.** Searchable access to apps, themes, settings, help and recovery. Audio/device control, useful clock/stats actions, and tray access. Reuse Windows panels where they do the job; a new calendar, notification system and plugin ecosystem are outside v1.
4. **Make installation portable and reversible.** Versioned component deployment, local overrides, protected elevated-keyboard config, offline bar assets, update rollback, off/on and uninstall that restore prior state.

Installation has a concrete open defect: elevated Kanata currently reads a repo spike config whose ACL grants Authenticated Users modify access. Deploying a protected config is a release prerequisite already present in the design. Also, PowerToys Run is configured to ignore its hotkey in fullscreen; that setting was observed, not behavior-tested. Both belong in the acceptance work, not in an assumptions list.

## Proposed release gate

- Install from a clean user account without editing repository paths or generating files by hand; repeat installation without damage.
- Follow a short on-screen guide through launch, focus, move, resize, float, fullscreen, scratch, workspace switching and recovery.
- Exercise physical keys through Kanata and Glaze, including history-dependent transition sequences.
- Resume from sleep, reconnect a display and recover a component failure with all user windows reachable.
- Switch dark/light themes and restore the previous one without destroying local overrides or browser-profile identity.
- Work for five consecutive days without manually repairing window state or the desktop stack.
- Turn it off, update with rollback, and uninstall with verified restoration.

The v1 demonstration should show one other developer installing it, opening their own tools, changing the look, arranging the work and recovering to normal Windows. That tests the actual promise: more time building, less time maintaining the desktop.
