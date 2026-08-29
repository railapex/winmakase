# One-time rig migration for the 2026-08-29 winsome -> winmakase rename.
# The repo, crates, and binaries are already renamed; this script moves the
# LIVE RIG: ~/.winsome -> ~/.winmakase (config patched), old-named scheduled
# tasks replaced with new ones, bar pack dir + zebar startup pointer renamed,
# then the stack is brought back up. BOUNCES THE DESKTOP (GlazeWM restart
# re-tiles windows).
#
# Run ELEVATED to migrate the WinsomeKanata/WinsomePanic tasks in the same
# pass. -RenameRepoDir (elevated only) also moves D:/dev/winsome ->
# D:/dev/winmakase and repoints the kanata task + config at the new path.
# Build first: cargo build --release (or pass -BuildDir target/debug).
#Requires -Version 7
param(
    [switch]$RenameRepoDir,
    [string]$BuildDir
)
$ErrorActionPreference = 'Stop'

$oldHome = Join-Path $env:USERPROFILE '.winsome'
$newHome = Join-Path $env:USERPROFILE '.winmakase'
$elevated = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
    ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($RenameRepoDir -and -not $elevated) {
    throw '-RenameRepoDir needs an elevated run (the kanata/panic task actions must be re-registered)'
}

# 1. Stop the stack through the OLD binary so the shutdown choreography
#    (completion-wait, taskbar restore) applies.
$oldExe = Join-Path $oldHome 'bin\winsome.exe'
if (Test-Path $oldExe) {
    & $oldExe down --timeout 30
    if ($LASTEXITCODE -ne 0) { Write-Warning 'winsome down reported a problem; continuing' }
}

# 2. Old-named tasks out. Hardcoded old names on purpose - the repo's
#    unregister-tasks.ps1 speaks the new names now.
foreach ($t in 'WinsomeSupervisor', 'WinsomeKanata', 'WinsomePanic') {
    schtasks /delete /tn $t /f 2>$null
}
if (-not $elevated) {
    Write-Warning 'not elevated: WinsomeKanata/WinsomePanic may remain - delete them from an elevated shell'
}

# 3. Home dir move + config patch. Capital-W replace hits the task name and
#    prose but leaves lowercase D:/dev/winsome paths alone (see step 4).
if ((Test-Path $oldHome) -and (Test-Path $newHome)) { throw "both $oldHome and $newHome exist - resolve by hand" }
if (Test-Path $oldHome) { Rename-Item $oldHome $newHome }
$cfg = Join-Path $newHome 'config.toml'
if (Test-Path $cfg) {
    (Get-Content $cfg -Raw).Replace('Winsome', 'Winmakase') | Set-Content $cfg -NoNewline
}

# 4. Repo dir (optional, elevated).
$repoOld = 'D:/dev/winsome'
$repoNew = 'D:/dev/winmakase'
$repoRoot = if ($RenameRepoDir) {
    if (Test-Path $repoOld) { Rename-Item $repoOld $repoNew }
    (Get-Content $cfg -Raw).Replace($repoOld, $repoNew) | Set-Content $cfg -NoNewline
    $repoNew
} elseif (Test-Path $repoNew) { $repoNew } else { $repoOld }

# 5. Bar pack dir + any zebar startup config pointing at it.
$zroot = Join-Path $env:USERPROFILE '.glzr\zebar'
if (Test-Path (Join-Path $zroot 'winsome')) {
    Rename-Item (Join-Path $zroot 'winsome') 'winmakase'
}
foreach ($f in Get-ChildItem $zroot -Filter *.json -File -ErrorAction SilentlyContinue) {
    $t = Get-Content $f.FullName -Raw
    if ($t.Contains('winsome')) { $t.Replace('winsome', 'winmakase') | Set-Content $f.FullName -NoNewline }
}

# 6. Deploy new-named binaries + register new tasks (elevated pass also
#    refreshes the kanata/panic pair with $repoRoot paths).
$reg = Join-Path $repoRoot 'installer\register-tasks.ps1'
if ($BuildDir) { & $reg -RepoRoot $repoRoot -BuildDir $BuildDir } else { & $reg -RepoRoot $repoRoot }

# 7. Back up.
schtasks /run /tn 'WinmakaseSupervisor' | Out-Null
Write-Output 'migrated - stack restarting under Winmakase names; check: winmakase status'
Write-Output 'remaining button: gh repo rename winmakase -R railapex/winsome'
if (-not $RenameRepoDir) {
    Write-Output "repo dir still $repoOld - rerun elevated with -RenameRepoDir to move it"
}
