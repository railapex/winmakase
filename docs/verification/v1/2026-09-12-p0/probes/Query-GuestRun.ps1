param([string]$Query,[string]$Label='run-query')
$ErrorActionPreference='Stop'
if($Label -notmatch '^[a-z0-9-]+$') { throw 'Invalid label' }
$queryEvidenceLabel=$Label
. C:/WinmakaseProbe/Probe-RunEvent.ps1 -Action Observe -Label "$Label-before"
$visible=@([RunWindowProbe]::Windows($runIds) | Where-Object { $_.visible -and $_.cloaked -eq 0 })
if($visible.Count -ne 1) { throw 'Expected one visible Run window' }
$root=[Windows.Automation.AutomationElement]::FromHandle([IntPtr]$visible[0].hwnd)
$edits=$root.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty,[Windows.Automation.ControlType]::Edit))
if($Query) {
  if($Query -notmatch '^Winmakase P0 - (Main Spaces|Main Renamed|Owned Dialog|Unicode Modal|Utility Popup)$') { throw 'Only fixture queries allowed' }
  if($edits.Count -ne 1 -or $edits[0].Current.Name -ne 'Query') { throw 'Expected Run Query editor' }
  $value=$edits[0].GetCurrentPattern([Windows.Automation.ValuePattern]::Pattern)
  $value.SetValue($Query)
  Start-Sleep -Milliseconds 1500
}
$all=$root.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
$rows=@(foreach($e in $all) {
  [ordered]@{name=$e.Current.Name; type=$e.Current.ControlType.ProgrammaticName; automationId=$e.Current.AutomationId; offscreen=$e.Current.IsOffscreen; focused=$e.Current.HasKeyboardFocus; patterns=@($e.GetSupportedPatterns() | ForEach-Object ProgrammaticName)}
})
[ordered]@{query=$Query; hwnd=$visible[0].hwnd; elements=$rows} | ConvertTo-Json -Depth 6 | Set-Content "C:/WinmakaseEvidence/$queryEvidenceLabel-ui.json" -Encoding UTF8
