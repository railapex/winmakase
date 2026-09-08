# P4 — Supervisor correctness and taskbar retirement

Status: not started. Lifecycle fixes may start after P0; final takeover depends on P1/P2 workflow coverage and protected deployment layout. Exit: R4.

## Keep, fix, remove

| Area | Action |
|---|---|
| Adoption, handles, bounded backoff, logs, reload, display/session-end events | Keep |
| Pair failure semantics; bar startup rollback; failed termination accounting | Fix against explicit state contract |
| PID singleton and overwriteable control slot | Replace with owned synchronization and request completion |
| Volatile taskbar baseline / takeover before readiness | Replace with durable ownership and readiness gate |
| Hard-crash restoration | Add minimal independent guard; share executable if practical |
| Dock repair | Verify actual widgets/reserves; retain bounded restart fallback until direct reconciliation passes |
| 50% available-commit gate | Reproduce and remove or bound; adoption bypasses new-start pressure wait |
| Display-change pair bounce | Keep observation; remove unused bounce option if no reproducible need |
| Edge-hover peek / foreground flyout list / delayed hide | Delete after replacements and suppression experiment pass |

The source review found these risks; this package must reproduce relevant behavior and test fixes. Do not describe every finding as a live incident.

## Implementation slices

### A. Lifecycle correctness

Write a native/starting/ready/degraded/stopping transition table first. On either input-pair member's failure, restore native access immediately and independently of survivor cleanup. Start cleanup alongside restoration; keep failures tracked and block pair restart until termination is confirmed. Keep healthy pair members running on bar/launcher failure. Initial Zebar failure must not roll back a healthy pair.

Use a real per-session singleton, with ownership lifetime rather than a stale PID check. Keep processes tracked until termination is confirmed; expose cleanup failure. Adopt only exact owned identities, avoiding PID reuse and unrelated instances.

Rare lifecycle requests can retain simple files: atomically claim requests, use unique tokens and per-request results, and prevent timeout cleanup from deleting another request. Test concurrent down/reload, supervisor replacement and stale results. Choose a pipe only if this cannot meet the bounded contract.

### B. Durable native restoration

Persist original taskbar preference and ownership generation before any mutation, including emergency showing, exactly once per takeover. Adoption/restart must reuse the baseline. Normal off/uninstall restores values still equal to our last write; preserve later user edits and report conflicts. Temporary emergency visibility is separate from preference restoration.

A small user-level native restore host, preferably a mode of the existing executable, is the single scheduled logon owner and launches the worker. Task Scheduler restarts the host only. The host handles worker loss; the worker handles host loss and refuses further takeover until ownership is re-established. Neither actor recursively respawns the other. Keep recovery outside Zebar/Run.

Use a generation/lease, clean-exit acknowledgement and stale-cleanup rejection. Retain process handles/creation identity and prove task-instance ownership before cleanup; path/PID alone is insufficient. Do not start a new generation while old cleanup is unresolved. Host readiness is mandatory before takeover. Protect elevated cleanup with P5's layout.

Never treat user-writable journal fields as authority for an arbitrary elevated path, command, task or process kill. Privileged cleanup validates protected installed ownership and actual task instance. Emergency visibility does not wait for the normal configuration/theme/distribution mutation lock.

Restoration handling must run independently of blocking task start/discovery/stop and reload calls. Loss of actor responsiveness expires readiness, even when a PID survives. Define the measured lease timeout and fault-to-detection budget in P0; the two-second target is not credible if restoration waits behind a five-second task call. On missing Explorer, failed task cleanup or an unavailable panic helper, report the precise degraded state, keep takeover revoked and retry only within bounded policy; do not claim recovery from a wedged Windows session.

A simultaneous worker/guard failure is tested and documented separately; independent native panic and next-login recovery remain required. Do not build guards for guards or promise automatic two-second repair after all restoration actors die.

Share restoration behavior with panic so enumeration and cleanup cannot drift. Panic remains reachable without Caps, Glaze or Zebar. Shutdown only reports success after owned termination and restoration complete.

### C. Readiness and suppression

Keep native taskbar available while starting. Ready requires responsive keyboard/tiler, reachable launcher, expected bar widgets/geometry, functioning tray and recovery routes. Specify bounded probes; a process or positive top reserve alone is insufficient.

On loss of bar/launcher, restore native taskbar while the healthy pair continues. On pair loss, restore immediately while cleaning up the survivor; retry only after confirmed cleanup. Terminal failures stay visible and bounded.

Use P0's smallest passing suppression method. Remove all hover/peek/flyout exception behavior after the P2 replacement matrix passes. Explorer remains running. Reconcile TaskbarCreated and topology/DPI events with fresh geometry; never rely on ABM_SETSTATE's return value as visual proof.

Zebar's tray backend creates its own Shell_TrayWnd and broadcasts TaskbarCreated. Identify actual Explorer-owned taskbars before suppressing/restoring; class names alone are insufficient. Treat the notification as a request to re-observe, not an unconditional bar restart. Test synthetic and real shell events without a feedback loop or damage to the tray provider.

Validate the HWND's actual owning process/image/session and re-resolve after Explorer recreation; never trust a stale HWND/PID or filename alone. Debounce repeated broadcasts, inspect actual changed state and act idempotently. Repeated synthetic notifications cause zero TraySpy suppression and zero component restart loop. Restoration does not broadcast TaskbarCreated itself.

### D. Retire temporary compensations

Measure pinned Glaze's allocation failure at startup/wake. Remove the pressure workaround if the pinned build no longer needs it; otherwise cap its wait, report the reason and distinguish available process commit from system-wide usage. Repeated failure yields native fallback, not permanent hidden startup.

Prove expected Zebar widget ownership and exact bounds/reserves after startup and monitor changes. Prefer direct dock reconciliation if supported; retain a bounded restart fallback with evidence until it is. Avoid pair restarts for healthy topology changes.

Leave ordinary process polling alone unless measurements show a problem. Removing code is justified by a passing replacement/removal test.

## Verification

R4 pure tests: each startup failure, either pair member failing, simultaneous pair/bar failures, failed kills, duplicate startup, concurrent requests, expired request ownership, invalid reload preserving current config, interrupted baseline writes and restart/adoption.

R4 isolated Windows: kill or suspend worker and host separately during task start/discovery, reload, shutdown and every takeover boundary; kill components, stall launcher/widget/provider, restart Explorer, alter DPI/primary, sustain commit pressure and exhaust retries. Test update/restart during old cleanup, simultaneous logon/interactive start, Task Scheduler/action failure, unavailable/replaced panic target, missing Explorer, and logoff during takeover. Test original taskbar visible and originally autohidden, later user preference edits, and Start/tray/notification/fullscreen interactions.

Proposed gates: native access within two seconds of single worker/host death or detected terminal replacement failure; zero duplicate owners; no living tracked process reported stopped; no bar-only failure restarts healthy pair; no hidden taskbar without usable replacement or fallback; original taskbar preference restored on normal off/uninstall only where still owned, with later edits preserved. Record detection latency separately from restoration latency and test during blocking lifecycle calls. Hung actors revoke takeover on bounded lease expiry. Simultaneous actor death uses the explicit panic/next-login recovery gate.

Run destructive cases in a disposable OS. R6 later validates normal day-to-day transitions on the authorized dogfood machine.
