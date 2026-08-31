# Handoff — WinMakase window policy W1

Created: 2026-08-30
Repo: `D:/dev/winmakase`
Next cut: W1 only — app definitions and generated workspace-home rules

## 2026-08-31 correction

The original Chromium gate inspected only stock Glaze's process/class/title
DTO and incorrectly generalized that absence to Windows. A same-session HWND
property-store probe found stable AppUserModelIDs: `Chrome` for Default,
`Chrome.UserData.Profile1` for Profile 1 and `Chrome.ember.Default` for an app
window. W1.1 adds a thin Glaze `window_app_id` matcher plus IPC exposure and
hardens unknown TOML keys, non-exact base overlap, explicit config paths and
manage-only home events. Live acceptance used an isolated Chromium profile:
exact AppID match, first state `floating` on hidden workspace `7`, four daily
windows unchanged, W0 YAML restored. WinMakase v0.1.5 and the unsigned
`ui_access=false` Glaze build plus watcher are deployed from
`~/.winmakase/bin`; no certificate install occurred. Glaze config reloads and
restarts replay `manage` rules against existing HWNDs. The historical
instructions below remain the W1 record; `docs/plans/winmakase-window-policy.md`
owns the corrected design.

## Resume state

W0 is complete, committed and live. WinMakase v0.1.3 renders the complete Glaze config from the portable keymap, machine-local keymap overrides and a binding-free machine-local Glaze base. The rendered source and live config have the same SHA-256: `3B2250505076453EF2521B3BD9A71E3EBD991DBA0F19D67C22C0A62AD58FAC36`.

Live stack at close:

- WinMakase supervisor, stock GlazeWM 3.10.1, kanata and Zebar are running with zero restarts.
- Zebar's display-churn hole is closed: every settled display-change burst re-arms the delayed dock check. Fresh v0.1.3 live proof registered three appbars and produced `[40, 40, 40]` reserves.
- W0 physical pass is green: Caps workspace navigation, reflow, float/layer cycle, scratch banish/summon, move+follow and silent move. Alt+Tab and physical left-Win+Tab remain native.
- Workspace map is `1/4 -> Dell`, `2/5 -> ultrawide`, `3/6 -> DualUp`; `7` is the occasional-display fallback and `scratch` is last/unbound.
- Main is clean. W0 is `0c8374f`; the separate parked Caps spike plan is `22c115c`. Pushing main remains Chris's button.
- Baseline verification: 164 tests pass; two real-task/taskbar tests are deliberately ignored. `cargo fmt --all -- --check`, Clippy with warnings denied and `git diff --check` pass.

## W1 outcome

Add first-class app definitions and compile their manage policy into deterministic Glaze `window_rules`:

- Ordinary app windows still tile by default.
- An app may match by process plus optional class/title refinements.
- An app may have a machine-local workspace home and initial state.
- Placement happens when Glaze manages the new window; no visible move-after-open helper.
- Portable defaults contain examples and identities, not Chris's numbered workspace map.

Proposed user shape from the plan; freeze names only after round-trip tests:

```toml
[apps.browser]
launch = 'chrome --profile-directory=Default'
process = 'chrome'
workspace = '2'
state = 'tiling'
```

`~/.winmakase/config.toml` is the intended home for these definitions. Do not confuse it with `~/.winmakase/keymap/local.toml`: that separate file only personalizes key bindings and app command placeholders today.

## First design gate — one `window_rules` owner

W0's machine-local `~/.winmakase/glazewm/base.yaml` already contains the Zebar ignore rule under top-level `window_rules:`. `render_glazewm_config()` currently appends only generated `keybindings:` and rejects a competing bindings owner.

W1 must not append a second top-level `window_rules:` key. Establish one deterministic composition path that:

1. preserves the base's narrow host/system rules, including Zebar ignore;
2. emits generated app-home rules after decisive ignore/dialog exceptions and before nothing broader that can undo them;
3. rejects ambiguous ownership or malformed base input loudly;
4. leaves no fact only in the deployed YAML;
5. preserves comments only if the chosen representation genuinely supports them — do not build a fragile line splicer to save comments.

Relevant Glaze behavior, source-verified against stock 3.10.1:

- Match fields inside one match object are AND; objects in a rule's `match` list are OR.
- All matching rules run in config order.
- `ignore` detaches the window and ends further rule processing for that window.
- Rules run during manage before normal use, which is the right placement seam.

The implementation may parse/model the base or move non-app rules into a typed source, but it must keep one owner and preserve W0's source/live contract. Decide from the actual data shape; do not add a generic YAML merge language.

## Code seams

