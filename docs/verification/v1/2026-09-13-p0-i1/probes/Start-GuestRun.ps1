$ErrorActionPreference='Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
try {
  Get-Process powershell -ErrorAction SilentlyContinue | Where-Object MainWindowTitle -eq 'Winmakase I1 focus control' | ForEach-Object { [void]$_.CloseMainWindow() }
  $installRoot=Join-Path $env:LOCALAPPDATA PowerToys
  $settingsPath=Join-Path $env:LOCALAPPDATA 'Microsoft/PowerToys/settings.json'
  $settings=Get-Content $settingsPath -Raw | ConvertFrom-Json
  if (!$settings.enabled.PSObject.Properties['PowerToys Run']) { throw 'PowerToys Run enable property absent' }
  $owned=@(Get-Process -Name 'PowerToys*' -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($installRoot+'\',[StringComparison]::OrdinalIgnoreCase) })
  foreach($process in $owned) { Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue }
  Start-Sleep -Seconds 1
  foreach($property in $settings.enabled.PSObject.Properties) { $property.Value=($property.Name -eq 'PowerToys Run') }
  [IO.File]::WriteAllText($settingsPath,($settings | ConvertTo-Json -Depth 10),(New-Object Text.UTF8Encoding($false)))
  $runner=Start-Process (Join-Path $installRoot PowerToys.exe) -WindowStyle Hidden -PassThru
  Start-Sleep -Seconds 5
  $run=@(Get-Process PowerToys.PowerLauncher -ErrorAction Stop)
  if($run.Count -ne 1) { throw "Expected one Run process, found $($run.Count)" }
  $hash=(Get-FileHash $run[0].Path -Algorithm SHA256).Hash
  if($hash -ne '062ED25AA8F343C83641277111BCCAC366B25EBF80021B1E4D783EEF25D2E688') { throw 'Run hash mismatch' }
  [ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o'); runnerPid=$runner.Id; runPid=$run[0].Id; runPath=$run[0].Path; runHash=$hash; enabled=$settings.enabled} | ConvertTo-Json -Depth 6 | Set-Content C:/WinmakaseI1Evidence/run-start.json -Encoding UTF8
} catch {
  [ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o'); error=$_.Exception.ToString(); processes=@(Get-Process -Name 'PowerToys*' -ErrorAction SilentlyContinue | Select-Object Id,ProcessName,Path)} | ConvertTo-Json -Depth 5 | Set-Content C:/WinmakaseI1Evidence/run-start-error.json -Encoding UTF8
  throw
}
