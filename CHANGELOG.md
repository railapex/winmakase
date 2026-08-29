# Changelog

All notable changes to Winsome. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- Supervisor hosts the live stack: adopt-first takeover (running components are watched, not restarted), kanata driven through the elevated `WinsomeKanata` scheduled task (`schtasks /run`//`/end` — the elevation door for a user-level supervisor), GlazeWM launched via `ShellExecuteExW` when its UIAccess manifest refuses plain `CreateProcess` (error 740), singleton guard against a second supervisor, and a display-change watch (always logged; `bounce_on_display_change` opt-in — glazewm#1233 insurance).
- `winsomed.exe`: windowless (GUI-subsystem) supervisor for the `WinsomeSupervisor` logon task; session end arrives as `WM_ENDSESSION` and runs the same graceful shutdown as Ctrl+C.
- `installer/register-tasks.ps1` / `unregister-tasks.ps1`: deploy binaries to `~/.winsome/bin` and register the logon task without elevation; an elevated run also refreshes the `WinsomeKanata`/`WinsomePanic` tasks.

### Changed
- `winsome down` now waits for the shutdown to finish rather than returning on the acknowledgement — callers acting on "down came back" were racing a shutdown still in flight (live incident: panic killed the supervisor mid-shutdown and orphaned the pair).
- `spike/panic.ps1` stops the supervisor first (via `winsome down`) and only then sweeps — killing components while the supervisor lives means the desktop refuses to die; writes a transcript to `~/.winsome/logs/panic-last.log`.
- `spike/winsome-up.ps1` reduced to taskbar-hide plus triggering `WinsomeSupervisor`; the supervisor owns bring-up.
- Keymap: full omarchy (quattro) grammar as tiler-agnostic data (`keymap/omarchy.toml` — 226 chords accounted for: 73 mapped to GlazeWM, the rest carrying explicit gap/native/app/omitted status with reasons) plus `winsome keymap check|render` to validate it and generate the GlazeWM `keybindings:` YAML. Golden-tested; coverage counts are a pinned drift alarm.
- `winsome` CLI core (Rust): supervisor with linked-pair/rollback semantics, rolling logs, health state, console-shutdown grace budget; `supervise`/`status`/`logs`/`down`. 61 tests.
- High-effort code review of both crates: 10 findings (6 confirmed correctness incl. an orphan-spawn bug and a shutdown-grace mismatch), all fixed same-day.
- Theme adapter core: omarchy v4 `colors.toml` parser (26-key schema, loud failure), Windows Terminal scheme renderer using omarchy's own `ansi_alias` mapping, light/dark detection matching omarchy's `resolve_theme_mode` precedence. 21 tests with real theme fixtures.
- Repo scaffold: design doc, work breakdown, README.
