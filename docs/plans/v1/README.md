# Coherent v1

Accepted scope: 2026-09-08. Status: planning complete; implementation and verification pending.

Deliver a solid daily desktop on **GlazeWM + Zebar + Kanata + PowerToys Run**: launch, tile, control, theme, recover. Omarchy is the compatibility target. Exact keys where supported, matching patterns elsewhere, explicit gaps with later convergence. No application catalog or platform replacement in v1.

[Design](../../DESIGN.md) owns contracts. [TODO](../../TODO.md) owns completion. [Engineering review](../../research/v1-engineering-review-2026-09-08.md) records evidence and uncertainty. These plans supersede the older W0/W1/W2/core roadmap; completed work is retained.

[Execution procedure](execution.md) defines bounded sessions, model roles, handoffs and check-ins. [Progress report](progress.html) is a generated snapshot, not a live controller. [Desktop interactions](../../research/desktop-interactions-2026-09-08.md) answers the tray/calendar/split/scratch questions and maps every old W cut.

## Delivery order

| Package | Work | Dependencies | Exit |
|---|---|---|---|
| [P0](00-baseline.md) | Baseline, correspondence, bounded integration experiments | Current source and disposable Windows environment | R0 |
| [P1](01-window-policy.md) | Dialog policy, homes, profiles, launch actions | P0 classification/profile findings | R1 |
| [P2](02-launcher-controls.md) | Shared command entries, offline bar, controls/help | P0 launcher/popup; P1 actions for role wiring | R2 |
| [P3](03-appearance.md) | Two complete presets, application/rollback | P2 assets/popup; P1 reload preservation | R3 |
| [P4](04-supervisor.md) | Lifecycle correctness, restoration, taskbar retirement | P0 experiments; P1/P2 workflow coverage; protected layout | R4 |
| [P5](05-package.md) | Install/update/uninstall and release | P1–P4, with installer skeleton allowed earlier | R5–R6 |

Start P4 pure lifecycle corrections and privileged-layout design alongside P1; they do not depend on final styling. Start P2 offline packaging independently of app actions. Agree P3/P5 ownership journal format before either writer is implemented. No taskbar takeover rollout until P2 controls and P4 restoration are verified.

Each package is split into reviewable changes, not one large feature branch. Implement a contract, test the meaningful failure cases, review it, then integrate. Plans specify outcomes and constraints; implementation chooses the smallest mechanism that passes.

## V1 acceptance

- Main windows tile; dialogs/utilities behave deliberately; existing layouts survive routine reload.
- Loose floats are listable/recoverable; split-next controls and one daily role layout work; a dedicated terminal pad is discoverable.
- Work/personal Chrome actions identify the right profile; launch-new and focus-or-launch have separate predictable semantics.
- Keyboard and mouse can launch apps, find running windows, access tray/settings/audio/notifications, inspect bindings and recover without edge-peeking a native taskbar.
- Bar and popup work offline on the supported display/DPI matrix.
- Tray overflow is a dropdown with configurable visible icons; date/time opens a working month calendar.
- Two visual presets are complete, readable and reversibly applied across declared surfaces.
- Failures yield a usable native desktop, truthful status and bounded recovery.
- Clean install, interrupted update and uninstall preserve prior settings and later user edits.
- R0–R6 pass with recorded evidence. Proposed timing gates are validated during implementation; changes require measured justification.

## Explicit exclusions

Packaged app catalog and automatic app installation; general webapp/PWA manager; another tiler or shell; native grouping/tabs; tap-Caps relay; arbitrary editor theme pass-through; a community-theme compatibility guarantee; a custom notification center; deep launcher skinning unsupported by PowerToys.

Native controls are valid v1 implementations. Unsupported Omarchy features stay documented; exclusions are sequencing decisions, not a new divergence policy.

## Reviews and change control

For each package: implementation self-review against acceptance, independent review of changed behavior, then integration verification. After R3/R4, re-review shared ownership, focus, reload and recovery assumptions together. R6 is a fresh release review against the user workflows, not a count of completed commits.

Any spike that fails records the observation and a bounded alternative. Do not introduce a new subsystem silently. Archive superseded plans; update design and TODO in the same commit as a changed contract.
