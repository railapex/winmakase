# Minimum reliable agent process tracking

## Verdict

First prove a **native Toolhelp snapshot plus retained, creation-validated lineage**. Winmakase already has a snapshot path, and configured full-path-validated Codex/Claude roots are candidates for discovery without a launch wrapper. This candidate aims to avoid false attribution by returning unknown when a complete observed chain is unavailable; snapshot/open races and cached-edge validation remain unproved. Recall is intentionally partial: an exited, previously unseen intermediate makes that window unknown.

Escalate only demonstrated gaps to a per-root named Job Object. Jobs give stronger retention, but require launcher cooperation for race-free enrollment and introduce compatibility questions absent from ancestry observation. Neither option needs a generic service.

## Proposed identity and query

Enrollment records `{root_pid, root_creation_time, provider}`. At enrollment, take one Toolhelp snapshot containing only PID and PPID and open candidate roots with `PROCESS_QUERY_LIMITED_INFORMATION`. Key a node by `{pid, creation_time}` only when its chain is examined. At `EVENT_OBJECT_SHOW`/management:

1. `GetWindowThreadProcessId(hwnd)`; reject an invalid/changed HWND or event-thread mismatch.
2. `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` for the owner and retain the handle through classification. `GetProcessTimes` supplies its creation identity.
3. If the node is not cached, take a fresh native snapshot and walk PPIDs. Accept only a chain reaching an enrolled root whose creation identity still matches. Validate each live parent creation time; use a retained parent only when its recorded child edge was observed while both identities were valid.
4. Cache the owner-to-root result. Any missing, reused, inaccessible, or changed identity means unknown and preserves placement.

The prototype would run this query in the same handling path as Glaze's Manage rules, avoiding reliance on WMI/WinEvent ordering. No such integration exists. Its same-event cost is unmeasured; the next proof must time the existing native snapshot rather than infer from WMI. Revalidate creation identity on cache hits too; test PID reuse between HWND lookup and process open, and between snapshot and ancestor open. Retained edges can preserve already-observed descendants after parent/root exit if validated correctly. Persisting the cache is optional for the first proof; after observer restart, reconstruct only live complete chains and leave the rest unknown.

Mid-session attach is supported when the root and a complete live/retained chain exist. It cannot recover prior history. Pre-existing mixed human/agent browser or terminal brokers remain ambiguous. A known exclusively-agent app server may itself be explicitly enrolled; metadata alone is insufficient.

## Alternatives

| Mechanism | Finding |
|---|---|
| HWND-time parent walk | Smallest code, but wrong after parent exit or PID reuse. Creation times reject a reused parent; they cannot reconstruct a missing bridge. |
| Native Toolhelp snapshot plus retained cache | Chosen first proof. Winmakase already has `CreateToolhelp32Snapshot` in `crates/winmakase/src/procs.rs`; extend entries with PPID and validate live PIDs with `GetProcessTimes`. Any missed short-lived bridge is unknown. Native snapshot cost was not measured because this probe did not enumerate live processes. |
| WMI start trace + snapshot | Cross-stream ordering against WinEvent is unspecified. On this host the filtered temporary subscription was denied to the standard token, and exact-PID WMI snapshots took about 0.86–0.91 s. This method/account result is not a universal WMI claim. |
| ETW process starts | Kernel process events are suitable data, but starting a SystemTraceProvider session as non-admin requires profile privilege and provider/session ACL grants. That is too much privilege/setup for the first proof. |
| Job Object | Best fallback if A1 exposes unacceptable missing-lineage cases. Kernel membership survives parent exit and is queryable at the window event. It requires launcher cooperation for a race-free root and cannot attribute work brokered by an already-running process. |

## Rights and compatibility limits

