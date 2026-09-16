# Layouts and PowerToys interoperability

Reader: Chris and the Winmakase lead deciding product boundaries. Checked 2026-09-16. This is source research, not a desktop interoperability test. Detailed build scope remains v1; broader integrations belong to the later roadmap.

## Three different meanings of layout

| Meaning | Owner and representation | Winmakase decision |
|---|---|---|
| Dynamic tiling | WM maintains a tree/algorithm as windows appear, disappear and move | Glaze owns managed windows throughout v1 |
| Zone layout | FancyZones defines drop targets/rectangles and snaps selected windows into them | Useful for a separately owned window set; not a second geometry writer for Glaze tiles |
| App-set recipe | Roles/apps launch or are selected, then placed deliberately | One optional main-left/right-stacked Winmakase recipe with local role assignments |

The v1 target defaults to ordinary dynamic tiling. The optional recipe will ship with main/side/lower unassigned; a user chooses applications/profiles locally and previews before running. Work browser/editor/terminal is a personal example, not an opinionated app default. No automatic login arrangement or required configuration step.

## Omarchy's actual definition

The pinned Omarchy v4.0.2 source defaults to Hyprland `dwindle`. Its navigation guide teaches launching/moving windows and flipping splits. Super+L switches the active workspace between dwindle and scrolling. This selects a tiling algorithm; it is not a three-app startup recipe. [Default layout](https://github.com/omacom/omarchy/blob/v4.0.2/default/hypr/looknfeel.lua), [navigation](https://github.com/omacom/omarchy/blob/v4.0.2/manual/04-navigation.md).

The toggle writes a per-workspace Lua rule under `~/.local/state/omarchy/workspace-layouts/<workspace-id>.lua`; the default configuration reloads those rules. Users can change the overall default in `~/.config/hypr/looknfeel.lua`. The inspected stock paths define algorithm preferences, not saved application/window snapshots. Third-party layout tools are separate products. [Toggle implementation](https://github.com/omacom/omarchy/blob/v4.0.2/bin/omarchy-hyprland-workspace-layout-toggle), [restore loader](https://github.com/omacom/omarchy/blob/v4.0.2/default/hypr/workspace-layouts.lua), [user override](https://github.com/omacom/omarchy/blob/v4.0.2/config/hypr/looknfeel.lua).

The official default branch is `quattro`; its observed head was `2fbac0c8e88eca704af1650ce721a494bd11a3d0`, dated 2026-09-15. The pinned compatibility baseline remains v4.0.2 (`346e69e1cec6c4e8924531874af6ba010a1bc99e`). The Winmakase recipe is an explicit convenience addition, not claimed Omarchy parity.

## PowerToys Workspaces

Workspaces can launch via its editor or a generated desktop shortcut. Its operation includes moving newly launched windows to saved positions. Turning off Move existing windows does not make it launch-only. Thus calling the shortcut from Winmakase is possible, but it does not establish safe coexistence with active Glaze tiling. [Official Workspaces behavior](https://learn.microsoft.com/en-us/windows/powertoys/workspaces).

Observed PowerToys source head: `8c604f779d98aefb92c63a830f3beb6902cee681`. `WorkspaceService` can internally list/launch IDs and starts WorkspacesLauncher; these are version-specific internals, not a supported external API. The arranger still handles newly launched windows. [Service source](https://github.com/microsoft/PowerToys/blob/8c604f779d98aefb92c63a830f3beb6902cee681/src/modules/Workspaces/Workspaces.ModuleServices/WorkspaceService.cs), [arranger source](https://github.com/microsoft/PowerToys/blob/8c604f779d98aefb92c63a830f3beb6902cee681/src/modules/Workspaces/WorkspacesWindowArranger/WindowArranger.cpp).

A supported list/launch/capture CLI is requested in open [#49900](https://github.com/microsoft/PowerToys/issues/49900), created 2026-08-14. An open feature request is not an available capability or roadmap promise. A supported export or launch-without-arrange option would make app-set reuse much cleaner; that latter option is our proposed upstream discussion, not a current feature.

## FancyZones

FancyZones now documents a CLI for monitor/layout discovery, active-layout selection, editor/settings and layout hotkeys. That makes a future adapter more plausible than the older notes suggest. Its documented command list does not assign a given HWND to a zone. Selecting zones is not equivalent to commanding a Glaze tree. [Official CLI](https://learn.microsoft.com/en-us/windows/powertoys/fancyzones#command-line-reference).

Its configurable automatic movement includes display/work-area changes, active-zone changes and an app's previous zone; Win-arrow overrides also overlap native/WM behavior. Public settings/DSC can describe these choices, but there is no window-manager ownership handshake. [FancyZones settings reference](https://github.com/microsoft/PowerToys/blob/8c604f779d98aefb92c63a830f3beb6902cee681/doc/dsc/modules/FancyZones.md).

Engineering inference: two active systems moving the same window can race. A clean experiment gives FancyZones only explicitly Glaze-ignored apps or transfers the entire session between owners. Merely pausing Glaze, zoning windows and resuming it does not prove that the arrangement survives. Profile-specific coexistence cannot be assumed from process-name exclusions.

## V1 decision and later proofs

V1 keeps one geometry owner for managed windows. Installer preview/doctor reports conflicting FancyZones auto-placement/hotkeys and preserves/restores any owned setting change. No silent module takeover and no private PowerToys JSON parser. Do not call Workspaces launch as the backend of the Winmakase recipe. Optional settings/editor entry points may use verified supported routes in the pinned installed version; they do not claim layout interoperability or become readiness dependencies.

Later, prefer these small proofs over replacing Glaze:

1. Supported Workspaces list/export/launch-only interface, if added; otherwise a separately approved read-only, exact-version app-set importer with preview and no geometry import. Never blindly execute imported command arguments.
2. FancyZones for explicitly unmanaged tools, with actual exclusion/negative-control evidence; or a deliberate alternate-session ownership transfer with rollback.
3. FancyZones Grid-to-Glaze-tree conversion only if the editor benefit warrants a private-schema dependency. Canvas rectangles need not map to a valid nested tiling tree; no general compatibility claim.

Each proof needs a concrete user workflow, pinned versions, one window-placement owner, restore behavior and failed-case evidence. None is a new v1 implementation prerequisite.

## Becoming part of PowerToys

The strongest near-term route is a separately distributed Command Palette extension using Winmakase's finite actions, then optional Dock bands for workspace/status/actions. Microsoft documents custom Dock bands through its extension SDK (`GetDockBands`); the integration need not first be accepted into the PowerToys repository. This is a recommended experiment, not a tested Winmakase extension. [Extension guide](https://learn.microsoft.com/en-us/windows/powertoys/command-palette/adding-dock-support).

Command Palette Dock is already a documented per-monitor edge toolbar with AppBar reservation and extension bands. Its existence changes the old assumption that PowerToys offers only a launcher. It does not prove that Dock satisfies Winmakase's tray, focus, controls or independent recovery matrix; replacing Zebar still needs that comparison. [Dock documentation](https://learn.microsoft.com/en-us/windows/powertoys/command-palette/dock), [ongoing Dock work](https://github.com/microsoft/PowerToys/issues/45584).

For first-party inclusion, the standard route is an issue/proposal, maintainer agreement on scope and design, then an implementation/PR conforming to the module conventions. The current new-PowerToy guide covers C++/C# modules and integration with settings, lifecycle and distribution. There is no automatic repository-adoption route. My assessment: the whole Winmakase stack is a poor first submission; a bounded Workspaces interface, app-action improvement or Dock integration has a smaller review/maintenance burden. Acceptance remains Microsoft's decision. [Contribution rules](https://github.com/microsoft/PowerToys/blob/8c604f779d98aefb92c63a830f3beb6902cee681/CONTRIBUTING.md), [new-module guide](https://github.com/microsoft/PowerToys/blob/8c604f779d98aefb92c63a830f3beb6902cee681/doc/devdocs/development/new-powertoy.md).

I found no committed automatic-tiling utility on the published roadmap. The longstanding automatic-tiling request is still open; a separate tiler remains a proposal, not an announced product. PowerToys also has an open effort to replace Run with Command Palette. Keep the accepted Run path for v1, but target new later extension experiments at Command Palette. [Published roadmap](https://github.com/microsoft/PowerToys#roadmap), [automatic tiling request](https://github.com/microsoft/PowerToys/issues/2694), [Run replacement epic](https://github.com/microsoft/PowerToys/issues/41696).

## Whim's current state

Whim is architecturally relevant: a C# extensible dynamic Windows manager with workspaces, layouts and optional bar/plugins. That overlaps more of Winmakase than zone snapping alone. However, the observed latest release remains `v0.8.6-alpha+37e06e85`, published 2025-11-29. Main is at `1e86b579206373a8e9939a5343b67696419ed284`, dated 2025-12-14, a dependency update. The repository is not archived/disabled. [Project](https://github.com/dalyIsaac/Whim), [release](https://github.com/dalyIsaac/Whim/releases/tag/v0.8.6-alpha%2B37e06e85), [main commit](https://github.com/dalyIsaac/Whim/commit/1e86b579206373a8e9939a5343b67696419ed284).

Conclusion: public product development appears inactive; abandonment is not established. Recent repository push metadata is not evidence of new WM functionality. A Whim migration would exchange a known patched Glaze integration for a broader migration onto a quiet alpha. Keep it as a later bounded comparison against a specific measured limitation, not a v1 dependency or presumed PowerToys successor.
