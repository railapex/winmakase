$ErrorActionPreference='Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
$installer='C:/WinmakaseAssets/vc-redist-14.51.36247.0-x64.exe'
if((Get-FileHash $installer -Algorithm SHA256).Hash -ne '843068991DAAA1F73AD9F6239BCE4D0F6A07A51F18C37EA2A867E9BECA71295C') { throw 'VC runtime hash mismatch' }
$timer=[Diagnostics.Stopwatch]::StartNew()
$proc=Start-Process -FilePath $installer -ArgumentList @('/install','/quiet','/norestart','/log','C:/WinmakaseEvidence/vc-runtime-install.log') -WindowStyle Hidden -PassThru
if(!$proc.WaitForExit(300000)) { throw 'VC runtime installation exceeded 300 seconds' }
[ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o');version='14.51.36247.0';sha256=(Get-FileHash $installer -Algorithm SHA256).Hash;exitCode=$proc.ExitCode;elapsedMs=$timer.Elapsed.TotalMilliseconds;runtimePresent=(Test-Path C:/Windows/System32/vcruntime140.dll)} | ConvertTo-Json | Set-Content C:/WinmakaseEvidence/vc-runtime-install.json -Encoding UTF8
if($proc.ExitCode -notin @(0,3010)) { throw 'VC runtime installation failed' }
