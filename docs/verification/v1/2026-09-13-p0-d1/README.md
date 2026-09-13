# P0.D1 dialog classification and reload acceptance

**Accepted for the bounded D1 contract.** Winmakase 0.1.7 renders main-window homes with native owner/resizability guards and a preserve-state default. Glaze `6bc83d1` supplies the native facts, initializes owned or fixed-size windows as floating, inserts a managed owned window on its owner's workspace, and does not replay completed Manage rules during routine config reload.

This is a paired-version change. The new Glaze DTO fields and YAML matchers are required by the Winmakase renderer; mixed old/new counterparts are unsupported. The daily Glaze remains `9c8e93b`. No host cutover occurred.

## Source and review

- Glaze source: `6bc83d1a9682e885c507ac489ca0208f20f985e0` on `winmakase-v1-input`.
- Release build: all three executables passed in 17.55 seconds; exact command, hashes and archive provenance are in [direct-caps-build.json](direct-caps-build.json).
- Winmakase: 14 keymap unit tests, 14 app-rule tests, 2 real-keymap tests and 16 config tests passed. Strict keymap Clippy passed.
- Glaze: the new reload, classification, owner-workspace and native matcher tests pass. The targeted production Clippy command passes. Full `cargo test -p wm` has 14 passes, the explicit native-placement test ignored, and the existing display-dependent `matches_corner_positions` failure; that same failure predates D1.
- Fresh Astra review found no remaining source correctness issue after two fixes: owner placement now wins over WmState's startup fallback, and ignore precedence compares identity selectors without letting native facts suppress every main rule.

## Disposable guest

Windows Sandbox `62a7b3fa-ff5f-43d8-b435-947267d601b0` ran Windows build 26100 with vGPU, networking, clipboard, printer and audio input disabled. Only fixture/probe/assets mappings were read-only; the isolated evidence directory was writable. The pinned VC runtime installed successfully in 243.27 seconds. Glaze started from the hash-pinned archive and answered its workspace query. The guest was stopped after capture; final host checks found no registered Sandbox or `vmmemWindowsSandbox` process, and `env:winmakase-uat` was released.

## Initial classification

| Fixture window | Native facts | Glaze result |
|---|---|---|
| Main | Unowned, resizable | Tiled, workspace 1 |
| Owned modeless dialog | Owner is its managed main; fixed-size | Floating, same workspace as owner |
| Modal dialog | Owner is its managed main; fixed-size | Floating, same workspace as owner |
| Utility tool window | Owned, fixed-size, tool-window style | Unmanaged; explicitly incomplete |

The corrected fixture home rule contains only `move --workspace 1` and matches `window_has_owner: false` plus `window_is_resizable: true`. Dialog state and placement therefore come from native classification rather than a process-wide rule.

## Routine reload

The main-only control was set floating at `(250, 200)`, size `700 × 450`, moved to workspace 2 and focused. `wm-reload-config` then reloaded the same config.

| Fact | Before reload | After reload | Verdict |
|---|---|---|---|
| HWND/process lifetime and Glaze ID | Recorded | Same | Pass |
| State | Floating | Floating | Pass |
| Workspace | 2 | 2 | Pass |
| Geometry | 250, 200; 700 × 450 | 250, 200; 700 × 450 | Pass |
| Display/focus | Shown, focused | Shown, focused | Pass |
| Completed Manage rules | Retained | Retained | Pass |

All represented main, owned and modal rows also preserve native identity/styles, Glaze management, state, workspace, geometry, parent, display state and focus. The utility comparison is `incomplete` because Glaze excludes that tool window before policy.

## Boundaries retained

The hidden-parent replay remains A3. After the managed owner moved to hidden workspace 2, a validated fixture child HWND created a fresh owned dialog (generation 3) with the correct native owner and fixed-size facts. Glaze did not admit it. This does not pass cloaked-owner admission or background-input behavior.

Real login/settings/Viscosity dialogs, utility policy, mixed-DPI/GPU displays and elevation remain unrun. P1/R0 does not close with D1.

Raw guest records are in [evidence](evidence/). The earlier failing baseline remains in [2026-09-12 dialog evidence](../2026-09-12-p0/dialog-runtime.md).
