# Register the Winsome scheduled tasks. Idempotent; re-run after a rebuild to
# refresh the deployed binaries.
#
# - WinsomeSupervisor (user-level, at logon): hosts winsomed.exe, the
#   windowless supervisor. Registering it needs NO elevation.
# - WinsomeKanata + WinsomePanic (elevated, on-demand): only (re)registered
#   when this script itself runs elevated; otherwise left exactly as they are.
#
# Binaries are copied to ~/.winsome/bin first so the running supervisor never
# holds a lock on target/ — a rebuild must never require stopping the desktop.
param(
    [string]$RepoRoot = (Split-Path $PSScriptRoot -Parent),
    [string]$BuildDir = (Join-Path (Split-Path $PSScriptRoot -Parent) 'target\release')
)
$ErrorActionPreference = 'Stop'

$bin = Join-Path $env:USERPROFILE '.winsome\bin'
New-Item -ItemType Directory -Force $bin | Out-Null
foreach ($exe in 'winsome.exe', 'winsomed.exe') {
    $src = Join-Path $BuildDir $exe
    if (-not (Test-Path $src)) { throw "$src not found - build first: cargo build --release" }
    try {
        Copy-Item $src (Join-Path $bin $exe) -Force
    } catch {
        throw "cannot replace $exe - if the supervisor is running, 'winsome down' first ($_)"
    }
}

$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
    -ExecutionTimeLimit (New-TimeSpan -Hours 0) -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1)
$action = New-ScheduledTaskAction -Execute (Join-Path $bin 'winsomed.exe')
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
# Limited principal on purpose: GlazeWM and everything it ever spawns must stay
# user-level. kanata gets its elevation from its own task, not from us.
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME
Register-ScheduledTask -TaskName 'WinsomeSupervisor' -Action $action -Trigger $trigger `
    -Principal $principal -Settings $settings -Force | Out-Null
Write-Output "WinsomeSupervisor registered (at logon, user-level, windowless; binaries in $bin)"

$elevated = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
    ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($elevated) {
    $kanata = New-ScheduledTaskAction `
        -Execute (Join-Path $RepoRoot 'spike\tools\kanata\kanata_windows_gui_winIOv2_cmd_allowed_x64.exe') `
        -Argument "--cfg $(Join-Path $RepoRoot 'spike\caps.kbd')"
    $panic = New-ScheduledTaskAction -Execute 'pwsh' `
        -Argument "-NoProfile -WindowStyle Hidden -File $(Join-Path $RepoRoot 'spike\panic.ps1')"
    $admin = New-ScheduledTaskPrincipal -UserId $env:USERNAME -RunLevel Highest
    $onDemand = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
        -ExecutionTimeLimit (New-TimeSpan -Hours 0)
    Register-ScheduledTask -TaskName 'WinsomeKanata' -Action $kanata -Principal $admin -Settings $onDemand -Force | Out-Null
    Register-ScheduledTask -TaskName 'WinsomePanic' -Action $panic -Principal $admin -Settings $onDemand -Force | Out-Null
    Write-Output 'WinsomeKanata + WinsomePanic (re)registered (elevated, on-demand, UAC-free to trigger)'
} else {
    Write-Output 'not elevated: WinsomeKanata / WinsomePanic left as-is (run elevated to (re)register them)'
}
