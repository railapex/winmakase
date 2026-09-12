# Windows agent process roots: source pass

Status: source and static-install review on 2026-09-12. No provider was launched; no live process/window inventory or UI action was performed. Runtime behavior remains an A1 proof obligation.

## Confirmed launch shapes

| Provider/path | Source-confirmed Windows shape | Ownership consequence |
|---|---|---|
| Codex, standalone/native | This host resolves bare `codex` first to the installed OpenAI/Codex/bin/codex.exe. | Candidate native provider root; helper/re-exec modes need A1 verification. This is static command resolution, not an observed running session. |
| Codex, npm | Local `@openai/codex` 0.145.0 and upstream source both run `cmd -> node codex.js -> packaged codex.exe`; the JS parent waits for the native child. The TypeScript SDK resolves and directly spawns the packaged native binary. | Recognize the observed native child, or enroll the launch instance when stronger coverage is needed. `cmd.exe`/`node.exe` alone are not provider identity. |
| Claude, native/current npm | This host resolves bare `claude` to `~/.local/bin/claude.exe` 2.1.269. Anthropic says npm versions >=2.1.198 install/link the same native binary and do not invoke Node at runtime. | Direct executable is the provider root. |
| Claude, legacy npm | Installed stale npm 2.1.59 shim is `cmd -> node cli.js`. | Here `node.exe` running `cli.js` is the provider runtime. Do not classify arbitrary Node processes by image name; the known launch registration supplies identity. |
| Grok, native with local shim | Follow-up static check: this host resolves `grok` first to a local `grok.cmd` that invokes the user's `.grok/bin/grok.exe`; the native executable also resolves on PATH. Upstream publishes a Windows binary. | Candidate native provider root under the same recognition contract. The shim's account-specific alternate target is not a runtime observation. Helper/re-exec, shell/PTY children and broker behavior still need A1; no Grok process was launched or inspected. |
| GitHub Copilot CLI | Official Windows installs include WinGet/direct executable; npm is also supported. Copilot is absent here, and the npm internal chain was not verified. ACP stdio is a child tied to its client; ACP TCP is an independently-lived listener. | Cover native and shim launch roots generically; leave npm details to A1. TCP needs separate enrollment if it is exclusively agent-owned. |
| Local stdio MCP/tool server | MCP requires the client to launch a stdio server as a subprocess. Claude documents the same behavior. | Covered as a descendant, unless that server forwards work into another existing process. |
| HTTP/TCP MCP, existing browser/CUA/terminal broker, singleton activation | The agent sends a request to a process it did not create. Windows app activation can redirect work to an existing instance. | No ancestry edge from agent to resulting HWND owner. Agent-only brokers may be enrolled for coarse agent classification; mixed human/agent brokers need explicit session/window registration or remain unchanged. |
| Codex app-server daemon | Upstream documents a detached, reusable app-server daemon; clients can share it and inherit the daemon's original environment. App-server exposes multiple threads. | An agent-only daemon can support the requested one-Agents-workspace classification. Process ancestry alone cannot assign its descendants to an exact thread. A mixed-use daemon is ambiguous. |

## Mechanism

Use automatic validated native-root discovery first, with one fallback launch/enrollment seam independent of Muxel:

1. Watch process starts and recognize configured, validated native Codex/Claude/Copilot executable paths as agent roots. Store `{root PID, root creation time, provider hint, scope}`; `scope=agent` is enough for the first-cut Agents workspace. Winmakase need not launch these native processes.
2. Record every observed process-start edge as `(PID, creation time, parent PID, parent creation identity)` and retain bounded lineage after `cmd`/PowerShell/Node shims exit. Match HWNDs through `GetWindowThreadProcessId`. PID or image-name matching alone is unsafe.
3. Use explicit launch/session enrollment for ambiguous generic runtimes, missed history, or finer per-session attribution. Legacy `node.exe` requires a locally verified package entry/script identity or registration; never classify arbitrary Node processes by image name.
4. Resolve `.exe`, `.cmd`, `.bat`, and `.ps1` launch targets. Muxel commit `b9c9f12e3a751f0f60b7df21531cfea5e06c6169`, `crates/muxel-terminal/src/session.rs`, exposes the exact PTY child PID after resolving Windows executable/batch shims. It is one optional enrollment adapter. Other terminals can use the generic wrapper/API. Never enroll a terminal host broadly.

