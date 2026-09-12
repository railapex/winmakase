# V1 verification rounds

**Status: all rounds pending.** This is a verification design, not a passing test report. Research and document reviews performed on 2026-09-08 do not close implementation gates.

## Evidence contract

For every case record: case ID, requirement, build/source/pins, OS/environment, display/DPI layout, initial state, procedure, expected result, actual observation, pass/fail, artifact path and reviewer. Include relevant timestamps, Glaze IDs/profile identity, workspace/state/geometry, focus, process/task ownership and settings deltas. Sanitize personal paths/titles/profile data before committing.

Store reports beneath docs/verification/v1/<build>/<round>/ when running them. A skipped required case blocks the round. Failures get a regression case and targeted retest; rerun later dependent rounds when a shared contract changes. No rerunning unrelated suites without a reason.

## Isolation

Unit/config tests run without live mutations. Hook, fixed-IPC, taskbar, Explorer, crash, installer and destructive pressure tests run in a disposable Windows OS environment. A separate account or WINMAKASE_HOME alone does not isolate global hooks, task names or IPC.

Record which Sandbox/VM hardware limitations exclude sleep, GPU, audio or physical monitor behavior; verify those on the authorized dogfood system after isolated recovery passes. Never start/stop/inspect/hash channel-selected muxel-live. Ordinary muxel.exe may be used with explicit UAT data/config roots for disposable app windows.

## Rounds

| Round | Required cases | Exit evidence |
|---|---|---|
| R0 — baseline | Exact provenance; pinned Omarchy comparison; Run indexing; popup; profile/classification; agent ownership/admission/background-input A1–A4; theme reload; suppression; guard; pressure; baseline performance | Resolved proof reports and supported environment matrix |
| R1 — windows/apps | Dialog/home/reload/adoption contracts; supported agent output placement and human/other-agent negative controls; profile/launch races; hidden/minimized/scratch selection; float recovery; nested split construction; named terminal; cold/warm/repeated daily layout; F/Alt+F/T and native collisions | Correct identity/state/focus with no extra or misrouted window |
| R2 — launcher/controls | Every replacement row; offline/fullscreen/elevated; popup focus/docks; mixed displays; tray dropdown/pins/identity/menu anchoring; 100 popup cycles; month calendar/date boundaries; float indicator | All workflows reachable, correct widgets/tray, input-ready measurements |
| R3 — appearance | Both full presets; screenshots at scales; ten theme alternations; preserved layout/focus; concurrent/interrupted/failing applies; apply versus update/off/uninstall; local edits | Visual approval and exact restoration/conflict evidence |
| R4 — lifecycle | Pair/bar failures; singleton/request/generation races; failed kill; worker/host death or hang during blocking calls; recovery task failure; missing/recreated Explorer; pressure; retry exhaustion; baseline/later-edit variants; logoff during takeover | Truthful ownership, bounded recovery and taskbar takeover proof |
| R5 — distribution | Clean install/repair/update/rollback/uninstall; ACLs and tampered journal; interrupted/concurrent mutation ownership; preexisting/later-edited settings; missing optional apps; offline runtime | Manifest/ACL/settings comparisons; no orphan ownership |
| R6 — integrated release | Five working days; three cold logins; five sleep/wakes; ten monitor cycles; full CI; independent workflow review | All prior gates pass, no blocking defect, release decision recorded |

W5 is a written switcher need/no-need decision after at least one week with verified app actions and float recovery. It is not an automatic switcher implementation or a reason to extend v1 without evidence.

## Measurable gates

These are proposed acceptance targets to validate, not current performance claims.

- Launcher and controls: 30 warm activations, five cold separately; warm p95 ≤200 ms to input-ready on recorded hardware.
- Focus-or-launch: twenty repeated invocations focus the intended profile without creating another window; simultaneous aliases/slow startup do not duplicate; intentional launch-new works.
- Agent output: the P0-supported window classes pass twenty creation/destruction cycles without transient human tile/focus changes, including concurrent agents and deliberate inspection/return. Human/unrelated/unknown windows are never reassigned by inferred ownership. Observer restart and expired identities preserve existing layout. Verify capture/input under the actual workspace hiding mode. [A1–A4 evidence contract](../../research/agent-window-ownership-2026-09-12.md).
- Theme: ten dark/light alternations preserve manually arranged state and focus; owned values restore after injected failures; later user edits survive.
- Recovery: native access within two seconds of a single worker/host death or detected terminal replacement failure, including during blocking lifecycle calls. Record fault-to-detection and detection-to-restoration separately. Hung actors revoke takeover on bounded measured lease expiry. Simultaneous host/worker loss requires working independent panic and next-login recovery; it has no false automatic two-second guarantee. Later user taskbar edits survive normal off/uninstall.
- Ownership: zero duplicate supervisors/guards, orphan widgets or forgotten live components reported stopped.
- Docks: exact expected bars and bounds/reserves, exactly one primary tray, zero extra popup reserve after each settled topology event.
- Runtime: no startup CDN/transpiler dependency; no per-monitor status subprocess loop or console flashes. Compare total backend/WebView2/private memory and CPU against R0 on one/three monitors. Investigate monotonic growth in a multi-hour soak.
- Usability: no wrong-profile focus, misplaced modal dialog, silent key failure or taskbar dependence without a documented working replacement.

Measure timings with monotonic timestamps and state observations; visually confirm taskbar/focus/controls. PID liveness, API success and positive reserved pixels are supporting data, not proof of readiness.

## Review rounds

1. **Package review:** author checks the diff against the package contract; independent reviewer challenges correctness and scope before integration.
2. **Cross-package review after R3/R4:** check app identity, reload behavior, catalog ownership, shared settings journal, readiness and fail-open behavior together. Re-test affected earlier cases after fixes.
3. **Release review after R6:** a fresh reviewer runs representative user workflows and examines failure evidence without relying on the author's completion narrative.

Documentation/link/source review can close a planning deliverable only. Release review needs executable results and real screenshots.

## Blocking defects

Data/settings loss, privileged-path replacement, trapped input, wrong-profile routing, modal windows displaced from parents, hidden taskbar without usable replacement/fallback, false shutdown success, duplicate ownership, unsupported claimed theme targets and failed uninstall restoration block v1.

Cosmetic exceptions may be accepted only with the exact defect, affected environment and reason recorded. A missing core workflow is not cosmetic. If taskbar suppression cannot pass, keep native access and leave that gate open rather than declaring it complete.
