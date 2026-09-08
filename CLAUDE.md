# Winmakase — Agent Instructions

Opinionated Windows 11 desktop. Public repository: railapex/winmakase.

## Start here

1. Read [docs/TODO.md](docs/TODO.md), then the active [v1 plan](docs/plans/v1/README.md).
2. Read [spike/NOTES.md](spike/NOTES.md), especially LIVE MACHINE STATE, before any machine action. A commit is not proof of deployment; recheck relevant live facts.
3. Read [docs/DESIGN.md](docs/DESIGN.md) for the touched area. It distinguishes shipped behavior from accepted targets. Update work state and design in the same commit as implementation.
4. An unchecked verification gate stays unchecked until its evidence exists. Plans, source review and unit tests are not live acceptance.
5. Build sessions follow [execution.md](docs/plans/v1/execution.md). At handoff, update the slice/decision state and regenerate the progress report; TODO remains package/round completion truth. Reconcile every old requirement when restructuring plans.

## Scope and safety

- V1 uses GlazeWM, Zebar, Kanata LLHOOK and PowerToys Run. Omarchy patterns and exact bindings where supported are the target. Record limitations and intended future convergence. No new WM, shell, application catalog or generic action framework.
- Machine changes need task authorization. Research/planning does not authorize deployment. A checklist item alone is not permission to alter a live desktop.
- Pin dependency versions and provenance. Never install an unpinned latest build. Glaze has a local AppUserModelID patch; record its upstream commit, patch and build configuration.
- WINMAKASE_HOME isolates files only. Glaze IPC, hooks, tasks and Explorer are shared. Crash, taskbar and Explorer tests belong in a disposable Windows OS environment, not a second stack on the daily desktop.
- Never touch channel-selected muxel-live binaries or processes. Any Muxel test uses ordinary muxel.exe and an explicit UAT data/config root.
- Elevated execution requires protected binaries, config, parent directories, panic implementation and update path. Flag privilege-boundary changes in the commit.
- Keep physical left Win native, a panic entry independent of remapped keys, and a recoverable native desktop.
- Theme imports are data: validated palettes and approved image assets. No theme-provided Lua execution, extension installation or arbitrary configuration pass-through.

## Implementation conventions

- Rust workspace under crates/ owns CLI, supervisor, native integration and theme logic. PowerShell 7 files under installer/ handle installation. No Python; do not extend the retired TypeScript adapter.
- Portable inputs live in the repository. Machine facts live in local config. Deployed YAML, shortcuts and CSS are generated output.
- Typed app definitions own app identity and launch arguments. The v1 migration removes duplicate launch strings in keymap overrides.
- colors.toml owns palette colors. The current parser requires known semantic fields and tolerates extras; validate values. Terminal ANSI mapping follows the pinned Omarchy template.
- Prefer bounded changes to the chosen components over another resident service. The narrow independent restoration guard is a specific crash-recovery need, not an action bus.
- Run checks appropriate to each change. Rust CI uses fmt, clippy and workspace tests; config verification includes Kanata validation and structured YAML parsing. Current theme coverage is two fixture themes, not the old claimed nineteen.
- Update CHANGELOG.md for user-visible changes. Commit messages describe the actual diff. Friday-authored work uses Friday <260232009+buildfriday@users.noreply.github.com>, with Chris co-author when he directs it.
- Historical plans in docs/plans/archive/ preserve old evidence and decisions; they are not current implementation instructions.

## Layout

- crates/ — Rust workspace; keymap/ — checked grammar and overrides.
- zebar/ — bar assets; installer/ — task registration and future packaging.
- docs/DESIGN.md — current architecture and accepted v1 contracts.
- docs/TODO.md and docs/plans/v1/ — work state, bounded packages, verification.
- docs/research/ — dated findings and sources; spike/ — experiments and live-state notes.
