# Future desktop platforms and Omarchy gaps

Status: parked until coherent v1 passes. Reviewed 2026-09-08.

V1 commits to GlazeWM, Zebar, Kanata and PowerToys. The user explicitly wants later versions to keep moving toward Omarchy patterns and capabilities. This file preserves future routes without turning them into prerequisites.

## Revisit when a concrete v1 limitation warrants it

| Route | Question to answer before implementation |
|---|---|
| Bounded Glaze improvements | Can an upstreamable/local patch add the needed primitive while retaining the current configuration and recovery model? |
| Native grouping/tabs | What WM-owned state, focus and visibility semantics are required for Omarchy's grouping family? No overlapping-floating-window substitute |
| Whim | Does a measured prototype improve a named limitation, and what configuration, API, licensing, maintenance and migration work does it add? |
| Quickshell on Windows | Is a maintained port available, or what platform work would monitors, shell/window access, tray, audio, notifications and packaging require? |
| PowerToys launcher evolution | Does Command Palette offer a demonstrated gain over the verified Run integration without adding another bar? |
| Input improvements | Can tap-Caps and missing chord families be added without hook conflicts or regressions to physical left Win? |
| App catalog / webapps | Is repeatable app installation/profile-aware webapp management now needed beyond configurable roles? |
| More theme targets | Is there a supported interface and safe owned-settings model for each editor/application? |

## Current evidence and limits

Whim is an existing Windows window-manager project. The release inspected was v0.8.6-alpha+37e06e85 dated 2025-11-29; its stable-latest endpoint did not provide a release. This is an observed release label, not a health or suitability verdict. Re-evaluate actual capabilities and current state at decision time. [Whim repository](https://github.com/dalyIsaac/Whim), [releases](https://github.com/dalyIsaac/Whim/releases)

Quickshell's official overview describes QtQuick shell construction and Wayland/X11 integration. This research did not establish a maintained Windows port. That is not proof that a port is impossible; a platform feasibility study would be separate work. Similarly named Windows projects are not evidence of a port. [Quickshell overview](https://quickshell.org/about/)

The archived [Glazemakaze core plan](archive/2026-09-08/glazemakaze-core.md) and [Caps ownership plan](archive/2026-09-08/glaze-caps-ownership.md) retain earlier analysis. Their proposed sequence and estimates are superseded. Re-derive any implementation from the current chosen stack and a concrete unmet workflow.

## Decision gate

A future platform proposal needs a reproducible limitation, comparable workflow/failure measurements, licensing/provenance review, migration/rollback plan and a bounded proof. Do not replace a working v1 component because another project's screenshots look closer to Omarchy.
