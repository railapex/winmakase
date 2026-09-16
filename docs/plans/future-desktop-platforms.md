# Future desktop platforms and Omarchy gaps

Status: platform implementations parked until coherent v1 passes. The W5 discovery decision runs during v1 dogfood after at least one week of app/float use. Updated 2026-09-16 with [layout/PowerToys/Whim source research](../research/layouts-powertoys-2026-09-16.md). Chris confirmed detailed plans through v1, later roadmap only.

V1 commits to GlazeWM, Zebar, Kanata and PowerToys. The user explicitly wants later versions to keep moving toward Omarchy patterns and capabilities. This file preserves future routes without turning them into prerequisites.

## Revisit when a concrete v1 limitation warrants it

| Route | Question to answer before implementation |
|---|---|
| Bounded Glaze improvements | Can an upstreamable/local patch add the needed primitive while retaining the current configuration and recovery model? |
| Native grouping/tabs | What WM-owned state, focus and visibility semantics are required for Omarchy's grouping family? No overlapping-floating-window substitute |
| Cross-workspace pinning | Add true workspace-independent visibility for intentional PiP/tools. Current O is one-way centered topmost on the current workspace, not a pin toggle |
| Special scratch workspace | Add a WM-owned multi-window show/hide overlay over the current monitor. Current S retrieves one parked window; do not extend HWND shuffling into an overlay engine |
| Window discovery/switcher (old W5) | During v1 dogfood, after at least one week using app actions and W3 float recovery, record whether finding current-workspace windows still fails. If it does, design only the missing interaction; otherwise close without code |
| Whim | Does a measured prototype improve a named limitation, and what configuration, API, licensing, maintenance and migration work does it add? |
| Quickshell on Windows | Is a maintained port available, or what platform work would monitors, shell/window access, tray, audio, notifications and packaging require? |
| PowerToys Command Palette extension | Expose the existing finite Winmakase actions through a separately distributed extension; prove focus, action outcomes and failure behavior before replacing the verified Run route |
| PowerToys Dock bands / optional shell replacement | Dock now exists with custom bands. Can workspace/status/action bands work first, and later can it pass the full tray/popup/focus/dock/recovery matrix before any Zebar replacement? Do not enable two default bars |
| Workspaces / FancyZones interop | Prefer supported app-set export/launch-only interfaces or explicitly Glaze-ignored tools. Current Workspaces also arranges windows; no concurrent geometry ownership. Private-schema import or session transfer needs its own bounded proof |
| PowerToys contribution | Discuss a narrow reusable capability with maintainers before large implementation. A first-party module is not automatic adoption of the whole Winmakase distribution |
| Input improvements | Can tap-Caps and missing chord families be added without hook conflicts or regressions to physical left Win? |
| App catalog / webapps | Is repeatable app installation/profile-aware webapp management now needed beyond configurable roles? |
| More theme targets | Is there a supported interface and safe owned-settings model for each editor/application? |

## Current evidence and limits

Whim was rechecked on 2026-09-16. Latest release remains v0.8.6-alpha+37e06e85 dated 2025-11-29; main remains a 2025-12-14 dependency update (`1e86b579`). The repository is not archived. Public product development appears inactive; abandonment is unproved. Its C# dynamic-WM/plugin model is relevant, but migration from the current paired Glaze stack still needs measured benefit. [Whim repository](https://github.com/dalyIsaac/Whim), [releases](https://github.com/dalyIsaac/Whim/releases), [observed main](https://github.com/dalyIsaac/Whim/commit/1e86b579206373a8e9939a5343b67696419ed284).

PowerToys now documents FancyZones layout CLI operations and Command Palette Dock extension bands. Workspaces' internal list/launch service is not a supported external API and still runs its arranger. A supported CLI is requested in #49900; launch-only is not an available mode. The published automatic-tiling request is not a roadmap commitment. These distinctions and source pins live in the [dated research](../research/layouts-powertoys-2026-09-16.md); no platform implementation was added to v1.

Quickshell's official overview describes QtQuick shell construction and Wayland/X11 integration. This research did not establish a maintained Windows port. That is not proof that a port is impossible; a platform feasibility study would be separate work. Similarly named Windows projects are not evidence of a port. [Quickshell overview](https://quickshell.org/about/)

The archived [Glazemakaze core plan](archive/2026-09-08/glazemakaze-core.md) and [Caps ownership plan](archive/2026-09-08/glaze-caps-ownership.md) retain earlier analysis. Their proposed sequence and estimates are superseded. Re-derive any implementation from the current chosen stack and a concrete unmet workflow.

## Decision gate

A future platform proposal needs a reproducible limitation, comparable workflow/failure measurements, licensing/provenance review, migration/rollback plan and a bounded proof. Do not replace a working v1 component because another project's screenshots look closer to Omarchy.
