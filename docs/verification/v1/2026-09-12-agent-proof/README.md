# A1 sequence-feed ancestry prototype

Status: bounded algorithm and controlled native fixture proof passed after review cycle 1. This does not pass A1, automatic provider recognition, HWND ownership, placement or background input.

## Review correction

The first commit's retrospective Toolhelp walk was unsafe. Process creation times are UTC timestamps, not monotonic identities. If a true unseen parent exits, the clock moves backward and an unrelated configured executable reuses that PPID with an earlier timestamp, two matching Toolhelp snapshots plus timestamp ordering can still produce false agent ownership. `GetProcessTimes` can also succeed for an exited process object while another handle keeps it alive.

The corrected prototype removes retrospective timestamp ancestry entirely. It uses Windows 11 26100.4770+ `SystemBasicProcessInformation`, whose `SequenceNumber` is documented as unique and intended to detect PID reuse. Microsoft also describes this information class as faster and lower-memory than `SystemProcessInformation`. No chronological meaning is assigned to sequence values. [Microsoft `NtQuerySystemInformation`](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntquerysysteminformation)

## Safe feed contract

- Each complete observation retains only PID, PPID and unique sequence number. The parser does not dereference or retain the returned global image-name pointers.
- A child edge is admitted only when the child sequence identity is absent from the immediately previous complete observation, present now, and the same unique parent identity is present in both observations. The parent must already be a retained agent root or descendant.
- Existing ancestry at observer start, a parent and child first seen together, a missing/inaccessible parent and any failed/truncated/over-cap observation remain unknown. An observation failure breaks continuity; the next success is baseline only.
- Observer restart starts with no lineage. There is no persistence or reconstruction from timestamps.
- Root enrollment is an explicit fixture-owned PID in this proof. The PID binds once to its first observed sequence identity. The exact configured executable path must match while the process is live in two consecutive complete observations before the root is promoted. The helper checks process liveness before and after its exact image query.
- Cached answers key on PID plus sequence. Lookup against a supplied current snapshot rejects a missing or reused PID. End-to-end native lookup first captures a fresh sequence snapshot, advances the feed, then classifies.
- Retained nodes, cache answers, root enrollments, snapshot entries and native query buffer all have hard caps. Only descendants of retained roots/descendants enter the dynamic graph.
- Unsupported `SystemBasicProcessInformation`, malformed/truncated data and cap overflow fail closed. There is no Toolhelp or creation-time attribution fallback.
- Known generic shell, runtime and terminal-host executable names are rejected as configured roots. Automatic provider candidate discovery and package-entry enrollment remain open.

The existing supervisor lookup still uses its original Toolhelp basename prefilter plus exact full-path match. No production policy calls the new feed.

## Deterministic coverage

Twelve ancestry tests cover prior-parent admission, existing ancestry, same-observation parent/child, missing parent, observation gaps, PID reuse, exact root binding twice, retained edges after ancestor exit, bounded memory, generic-root rejection, native sequence capture and deliberately nonmonotonic sequence values. Nine process-helper tests include the existing supervisor exact-path regressions, native sequence identity for the current process, rejection of an exited-but-held hidden child, and parser rejection of truncation, PID overflow and entry-cap overflow.

The hidden native fixture stages each edge across separate observations:

1. Explicitly enroll the copied root PID and validate its sequence/path twice.
2. Signal the root to create the bridge, then observe the bridge edge.
3. Signal the bridge to create the leaf, then observe the leaf edge.
4. Classify the leaf, exit root and bridge, refresh, and classify the surviving leaf through retained edges.

The fixture queried image/liveness only for its explicitly enrolled copied root. It did not launch a provider or inspect any global image name, command, environment, HWND or `muxel-live` process/path.

## Release timings

The final release run passed two staged fixture edges, 200/200 cache lookups against an already supplied current snapshot and 50/50 end-to-end cache hits with a fresh sequence snapshot. One warm-up classification missed cache because a new owned descendant appeared and changed the graph; ownership remained agent and the cache repopulated.

| Operation | Median | p95 | Maximum |
|---|---:|---:|---:|
| Lookup with supplied current snapshot | 0.0001 ms | 0.0001 ms | 0.0001 ms |
| Fresh sequence snapshot | 0.5683 ms | 0.6891 ms | 0.7118 ms |
| Exact enrolled-root live image validation | 0.0514 ms | 0.0803 ms | 0.0955 ms |
| Feed update | 0.0434 ms | 0.0647 ms | 0.0718 ms |
| Fresh snapshot to answer | 0.6611 ms | 0.8114 ms | 0.8403 ms |

Exact nanosecond distributions are in [benchmark.json](benchmark.json). They contain no raw PIDs or global process data.

The 0.6611 ms median fresh-snapshot path is evidence against capturing a new global snapshot unconditionally for every Manage event. A caller that already owns the current observation can use the supplied-snapshot lookup; event cadence and batching still need integration proof.

## Verification and limits

- `cargo test -p winmakase agent_ancestry -- --nocapture`: 12 passed.
- `cargo test -p winmakase procs -- --nocapture`: 9 passed.
- `cargo test -p winmakase --lib`: 136 passed, 1 explicit real-taskbar test ignored.
- Scoped Clippy with the two known untouched lint classes allowed passed for all `winmakase` targets; `git diff --check` passed.
- Supervisor process-spawning integration remains environment-blocked by the existing host commit-pressure gate; it was not rerun in this review cycle.

No fixture process remains. The local hook rejected recursive cleanup of the first two paths below. Three failed example-run directories also remain; no bypass was attempted. Exact cleanup targets are:

- `D:/temp/windows/winmakase-test-110912-2-adopt-bin`
- `D:/temp/windows/winmakase-test-60720-1-start-failure`
- `D:/temp/windows/winmakase-agent-ancestry-102864-1789224733192744500`
- `D:/temp/windows/winmakase-agent-ancestry-56976-1789224803210753900`
- `D:/temp/windows/winmakase-agent-ancestry-94400-1789224786174435000`

The proof does not cover automatic provider discovery, real provider helpers, event cadence/loss, two simultaneous agents, detached descendants, observer restart recovery, warm brokers, HWND ownership or placement timing. This result supports the sequence-feed identity and edge rule on the controlled fixture. It does not establish production readiness or provider coverage.
