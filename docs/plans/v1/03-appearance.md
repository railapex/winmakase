# P3 — Appearance and reversible theme application

Status: not started. Depends on P2 assets/popup and P1 reload preservation. Exit: R3.

## Deliverables

- Tokyo Night dark and Catppuccin Latte light, complete across the declared core targets.
- Pinned palette manifest, parser coverage and rendered fixtures. Current research counted 22 palettes in Omarchy v4.0.2; derive the manifest from the pinned tree, not a hardcoded historical total.
- Theme list/set through CLI and the same popup action, with static palette/wallpaper previews that do not mutate the desktop. No trial-apply/cancel state machine in v1. Configurable target opt-outs.
- Staged apply, recovery journal and shared-settings ownership compatible with P5.

## Changes

1. Validate required palette fields and color values; tolerate extra fields unless a version rule requires rejection. Import only palette data and approved local image assets. Retain the pinned Ghostty ANSI mapping.
2. Use semantic colors throughout bar/popup states, focus borders and theme selector. Fix hardcoded white/dark assumptions. Consistent geometry, typography, spacing, icon weights and hit areas matter as much as colors.
3. Apply Terminal schemes through owned fragments where supported. Select the scheme with a narrow backed-up edit if needed; respect per-profile overrides and application absence.
4. Apply per-monitor wallpaper and optional Windows app/system mode. Probe Run's actual appearance response; document limited native matching rather than claiming full palette skinning.
5. Preserve Chrome profile colors. Leave arbitrary editor configs, extension installation and Muxel integration outside this package.
6. Serialize apply. Validate/render a new generation, journal original/current owned values, commit owned files atomically, apply native values, reload only affected surfaces, verify and report.
7. Recover interrupted apply at the next invocation/start. Roll back already-applied targets on failure; report a degraded result if a restoration fails. Independent applications cannot commit globally atomically.
8. Restore shared values only while equal to our last write. Keep later user edits and report conflicts. Do not overwrite whole settings files or local overrides.
9. Share one per-install mutation owner with configuration, update, off and uninstall. Those commands quiesce or reject in-flight applies; verify the active generation before theme commit or rollback. Emergency native restoration bypasses this lock and revokes takeover independently. No additional service is needed.
10. Generate Glaze output through the same base/role/keymap/theme composition path as P1. Test concurrent role-change/theme-apply and keymap-change/theme-apply: each accepted input survives in the final generation.

## Verification

R3 captures both presets at supported scales, all bar/popup states, long labels, busy tray, active/inactive windows and wallpapers on mixed displays. Review actual screenshots for clipping, contrast, readable focus and alignment. Proposed text contrast gate: WCAG AA 4.5:1 for normal text, 3:1 for large text; focus and disabled states also need visual review.

Use manually arranged windows before every theme cycle. Ten alternating applies preserve workspace, float/fullscreen state, geometry and focus. No pair restart solely for colors. Record which targets need application restart and do not claim immediate application when a target has not changed.

Fault cases: malformed palette, missing required field, unknown extra field, invalid path/image, locked target, permission denial, absent Terminal, later user edit, concurrent apply, apply interleaved with update/off/uninstall, killed mutation owner, apply process killed between each commit step and deleted wallpaper during rollback. Verify exact owned-file restoration, obsolete-generation rejection and conflict reports.

R3 passes only when both complete presets and failure recovery work. A parser/golden pass for other palettes does not make them shipped visual presets.
