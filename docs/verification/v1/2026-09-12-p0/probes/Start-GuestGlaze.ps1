$ErrorActionPreference='Stop'
trap { ($_ | Out-String) | Set-Content C:/WinmakaseEvidence/glaze-start-error.txt -Encoding UTF8; exit 1 }
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
if(Get-Process -Name glazewm -ErrorAction SilentlyContinue) { throw 'Glaze already exists in guest' }
$archive='C:/WinmakaseAssets/glazewm-p0-dd8fb7d-x64.zip'
if((Get-FileHash $archive -Algorithm SHA256).Hash -ne '8ABC5D6A77FA1BB835A61C0AF9E3B7F898F3529F46F6BCD41F7040F16ACB7C78') { throw 'Archive hash mismatch' }
$destination='C:/WinmakaseP0/glaze'
Expand-Archive -LiteralPath $archive -DestinationPath $destination -Force
$glaze=Join-Path $destination glazewm.exe
if((Get-FileHash $glaze -Algorithm SHA256).Hash -ne '226E0E4BFCD4B308A9415DC6DE011CF0EA1E01EC0E2621A4DC06078D08401BA3') { throw 'Glaze hash mismatch' }
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
