$ErrorActionPreference='Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
$installRoot=Join-Path $env:LOCALAPPDATA PowerToys
$settingsPath=Join-Path $env:LOCALAPPDATA 'Microsoft/PowerToys/settings.json'
$settings=Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json
if(!$settings.enabled.PSObject.Properties['PowerToys Run']) { throw 'Expected enable flag absent' }
$owned=@(Get-Process -Name 'PowerToys*' -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($installRoot+'\',[StringComparison]::OrdinalIgnoreCase) })
$stopped=@()
foreach($process in $owned) { $stopped += $process.Id; Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue }
Start-Sleep -Seconds 1
foreach($property in $settings.enabled.PSObject.Properties) { $property.Value=($property.Name -eq 'PowerToys Run') }
[IO.File]::WriteAllText($settingsPath,($settings | ConvertTo-Json -Depth 10),(New-Object Text.UTF8Encoding($false)))
$runner=Start-Process -FilePath (Join-Path $installRoot PowerToys.exe) -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 5
[ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o'); stoppedGuestPids=$stopped; runnerPid=$runner.Id; enabled=$settings.enabled; runProcesses=@(Get-Process -Name PowerToys.PowerLauncher -ErrorAction SilentlyContinue | Select-Object Id,MainWindowHandle,MainWindowTitle)} | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath C:/WinmakaseEvidence/run-enabled.json -Encoding UTF8
