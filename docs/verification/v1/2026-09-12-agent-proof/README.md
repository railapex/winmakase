# A1 native ancestry prototype

Status: bounded algorithm and native fixture proof passed. This does not pass A1, provider recognition, HWND ownership, placement or background-input coverage.

## Result

The conservative lineage algorithm worked for the tested identity and retention cases. It recognizes only configured existing executable paths, keys every retained node by PID plus creation time, retains only validated parent identities, revalidates the queried owner on every cache hit and returns unknown when history or access is missing. Dynamic retention and cached answers share one fixed node cap.

The native miss path is evidence against running the full scan unconditionally on every window event. In the final release benchmark, two Toolhelp PID/PPID snapshots dominated a three-node capture: 58.1283 ms median and 76.2888 ms p95 end to end. Exact-chain image and creation queries were 0.1127 ms median. After the root and intermediate exited, retained ancestry classified the surviving leaf in 200/200 cache-hit queries; owner creation revalidation was 0.0013 ms median.

This supports retained observation plus cheap cache-hit validation. It does not establish which process-start observer can feed the cache reliably or whether a miss can complete in the same Glaze Manage pass. PPID plus an older parent creation time still cannot restore an edge that was never observed.

## Implemented contract

- `agent_ancestry.rs` is an unwired prototype module. `AgentRootPaths` accepts only existing absolute `.exe` files and compares normalized full paths. It rejects known generic shell, runtime and terminal-host executable names even when configured; caller-side provider validation must reject any additional broker. This proof configured only its copied fixture root.
- `ObservedProcess` carries the child identity and an optional already-validated parent identity. Missing/inaccessible parents have no edge. A parent newer than its child is rejected as a reused PID.
- `AncestryTracker` follows retained creation identities to a recognized root. A gap, stale child identity, inaccessible owner, cycle, evicted node or reused PID yields `Unknown`.
- Cache lookup first reopens the queried PID for its current creation identity. Unknown identities are not cached, so arbitrary misses cannot grow memory beyond the retention cap.
- Native capture takes an initial PID/PPID-only Toolhelp snapshot, queries image and creation data only for the exact requested chain, takes a verification PID/PPID snapshot, reopens every queried identity and requires the child edges to agree before retention. Global process paths, commands and environment were neither collected nor printed.
- The existing supervisor image-path lookup still uses its basename prefilter plus exact full-path check. The shared Toolhelp enumerator was refactored without changing that contract.

## Deterministic coverage

Eleven focused tests cover:

- complete observed root/bridge/leaf classification;
- exact configured path versus same-basename and generic-shell negatives;
- rejection of a configured exact `cmd.exe` path as a generic root;
- missing and inaccessible parent history;
- a reused parent PID with a different creation identity;
- a currently newer parent identity rejected during observation;
- retained classification after parent and root exit;
- cache-hit owner revalidation, inaccessible owner and reused owner PID;
- bounded node/cache memory and loss of classification after required history is evicted;
- a native self-root capture using the real Toolhelp and process-time path.

The hidden native benchmark used two copied fixture executables: one configured provider root and one bridge/leaf image. The observer launched the root with `CREATE_NO_WINDOW`; root, bridge and leaf were the only processes whose paths/creation identities were queried. Twenty-five cold captures passed with three observed nodes and 25 cache misses. The fixture then exited root and bridge while leaving the leaf alive; 200 retained cache-hit classifications passed. Fixtures exited and the temporary directory was removed.

Exact release timings are in [benchmark.json](benchmark.json). The benchmark reports distributions in nanoseconds and contains no raw PIDs or global process data.

## Verification

- `cargo test -p winmakase agent_ancestry -- --nocapture`: 11 passed, 0 failed.
- `cargo run --release -p winmakase --example agent_ancestry_proof`: 25 captures and 200 retained cache hits passed; timings recorded in `benchmark.json`.
- `cargo test -p winmakase --lib`: 130 passed with one real-taskbar test ignored, including all four existing exact-path process-lookup regressions.
- `cargo test --workspace`: an earlier library phase passed 129 tests with one real-taskbar test ignored before the final generic-root test was added. The supervisor integration phase was environment-blocked: 18 fixture-start cases failed after the host's existing commit-pressure gate refused fresh Glaze fixture starts. A serial rerun reproduced the gate; its own supervisor log said `Windows commit pressure is above 50% — delaying GlazeWM until headroom returns`. This is not a test pass.
- `cargo clippy -p winmakase --all-targets -- -D warnings`: blocked by three pre-existing warnings in untouched `supervisor.rs` and `taskbar.rs` (`collapsible_if`, `new_without_default`). `cargo clippy -p winmakase --all-targets -- -D warnings -A clippy::collapsible_if -A clippy::new_without_default` passed, preserving strict checks for the changed code while allowing only those known baseline classes.
- `cargo fmt --all -- --check`: blocked by pre-existing formatting drift in untouched `taskbar.rs`. The changed Rust files were formatted directly with Rustfmt.

Requested worker routing was `gpt-5.6-sol` at high effort. Actual model metadata was not exposed to this worker runtime, so no substitution claim is made.

## Limits and next decision

No provider, terminal, browser, CUA, HWND, Glaze, guest, task, service or live configuration was launched or changed. The proof does not cover process-start event loss/order, short-lived unseen intermediates, observer restart, two simultaneous real agents, warm brokers, detached provider children, console HWND ownership or placement latency. Those remain A1 cases.

The failed workspace run left one reported test stub, PID 10916; it was stopped explicitly. A later exact check found neither PID 10916, the serial runner PID 60720 nor the fixture-specific `winmakase-adoptee` image running. The Friday PreToolUse hook rejected recursive removal of `D:/temp/windows/winmakase-test-110912-2-adopt-bin` and `D:/temp/windows/winmakase-test-60720-1-start-failure` even after both paths were resolved beneath the temp root. Those two scratch directories remain; they contain only test output and a copied fixture executable.

The next tracker experiment should populate this bounded identity graph before window admission and use the 1.3 µs median cache-hit path during admission. A cold two-snapshot miss can remain conservative and return unknown, or run outside the immediate placement path. Selecting an event source or integration seam still needs evidence; this prototype does not justify WMI, a Job Object or production policy wiring.
