$ErrorActionPreference='Stop'
trap { ($_ | Out-String) | Set-Content C:/WinmakaseEvidence/glaze-start-error.txt -Encoding UTF8; exit 1 }
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
if(Get-Process -Name glazewm -ErrorAction SilentlyContinue) { throw 'Glaze already exists in guest' }
$archive='C:/WinmakaseAssets/glazewm-p0-6bc83d1-x64.zip'
if((Get-FileHash $archive -Algorithm SHA256).Hash -ne '9E975A3E5906760A4E718B57C70614DECF6364FFB6661ABC8E8AD21B4581D14C') { throw 'Archive hash mismatch' }
$destination='C:/WinmakaseP0/glaze'
Expand-Archive -LiteralPath $archive -DestinationPath $destination -Force
$glaze=Join-Path $destination glazewm.exe
if((Get-FileHash $glaze -Algorithm SHA256).Hash -ne '5B5EEE5068E44C4B93ACD1F3448C483B3AC78673BBEFBC3FC3A76D1CEA8B8A15') { throw 'Glaze hash mismatch' }
Copy-Item C:/WinmakaseProbe/glaze-fixture.yaml C:/WinmakaseP0/glaze-fixture.yaml -Force
$proc=Start-Process -FilePath $glaze -ArgumentList @('start','--config','C:/WinmakaseP0/glaze-fixture.yaml') -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 3
$proc.Refresh()
if($proc.HasExited) { throw "Guest Glaze exited: $($proc.ExitCode)" }
$response=& (Join-Path $destination glazewm-cli.exe) query workspaces
$queryExit=$LASTEXITCODE
$parsed=$response | ConvertFrom-Json
if($queryExit -ne 0 -or !$parsed.success) { throw "Glaze query failed: $response" }
[ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o');pid=$proc.Id;exe=$glaze;sha256=(Get-FileHash $glaze -Algorithm SHA256).Hash;configSha256=(Get-FileHash C:/WinmakaseP0/glaze-fixture.yaml -Algorithm SHA256).Hash;querySuccess=$parsed.success} | ConvertTo-Json | Set-Content C:/WinmakaseEvidence/glaze-start.json -Encoding UTF8
