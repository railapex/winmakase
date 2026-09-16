# Appearance, dogfood and release

Reader: Sol appearance/evidence builders and the Astra release lead. Consume [C3/C6/C8](../build-contracts.md); retain every [P3](../03-appearance.md) and [P5/R6](../05-package.md) case.

## T0 — Two complete preset renderers

Requires F0 types and accepted S0 local-asset layout; no live apply. Own `crates/winmakase-theme/`, preset palette/wallpaper inputs and their manifest/notices, with the asset destination agreed with S0. Do not edit runtime bar layout/styles concurrently with S2; return semantic tokens for that owner.

Keep the current Rust palette/Terminal renderers and two fixture themes. Pin the upstream palette revision. Deliver Tokyo Night dark and Catppuccin Latte light for bar/popup tokens, Glaze border/geometry inputs, Terminal scheme, wallpaper choices and optional native mode. A palette parser accepting extras does not prove full visual support for other themes. Define required semantic tokens and validate colors/images/paths; no imported Lua or arbitrary config execution.

The existing Catppuccin fixture is dark Mocha, so add actual Latte rather than relabeling it. Reconcile v4.0.1 comments/tests with the plan's pinned v4.0.2 source and derive the manifest from that tree. Windows app/system mode changes default off, available by explicit target opt-in.

Tests cover missing/unknown/invalid fields, both expected Terminal mappings, readable focus/disabled tokens, safe image paths and deterministic render output. Generate representative bar/popup/control states for actual screenshot review at 100/150/200% scale. The reviewer inspects clipping, long labels, busy tray, contrast and alignment. Use P3's proposed 4.5:1 normal/3:1 large-text targets; numeric checks supplement screenshots.

Stop at renderable, complete presets and a precise target matrix. Native application support that P0 has not observed remains pending. Preserve Chrome profile colors; Muxel and arbitrary editor configuration are excluded.

## T1 — Journaled application and restoration

Requires T0, J0, P0 appearance primitives, INST1 mutation boundary and preserved P0.D1 Glaze reload. Own new `crates/winmakase/src/theme_apply.rs`, integration tests and explicitly transferred CLI/composition wiring. Consume the shared journal; do not add a private theme transaction format.

Validate/stage before mutation; apply only owned file/setting keys; reload only affected surfaces; verify target observations and report unsupported/restart-required behavior. Prefer a Terminal scheme fragment plus small selection edit; preserve unrelated settings and profiles. Native wallpaper/mode operations use typed allowlisted adapters. Roll back applied entries on failure; retain conflicts and partial restoration errors. No input/tiler restart solely for color changes.

After S2 transfers its theme view, finish a serial T1 UI integration sub-slice owning that view and semantic styles. Supply theme list/set through the CLI and the same finite popup action, static palette/wallpaper previews with zero desktop mutation, and configurable per-target opt-outs. There is no trial-apply/cancel mode. A failed or pending apply displays its actual result; the UI cannot show the selected preset as applied before verification.

R3 guest cases: CLI/popup parity, previews leave files/settings/layout unchanged, every target opt-out, ten dark/light alternations on manually arranged windows, interrupted apply at every durable/write boundary, two simultaneous applies, role/key/theme interleavings, update/off/uninstall races, killed owner, stale generation, malformed palette, locked target, absent Terminal, denied native write, later user edit and missing rollback wallpaper. Assert workspace, state, geometry, focus and exact setting restoration. Screenshots cover both full presets and supported scales. Stop at accepted R3 evidence, not a parser-only pass.

## REL0 — Dogfood and release evidence

Lead owns final acceptance; Sol can build evidence tooling/docs in `docs/verification/v1/<candidate>/`, release documentation and `packaging/` after INST1 transfers it. This is slice REL0, distinct from verification round R0.

Before daily cutover, assemble one concrete packet: candidate/version and paired hashes; selected input mode; supported/unsupported workflows; four-cut evidence; protected panic and visible native fallback; config/task backup manifest; exact install/rollback commands; remaining gates. Ask Chris for the prepared machine change and whether the candidate becomes the default session. Physical input/hardware cases run only within that authorization. Historical spike PIDs/deployment notes are not current evidence.

Record the dogfood candidate in the manifest and, when authorized, a clearly named prerelease tag/release section. Development Cargo bumps stay distinct from release tags. No public v1 claim while R0–R6 remain open. Required behavior may not be waived as cosmetic.

During real use, log command/action, build, trigger, observed failure, recovery and artifact reference. Fix data/settings loss, focus theft, trapped input, stranded windows and restoration failures promptly; invalidate and repeat affected gates. Measure startup/recovery/launcher latency, restart counts and recurring manual interventions. A changed shared contract requires affected retests, not erasing the clock or assuming every earlier pass still applies.

R6 requires five working days, three cold logins, five sleep/wakes, ten occasional-monitor cycles, both presets and all required earlier gates. Record native taskbar access and the missing/recovery reason. W5 separately needs at least seven elapsed days after verified app actions and float recovery, then a written switcher need/no-need verdict. Do not simulate elapsed time or automatically build a switcher from a need finding.

Finish clean supported-Windows install/repair/update/rollback/uninstall and cold-cache offline runtime; freeze provenance/patch/build/license/notices, migration range and supported matrix. Documentation covers setup, key fidelity/gaps, roles/profiles, controls, themes, recovery, updates and removal. Resolve required CI failures; ignored `continue-on-error` desktop tests cannot substitute for recorded visual/OS acceptance. Fresh review exercises representative workflows and reads failure evidence. Chris makes public release/announcement decisions after that packet is ready.

## Glaze contribution lane

Upstream work is independent maintenance and cannot block dogfood. It uses an available builder slot, its own worktree and current official upstream base verified at the time. Start with cleanup-uncloak reproduction/port, then AppUserModelID; reduce placement-denial and split native facts from reload semantics. Direct-Caps upstream work waits for its local decision. Never submit the integration branch or assume the current remote URL establishes ancestry.

Every port records original/local commit, official target, minimal patch, reproduction/regression evidence, required paired Winmakase capabilities, source/artifact hashes and PR state. Before opening an external PR, prepare the actual diff/body and resolve Chris's publication timing/authorship. This plan itself sends nothing externally.

## Beyond v1

Detailed builder scope ends at v1. v1.x takes recurring measured friction, theme expansion and diagnostics/migration improvements. v2 grouping/tab stacks, multi-window scratch, sticky windows, deeper role/browser identity and missing chord families need an observed limitation, bounded Glaze primitive proof, user workflow and rollback before a work order. v3 platform/provider changes require comparative evidence and migration, not a speculative abstraction today. [Future platforms](../../future-desktop-platforms.md) and [gap research](../../../research/gap-features.md) retain those triggers.

Chris's 2026-09-16 questions add concrete later candidates: a Command Palette extension/Dock bands over the finite action contract, supported Workspaces app-set access, FancyZones for explicitly unmanaged tools, and a maintainer-agreed narrow PowerToys contribution. Whim remains a measured alternative, with current inactivity recorded rather than assumed abandonment. [Dated primary-source findings](../../../research/layouts-powertoys-2026-09-16.md). No v1 backend changes follow from this research.
