param([ValidateSet('Inspect','CloseMove','Open')][string]$Action)
$ErrorActionPreference='Stop'
trap { ($_ | Out-String) | Set-Content C:/WinmakaseEvidence/owned-probe-error.txt -Encoding UTF8; exit 1 }
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes
Add-Type -Path C:/WinmakaseP0/source/DialogFactInterop.cs
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class OwnedFixtureClose {
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,uint message,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr GetParent(IntPtr h);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,out uint pid);
 [DllImport("user32.dll")] public static extern IntPtr SendMessageTimeout(IntPtr h,uint message,IntPtr w,IntPtr l,uint flags,uint timeout,out IntPtr result);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h,StringBuilder text,int capacity);
}
'@
$registration=Get-Content C:/WinmakaseP0/data/owned-dialog-windows.json -Raw -Encoding UTF8 | ConvertFrom-Json
if($Action -eq 'Inspect') {
  $focus=& C:/WinmakaseP0/glaze/glazewm-cli.exe command focus --workspace 1 | ConvertFrom-Json
  if(!$focus.success) { throw 'Guest focus failed' }
  Start-Sleep -Milliseconds 200
}
$main=@($registration.windows | Where-Object role -eq main)[0]
$observed=[Winmakase.P0.DialogFactReader]::Snapshot([long]$main.handle,[int]$registration.processId,[long]$registration.processCreationTimeUtcTicks,[long]$main.windowGeneration)
$element=[Windows.Automation.AutomationElement]::FromHandle([IntPtr]$main.handle)
$buttons=$element.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
if($Action -eq 'Inspect') {
  [ordered]@{mainHandle=$main.handle;buttons=@($buttons | ForEach-Object { [ordered]@{name=$_.Current.Name;type=$_.Current.ControlType.ProgrammaticName;enabled=$_.Current.IsEnabled;handle=$_.Current.NativeWindowHandle;patterns=@($_.GetSupportedPatterns()|ForEach-Object ProgrammaticName)} })} | ConvertTo-Json -Depth 5 | Set-Content C:/WinmakaseEvidence/owned-main-buttons.json -Encoding UTF8
} elseif($Action -eq 'CloseMove') {
  $owned=@($registration.windows | Where-Object role -eq owned)[0]
  $check=[Winmakase.P0.DialogFactReader]::Snapshot([long]$owned.handle,[int]$registration.processId,[long]$registration.processCreationTimeUtcTicks,[long]$owned.windowGeneration)
  if(!$check.hasOwner -or $check.ownerHandle -ne $main.handle) { throw 'Owned dialog relationship changed' }
  [void][OwnedFixtureClose]::PostMessage([IntPtr]$owned.handle,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
  $timer=[Diagnostics.Stopwatch]::StartNew()
  while([OwnedFixtureClose]::IsWindow([IntPtr]$owned.handle) -and $timer.ElapsedMilliseconds -lt 2000) { Start-Sleep -Milliseconds 20 }
  if([OwnedFixtureClose]::IsWindow([IntPtr]$owned.handle)) { throw 'Owned dialog did not close' }
  $prior=Get-Content C:/WinmakaseP0/data/dialog-proof/owned-dialog-after-reload.json -Raw -Encoding UTF8 | ConvertFrom-Json
  $id=@($prior.glazeWindows | Where-Object role -eq main)[0].id
  $cli='C:/WinmakaseP0/glaze/glazewm-cli.exe'
  foreach($commandArgs in @(@('command','--id',$id,'move','--workspace','2'),@('command','focus','--workspace','1'))) {
    $result=& $cli @commandArgs | ConvertFrom-Json
    if($LASTEXITCODE -ne 0 -or !$result.success) { throw 'Fixture move/focus failed' }
  }
} else {
  $target=@($buttons | Where-Object { $_.Current.Name -eq 'Owned dialog' -and $_.Current.IsEnabled })
  $usedRetainedHandle=$false
  if($target.Count -eq 1) { $button=[IntPtr]$target[0].Current.NativeWindowHandle }
  elseif($target.Count -eq 0) {
    $known=Get-Content C:/WinmakaseEvidence/owned-main-buttons.json -Raw -Encoding UTF8 | ConvertFrom-Json
    $knownButton=@($known.buttons | Where-Object name -eq 'Owned dialog')
    if($known.mainHandle -ne $main.handle -or $knownButton.Count -ne 1) { throw 'No previously inspected exact fixture button' }
    $button=[IntPtr]$knownButton[0].handle
    $usedRetainedHandle=$true
  } else { throw 'Owned dialog button ambiguous' }
  $buttonText=New-Object Text.StringBuilder 128
  [void][OwnedFixtureClose]::GetWindowText($button,$buttonText,128)
  if($buttonText.ToString() -cne 'Owned dialog') { throw 'Fixture button text changed' }
  [uint32]$buttonPid=0
  [void][OwnedFixtureClose]::GetWindowThreadProcessId($button,[ref]$buttonPid)
  if($buttonPid -ne $registration.processId -or [OwnedFixtureClose]::GetParent($button).ToInt64() -ne $main.handle) { throw 'Fixture button identity changed' }
  $nativeResult=[IntPtr]::Zero
  if([OwnedFixtureClose]::SendMessageTimeout($button,0xF5,[IntPtr]::Zero,[IntPtr]::Zero,2,2000,[ref]$nativeResult) -eq [IntPtr]::Zero) { throw 'Fixture button click timed out' }
  [ordered]@{usedPreviouslyInspectedHandle=$usedRetainedHandle;currentUiaMatchingButtons=$target.Count;nativePidParentTextValidated=$true;method='BM_CLICK to one fixture child HWND; no global input'} | ConvertTo-Json | Set-Content C:/WinmakaseEvidence/owned-away-input-method.json -Encoding UTF8
  Start-Sleep -Milliseconds 500
  & C:/WinmakaseP0/source/Capture-P0DialogProof.ps1 -FixtureRoot C:/WinmakaseP0 -ActionId owned-dialog -Phase owner-away-new-dialog -GlazeExecutablePath C:/WinmakaseP0/glaze/glazewm.exe -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
  Copy-Item C:/WinmakaseP0/data/dialog-proof/owned-dialog-owner-away-new-dialog.json C:/WinmakaseEvidence -Force
}
