# P0 source binding comparison

Date: 2026-09-08

Winmakase base: `b73f48f52c50036673a8c77baaa705653e7a58e9`

Scope: source comparison only. No desktop process, input hook, task, Glaze IPC, deployed configuration, or Muxel state was inspected or changed.

## Verdict

`keymap/omarchy.toml` is useful coverage data, but its claim to contain every Quattro binding verbatim is false against Omarchy v4.0.2. The local stock file has 104 grouped rows and expands to 226 chords. The pinned upstream has 233 distinct chord contexts. The difference is exact: Winmakase carries three old/local aliases that v4.0.2 does not have, while omitting ten v4.0.2 contexts.

The row-level disposition of all 104 stock mappings is 22 exact, 35 Windows/Glaze equivalents, 28 unavailable, and 19 deferred. Of 50 v1-critical rows, 20 are exact, 18 equivalent, 1 unavailable, and 11 deferred. These are source judgments, not runtime acceptance.

The full row ledger is [bindings-comparison.csv](bindings-comparison.csv). It also records the ten upstream-only contexts and seven sanitized local-override effects. No command, profile name, workspace label, path, account, or private app value from the personal override file is present.

## Pin and reproduction

The official repository is `omacom/omarchy`; the local header still says `basecamp/omarchy` and `ref = "quattro"`. GitHub's tag API resolves `v4.0.2` directly to commit [`346e69e1cec6c4e8924531874af6ba010a1bc99e`](https://github.com/omacom/omarchy/commit/346e69e1cec6c4e8924531874af6ba010a1bc99e), committed 2026-08-30T22:47:46Z. There is no annotated-tag indirection.

