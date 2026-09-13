# D1 replay overlay

Use the guarded 2026-09-12 probe set as described in its [replay guide](../../2026-09-12-p0/probes/README.md), replacing these four files with this directory's versions:

- `glaze-fixture.yaml` — main-only home rule with native owner/resizability guards and no forced state.
- `Start-GuestGlaze.ps1` — Glaze `6bc83d1` archive and executable hashes.
- `Arrange-GuestDialogProof.ps1` — paired CLI hash.
- `Install-GuestVCRuntime.ps1` — 300-second observer budget matching the pinned installer's measured runtime.

Do not alter the dated 2026-09-12 files; they reproduce the failing baseline.
