$ErrorActionPreference='Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
$log=Join-Path $env:USERPROFILE '.glzr/zebar/errors.log'
if(Test-Path $log){Copy-Item $log C:/WinmakaseI1Evidence/zebar-errors.log -Force}
