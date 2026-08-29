# Remove the Winmakase scheduled tasks. Counterpart of register-tasks.ps1.
# Stops the stack gracefully first (winmakase down), then removes the
# user-level supervisor task; -All also removes the elevated pair (needs an
# elevated shell). Deployed binaries in ~/.winmakase/bin are left in place.
param(
    [switch]$All
)
$winmakase = Join-Path $env:USERPROFILE '.winmakase\bin\winmakase.exe'
if (Test-Path $winmakase) {
    # Graceful: GlazeWM's watcher restores window positions. Harmless when no
    # supervisor is running (it says so and exits nonzero).
    & $winmakase down --timeout 20
}
Unregister-ScheduledTask -TaskName 'WinmakaseSupervisor' -Confirm:$false -ErrorAction SilentlyContinue
Write-Output 'WinmakaseSupervisor removed'
if ($All) {
    foreach ($t in 'WinmakaseKanata', 'WinmakasePanic') {
        Unregister-ScheduledTask -TaskName $t -Confirm:$false -ErrorAction SilentlyContinue
    }
    Write-Output 'WinmakaseKanata + WinmakasePanic removed'
}
