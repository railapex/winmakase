$ErrorActionPreference = 'Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
$installer = 'C:/WinmakaseAssets/PowerToysUserSetup-0.100.2-x64.exe'
$expected = '945FDF327E4D38E4CED61B0727B7AB8A1222958782982052989DDF7CB7096F62'
if ((Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash -ne $expected) { throw 'Installer hash mismatch' }
$at = [DateTimeOffset]::UtcNow.ToString('o')
$timer = [Diagnostics.Stopwatch]::StartNew()
$process = Start-Process -FilePath $installer -ArgumentList '/quiet','/norestart','/log','C:\WinmakaseEvidence\powertoys-install.log' -WindowStyle Hidden -PassThru
$completed = $process.WaitForExit(180000)
if ($completed) { $process.Refresh() }
$record = [ordered]@{ startedAt=$at; completed=$completed; elapsedMs=$timer.ElapsedMilliseconds; installerPid=$process.Id; exitCode=$null; user=[Environment]::UserName; networking='Disabled by Sandbox config' }
if ($completed) { $record.exitCode=$process.ExitCode }
$record | ConvertTo-Json | Set-Content -LiteralPath C:/WinmakaseEvidence/powertoys-install-result.json -Encoding UTF8
if (!$completed) { throw 'Installer still running after bounded wait; inspect before retrying' }
