# F0/S0 source checkpoint

**Status:** source implementation accepted; desktop runtime acceptance remains open.

## Build

- Baseline: `5969051` (`main`, published to `origin/main`).
- F0 contracts: `74586c6`; frozen serialized fixtures: `a97d24d`.
- S0 offline bar assets: `2e2bc2f` (reviewed cherry-pick of builder commit `2402fa0`), with integration corrections `eee678b` removing checked-in host paths and `cb2555b` fixing deterministic checks across Windows line-ending checkout.
- Integration branch: `build/f0-s0`.
- Environment: Windows host; Rust 1.98 toolchain; pinned Node dependencies from `zebar/package-lock.json`.
- No desktop process, hook, task, Explorer state, installed configuration or daily deployment was changed.

## F0 observations

- Added versioned role identity, finite action/result, popup snapshot, lifecycle/request, health-generation and mutation-journal contracts.
- Existing `[apps]` configuration normalizes into roles. Existing configuration defaults to `input_mode = "kanata"`; direct Caps is explicit.
- One pure composition path accepts the base, optional rendered theme base, keymap, validated roles and input mode, then delegates to `winmakase-keymap`.
- Health schema 2 exposes owner/WM generation, input mode, lifecycle, takeover, responsiveness and degraded/cleanup state. Schema 1 is rejected as unsupported instead of being shown healthy.
- Literal JSON/TOML fixtures freeze role identity, action result, health, popup snapshot, mutation intent and mutation result shapes under `crates/winmakase/tests/fixtures/contracts`.

Validation:

- `cargo check -p winmakase --all-targets` — pass.
- `cargo test -p winmakase --lib` — 159 passed, 1 ignored.
- `cargo test -p winmakase-keymap` — 33 passed across its test binaries.
- `cargo clippy -p winmakase --all-targets -- -D warnings -A clippy::collapsible-if -A clippy::new-without-default` — pass. The two allowed lint classes cover three findings reproduced on untouched baseline: one `collapsible_if` in `supervisor.rs`, one `new_without_default` and two `collapsible_if` occurrences in `taskbar.rs`. They remain release cleanup, not F0 regressions.
- `git diff --check` — pass.

The supervisor integration suite could not provide a valid result on this host. Current Windows committed-memory use was about 94%, while the existing safety guard deliberately refuses fresh Glaze-named test processes above 50%. The first start therefore timed out and parallel tests cascaded. A serial replay reproduced the pressure block. The guard was not weakened. No integration case is marked passed from that run.

## S0 observations

- Pinned React 18.3.1, React DOM 18.3.1, Zebar 3.3.1 and esbuild 0.28.2.
- Replaced runtime Babel, React/Zebar CDNs and Nerd Font downloads with committed local JS/CSS and inline SVG icons.
- Preserved workspaces, one primary tray heuristic, clock, CPU/memory/network/weather, tiling controls and the configured launcher control.
- Removed the per-monitor `status --json` subprocess poll. Health remains visibly unknown until S3 consumes the shared status source; a missing or incompatible source cannot render green.
- Removed the checked-in user-specific launcher path. The source pack disables launcher execution until installation supplies an exact machine path in both runtime configuration and Zebar's matching privilege. Build validation rejects user-specific source paths and mismatched program/privilege pairs.
- Added deterministic build checks, runtime remote-reference scanning, a local runtime-config file and generated third-party notices with exact archive/integrity data.

Validation from a clean dependency install:

- `npm --prefix zebar ci` — pass, 0 reported vulnerabilities.
- `npm --prefix zebar run build` — pass.
- `npm --prefix zebar run check` — TypeScript pass; generated assets deterministic/current; no remote runtime asset references.

Cold-cache disconnected Zebar startup, rendered controls, monitor/tray behavior and launcher focus were not exercised. Those remain S0 runtime evidence for R0/R2 and block any offline-runtime or taskbar-replacement acceptance claim.

## Disposition

F0 is ready for W0/L0/J0 importers. S0 is ready for later S1/S3 wiring. No P0-P5 package or R0-R6 round closes here. The next source wave is W0 role identity proof/schema and L0 singleton/request mechanics; P0.I2 physical Caps remains a separately prepared hardware gate.
