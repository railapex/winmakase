# Placement-denied tile recovery — 2026-09-10

## Observed failure

Workspace 2 reserved a 475 × 1544 tile at `(3356, 48)` for HWND `3673870`, titled `CUA disposable smoke test`. The actual window was 442 × 215 at `(184, 214)`, behind other windows. A targeted Glaze `ignore` removed the reservation; the two remaining tiles filled the workspace. The native app remained open.

The original window closed before its token could be inspected. The CUA daemon had high integrity (`12288`); Glaze had medium integrity (`8192`) and `uiAccess=0`. An owned probe launched through CUA reported `elevated=True` and reproduced the mismatch: native 442 × 215, Glaze allocation 950 × 1544. Thus the elevated launch route is reproduced, while the original process's token is not claimed as directly observed.

Glaze's existing placement path logged errors and kept the tiling allocation. Windows access-denied messages in the old logs omit HWNDs, so those historical messages cannot identify the original window individually.

## Implemented recovery

Glaze commit `9c8e93bac0e93a6db4e76b1ecca104ee5b1fcf17` on `fix/placement-denied`, worktree `D:/dev/glazewm-wt-placement-denied`, is based directly on the deployed AppID source `6d62d6616e6acdfd1c1d35ad42ba468f9bb7972e`. It contains no direct-Caps changes.

- A typed Windows `E_ACCESSDENIED` from a combined placement/stacking call gets one geometry-only retry. Only denial of that retry can release a tile. Message text, I/O errors, stale handles, property/visibility errors and stacking-only errors do not trigger release.
- Automatic release applies only to displayed, natively visible, non-minimized tiled windows outside an active drag. Native visibility also excludes cloaked windows. Hidden windows retain ownership rather than being abandoned invisibly.
- Recovery finishes the current sync pass, then uses normal unmanagement to repair the tree, emit events and queue neighbour redraw. A new pass consumes that work; each retry removes a tile, bounding the loop.
- If a released app is actually in the foreground, recovery preserves its native focus across passes while repairing internal selection. It cancels native focus/cursor movement and the close/minimize focus override.
- Released HWNDs cannot re-enter through repeated SHOW events. Destruction clears the exclusion so handle reuse does not inherit it.
- Failure logs identify the window and intended rectangle.

The geometry-only retry uses `SWP_NOZORDER`, which ignores the relative stacking target. [Microsoft SetWindowPos reference](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos).

## Verification

- Seven focused WM regression tests pass: unfocused/focused/last/nested removal, tiling shares, event ordering, foreground preservation, pending work, admission/lifetime checks and rejection of misleading error text.
- Four platform error-classification tests pass.
- Native denied probe passed: elevated HWND `8130092`, requested x/width change rejected, native frame unchanged, synthetic tile detached, sibling share recovered, app still visible and alive.
- Native ordinary probe passed: non-elevated HWND `20843776`, placement succeeded and native geometry changed; it stayed attached and no ignore/event was produced.
- Both native component tests used explicitly owned marker windows. No second Glaze daemon was started. Probes were removed from the daily tiler before testing the candidate helper and closed themselves afterward.
- Strict Clippy for `wm` and `wm-platform` passes. Formatting and diff checks pass. Independent review covered placement attribution, hidden-window safety, sync iteration, focus and handle lifetime.
- The full WM suite has one existing failure, `matches_corner_positions` at `handle_window_moved_or_resized.rs:556`. The exact test also fails on untouched deployed-source `6d62d66`. Hotfix run: nine passed, one failed, one native test ignored; the ignored native test was run explicitly in both modes above.

Limits: native component tests cover the actual placement helper plus tree recovery, not a running candidate daemon's full sync loop. The admission test uses synthetic HWNDs and is supplemented by source review of the guard order. This is permission-denied recovery, not a general watcher for windows that accept a placement request and later move themselves.

## Delivery

The release build passes and is staged at the normal next-start path, `C:/Users/chris/.winmakase/bin/glazewm.exe`. SHA-256: `06548C993F98B477327AFD1E923FEB7F114708F549BCDFEA1C1A06D433BFDFD8`.

The old executable is retained at `C:/Users/chris/.winmakase/bin/glazewm.pre-placement-denied-20260910.exe`, SHA-256 `4EA8BFD012915907628A21C8E1907F096BDA7253BEDFCD7D55952232C6F79988`. Both copies were hash-verified. Staging renamed the old disk image and installed the new one; the mapped running image continues unchanged. Glaze PID `19880`, start `2026-09-10T04:25:05.5984669-07:00`, executable path and supervisor restart count (`0`) stayed unchanged afterward. The fix takes effect when Glaze next starts. Rollback is the retained old binary; replace the on-disk main executable with that copy before the next start to cancel staging.

The fix was also cherry-picked into the future input branch as `dd8fb7d`; its seven focused recovery tests pass. That branch remains a separate, unactivated candidate. This fix does not pass R0 or R1 and does not activate direct Caps. The original gap was repaired immediately with targeted `ignore`; no running candidate-daemon acceptance is claimed.
