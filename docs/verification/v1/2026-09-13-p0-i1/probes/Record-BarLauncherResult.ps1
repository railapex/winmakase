$ErrorActionPreference = 'Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
. C:/WinmakaseI1Probe/Probe-Launcher.ps1 -FunctionsOnly

$state = Read-RunState
$visible = @($state.launcher | Where-Object { $_.visible -and $_.cloaked -eq 0 })
if ($visible.Count -ne 1 -or $state.foregroundHwnd -ne $visible[0].hwnd -or !$state.focusedRunElement.keyboard) {
  throw 'Observed bar click did not leave one focused Run launcher with keyboard focus'
}

[ordered]@{
  at = [DateTimeOffset]::UtcNow.ToString('o')
  activation = 'CUA foreground pixel click on the rendered Zebar launcher button'
  state = $state
  verdict = 'pass'
} | ConvertTo-Json -Depth 9 | Set-Content C:/WinmakaseI1Evidence/bar-launcher-observation.json -Encoding UTF8