If needed, create a named job with no limits or `KILL_ON_JOB_CLOSE`; associate the root before it executes using `PROC_THREAD_ATTRIBUTE_JOB_LIST` or create suspended, assign, then resume. Creating a job uses the creator-token ACL. `AssignProcessToJobObject` needs `JOB_OBJECT_ASSIGN_PROCESS` plus `PROCESS_SET_QUOTA | PROCESS_TERMINATE`; `IsProcessInJob` needs process query-limited and job query rights. No elevation requirement is documented for same-user processes, but prove it under the supported token. A named job can be reopened after observer restart while members remain. Never terminate it.

Windows 8+ supports nested jobs, but assignment can fail for incompatible existing job hierarchy/UI limits. With no breakaway allowance, a program requesting breakaway may fail to launch; allowing breakaway admits attribution escapes. `Win32_Process.Create` children explicitly do not inherit job membership. ConPTY itself has no documented Job Object incompatibility, but its process topology is not proof. Browser sandbox jobs, broker reuse, explicit breakaway, and authentication handoff all remain unproven.

Job completion-port process notifications are optional diagnostics, not authority: Microsoft says delivery is not guaranteed and warns of PID/race issues. Query membership on the live HWND owner instead.

## Bounded local probe

Host: Windows build `26200`, PowerShell 7.6.6, non-elevated. The exact filtered event core was:

```powershell
$src='WinmakaseOwnProbe-20260912-sol-wmi'
try {
  $q="SELECT * FROM Win32_ProcessStartTrace WHERE ParentProcessID = $PID"
  Register-WmiEvent -Namespace root/cimv2 -Query $q -SourceIdentifier $src -ErrorAction Stop
  $child=Start-Process $env:ComSpec -ArgumentList '/d','/c','ping -n 3 127.0.0.1 >nul' -WindowStyle Hidden -PassThru
  Wait-Event -SourceIdentifier $src -Timeout 5
} finally {
  Remove-Event -SourceIdentifier $src -ErrorAction SilentlyContinue
  Unregister-Event -SourceIdentifier $src -ErrorAction SilentlyContinue
  if($child -and !$child.HasExited){Stop-Process -Id $child.Id -Force}
}
```

Windows PowerShell 5.1 returned `Access denied` at registration, before launch. PowerShell 7 `Register-CimIndicationEvent` returned `Call cancelled`. Separate filtered `Get-CimInstance Win32_Process -Filter "ProcessId = <own hidden child>" -Property ProcessId,ParentProcessId,CreationDate` matched 3/3 at 913.8, 885.2, and 857.5 ms. Exact-own-PID native `OpenProcess` + `GetProcessTimes` succeeded 100/100 without elevation: median 0.0065 ms, p95 0.0187 ms, max 5.6147 ms.

## First proof

Extend the existing snapshot helper in a disposable prototype to retain PPID and creation identity, without logging global entries. Enroll validated Codex/Claude/Copilot fixture roots already running; exercise direct/grandchild native windows, a short-lived bridge, root exit, attach mid-session, observer restart, two agents, and a human negative. Time snapshot-to-answer inside the WinEvent path and record whether ownership is ready for that same Manage pass. Test one dedicated browser and ConPTY topology. If supported cases classify correctly and quickly, stop. Trial a launch-time Job Object only for observed misses; assignment failure, broker reuse, or escaped children remain unsupported.

## Primary sources

[Window owner](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid) · [PID/parent reuse](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-process) · [process creation time](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes) · [Toolhelp snapshots](https://learn.microsoft.com/en-us/windows/win32/toolhelp/snapshots-of-the-system) · [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) · [nested jobs](https://learn.microsoft.com/en-us/windows/win32/procthread/nested-jobs) · [creation-time job list](https://learn.microsoft.com/en-us/windows/desktop/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute) · [assignment rights](https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-assignprocesstojobobject) · [membership query](https://learn.microsoft.com/en-us/windows/win32/api/jobapi/nf-jobapi-isprocessinjob) · [job notification races](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_associate_completion_port) · [WMI start trace](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/krnlprov/win32-processstarttrace) · [ETW SystemTraceProvider privilege](https://learn.microsoft.com/en-us/windows/win32/etw/configuring-and-starting-a-systemtraceprovider-session) · [WinEvent ordering](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook)
