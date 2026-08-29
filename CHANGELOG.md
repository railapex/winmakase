# Changelog

All notable changes to Winsome. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- `winsome` CLI core (Rust): supervisor with linked-pair/rollback semantics, rolling logs, health state, console-shutdown grace budget; `supervise`/`status`/`logs`/`down`. 61 tests.
- High-effort code review of both crates: 10 findings (6 confirmed correctness incl. an orphan-spawn bug and a shutdown-grace mismatch), all fixed same-day.
- Theme adapter core: omarchy v4 `colors.toml` parser (26-key schema, loud failure), Windows Terminal scheme renderer using omarchy's own `ansi_alias` mapping, light/dark detection matching omarchy's `resolve_theme_mode` precedence. 21 tests with real theme fixtures.
- Repo scaffold: design doc, work breakdown, README.
