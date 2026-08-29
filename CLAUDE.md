# Winmakase — Agent Instructions

Omakase omarchy-style desktop environment for Windows 11. Public repo (railapex/winmakase).

## Session start

1. Read `docs/TODO.md` — the work-state source of truth. Active milestone = first with unchecked items; pick up at the first unchecked item.
2. Read `spike/NOTES.md` § LIVE MACHINE STATE — what is actually running on Chris's rig right now (dogfood config, scheduled tasks, known landmines). Machine state is NOT derivable from the repo.
3. Read `docs/DESIGN.md` for any area you're touching. **If code disagrees with DESIGN.md, one of them is wrong — fix whichever it is, in the same PR/commit.**
3. Check items in TODO.md **in the same commit as the work**. `[v]` items need verification on the real rig before checking.

## Hard rules

- **Never** install components as scheduled tasks / startup, enable kanata beyond console-mode, or modify Chris's live PowerToys/taskbar config unless the TODO item explicitly says so and the milestone is past M0. Spike work is console-mode and reversible by closing the console.
- All tool versions are pinned (scoop bucket). Never install "latest" of kanata/GlazeWM/Zebar; bump pins deliberately in their own commit.
- The keymap mapping file is omarchy-grammar-verbatim. Deviations require a DESIGN.md edit explaining why, in the same commit.
- Theme adapter: colors.toml is the sole palette source (26 semantic keys incl. `mode`, omarchy 4 format — see DESIGN.md). Terminal ANSI mapping mirrors omarchy's `default/themed/ghostty.conf.tpl`, never invented. Missing or unknown shape = loud failure, never a silent wrong palette. No parsing of waybar.css — ever.
- Elevated-kanata config lives in an ACL-protected path; anything that changes that path or its permissions is security-sensitive — flag it in the commit message.

## Conventions

- Rust for the winmakase binary (CLI + supervisor + theme adapter; cargo workspace, `crates/`). PowerShell 7 (`pwsh`) for install-time scripts as `.ps1` files. The TS under `adapter/`+`tests/` is the retired reference implementation — do not extend it; it's removed once the Rust port's parity is verified. No Python.
- Tests: adapter = golden files under `tests/golden/` (19 stock omarchy themes); installer = Pester; configs = `kanata --check` + YAML parse in CI.
- CHANGELOG.md (Keep a Changelog) updated with anything user-visible.
- Commits: subject describes the actual diff. Author: Friday (`Friday <260232009+buildfriday@users.noreply.github.com>`) with Chris co-authored when he directed the work; plain Chris when he authored.

## Layout

- `docs/` — DESIGN.md, TODO.md
- `keymap/` — tiler-agnostic grammar mapping (planned)
- `adapter/` — theme adapter TS (planned)
- `installer/` — install.ps1, uninstall.ps1, bucket (planned)
- `supervisor/`, `helper/` — process supervision, winmakase-helper (planned)
- `themes/` — omarchy themes as data dependency (submodule or synced; planned)
