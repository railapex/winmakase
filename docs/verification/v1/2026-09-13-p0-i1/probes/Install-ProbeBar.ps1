$ErrorActionPreference = 'Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
$exe='C:/WinmakaseAssets/zebar-portable/zebar.exe'
$zebarHash=(Get-FileHash $exe -Algorithm SHA256).Hash
if ($zebarHash -ne '16EF0B5B586F30DF132DE93CFB31C4D9105571EC3525685B4BDA6D8D6D5C84D0') { throw 'Staged Zebar binary hash mismatch' }

$root=Join-Path $env:USERPROFILE '.glzr/zebar'
$pack=Join-Path $root winmakase
New-Item -ItemType Directory -Force $pack | Out-Null
Copy-Item C:/WinmakaseI1Probe/bar.html,C:/WinmakaseI1Probe/styles.css,C:/WinmakaseI1Probe/zpack.json $pack -Force
foreach($file in 'bar.html','zpack.json') {
  $path=Join-Path $pack $file
  [IO.File]::WriteAllText($path,([IO.File]::ReadAllText($path).Replace('C:/Users/chris/.winmakase/bin','C:/WinmakaseBuild')),(New-Object Text.UTF8Encoding($false)))
}
$settings=@{ '$schema'='https://github.com/glzr-io/zebar/raw/v3.3.1/resources/settings-schema.json'; startupConfigs=@(@{pack='winmakase';widget='bar';preset='default'}) }
$settingsJson=$settings | ConvertTo-Json -Depth 5
[IO.File]::WriteAllText((Join-Path $root settings.json),$settingsJson,(New-Object Text.UTF8Encoding($false)))
$zebar=Start-Process $exe -PassThru
Start-Sleep -Seconds 8
[ordered]@{ at=[DateTimeOffset]::UtcNow.ToString('o'); binary=$exe; sha256=$zebarHash; zebarPid=$zebar.Id; pack=$pack; settings=$settings } | ConvertTo-Json -Depth 6 | Set-Content C:/WinmakaseI1Evidence/zebar-start.json -Encoding UTF8
