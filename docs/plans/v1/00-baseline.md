# P0 — Baseline and integration proofs

Status: not started. Exit: R0. Research findings are not experiment results.

## Deliverables

1. Record exact installed/source versions, upstream commits, local patches, build flags, artifact hashes and licenses for Winmakase, Glaze, Zebar, Kanata and PowerToys. Record Windows build, display layout/DPI, task definitions, config ownership and a sanitized process/startup inventory. A file version of 0.0.0 is not provenance.
2. Freeze the comparison against Omarchy v4.0.2, with exact upstream revision recorded. Compare the entire current mapping, then mark v1-critical chords and each exact/equivalent/unavailable/deferred result. Include generic launch-new versus app-specific focus behavior, workspace families and Windows-reserved keys.
3. Produce disposable-environment fixtures and recovery instructions. WINMAKASE_HOME alone is not isolation: fixed Glaze IPC, hooks, task names and shell changes require a separate Windows OS environment.
4. Capture current warm/cold launcher timings, one/three-monitor bar CPU and private memory, process creation rate, docks, palette screenshots and reload effects. Keep raw measurements, not a single favorable sample.

## Bounded experiments

| Proof | Procedure and pass evidence | If it fails |
|---|---|---|
| Run action indexing/activation | Create owned shortcuts to one test binary with different argument arrays, including spaces/Unicode/profile labels. Search, invoke, rename/remove, restart Run; verify exact action and no console flash. Prove key and bar activation reach one launcher on the focused monitor, including fullscreen/elevated contexts | Compare PowerToys-native command entries/bookmarks within the chosen stack; document result before adopting a plugin |
| Zebar popup | One undocked controls widget; keyboard toggle, Escape restoring origin only while owning focus, click-away keeping clicked target, action keeping destination, fullscreen, mixed DPI/negative coordinates, repeated open/close, monitor removal | Simplify to native settings/help surfaces or fix a bounded widget integration issue; no replacement shell |
| Classification | Inspect sanitized Glaze/Win32 facts for main windows, owned/modal/non-resizable dialogs, browser popups, launchers and utilities | Specify minimum classification facts/patch needed in the chosen Glaze base; do not infer from title alone |
| Chrome profiles | Read actual per-window identity for explicit work/personal launch, multiple windows, incognito, picker, installed app, existing background process and delayed first window | Tighten main-window eligibility/discovery; never silently fall back to any chrome.exe |
| Theme reload | Render colors and reload the current stack with manually moved/floated windows present; record IDs/state/geometry/focus | P1 fixes manage-rule replay before P3 applies themes |
| Native taskbar suppression | In isolated Windows, compare native autohide alone with minimal suppression; test edge, left Win, Start, notifications/tray flyouts, fullscreen, Explorer recreation and DPI change | Keep native taskbar available until a mechanism passes; do not retain accidental hover UX as the target |
| Crash restoration | Worker/guard prototype, durable baseline, kill each separately during takeover; verify exact cleanup without recursive restarting | Refine the narrow owner contract; no readiness claim from PID alone |
| Commit-pressure workaround | Test pinned Glaze start/adoption/wake at measured available commit levels; retain crash evidence and distinguish adoption from new allocation | Keep a bounded documented mitigation pending a Glaze fix; do not indefinitely block a healthy adoption |
| Appearance primitives | Verify Terminal scheme fragment/selection reload, native modes/accent response in Run, wallpaper per monitor and opt-out behavior | Publish a precise supported-target matrix; no promise based on registry writes alone |

## Completion evidence

A dated R0 report includes observed versus proposed behavior, source/build identifiers, command transcripts, screenshots where visual, and dispositions for every failed proof. Freeze supported OS/display assumptions. R0 does not require implementing the full product; each dependent package needs its relevant proof resolved first.

Proposed performance baseline: 30 warm and 5 cold launcher/popup activations, cold reported separately; warm p95 target 200 ms to input-ready. Record hardware and method. Test setup may change the proposed target with evidence, never silently lower it to turn a failure green.
