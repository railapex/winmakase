# Gap features — scratchpad · grouping/tabs · in-place reflow

Decision doc, researched 2026-08-28. These are Hyprland built-ins that omarchy merely binds (mod+S scratchpad, mod+G group, mod+J split-toggle); no Windows tiler ships all three. Options compared per feature: (a) build winsome-side via GlazeWM IPC, (b) upstream GlazeWM PR, (c) komorebi power option.

## Verdicts

| Feature | Verdict | Route |
|---|---|---|
| In-place reflow (mod+J) | **Build** | winsome `reflow` verb via IPC (bench-proven feasible) |
| Scratchpad (mod+S) | **Build** | winsome `scratchpad` verb via IPC — every primitive source-verified; needs a small behavior spike |
| Grouping/tabs (mod+G) | **Documented gap** in v1 | Only real route is komorebi (arc power option). No GlazeWM primitive exists; upstream is dormant |

Upstream PRs: file as courtesy/opportunistic only. Nothing blocks on glzr-io merging anything.

## Finding that reframes option (b): GlazeWM is dormant, not merge-quiet

Evidence gathered 2026-08-28 via GitHub API:

- **Last commit to `main`: 2026-04-08.** Zero commits in 4.5 months. Last release v3.10.1 (2026-03-21).
- The whole glzr-io org went dark together: Zebar's last commit 2026-03-31; the only post-April touch anywhere is one commit on an `animations-test` experiment branch (2026-06-20). The maintainer (lars-berger) has **zero public GitHub events** in the API's ~90-day window.
- The repo was healthy right up to the cliff: 13 outside authors + 24 maintainer PRs merged Jan–Apr 2026 (upgrades the earlier "7 outside authors in Q1" estimate). This reads as *maintainer stepped away*, not contribution-hostile — but for planning it's the same thing.
- **40+ open PRs queued Feb–Aug 2026**, including feature-sized work from strong authors sitting unreviewed: #1281 (craigloewen-msft — tiling strategies/toggle-vertical, Feb 24), #1406 (declarative column layouts), #1392 (animations), #1407 (grab-and-move). New PRs still arrive (latest Aug 25); none merge.
- Stacking/grouping specifically: [#301](https://github.com/glzr-io/glazewm/issues/301) "Stacking like in i3" (open since 2023-04, **89 reactions** — the repo's most-upvoted FR) and [#1005](https://github.com/glzr-io/glazewm/issues/1005) tabbed layout (2025-03, 57 reactions). The maintainers' own attempts exist as WIP branches `feat/master-stack` and `feat/accordian-v1` — **abandoned March 2025**.
- Scratchpad FR [#662](https://github.com/glzr-io/glazewm/issues/662) (sway-like scratchpad): open since 2024-08, 5 comments, 9 reactions, last activity 2025-04. Stale as the card suspected.

**Implication beyond this card**: winsome's base tiler is currently unmaintained. Mitigations already in the design hold — pinned versions, supervisor owns crash/restart, GPL-3.0 means a fork is the worst-case escape. DESIGN.md carries a one-line risk note as of this commit. If glzr-io wakes, revisit; the winsome-side verbs below lose nothing either way (they're IPC clients, not forks).

## Field re-sweep — is there a better base tiler? (Chris's challenge, answered with data)

With GlazeWM revealed dormant, the "was GlazeWM right" question got re-run against the whole 2026 field, not just komorebi (all repo stats pulled 2026-08-28):

| Tiler | Stars | License | Last activity | Verdict |
|---|---|---|---|---|
| GlazeWM | 12.7k | GPL-3.0 | main 2026-04-08 (dormant) | **Keep as default** — see below |
| komorebi | 15.1k | Komorebi 2.0.0 (PolyForm-Strict fork) | active (2026-08-24) | Power option only — license (workplace use paid, no redistribution) |
| Seelen UI | 17.6k | AGPL-3.0 | very active (pushed 2026-08-28, v2.8.2 Aug) | Watch, not adopt — see below |
| FancyWM | 1.2k | **PolyForm Perimeter** (relicensed MIT→GPL-2→Perimeter, 2022→2024→2025-07; website's "free and open source" claim is stale) | releases active (v2.19.0 2026-03) | Out — Perimeter's noncompete ("any purpose... except providing to others any product that competes") arguably excludes winsome as a *licensee at all*; two license tightenings in 3 years = relicense risk; no IPC/CLI surface |
| Whim | 458 | MIT | alpha; last commit 2025-12-14 | Out — alpha, quiet 8 months, tiny community |
| workspacer | 1.8k | MIT | 2025-06, maintenance-mode | Dead |
| bug.n | 3.4k | GPL-3.0 | 2023 | Dead |

**Seelen UI is the only live challenger** and deserves the honest paragraph: bigger than GlazeWM and komorebi combined, shipping monthly, AGPL (fine — we install, never link), and its WM has a real `slu` CLI (source-verified: `focus/move {side}`, `move-to-monitor`, `toggle-float`, `toggle-monocle`, `reserve stack|float`, `cycle-stack`) — including **genuine stacking**. Why it still loses for winsome's base: (1) it's a whole Tauri/webview desktop environment — dock (weg), toolbar, task switcher, apps menu, wallpaper manager — i.e. a *competitor to winsome's thesis*, not a component; adopting only its TWM means running its shell architecture anyway. (2) The tiling WM is historically its weakest organ (state_v2 churn; the stars are for the dock/toolbar). (3) Its workspace system is a separate "virtual desktops" layer with **readonly hardcoded chords** (Ctrl+Win+D/F4/arrows) — omarchy grammar can't be fully expressed. (4) No scratchpad, no tiling-direction/reflow concept — it closes one of our three gaps (grouping) while reopening keybinding fidelity, which is winsome's goal #2. **Re-evaluate only if glzr-io stays dark another 6+ months AND Seelen's TWM matures into grammar-complete territory.**

GlazeWM keeps the seat because dormant-but-pinned is livable for a component with this profile: GPL (fork escape exists), native and lean, deepest scriptable surface of the field (per-window IPC targeting, 16 event types — spike-verified), proven on the rig, and the supervisor already owns its failure modes. The gap verbs below are IPC clients, so they survive any future tiler wake-up or fork.

## Feature 1 — In-place reflow (omarchy Super+J parity)

Spike already bench-confirmed (IPC rect comparison): GlazeWM `toggle-tiling-direction` sets the *insert* direction only; it never re-flows an existing split. Omarchy's Super+J re-arranges in place.

- **(a) winsome-side — CHOSEN.** `winsome reflow`: read the focused split's children via `query windows`/`query tiling-direction`, re-arrange via targeted `move` + `set-tiling-direction`. Relay path, latency-insensitive, proven feasible in the spike. Already on the M1 list as the interim verb; this decision makes it the permanent one.
- (b) Upstream: correct fix architecturally, but see dormancy above. File the FR referencing the bench data; don't wait on it.
- (c) komorebi `flip-layout` genuinely re-flows in place — but switching tilers for one verb we can build in a day is absurd.

## Feature 2 — Scratchpad (omarchy mod+S, dropdown-terminal pattern)

Card rated this "plausible." Source review (packages/wm-common/src/app_command.rs, ipc_server.rs @ main 2026-08-28) upgrades it: **every needed primitive exists in the IPC surface**:

- `command --id <uuid> <verb>` — every command takes an optional subject container (`subject_container_id: Option<Uuid>`), so winsome can act on tracked windows regardless of focus.
- `focus --container-id <uuid>` — direct focus-by-id.
- `move --workspace <name>` — targeted move to a named workspace (hide = move to a workspace bound to no monitor).
- `set-floating --centered --shown-on-top --width --height` — summon with geometry, above the tiles: the dropdown feel.
- `set-minimized` / `toggle-minimized` — alternate hide mechanism if the hidden-workspace route misbehaves.
- `query windows` + `window_managed`/`window_unmanaged` events — track the scratch window's lifecycle; spawn via `shell-exec` when absent.

**Design sketch**: `winsome scratchpad toggle <name>` — config maps name → launch command + match rule + geometry. Hidden state = parked on workspace `scratch` (declared, bound to no monitor). Summon = targeted `move --workspace <current>` + `set-floating --centered --shown-on-top` + `focus --container-id`. Banish = targeted `move --workspace scratch`. Matches Hyprland special-workspace behavior for the single-window case, which is omarchy's actual usage.

**Residual risk (spike before checking the TODO item)**: behavior edges the source doesn't answer — does moving a window to a never-displayed workspace fully hide it; does `set-floating` on it restore/raise correctly; focus-event ordering; flicker. One dogfood session answers all four.

- (b) Upstream #662: stale two years in a dormant repo. No.
- (c) komorebi: **doesn't have one either.** Open FR [LGUG2Z/komorebi#1446](https://github.com/LGUG2Z/komorebi/issues/1446) (May 2025, no maintainer response, last activity Dec 2025). The card's "scratchpad unverified" resolves to *absent* — so the power option wouldn't even buy this feature. Winsome-side is the only route on Windows, full stop.

## Feature 3 — Grouping/tabs (omarchy mod+G)

- **(a) winsome-side — confirmed impossible, not just "likely."** The full `InvokeCommand` enum (source, main) contains no grouping/stacking/tab primitive — nothing puts two windows in one tile. Faking it (stacked floating windows + a tab widget + focus juggling) means building a WM inside the WM and fighting the tiler on every event. Honest no, per design goal 5.
- **(b) Upstream — dead end for v1.** Grouping is GlazeWM's most-demanded feature (89 + 57 reactions), the maintainers started it twice (`feat/master-stack`, `feat/accordian-v1`) and abandoned both in March 2025, and the repo is dormant. A feature-sized core-tree PR from outside, with no outside-feature precedent even when merges flowed, into a queue nobody drains — no.
- **(c) komorebi — the only genuine route, with an asterisk.** Verified: real grouping — multiple windows share one tile, cycled tab-style (`komorebic stack <dir>` / `unstack` / `cycle-stack` / `focus-stack-window <index>` / `stack-all`), with a configurable tab bar (stackbar: height, mode `OnStack`, per-tab styling). The asterisk: komorebi's own docs call the stackbar *"not considered stable"* with visual artifacts ([docs](https://lgug2z.github.io/komorebi/usage/stacking-windows.html)). Genuine, shipping, imperfect.

**v1 posture**: mod+G becomes a documented gap (DESIGN.md § Documented gaps) — bound to a notification stub ("grouping needs the komorebi power option — see docs") rather than silently dead, so trained fingers get told, not ignored. The komorebi power option stays arc'd (Deferred), now carrying grouping as its headline justification rather than general "power."

## komorebi power-option cost sheet

Researched 2026-08-28 (agent pass, sources verified against LGUG2Z repos + docs):

| Cost | Reality |
|---|---|
| License | Komorebi License 2.0.0 (source-available, not OSI). Personal use free — narrowly defined (hobby, study, "no anticipated commercial application"); **work use requires the ICUL: $10/mo or $110/yr** (Windows tier; bundle covers whkd + masir too). Redistribution of binaries blocked; whether "installer scripts `scoop install komorebi` on the user's machine" counts as distribution is **unverified** — no provision either way. BYO-license posture is mandatory: winsome scripts the install, the user acquires komorebi through its own channel and owns the license question. |
| whkd | Same license, same ICUL bundle — no separate cost. |
| Bar pairing | **Cheaper than the card assumed: Zebar has a first-party komorebi provider** — the "Zebar pairing loss" cost doesn't exist. Options: keep Zebar (theming + health dot carry over), or komorebi-bar (in-repo, same license) or yasb. |
| Config migration | GlazeWM YAML → komorebi.json + whkd. The tiler-agnostic keymap file was designed for exactly this; wm-switch adds a second render target. Bounded, real work. |
| Feature trade | Gains grouping (native stacks + stackbar — docs self-label "not stable", visual artifacts) and native in-place reflow (`flip-layout`). **Does not gain scratchpad** — komorebi lacks it too (FR #1446, open since 2025-05, no maintainer response). The winsome scratchpad verb is GlazeWM-IPC; a komorebic port is plausible (similar primitives) but unbuilt — komorebi mode ships without scratchpad until someone ports it. |
| Maintenance | Healthy: v0.1.41 stable 2026-05-03, nightly 2026-08-23, repo pushed 2026-08-24. The one *actively maintained* deep tiler on Windows right now. |

Net: the power option is more attractive than at scoping (Zebar survives, ICUL is cheap, komorebi is the only deep tiler still shipping) — but it still can't be the default (license) and still doesn't cover scratchpad. Verdicts unchanged; priority of the arc'd wm-switch item can rise if GlazeWM's dormancy starts to bite.

## What this changes

1. M1 TODO: research card → checked, replaced by two build items (`reflow` verb; `scratchpad` verb + behavior spike) and one keymap/gap item (mod+G stub + DESIGN gap entry).
2. DESIGN.md: one-line upstream-dormancy risk note in the GlazeWM decision paragraph; grouping added to Documented gaps.
3. Deferred/arc: komorebi power option annotated as the grouping route.

## Patchset build — fork-ladder rung 1, evaluated 2026-08-28 (evening)

**Question**: cherry-pick upstream PRs onto v3.10.1 as a winsome-built GlazeWM? Candidates: [#1347](https://github.com/glzr-io/glazewm/pull/1347) + [#1348](https://github.com/glzr-io/glazewm/pull/1348) (crash fixes, #1348 depends on #1347), [#1424](https://github.com/glzr-io/glazewm/pull/1424) (shell-exec quoted-path parsing, fixes #1417).

**Verdict: not now.** Four legs:

1. **The motivating incident was retracted.** The 2026-08-28 "GlazeWM died on the teleprompter drag" theory is disproven by the supervisor log (display change 02:55:21/28Z, GlazeWM alive at 02:55:41Z; the dead stack was a WinsomePanic live test). Observed GlazeWM crashes on this rig: **zero**.
2. **What #1347/#1348 actually fix is narrower than billed**: not display-change crashes in general but an **OOM fast-fail during the commit-pressure spike after sleep/wake** (Windows pages processes back in; `DisplaySettingsChanged` fires mid-spike; allocation hits `rust_oom → __fastfail`, uncatchable). The fix is a pressure check + 1s debounce/re-arm. The upstream maintainer is himself hesitant on #1347 — single reporter, possibly environment-specific.
3. **The supervisor already converts this crash class into a ~2s blip** — and its restart backoff (1s, doubling) lands the retry *after* the transient pressure spike, which is functionally the same debounce #1348 adds. The crash-loop the PR author describes is exactly what the backoff machinery absorbs.
4. **The fork costs more than "two cherry-picks" because of the UIAccess manifest** (verified live 2026-08-28: `asInvoker` + `uiAccess="true"`, enforced at process launch). A self-built glazewm.exe must either be Authenticode-signed and installed to a secure path (self-signed root cert = a machine-trust change winsome should not ship) or built with `uiAccess=false` (unquantified behavior loss). That's build-and-trust engineering, not a patch.

**#1424 is adopted as a constraint, not a build**: shell-exec mis-parses quoted paths containing spaces. The live scratchpad binding survives only because `D:\dev\winsome\spike\summon.ps1` has no spaces. Rule until upstream fixes it (or rung 1 fires for other reasons): **no spaces in any shell-exec path** — config-gen and keymap render must enforce it; `~/.winsome` paths are safe, `Program Files` paths are not.

**Re-open triggers** (concrete, so this isn't a vibe): a real GlazeWM crash on this rig with exit code `0xc0000409` (-1073740791 in the supervisor log — exits are recorded with codes now) correlated with sleep/wake or a display change. Two such incidents → rung 1 re-opens with this doc as the starting point.
