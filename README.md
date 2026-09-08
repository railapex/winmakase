# Winmakase

An opinionated Windows 11 desktop, following [Omarchy](https://omarchy.org)'s patterns for launching, tiling, controls and appearance.

**Status: working dogfood stack; v1 is planned, not released.** GlazeWM, Zebar, Kanata and PowerToys are the committed v1 components. The [v1 plan](docs/plans/v1/README.md) defines the remaining work and release gates.

## What works today

- Caps Lock acts as right Win through Kanata; GlazeWM handles the window chords. Physical left Win keeps native Windows shortcuts. Caps+Space opens PowerToys Run.
- Generated keybindings, machine-local overrides and typed app-home rules, including a small GlazeWM patch for Chrome profile identity.
- Window actions including current-state float/tile, reflow and scratchpads.
- A supervised stack with scheduled-task hosting, component recovery, status/logging and a per-monitor Zebar pack.
- A palette parser and Windows Terminal scheme renderer. Complete theme application is still planned.

This is not yet a clean-machine installer or a finished desktop. See [current work](docs/TODO.md) and the dated [assessment](docs/research/v1-assessment-2026-09-08.md).

## V1

Reliable dialogs and app homes; explicit work/personal browser roles; predictable launch-new and focus-or-launch actions; searchable commands; useful Zebar controls; two complete appearance presets; taskbar recovery that survives a supervisor crash; install, update and uninstall with tested restoration.

Omarchy is the compatibility target. Unsupported bindings remain recorded gaps to close over time. V1 excludes an application catalog, another window manager, a replacement Windows shell and broad editor integration.

[Design](docs/DESIGN.md) · [Implementation plans](docs/plans/v1/README.md) · [Verification rounds](docs/plans/v1/verification.md) · [Future platforms](docs/plans/future-desktop-platforms.md)

## Credits and license

Built on [Omarchy](https://github.com/omacom/omarchy), [GlazeWM and Zebar](https://github.com/glzr-io), [Kanata](https://github.com/jtroo/kanata) and [PowerToys](https://github.com/microsoft/PowerToys). Inspired also by [Omacosy](https://github.com/paulsp94/omacosy).

MIT; bundled dependencies retain their own licenses.
