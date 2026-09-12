$ErrorActionPreference='Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
$record=[ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o');osVersion=[Environment]::OSVersion.Version.ToString();runtimeVersion=(Get-Item C:/Windows/System32/vcruntime140.dll).VersionInfo.FileVersion;ownedProcesses=@(Get-Process -Name glazewm,glazewm-watcher,PowerToys,PowerToys.PowerLauncher,WinmakaseP0Fixture -ErrorAction SilentlyContinue | Select-Object Id,ProcessName);fixtureScripts=@(Get-FileHash C:/WinmakaseP0/source/Capture-P0DialogProof.ps1,C:/WinmakaseP0/source/DialogProof.Common.psm1,C:/WinmakaseP0/bin/WinmakaseP0Fixture.exe -Algorithm SHA256 | Select-Object Path,Hash)}
$record | ConvertTo-Json -Depth 5 | Set-Content C:/WinmakaseEvidence/guest-final.json -Encoding UTF8
