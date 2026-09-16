# Offline shell and controls

Reader: Sol shell builders and their integration reviewer. Consume [C2/C4/C5](../build-contracts.md), the entire [P2 replacement matrix](../02-launcher-controls.md) and relevant P0 proofs. Current source is three files under `zebar/`; new module paths below are the proposed split, not existing APIs.

## S0 — Local production assets

Independent of physical Caps and app actions. Own new `zebar/package.json`, lockfile, `tsconfig.json`, `build.mjs`, `src/bar/`, `src/shared/icons/`, `assets/`, notices and generated `bar.html`/`styles.css`. The lead approves the one package boundary; do not extend the retired TypeScript adapter or silently repurpose the old root theme tests.

Use one pinned esbuild production pipeline to bundle the existing React UI and the exact compatible Zebar API. Its documented browser/JSX bundling is sufficient; no framework migration is needed ([official build documentation](https://esbuild.github.io/getting-started/)). Select/record exact versions from the pinned runtime/provenance proof before installation; the current broad `zebar@3.0` import and 3.3.1 zpack schema are not a verified version pair. Commit the lockfile and third-party notices.

Use system UI text fonts and local inline SVGs for the finite built-in icon set; remove the remote Nerd Font dependency. Preserve current controls while replacing runtime Babel/CDNs and hardcoded host paths. Runtime configuration supplies machine paths/monitor/clock choices. Optional network providers may fail gracefully; required shell rendering uses no network asset fetch.

Checks: clean production build, deterministic emitted assets, dependency/license inventory, scan scripts/styles/font/image references for remote runtime assets, cold-cache disconnected startup and restart, rendered launcher button still invokes the accepted helper without console/focus regression. A text scan is supporting evidence; observe runtime requests because dependencies can fetch dynamically. Missing weather/network data retains required controls with honest unavailable/stale state.

Stop at offline behavior preserved. Do not build tray UI against unproved transport/provider ownership. S1 proof fixtures can run independently of this asset slice.

## S1 — Popup/provider proof, then one host

First sub-slice owns only `spike/p0/zebar-shell/` and its case-specific evidence directory. Record pinned Zebar/API/WebView versions. Prove one undocked focusable widget; real inter-widget transport; zero reserve; focus/Escape/click-away/action/native-menu transitions; long-lived systray ownership; disposal and primary handover; image transfer; metadata coverage; keyboard menu anchoring; monitor removal; 100 open/close cycles. Use a disposable Windows lane for shell/provider experiments. No production transport is selected from an assumed API.

If the seam passes, record the actual API/signature/lifetime and C4 fixture messages in the handoff, then implement `zebar/src/shared/provider-client.ts`, `src/bar/provider-owner.ts`, `src/popup/` host and `zpack.json`. S0 must have released the parent bar paths first. If clean owner transfer cannot be proved, retain bounded full-Zebar restart for primary changes. If transport/menu metadata is insufficient, return one bounded helper/provider patch with evidence or the accepted native fallback; no replacement shell.

The popup has controls/tray/calendar/themes/key-help views under one host, not separate provider-owning windows. Implement activation-token focus state; blur never restores origin. View closure releases listeners/images only. Stale owner generation, sequence or item IDs reject actions. Persistent bars stay non-activating. Return exact owner/handover and focus fixtures for S2/S3.

## S2 — Tray, calendar and finite controls

Requires S0, accepted S1, F0 finite action types and J0 before persistent preferences are enabled. W2/W3 action routes become available only after their acceptance. Own `zebar/src/model/`, model tests, popup views, bar tray/clock controls and styles after S1/S0 transfer; new Rust `native_shell.rs`, `bin/winmakase-action.rs` and action tests are a separate serial sub-slice. Lead wires Cargo/lib/CLI and the finite catalog after other writers release those files.

Use one GUI-subsystem `winmakase-action.exe <catalog-id>` for finite native/bar actions, keeping `winmakase-run.exe` as the compatibility launcher shim. Unknown IDs/arguments fail; no shell text, arbitrary executable or arbitrary settings URI from a view. Generate shortcuts/help/control entries from the same catalog. Settings, notification/quick-settings routes, output selection and confirmed power operations use the P2 native routes. Missing optional audio/media capability is labeled, not a blank popup.

Allow optional PowerToys Workspaces/FancyZones settings or editor entries when the pinned installed version has a verified route. These open native configuration surfaces, do not start a Workspaces arrangement or silently switch zone layouts, and are not readiness requirements. No private-storage parser or second geometry engine in S2. Command Palette/Dock extension work stays on the later roadmap.

Tray: stable ordered identity preferences, pinned visible icons and overflow grid; no inline dock expansion. Reconcile GUID/image+UID coverage from S1. Late/ambiguous identity requires explicit fallback scope; no tooltip-regex persistence. Removed icons reject pending actions; content-addressed image data creates per-view URLs with explicit disposal. Native right-click/keyboard menus retain their destination focus and proven anchor.

Calendar: pure integer civil-date arithmetic plus zone-aware `Intl.DateTimeFormat(...).formatToParts()` for current date. Reuse/test the existing civil-date approach in `src/timefmt.rs` rather than moving UTC millisecond dates across DST. Follow system locale/12-hour/first-weekday defaults where observed; explicit user settings override. Render Today/selection/month navigation, keyboard focus and localized full date. Refresh after midnight/wake/zone change. No system-date mutation or event account dependency.

Float count appears only when nonzero and uses W2's classification; float list/cycle, split direction, named terminal and layout actions use accepted Rust APIs. Keep dialogs/scratch distinct and native Alt+Tab discoverable. Do not advertise not-yet-accepted actions as functional controls.

Pure tests: pin order and stale-generation rejection, duplicate/late/missing icons, civil leap/year/month grids, locale/weekday/DST variants, focus reducer destruction/action/menu/toggle cases, finite native action argument rejection. Guest R2 cases cover every replacement row, zero/some/all/many pins, icon removal while acting, keyboard menus with pointer on another monitor, cold offline controls, primary removal with popup open and 100 cycles without provider restart/listener/image/process growth. Persistent preference writes participate in J0 and survive concurrent theme/update/off/uninstall correctly.

## S3 — Monitor/status composition and shell acceptance

Requires S1/S2, L1 health types and W2/W3 routes. Own new `crates/winmakase/src/shell_snapshot.rs`, monitor snapshot tests and shared frontend snapshot consumption. `monitors.rs`/`display_watch.rs` transfer from L1 for a serialized integration sub-slice; do not create a second topology watcher.

Extend current bounds/reserve data with a topology generation, observed device identity, primary flag, bounds, work area and effective DPI. Re-resolve after changes; native handles are generation-local. Publish one shared shell status source from existing host/worker/state mechanisms, with fresh owner-liveness/lease observation. Remove each bar's five-second `status --json` subprocess. Transition `health.updated` is not liveness. Unsupported schema/dead/unresponsive owner is unknown/degraded.

Reconcile exact configured widgets, their bounds/reserves and exactly one tray on the current primary. Coordinate provider handover using S1, closing/reanchoring popup on removal. Failed bar restarts cannot strand native fallback. Keep direct dock reconciliation conditional on proof; bounded restart is an explicit fallback, not a claim of successful docking.

R2 display matrix: one/three monitors, primary away from origin, negative coordinates, secondary above/below at x=0, 100/150/200% DPI, primary switch and occasional hotplug. Popup stays within fresh work area and adds zero reserve; focus survives passive bar restarts. Repeat from tiles/floats/Glaze fullscreen/application F11/elevated apps with keyboard and mouse. Record 30 warm/5 cold input-ready samples; warm p95 target remains proposed 200 ms until measured. Compare total backend/WebView/private memory/CPU and process creation under identical providers, 30-minute idle and multi-hour growth checks.

Return the completed taskbar replacement matrix and actual dock/provider/recovery observations. L1 may remove hover takeover only after this evidence and restoration/suppression gates pass. Native fallback remains available throughout the unfinished shell work.
