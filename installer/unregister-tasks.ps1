# Remove the Winsome scheduled tasks. Counterpart of register-tasks.ps1.
# Stops the stack gracefully first (winsome down), then removes the
# user-level supervisor task; -All also removes the elevated pair (needs an
# elevated shell). Deployed binaries in ~/.winsome/bin are left in place.
param(
    [switch]$All
)
$winsome = Join-Path $env:USERPROFILE '.winsome\bin\winsome.exe'
if (Test-Path $winsome) {
    # Graceful: GlazeWM's watcher restores window positions. Harmless when no
    # supervisor is running (it says so and exits nonzero).
    & $winsome down --timeout 20
}
Unregister-ScheduledTask -TaskName 'WinsomeSupervisor' -Confirm:$false -ErrorAction SilentlyContinue
Write-Output 'WinsomeSupervisor removed'
if ($All) {
    foreach ($t in 'WinsomeKanata', 'WinsomePanic') {
        Unregister-ScheduledTask -TaskName $t -Confirm:$false -ErrorAction SilentlyContinue
    }
    Write-Output 'WinsomeKanata + WinsomePanic removed'
}
