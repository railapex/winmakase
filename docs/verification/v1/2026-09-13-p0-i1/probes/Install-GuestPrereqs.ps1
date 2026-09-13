$ErrorActionPreference = 'Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null

$vc = 'C:/WinmakaseAssets/vc-redist-14.51.36247.0-x64.exe'
if ((Get-FileHash $vc -Algorithm SHA256).Hash -ne '843068991DAAA1F73AD9F6239BCE4D0F6A07A51F18C37EA2A867E9BECA71295C') { throw 'VC runtime hash mismatch' }
$vcTimer = [Diagnostics.Stopwatch]::StartNew()
$vcProc = Start-Process $vc -ArgumentList @('/install','/quiet','/norestart','/log','C:/WinmakaseI1Evidence/vc-runtime-install.log') -WindowStyle Hidden -PassThru
if (!$vcProc.WaitForExit(300000)) { throw 'VC runtime installation exceeded 300 seconds' }
if ($vcProc.ExitCode -notin @(0,3010)) { throw "VC runtime installation failed: $($vcProc.ExitCode)" }

$pt = 'C:/WinmakaseAssets/PowerToysUserSetup-0.100.2-x64.exe'
if ((Get-FileHash $pt -Algorithm SHA256).Hash -ne '945FDF327E4D38E4CED61B0727B7AB8A1222958782982052989DDF7CB7096F62') { throw 'PowerToys installer hash mismatch' }
$ptTimer = [Diagnostics.Stopwatch]::StartNew()
$ptProc = Start-Process $pt -ArgumentList '/quiet','/norestart','/log','C:/WinmakaseI1Evidence/powertoys-install.log' -WindowStyle Hidden -PassThru
if (!$ptProc.WaitForExit(300000)) { throw 'PowerToys installation exceeded 300 seconds' }
$ptProc.Refresh()
if ($ptProc.ExitCode -notin @(0,3010)) { throw "PowerToys installation failed: $($ptProc.ExitCode)" }

$installRoot = Join-Path $env:LOCALAPPDATA PowerToys
$settingsPath = Join-Path $env:LOCALAPPDATA 'Microsoft/PowerToys/settings.json'
if (!(Test-Path $settingsPath)) {
  $bootstrap = Start-Process (Join-Path $installRoot PowerToys.exe) -WindowStyle Hidden -PassThru
  $settingsDeadline = [DateTimeOffset]::UtcNow.AddSeconds(20)
  while (!(Test-Path $settingsPath) -and [DateTimeOffset]::UtcNow -lt $settingsDeadline) {
    Start-Sleep -Milliseconds 100
  }
  if (!(Test-Path $settingsPath)) { throw 'PowerToys did not create settings.json within 20 seconds' }
  Start-Sleep -Seconds 1
}
$settings = Get-Content $settingsPath -Raw | ConvertFrom-Json
$owned=@(Get-Process -Name 'PowerToys*' -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($installRoot+'\',[StringComparison]::OrdinalIgnoreCase) })
foreach($process in $owned) { Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue }
Start-Sleep -Seconds 1
foreach ($property in $settings.enabled.PSObject.Properties) { $property.Value = ($property.Name -eq 'PowerToys Run') }
[IO.File]::WriteAllText($settingsPath,($settings | ConvertTo-Json -Depth 10),(New-Object Text.UTF8Encoding($false)))
$runner = Start-Process (Join-Path $installRoot PowerToys.exe) -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 5
$run = @(Get-Process PowerToys.PowerLauncher -ErrorAction Stop)
if ($run.Count -ne 1) { throw "Expected one PowerToys Run process, found $($run.Count)" }
if ((Get-FileHash $run[0].Path -Algorithm SHA256).Hash -ne '062ED25AA8F343C83641277111BCCAC366B25EBF80021B1E4D783EEF25D2E688') { throw 'Installed Run hash mismatch' }

[ordered]@{
  at = [DateTimeOffset]::UtcNow.ToString('o')
  vc = [ordered]@{ exitCode=$vcProc.ExitCode; elapsedMs=$vcTimer.ElapsedMilliseconds }
  powerToys = [ordered]@{ exitCode=$ptProc.ExitCode; elapsedMs=$ptTimer.ElapsedMilliseconds; runnerPid=$runner.Id; runPid=$run[0].Id; runPath=$run[0].Path; runHash=(Get-FileHash $run[0].Path -Algorithm SHA256).Hash }
} | ConvertTo-Json -Depth 5 | Set-Content C:/WinmakaseI1Evidence/prereqs.json -Encoding UTF8
