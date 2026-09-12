$ErrorActionPreference = 'Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
$installRoot = Join-Path $env:LOCALAPPDATA PowerToys
$runPath = Join-Path $installRoot 'WinUI3Apps/PowerToys.PowerLauncher.exe'
if (!(Test-Path -LiteralPath $runPath)) { $runPath = Join-Path $installRoot 'PowerToys.PowerLauncher.exe' }
if (!(Test-Path -LiteralPath $runPath)) {
  Get-ChildItem -LiteralPath $installRoot -Filter PowerToys.PowerLauncher.exe -Recurse | Select-Object FullName | ConvertTo-Json | Set-Content C:/WinmakaseEvidence/powertoys-run-location.json
  throw 'Run binary was not at the expected root or WinUI3Apps path; inspect location before launch'
}
if ((Get-FileHash -LiteralPath $runPath -Algorithm SHA256).Hash -ne '062ED25AA8F343C83641277111BCCAC366B25EBF80021B1E4D783EEF25D2E688') { throw 'Installed Run hash mismatch' }
$runnerPath = Join-Path $installRoot PowerToys.exe
$existing = Get-Process -Name PowerToys -ErrorAction SilentlyContinue
if (!$existing) { $existing = Start-Process -FilePath $runnerPath -WindowStyle Hidden -PassThru }
Start-Sleep -Seconds 5
$record=[ordered]@{
  at=[DateTimeOffset]::UtcNow.ToString('o')
  runnerPid=@($existing | ForEach-Object Id)
  runPath=$runPath
  runFileVersion=(Get-Item -LiteralPath $runPath).VersionInfo.FileVersion
  runProcesses=@(Get-Process -Name PowerToys.PowerLauncher -ErrorAction SilentlyContinue | Select-Object Id,MainWindowHandle,MainWindowTitle)
  generalSettings=$null
}
$settings=Join-Path $env:LOCALAPPDATA 'Microsoft/PowerToys/settings.json'
if (Test-Path -LiteralPath $settings) { $record.generalSettings=Get-Content -LiteralPath $settings -Raw | ConvertFrom-Json }
$record | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath C:/WinmakaseEvidence/powertoys-start.json -Encoding UTF8
