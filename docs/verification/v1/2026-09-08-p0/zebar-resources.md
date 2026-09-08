# Zebar resource baseline

Captured 2026-09-08T23:14:26Z–23:14:56Z. This is a read-only observation of the current, uncontrolled host workload. It is not an idle benchmark.

## Result

The backend and its 14 connected descendant WebView processes remained the same 15 PID/start-time identities for all 31 samples. The 30 adjacent intervals contained 562.5 ms of valid aggregate CPU-time delta. Mean interval delta was 18.75 ms; p95 was 62.5 ms.

| Counter summed over backend and descendant WebViews | Minimum | Mean | p95 | Maximum |
|---|---:|---:|---:|---:|
| Private bytes | 1,249.0 MiB | 1,252.6 MiB | 1,260.2 MiB | 1,261.2 MiB |
| Working set | 427.6 MiB | 443.9 MiB | 455.0 MiB | 455.3 MiB |
| Process count | 15 | 15 | 15 | 15 |
| Descendant WebView count | 14 | 14 | 14 | 14 |

No counter read failed. No identity appeared, disappeared, reused a PID, or reset its CPU counter between adjacent samples. Those zero invalidation counts are observations for this capture only.

Raw evidence: [zebar-resources.json](zebar-resources.json). Sampler: [`spike/p0/measure-zebar-resources.ps1`](../../../../spike/p0/measure-zebar-resources.ps1).

## Context

The accepted P0 baseline observed three displays on 2026-09-08, all at 96 effective DPI, with `DISPLAY2` primary. This capture reused those facts as dated context from [baseline.json](baseline.json); it did not query or alter display topology, widgets, settings, or bar windows. It makes no one-monitor comparison and no fresh claim about bar visibility or dock state.

The configured target and accepted installed artifact referred to the same backend location before sampling. The evidence omits executable paths. No component was started, stopped, reloaded, focused, or otherwise controlled.

## Method

The script selected exactly one running backend by its exact executable name, fixed its PID and UTC start time, and aborted without writing evidence if that identity disappeared or changed. Each sample took one allowlisted `Win32_Process` snapshot, built the descendant graph from `ParentProcessId`, and retained only connected WebView descendants. `Get-Process` supplied cumulative CPU time, private bytes, working set, PID, and start time.

Thirty-one samples targeted one-second offsets from a monotonic stopwatch. Actual capture duration was 30,754 ms. Adjacent recorded sample spacing was 829–1,140 ms, mean 993.467 ms. CPU deltas require the same PID and start time in adjacent samples plus a nondecreasing counter. New, missing, reused, and reset identities are counted and excluded. The JSON retains all 465 per-process counter rows and all 30 interval-delta records.

Run from the repository root:

```powershell
pwsh -NoProfile -File spike/p0/measure-zebar-resources.ps1 -OutputPath docs/verification/v1/2026-09-08-p0/zebar-resources.json
```

The sampler emits roles, PIDs, parent PIDs, start identities, counters, and aggregates. It emits no window titles, command lines, executable paths, or unrelated process names.

## Limits

The CIM ancestry snapshot and `Get-Process` counter read are sequential, not atomic. Processes that live entirely between samples are absent. The valid CPU total excludes every missed lifetime and any identity/counter invalidated by the stated rules. This therefore cannot establish a total process-creation rate.

Summed working sets can count shared pages in more than one process. Private bytes and working set describe this process tree at these samples; they do not isolate GPU allocation or explain why memory is resident. A 30-second window cannot establish long-term growth, cold-start cost, or steady-state behavior under a controlled workload.

## Model metadata

Requested builder: `gpt-5.6-sol`, reasoning effort `high`. The execution surface did not independently expose actual model or effort; no substitution was reported. Lead allocation: Astra.
