param([string]$Phase='initial')
$ErrorActionPreference='Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
if($Phase -notmatch '^[a-z0-9-]+$') { throw 'Invalid phase' }
$results=@()
foreach($actionId in @('main-spaces','owned-dialog','unicode-modal','utility-popup')) {
  try {
    & C:/WinmakaseP0/source/Capture-P0DialogProof.ps1 -FixtureRoot C:/WinmakaseP0 -ActionId $actionId -Phase $Phase -GlazeExecutablePath C:/WinmakaseP0/glaze/glazewm.exe -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
    Copy-Item "C:/WinmakaseP0/data/dialog-proof/$actionId-$Phase.json" "C:/WinmakaseEvidence/dialog-$actionId-$Phase.json" -Force
    Copy-Item "C:/WinmakaseP0/data/$actionId-windows.json" "C:/WinmakaseEvidence/launcher-$actionId-windows.json" -Force
    $results+=[ordered]@{action=$actionId;success=$true}
  } catch { $results+=[ordered]@{action=$actionId;success=$false;error=($_ | Out-String)} }
}
$results | ConvertTo-Json -Depth 5 | Set-Content "C:/WinmakaseEvidence/dialog-$Phase-results.json" -Encoding UTF8