A late snapshot can classify a surviving validated native root and descendants whose complete chain remains observable. If a short-lived intermediary already exited, current parent fields cannot reliably reconstruct that edge; leave the broken lineage unknown.

Direct provider descendants (shell commands, native fixtures, SDK-started Codex, per-session stdio servers) are candidates for automatic placement. Services and scheduled Winmakase processes remain a separate ownership class. Unknown ownership keeps current placement.

## Required runtime probes

- For native Codex/Claude/Grok and installed Copilot: record only the bounded enrolled subtree's image name, PID/creation identity/parent, and resulting HWND owner. Verify direct native roots, legacy/current shims, child cmd/pwsh, detached child, short-lived intermediary, agent exit, and two concurrent roots.
- Probe console-host and terminal-broker ownership; a console HWND may not belong to the shell process whose output it shows.
- Compare per-session stdio with Codex daemon/Copilot ACP TCP and a warm browser/CUA broker. Prove agent-only coarse classification and zero capture of mixed human windows.
- Measure snapshot-plus-start-event ordering, loss, observer restart, and PID reuse before choosing WMI trace, ETW, or a Job Object. No disposable ancestry experiment was run in this pass.

## Sources

- Winmakase `c474786a1b058056711b9ee3b6ac974d16faf942`: `docs/research/agent-window-ownership-2026-09-12.md`, `docs/plans/v1/01-window-policy.md`.
- OpenAI Codex `727e48697d8e89a88c024cc6831bd4816ffd3b3e`: [`codex-cli/bin/codex.js`](https://github.com/openai/codex/blob/727e48697d8e89a88c024cc6831bd4816ffd3b3e/codex-cli/bin/codex.js), [`sdk/typescript/src/exec.ts`](https://github.com/openai/codex/blob/727e48697d8e89a88c024cc6831bd4816ffd3b3e/sdk/typescript/src/exec.ts), [`app-server-daemon/README.md`](https://github.com/openai/codex/blob/727e48697d8e89a88c024cc6831bd4816ffd3b3e/codex-rs/app-server-daemon/README.md), [app-server protocol](https://github.com/openai/codex/blob/727e48697d8e89a88c024cc6831bd4816ffd3b3e/codex-rs/app-server/README.md).
- Anthropic: [Claude Code setup](https://code.claude.com/docs/en/getting-started), [MCP transports](https://code.claude.com/docs/en/mcp).
- Grok follow-up, 2026-09-12: local `Get-Command grok*`, static `grok.cmd` contents and native file presence; [official Grok Build repository and Windows release installation](https://github.com/xai-org/grok-build#installing-the-released-binary). Native file version fields were empty; installed version and runtime topology remain unverified.
- GitHub: [Copilot CLI install](https://docs.github.com/en/copilot/how-tos/copilot-cli/set-up-copilot-cli/install-copilot-cli), [ACP server lifecycle](https://docs.github.com/en/copilot/reference/copilot-cli-reference/acp-server), repository `github/copilot-cli` at `1032cd05bee5d1f7950b82e873c91a6b1ddb5c69`.
- Microsoft: [`GetWindowThreadProcessId`](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid), [`Win32_Process`](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-process), [activation redirection](https://learn.microsoft.com/en-us/windows/windows-app-sdk/api/winrt/microsoft.windows.applifecycle.appinstance.redirectactivationtoasync).
- MCP specification: [stdio versus Streamable HTTP](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports).