The comparison used the six modules loaded by [`default/hypr/omarchy.lua`](https://github.com/omacom/omarchy/blob/346e69e1cec6c4e8924531874af6ba010a1bc99e/default/hypr/omarchy.lua#L8-L14), plus the helper and launch scripts needed to interpret action semantics:

| Source | Git blob | Canonical raw SHA-256 |
|---|---|---|
| [`tiling.lua`](https://github.com/omacom/omarchy/blob/346e69e1cec6c4e8924531874af6ba010a1bc99e/default/hypr/bindings/tiling.lua) | `7e2408d351afbd2b197a4f5088c3d4a7099067de` | `85AAC24C8A44FC3D0513E77AC2C59EEFB179228834D6670EC76FB310D9F99AAF` |
| [`applications.lua`](https://github.com/omacom/omarchy/blob/346e69e1cec6c4e8924531874af6ba010a1bc99e/default/hypr/bindings/applications.lua) | `0c990e64ae6a23b1b8a42e61ab8e6f2211783c0a` | `6EF93BFE3DF6F3B93379E0B194F3EC925CD84CFDB30115C6ECE18198F21E0E3E` |
| [`clipboard.lua`](https://github.com/omacom/omarchy/blob/346e69e1cec6c4e8924531874af6ba010a1bc99e/default/hypr/bindings/clipboard.lua) | `f095ea5cd6484156e1f4d5dc43867d422f14d3d3` | `3C197296F8EFD7C3EB4B048D05C452FD9BBE0AD3C4969D938EA24A1E2D034717` |
| [`utilities.lua`](https://github.com/omacom/omarchy/blob/346e69e1cec6c4e8924531874af6ba010a1bc99e/default/hypr/bindings/utilities.lua) | `406b773f6b6fafceb3174f07a8fc1e19baf22abc` | `60B4AD67084D71222F04B118D6B5DF0DA7BC7B48169655674B84AE6D2F36B5FF` |
| [`media.lua`](https://github.com/omacom/omarchy/blob/346e69e1cec6c4e8924531874af6ba010a1bc99e/default/hypr/bindings/media.lua) | `52779baa8a43c94b58c27b2b1c60fb8b5871d57a` | `956980E5BC440E51D305F0E40CDEE7F791EFF6D4692F866B70275036FF82D2C1` |
| [`voxtype.lua`](https://github.com/omacom/omarchy/blob/346e69e1cec6c4e8924531874af6ba010a1bc99e/default/hypr/bindings/voxtype.lua) | `df594b83056fd6166eb0cab326e2761eb7c8dc67` | `FF39DCAB7A834082ABC12A5563C3B37CE434A39A1969A1ECB2F025553B69C446` |

The SHA-256 values are over bytes downloaded from `raw.githubusercontent.com` at the pinned commit, before Git checkout or line-ending conversion. The Git blob IDs independently identify the same canonical objects.

Local `keymap/omarchy.toml` Git blob: `22f89ba9cb7da3397c2bc13be476df00bf0a59ce`.

Reproduce the source set and counts:

```powershell
gh api --paginate repos/omacom/omarchy/git/ref/tags/v4.0.2
gh api --paginate repos/omacom/omarchy/commits/v4.0.2 --jq '.sha, .commit.committer.date, .html_url'
gh api --paginate 'repos/omacom/omarchy/git/trees/v4.0.2?recursive=1' --jq '.tree[] | select(.path | startswith("default/hypr/bindings/")) | [.type,.path,.sha] | @tsv'
curl.exe -L --fail -o tiling.lua https://raw.githubusercontent.com/omacom/omarchy/346e69e1cec6c4e8924531874af6ba010a1bc99e/default/hypr/bindings/tiling.lua
Get-FileHash ./tiling.lua -Algorithm SHA256
cargo run --quiet -p winmakase --bin winmakase -- --home docs/verification/v1/2026-09-08-p0/nonexistent-home keymap check --keymap keymap/omarchy.toml
Import-Csv docs/verification/v1/2026-09-08-p0/bindings-comparison.csv | Group-Object scope,classification | Sort-Object Name | Select-Object Name,Count
```

The isolated stock check prints `76 mapped, 17 gap, 18 native, 2 app, 30 omitted` and `76 of 226` chords bound. A sanitized check with the present local override reports two app placeholders, three replaced rows, six added rows, no disabled rows, and `73 of 223` chords bound. The current override shortens all three numeric workspace families by three members, removing nine stock chords, then adds six private/local chords. Only these structural effects were retained.

Upstream's module totals are tiling 98, applications 28, clipboard 4, utilities 73, media 28, and voxtype 2: 233 distinct chord contexts. This expands the three 1–10 workspace loops, the 1–5 group selector, the 1–9 bar panel loop, and four directional capture-selection keys. Repeated dispatches on the same chord are counted once: Omarchy binds each Alt+Tab direction twice and F9 on press and release. The eight capture-selection keys count because they are real, usable bindings while the selection layer exists.

## Classification

- **Exact**: same chord and same user-visible operation, with no known semantic loss in source.
- **Equivalent**: the same intent is present through Windows, PowerToys, or a bounded Glaze/Winmakase approximation. Alternate chords and material semantic differences prevent an exact label.
- **Unavailable**: the chosen Windows/Glaze stack lacks the primitive, the chord is reserved, or the feature is outside v1 with no accepted implementation.
- **Deferred**: the accepted design names later P1–P3 work or a bounded candidate, but current source does not provide the required behavior.

Grouped rows receive the weakest disposition of their members. The CSV is a mapping-row comparison, so its counts are not chord counts.

## Binding findings

The exact v1 core is narrower than the current header implies. Directional focus, numeric workspace switch/move/silent-move in the stock file, workspace next/previous/former, workspace-to-monitor, directional swaps, close, and explicit maximized fullscreen have direct source matches. The effective local map omits the final three upstream workspace indices.

Several same-key mappings are only equivalents:

- `SUPER+J` invokes Winmakase reflow. It matches split-toggle intent but refuses some nested structures; Omarchy dispatches Hyprland `togglesplit` directly.
- Scratchpad toggle/move emulates one-window rescue/parking. It does not implement Hyprland's special-workspace family.
- Float/tile intentionally sends every non-tiled state to tiled. Omarchy uses its compositor's toggle semantics.
- Height resize uses percentages instead of Omarchy's fixed pixel deltas.
- `SUPER+O` floats, centers, and raises a window, but cannot pin it across workspaces.

The six width-resize mappings remain deferred because source alone does not prove their user-visible direction. Omarchy describes `code:20` as expansion left and passes negative relative x deltas; `code:21` gets positive deltas. Winmakase passes negative and positive width deltas respectively, using percentages instead of pixels. The matching signs do not establish equivalence or inversion: tiled behavior depends on split position and compositor resize semantics, while floating behavior is a separate case. R1 must test normal, little, and large steps across tiled split positions and floating windows. Do not change these bindings from the English labels alone.

Windows-reserved or occupied chords remain explicit losses. `CTRL+ALT+DELETE` is the secure attention sequence. `SUPER+L` is occupied by Windows lock and cannot safely host Omarchy's workspace-layout cycle; this source slice did not test extra-modifier variants. `CTRL+ALT+TAB` is Windows' persistent task switcher, so it is not an exact monitor-cycle binding. Physical left Win remains native by product contract.

Two aggregate native rows also need the weakest-member rule. Windows Notification Center covers history, but source identifies no equivalent for Omarchy's dismiss-one, dismiss-all, silence, or invoke-last actions, so `notifications` is unavailable as a family. Windows voice typing covers toggle dictation, but not Omarchy's F9 press/release push-to-talk pair, so `dictation` is also unavailable as a family.

### Current fullscreen and reload defects

The `SUPER+F` row is not exact. It emits bare `toggle-fullscreen`, while `SUPER+ALT+F` explicitly requests maximized mode. The accepted source review records that the configured Glaze fullscreen default is maximized, so both chords currently converge on maximized behavior. P1 must make F explicitly true fullscreen, keep Alt+F maximized, and verify tile/float restoration. No runtime check was performed here.

The local `wm-reload` override reaches Glaze's reload command, but reload is not behavior-preserving today. Generated app-home rules always emit `move --workspace` when configured and one of `set-tiling`, `set-floating`, or `ignore`, on `manage`. Glaze replays manage rules on reload. Existing windows can therefore be rehomed or retiled, and a broad default-tiling app rule can override dialog classification. The row stays deferred until P1 separates first management from reconciliation and proves reload preservation.

### Launch is not one behavior

Omarchy's essential terminal and browser keys are launch commands. The terminal helper opens in the active terminal's current directory. The browser helper resolves the selected browser, translates private mode per browser, starts a new launch request, and only performs a best-effort focus after a URL launch. None promises a new app window for every invocation.

Conditional application bindings mix three behaviors in one upstream file:

- ordinary launch: most essential apps, several preinstalled apps, and ordinary web apps;
- explicit focus-or-launch: the preinstalled notes app, selected web apps, and TUI entries marked `focus = true`;
- intentional new surface by argument: for example the new-message URL.

Winmakase's two packed rows preserve the chords but erase that behavior split. The current essential rows directly shell-execute generic commands, and a sanitized local browser-role chord directly launches one profile. P1 must migrate these to typed roles while retaining separate `focus-or-launch` and `launch-new` actions. Treating every app key as focus-existing would be less faithful than the current upstream source.

## Missing upstream contexts and whole families

Two ordinary v4.0.2 aliases are absent: `XF86PowerOff` for the system/power menu and `XF86Calculator` for calculator. The local `SUPER+CTRL+Q` calculator row does not account for the hardware alias.

The larger omission is the transient capture-selection family. While the screenshot selection layer is open, Omarchy binds Return, Ctrl+Return, Tab, Ctrl+Tab, and all four arrows to take the highlighted/full screen or change the selected window. Those eight contexts are not represented anywhere in `keymap/omarchy.toml`. This is a whole conditional family, not eight ordinary global chords.

Other whole upstream behavior families are represented only as aggregate placeholders or gaps:

| Family | Current source disposition | V1 position |
|---|---|---|
| Grouping/tabs: toggle, move in/out, cycle, select 1–5, mouse cycle | Three catalog rows, no Glaze primitive | Unavailable in v1; do not imply implementation because the chords are listed |
| Preinstalled apps and web apps | Two packed rows with `status = "app"` | Deferred; expand typed action semantics when P1 owns roles |
| Omarchy menus and shell panels | Aggregate omitted rows | P2 supplies Windows controls/help equivalents; Linux-specific menus remain unavailable |
| Mouse workspace scroll, drag, resize | Two gap rows | Unavailable in current Glaze binding model |
| Capture selection layer | Missing entirely | Record before deciding whether a Windows capture surface needs equivalent keyboard control |
| Hardware/locked/repeating semantics | Chords grouped as native | Equivalent at best until the supported device/Windows matrix proves repeat, lock-screen, brightness, and touchpad behavior |

This distinction matters: enumerating a family in a non-mapped row is useful correspondence data, but it is not upstream feature coverage.

## Input ownership: current and candidate

Current Winmakase source assumes Caps is converted to right Win by Kanata and Glaze binds the resulting `rwin` chords. That is the proven W0–W4 architecture, not evidence that Kanata is structurally mandatory.

The parked direct-Caps spike at `D:/dev/glazewm-wt-caps-leader` records commits `ba1c7e4` and `f17c552`. Its pure reducer, hook harness, and automated/live command-routing A/B passed, including graceful hook teardown. Physical feel, elevated foreground behavior, PowerToys/reboot hook ordering, and a fullscreen game remain open. If those gates pass, the accepted candidate is for Glaze to own Caps directly as a tap/hold leader and for Winmakase to remove Kanata. No direct-Caps source was merged or runtime-tested in this slice.

## Model and limits

Requested builder: `gpt-5.6-sol`, reasoning effort `high`. The execution surface did not independently expose actual model or effort; no substitution was reported. Lead allocation: Astra. This report records source observations only. Every runtime result, collision check, physical-input judgment, and P1 acceptance case remains open until executed in the authorized environment.

## Validation

- `winmakase keymap check` passed for the isolated 226-chord stock file and the sanitized 223-chord effective structure.
- The CSV stock IDs match all 104 `[[bind]]` IDs exactly: 104 rows, 104 unique, no missing or extra IDs.
- `cargo test -p winmakase-keymap --lib` passed all 14 unit tests. `real_keymap_is_valid` also passed.
- The two full golden suites failed only because Git checked the LF golden YAML files out as CRLF under the user-level `core.autocrlf=true`, while the renderer returns LF. `git ls-files --eol` reports `i/lf w/crlf` for both goldens. No product assertion was inferred from those failed byte comparisons, and this source-only slice did not alter unrelated test fixtures.
- `git diff --check` passed. A case-insensitive scan found none of the inspected personal override values in either committed artifact.
