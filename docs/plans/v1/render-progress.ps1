param(
    [string]$StatePath = (Join-Path $PSScriptRoot 'execution-state.json'),
    [string]$OutputPath = (Join-Path $PSScriptRoot 'progress.html')
)
$ErrorActionPreference = 'Stop'
$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '../../..')).Path
$state = Get-Content -Raw -LiteralPath $StatePath | ConvertFrom-Json
if ($state.schemaVersion -ne 1) { throw 'Unsupported execution-state schema.' }
$todo = Get-Content -Raw -LiteralPath (Join-Path $repoRoot 'docs/TODO.md')
$packages = [regex]::Matches($todo, '(?m)^- \[(?<done>[ x])\] \[(?<id>P\d+) — (?<title>.+?)\]\((?<link>.+?)\): (?<detail>.+)\r?$')
$rounds = [regex]::Matches($todo, '(?m)^- \[(?<done>[ x])\] (?<id>R\d+) — (?<title>.+)\r?$')
if ($packages.Count -ne 6 -or $rounds.Count -ne 7) { throw 'TODO package/round structure changed; reconcile the report renderer.' }
function Encode([object]$value) { [System.Net.WebUtility]::HtmlEncode([string]$value) }
$packageRows = foreach ($entry in $packages) {
    $status = if ($entry.Groups['done'].Value -eq 'x') { 'Verified' } else { 'Pending' }
    $target = '../../../docs/' + $entry.Groups['link'].Value
    '<tr><th scope="row"><a href="' + (Encode $target) + '">' + (Encode $entry.Groups['id'].Value) + '</a></th><td>' + (Encode $entry.Groups['title'].Value) + '</td><td>' + $status + '</td></tr>'
}
$roundCards = foreach ($entry in $rounds) {
    $status = if ($entry.Groups['done'].Value -eq 'x') { 'Passed' } else { 'Pending' }
    '<li><strong>' + (Encode $entry.Groups['id'].Value) + '</strong><span>' + (Encode $entry.Groups['title'].Value.Trim().TrimEnd('.')) + '</span><em>' + $status + '</em></li>'
}
$passed = @($rounds | Where-Object { $_.Groups['done'].Value -eq 'x' }).Count
$decisionRows = if ($state.decisions.Count -eq 0) {
    '<p>No decisions awaiting Chris. The next checkpoint will show concrete options and evidence.</p>'
} else {
    @($state.decisions | ForEach-Object {
        '<article class="decision"><h3>' + (Encode $_.title) + '</h3><p>' + (Encode $_.question) + '</p><p><strong>Recommendation:</strong> ' + (Encode $_.recommendation) + '</p><p><strong>Blocks:</strong> ' + (Encode $_.blockingScope) + '</p></article>'
    }) -join [Environment]::NewLine
}
$blockerRows = if ($state.blockers.Count -eq 0) {
    '<p>No active build blocker recorded. Prerequisite experiments are queued, not passed.</p>'
} else {
    '<ul>' + (@($state.blockers | ForEach-Object { '<li>' + (Encode $_) + '</li>' }) -join '') + '</ul>'
}
$recentRows = @($state.recentDecisions | ForEach-Object { '<li><strong>' + (Encode $_.title) + '</strong><p>' + (Encode $_.detail) + '</p></li>' }) -join [Environment]::NewLine
$evidenceRows = @($state.evidence | ForEach-Object { '<li><a href="' + (Encode $_.href) + '">' + (Encode $_.label) + '</a><p>' + (Encode $_.detail) + '</p></li>' }) -join [Environment]::NewLine
$checkpointRows = @($state.checkpoints | ForEach-Object { '<li><time>' + (Encode $_.at) + '</time><p>' + (Encode $_.summary) + '</p></li>' }) -join [Environment]::NewLine
$verifiedBuild = if ($state.lastVerifiedBuild) { Encode $state.lastVerifiedBuild } else { 'No v1 build has passed these rounds' }
$updatedAt = Encode $state.updatedAt
$html = @"
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Winmakase v1 · Build progress</title>
<style>
:root{color-scheme:dark light;--bg:#171922;--panel:#20232f;--fg:#e4e7ef;--muted:#adb4c8;--line:#393f53;--accent:#9db9ff;--warm:#f0cb88}
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);font:16px/1.55 system-ui,Segoe UI,sans-serif}
main{max-width:1100px;margin:auto;padding:42px 24px 64px}h1{font-size:clamp(30px,5vw,48px);line-height:1.1;letter-spacing:-.035em;margin:12px 0}h2{font-size:21px;margin:0 0 18px}h3{font-size:17px;margin:0 0 8px}p{margin:8px 0 16px}a{color:var(--accent);text-underline-offset:3px}a:focus-visible,summary:focus-visible{outline:3px solid var(--warm);outline-offset:4px}.eyebrow{color:var(--accent);letter-spacing:.1em;text-transform:uppercase;font-size:12px;font-weight:700}.subtle,time{color:var(--muted);font-size:14px}.stage{font-size:19px;max-width:760px}.notice{border-left:3px solid var(--warm);padding:9px 16px;color:var(--warm);margin:24px 0}.grid{display:grid;grid-template-columns:1.3fr 1fr;gap:24px}.panel{padding:24px;border:1px solid var(--line);border-radius:12px;background:var(--panel);margin:24px 0 0}.lead{border-top:3px solid var(--accent)}.small-label{color:var(--muted);font-size:13px;margin-bottom:4px}.slice{font-size:23px;line-height:1.3;margin:8px 0}.pill{display:inline-block;border:1px solid var(--line);border-radius:99px;padding:2px 9px;font-size:12px;color:var(--warm);margin-left:8px}table{width:100%;border-collapse:collapse}td,th{padding:13px 9px;border-top:1px solid var(--line);text-align:left;font-size:14px}th{font-weight:600}td:last-child{color:var(--muted);white-space:nowrap}.rounds{list-style:none;padding:0;margin:0}.rounds li{display:grid;grid-template-columns:34px 1fr auto;gap:12px;padding:11px 0;border-top:1px solid var(--line);font-size:14px}.rounds em{font-style:normal;color:var(--muted);font-size:12px}.list{list-style:none;margin:0;padding:0}.list>li{padding:12px 0;border-top:1px solid var(--line)}.list p{font-size:14px;color:var(--muted);margin:5px 0}.decision{margin:16px 0;padding-top:12px;border-top:1px solid var(--line)}.footer{margin-top:30px;padding-top:20px;border-top:1px solid var(--line);color:var(--muted);font-size:13px}summary{cursor:pointer;font-weight:600}.meta{display:flex;gap:20px;flex-wrap:wrap;margin-top:24px}
@media(max-width:720px){main{padding:24px 16px}.grid{grid-template-columns:1fr;gap:0}.panel{padding:18px}td,th{padding:11px 5px}table{table-layout:fixed}th:first-child{width:42px}td:last-child{width:76px}.rounds li{gap:8px}}
@media(prefers-color-scheme:light){:root{--bg:#f4f5f9;--panel:#fff;--fg:#252936;--muted:#596277;--line:#d9deea;--accent:#285bb4;--warm:#8b5500}}
@media print{body{background:white;color:black}.panel{break-inside:avoid}.grid{display:block}}
</style>
</head>
<body>
<main>
<header>
<div class="eyebrow">Winmakase / v1</div>
<h1>Build progress</h1>
<p class="stage">$(Encode $state.stage)</p>
<p class="subtle">Updated <time id="updated" datetime="$updatedAt">$updatedAt</time> · Read-only build snapshot</p>
<p class="notice" id="freshness">This report records evidence. It does not run builds, change the desktop or submit decisions.</p>
<nav aria-label="Build references"><a href="NEXT.md">Resume</a> · <a href="README.md">Plan</a> · <a href="execution.md">Build procedure</a> · <a href="verification.md">Verification</a> · <a href="../../research/desktop-interactions-2026-09-08.md">Desktop behavior</a></nav>
</header>
<div class="grid">
<section class="panel lead"><div class="small-label">Current slice</div><h2 class="slice">$(Encode $state.currentSlice.id) · $(Encode $state.currentSlice.title)<span class="pill">$(Encode $state.currentSlice.status)</span></h2><p>$(Encode $state.currentSlice.nextAction)</p><div class="small-label">Next check-in</div><p>$(Encode $state.nextCheckpoint)</p></section>
<section class="panel"><h2>Decisions for Chris</h2>$decisionRows<div class="small-label">Blockers</div>$blockerRows</section>
</div>
<div class="grid">
<section class="panel"><h2>Build packages</h2><table><thead><tr><th>ID</th><th>Outcome</th><th>Status</th></tr></thead><tbody>$($packageRows -join [Environment]::NewLine)</tbody></table></section>
<section class="panel"><h2>Verification · $passed / 7 passed</h2><ul class="rounds">$($roundCards -join [Environment]::NewLine)</ul></section>
</div>
<div class="grid">
<section class="panel"><h2>Scope decisions</h2><ul class="list">$recentRows</ul></section>
<section class="panel"><h2>Evidence</h2><ul class="list">$evidenceRows</ul><div class="meta"><div><div class="small-label">Last verified build</div>$verifiedBuild</div><div><div class="small-label">Planning base</div>$(Encode $state.planBaseCommit)</div></div></section>
</div>
<section class="panel"><details><summary>Checkpoint history</summary><ul class="list">$checkpointRows</ul></details></section>
<footer class="footer">Generated from docs/TODO.md and execution-state.json. Answer decisions in the active session; the lead records the decision and updates this snapshot. No hidden build is running behind this page.</footer>
</main>
<script>
const stamp = Date.parse(document.getElementById('updated').dateTime);
if (Number.isFinite(stamp) && Date.now() - stamp > 86400000) {
  document.getElementById('freshness').textContent = 'Stale snapshot: last updated more than 24 hours ago. Check the latest handoff before relying on this status.';
}
</script>
</body>
</html>
"@
[System.IO.File]::WriteAllText($OutputPath, $html, [System.Text.UTF8Encoding]::new($false))
Write-Output ("Rendered {0} packages, {1} rounds, {2} passed: {3}" -f $packages.Count, $rounds.Count, $passed, $OutputPath)