- `crates/winmakase/src/config.rs`
  - `Config` owns supervisor, components, keyboard and scratchpads.
  - Add the app map and typed app/state/match definitions here unless evidence points elsewhere.
  - Reuse scratchpad matching lessons, not its type: app homes and scratchpads have different lifecycle policy.
- `crates/winmakase-keymap/src/lib.rs`
  - `render_glazewm_config()` owns complete Glaze composition after W0.
  - Keep rendering deterministic and pure enough for golden tests.
- `crates/winmakase/src/main.rs`
  - `KeymapAction::Render` / `keymap_cmd()` load local keymap and optional base.
  - Wire app definitions into rendering without making a second config-loading convention.
- `C:/Users/chris/.winmakase/glazewm/base.yaml`
  - Machine-local workspace order, monitor pins and current Zebar ignore source.
- `C:/Users/chris/.winmakase/config.toml`
  - Machine-local product config; currently contains the stack and named scratchpad.
- Glaze reference: `D:/dev/glazewm-wt-caps-leader/resources/assets/sample-config.yaml` and `packages/wm/src/user_config.rs` / `commands/window/run_window_rules.rs`.

## Validation contract

At minimum reject:

- an app without a launch command;
- an app without any process/class/title match field;
- empty match strings;
- unsupported state values;
- a workspace not declared in the Glaze source being rendered;
- state/workspace combinations the generated command sequence cannot honor;
- ambiguous duplicate ownership of `window_rules`;
- overlapping definitions whose ordered result is contradictory, unless the order is explicit and covered by a test.

Keep state an enum. The expected v1 set is `tiling`, `floating`, and `ignored`; prove command order and reject nonsense such as an ignored app with a workspace home if Glaze cannot honor both.

## Tests

Add round-trip/parser tests plus deterministic YAML goldens for:

- tiled app with a workspace home;
- floating app with class/title refinement;
- ignored bar/system window preserved ahead of app rules;
- overlapping narrow and broad rules;
- invalid/missing match fields, state and workspace;
- base/config ownership conflicts;
- no apps, which must reproduce the exact W0 live render byte-for-byte.

Do not use arbitrary daily windows as fixtures. The W0 temp suite was reversible but brittle when its chosen Chrome window moved. Build or reuse a disposable uniquely named test window, snapshot every reachable Glaze window state, and never involve `target/debug/muxel-live.exe` directly or as a focused fallback. Do not use Windows 11 Notepad: it restores saved user documents/windows.

## Chromium identity gate

Before promising profile-specific homes, observe Glaze's actual `processName`, `className` and `title` for both Chrome profiles. `chrome.exe` alone does not identify a profile. If no stable marker survives normal navigation/title changes, explicitly leave profile homes unsupported in W1. Launch commands may remain profile-specific; manage rules must not pretend they can correlate the resulting HWND.

## Live acceptance

1. Render to staging and diff before touching the live config.
2. Preserve every W0 machine fact and keep `config.yaml.pre-w0-20260830` available.
3. Add one machine-local disposable app home.
4. Reload through Glaze's existing path.
5. Launch the disposable app and prove it appears directly on the configured workspace in the configured state.
6. Prove Zebar remains ignored and all three 40px reserves remain.
7. Prove the W0 rendered config is unchanged when no apps are configured.
8. Restore/remove the disposable definition and fixture unless it is a real chosen app home.

Exit: a test app opens directly in its configured workspace/state; source, staged output and live YAML agree; tests and real-rig acceptance are green.

## Guardrails

- W1 only. Do not implement W2 `focus-or-launch` or retarget app keys yet.
- App homes are manage rules, not persisted pixels, slots or layout geometry.
- Numeric homes remain machine-local.
- Normal windows tile. Repeated overlays are scratchpads; loose floats are exceptions.
- Do not intercept Alt+Tab or left-Win shortcuts.
- Do not pull in Caps ownership, kanata removal, games, Glazemakaze, Groupy, theming, bar features, layouts or the switcher.
- Use graceful `wm-exit`; never force-kill a live Glaze test process. A prior forced test kill stranded loopback port 6123 behind a dead PID until Windows aged it out.

## Fresh-session prompt

> Work in `D:/dev/winmakase`. Read `CLAUDE.md`, `docs/TODO.md`, `spike/NOTES.md` § LIVE MACHINE STATE, `docs/plans/winmakase-window-policy.md`, and `docs/plans/handoff-w1-app-homes.md`. Execute W1 only: add typed app definitions and deterministic generated Glaze window rules, settle one-owner composition with the base's existing Zebar rule, keep numeric homes machine-local, and observe Chrome profile identity before claiming support. Stage/diff before live reload; use a disposable fixture and never touch muxel-live. Commit only after tests plus real-rig first-managed-state acceptance pass. Do not start W2 or any parked track.
