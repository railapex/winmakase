param([ValidateSet('Arrange','Reload')][string]$Action)
$ErrorActionPreference='Stop'
trap { ($_ | Out-String) | Set-Content C:/WinmakaseEvidence/dialog-arrange-error.txt -Encoding UTF8; exit 1 }
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
$cli='C:/WinmakaseP0/glaze/glazewm-cli.exe'
if((Get-FileHash $cli -Algorithm SHA256).Hash -ne '9CB864C7BFF00FD7EA336DDDC50A3C3A73453E9E2D3AC5EBE9A25EE03D4105FC') { throw 'Unexpected CLI' }
function Invoke-CheckedCommand([string[]]$Arguments) {
  $output=& $cli @Arguments
  if($LASTEXITCODE -ne 0) { throw 'Glaze command failed' }
  $response=$output | ConvertFrom-Json
  if(!$response.success) { throw 'Glaze rejected fixture command' }
}
if($Action -eq 'Arrange') {
  & C:/WinmakaseP0/source/Capture-P0DialogProof.ps1 -FixtureRoot C:/WinmakaseP0 -ActionId main-spaces -Phase arrange-validation -GlazeExecutablePath C:/WinmakaseP0/glaze/glazewm.exe -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
  $proof=Get-Content C:/WinmakaseP0/data/dialog-proof/main-spaces-arrange-validation.json -Raw -Encoding UTF8 | ConvertFrom-Json
  $main=@($proof.glazeWindows | Where-Object role -eq main)[0]
  if(!$main.managed -or !$main.id) { throw 'Main fixture is not managed' }
  Invoke-CheckedCommand @('command','--id',$main.id,'set-floating','--centered=false','--x-pos','250','--y-pos','200','--width','700px','--height','450px')
  Invoke-CheckedCommand @('command','--id',$main.id,'move','--workspace','2')
  Invoke-CheckedCommand @('command','focus','--workspace','2')
} else {
  Invoke-CheckedCommand @('command','wm-reload-config')
}
Start-Sleep -Milliseconds 500
& C:/WinmakaseProbe/Capture-GuestDialogs.ps1 -Phase $(if($Action -eq 'Arrange') {'before-reload'} else {'after-reload'})
if($Action -eq 'Reload') {
  foreach($actionId in @('main-spaces','owned-dialog','unicode-modal','utility-popup')) {
    & C:/WinmakaseP0/source/Compare-P0DialogProof.ps1 -FixtureRoot C:/WinmakaseP0 -ActionId $actionId -BeforePhase before-reload -AfterPhase after-reload -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
  }
  Get-ChildItem C:/WinmakaseP0/data/dialog-proof -Filter '*comparison*' | Copy-Item -Destination C:/WinmakaseEvidence -Force
}
