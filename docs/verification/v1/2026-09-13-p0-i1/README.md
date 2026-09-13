# P0.I1 — guarded PowerToys Run activation

Verdict: **accepted for the bounded I1 slice**. Winmakase 0.1.8 now owns one open-or-focus action used by the generated launcher chord and Zebar button. The final release binaries passed source review, focused tests and a fresh disposable Windows Sandbox replay. No daily-stack cutover occurred.

## Accepted behavior

- The adapter admits one exact PowerToys Run process in the current logon session and authentication LUID from the supported machine or per-user install paths. It retains the process handle and creation time, requires one exact launcher window, and observes visible/uncloaked and foreground state before acting.
- A visible focused launcher returns without signalling. A visible unfocused launcher gets one focus attempt. A hidden launcher gets one native invoke-event signal. A per-logon mutex serializes callers; an uncertainty marker is written before signalling and remains until the process generation changes or a confirmed state clears it.
- Foreground is captured before mutex acquisition. If the user changes destination while a request waits, the adapter refuses to mutate focus.
- `winmakase-run.exe` is a Windows-GUI-subsystem helper, so the key and bar routes do not create a console window. The CLI retains structured JSON outcomes for diagnosis.
- `keymap render --super-key caps-lock` emits the patched GlazeWM Caps leader and translates the abstract `SUPER` grammar to `caps_lock`. The default remains `right-win`, preserving the current Kanata path.
- The stock launcher entry calls the exact helper with no arguments. Zebar's permission is restricted to that executable and an empty argument string; its handler rejects resolved nonzero exits and shows a short error state.

## Runtime results

The final release hashes and pinned dependency hashes are in [provenance.json](evidence/provenance.json).

- Hidden open: `opened`, one signal, 31 ms. A second focused call returned `already_focused`, no signal, 13 ms, and preserved the exact `winmakase-proof` query.
- Two concurrent CLI callers produced one `opened` result and one `already_focused` result against the same PID/HWND; exactly one caller signalled. The measured calls were 34 ms and 13 ms.
- The GUI helper exited 0 and left the one launcher visible, foreground and keyboard-focused. With the PowerToys process stopped, the CLI returned structured `not_running` with `signalled: false`.
- A real pixel click on the rendered Zebar magnifier opened Run. The guest then observed one visible/uncloaked launcher, its HWND as foreground, and keyboard focus on `QueryTextBox`. Compare [before](evidence/bar-before-click.png), [after](evidence/bar-after-click.png), and the guest-native [window/focus record](evidence/bar-launcher-observation.json).
- The direct-Caps render contains `general.keybinding_leader.key: caps_lock`, `tap_max_ms: 180`, `caps_lock+space` for the launcher and no standalone Caps binding. See [direct-caps-render.yaml](evidence/direct-caps-render.yaml).

The primary adapter replay is [launcher-proof.json](evidence/launcher-proof.json). [run-start.json](evidence/run-start.json) records the pinned per-user Run process, and [zebar-start.json](evidence/zebar-start.json) records the final bar process and pack.

## Rejected or open evidence

The first bar attempt ran with guest networking disabled. The production bar is still buildless and imports React, Babel and the Zebar API from CDNs, so only a blank widget loaded. The final interaction pass enabled guest networking and passed. This proves the I1 button route; it does **not** satisfy P2's offline-bar requirement. Vendoring the frontend and licenses remains required.

Zebar's webview did not expose the HTML button through Windows UI Automation. The UIA probe's zero-button result was rejected after the rendered magnifier was visible and the CUA pixel click plus guest-native focus observation agreed.

PowerToys Run hides immediately when another guest window takes foreground. A stable visible-but-unfocused launcher could not be held: an exploratory pre-final case observed the click-away hide, then the adapter reopened the same launcher and preserved `winmakase-unfocused-proof`. [visible-unfocused-proof.json](evidence/visible-unfocused-proof.json) retains that failed setup rather than calling the unexecuted focus-only branch passed.

Physical Caps input, fullscreen and elevated origins, focused-monitor placement, multi-monitor behavior and daily deployment remain open. P2 offline packaging and broader R0 interaction cases also remain open.

## Verification

- `cargo test -p winmakase-keymap`: 33 tests passed (17 unit, 14 app-rule, 2 real-keymap).
- `cargo test -p winmakase launcher`: 4 launcher tests passed.
- `cargo check -p winmakase --all-targets`: passed.
- `cargo clippy -p winmakase --all-targets -- -D warnings -A clippy::collapsible-if -A clippy::new-without-default`: passed. The two allows are existing workspace debt in untouched supervisor/taskbar code.
- `cargo build --release -p winmakase --bins`: passed.
- `llvm-readobj --file-headers target/release/winmakase-run.exe`: `IMAGE_SUBSYSTEM_WINDOWS_GUI`.
- Independent Astra review found five substantive races or boundary defects across two passes; all were corrected. The final foreground-capture review reported no remaining acceptance blocker.

The guest was stopped and the `env:winmakase-uat` Wire claim released after capture.
