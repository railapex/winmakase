param([ValidateSet('RenameRemove','Restore')][string]$Action)
$ErrorActionPreference='Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
$directory=Join-Path ([Environment]::GetFolderPath('Programs')) 'Winmakase P0 Fixtures'
if((Get-Content (Join-Path $directory '.winmakase-p0-published') -Raw).Trim() -cne 'winmakase-p0-disposable-publication-v1') { throw 'Shortcut publication marker missing' }
$main=Join-Path $directory 'Winmakase P0 - Main Spaces.lnk'
$renamed=Join-Path $directory 'Winmakase P0 - Main Renamed.lnk'
$utility=Join-Path $directory 'Winmakase P0 - Utility Popup.lnk'
$parked='C:/WinmakaseP0/utility-removed.lnk'
if($Action -eq 'RenameRemove') {
  if((Test-Path $renamed) -or (Test-Path $parked)) { throw 'Rename/remove already done' }
  Move-Item -LiteralPath $main -Destination $renamed
  Move-Item -LiteralPath $utility -Destination $parked
} else {
  Move-Item -LiteralPath $renamed -Destination $main
  Move-Item -LiteralPath $parked -Destination $utility
}
[ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o');action=$Action;mainPresent=(Test-Path $main);renamedPresent=(Test-Path $renamed);utilityPublished=(Test-Path $utility)} | ConvertTo-Json | Set-Content "C:/WinmakaseEvidence/shortcuts-$Action.json" -Encoding UTF8
