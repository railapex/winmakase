# The admin half of the F13 cutover (input recut slices 3 and 4). Writes the
# Caps -> F13 / ScrLk -> Caps Scancode Map through caps-remap.ps1 and enables
# the WinmakaseSupervisor logon task, which starts GlazeWM and Zebar in f13
# mode. Run it from an elevated PowerShell; running it again is harmless.
#
#   pwsh -File enable-f13.ps1            map + task; the stack starts at next logon
#   pwsh -File enable-f13.ps1 -StartNow  also start the supervisor now, before any reboot
#   pwsh -File enable-f13.ps1 -DryRun    show what would change and write nothing
#
# A machine without the WinmakaseSupervisor task (the laptop, which only
# sends keys over Parsec) gets the map alone.
#
# The map takes effect at the next boot. Until then PowerToys Keyboard Manager
# keeps sending Caps as F13, so nothing breaks in between. After that boot,
# remove the Caps and ScrLk rules from Keyboard Manager. With both active,
# ScrLk also becomes F13 and the raw-CapsLock escape is gone.
#
# Rollback: caps-remap.ps1 -Remove (then reboot), and
# Disable-ScheduledTask -TaskName WinmakaseSupervisor.
param(
    [switch]$StartNow,
    [switch]$DryRun
)
$ErrorActionPreference = 'Stop'
$taskName = 'WinmakaseSupervisor'

$elevated = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $elevated -and -not $DryRun) { throw 'Run this from an elevated PowerShell.' }

Write-Output '== Scancode Map'
& (Join-Path $PSScriptRoot 'caps-remap.ps1') -Apply -DryRun:$DryRun

Write-Output ''
Write-Output "== $taskName"
$task = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
if ($null -eq $task) {
    Write-Output "No $taskName task on this machine; the map is the whole job here."
    exit 0
}
if ($DryRun) {
    Write-Output "Would enable $taskName (now $($task.State))$(if ($StartNow) { ' and start it' })."
    exit 0
}
if ($task.State -eq 'Disabled') {
    Enable-ScheduledTask -TaskName $taskName | Out-Null
    Write-Output "$taskName enabled: it starts the stack at every logon."
} else {
    Write-Output "$taskName already enabled ($($task.State))."
}
if ($StartNow) {
    Start-ScheduledTask -TaskName $taskName
    Write-Output "$taskName started. Check with: winmakase status"
} else {
    Write-Output 'Reboot to start the stack and activate the map in one go.'
}
