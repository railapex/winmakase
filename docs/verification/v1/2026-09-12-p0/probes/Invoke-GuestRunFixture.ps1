param([ValidateSet('main-spaces','owned-dialog','unicode-modal','utility-popup')][string]$ActionId)
$ErrorActionPreference='Stop'
$fixtureActionId=$ActionId
. C:/WinmakaseProbe/Probe-RunEvent.ps1 -Action Observe -Label "invoke-$fixtureActionId-before"
$manifest=Get-Content C:/WinmakaseP0/fixture-actions.json -Raw -Encoding UTF8 | ConvertFrom-Json
$actionDef=@($manifest.actions | Where-Object id -eq $fixtureActionId)[0]
$queryText=[IO.Path]::GetFileNameWithoutExtension($actionDef.shortcutFile)
$visible=@([RunWindowProbe]::Windows($runIds) | Where-Object { $_.visible -and $_.cloaked -eq 0 })
if($visible.Count -ne 1) { throw 'Expected one visible Run window' }
$root=[Windows.Automation.AutomationElement]::FromHandle([IntPtr]$visible[0].hwnd)
$results=$root.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty,[Windows.Automation.ControlType]::ListItem))
$matches=@($results | Where-Object { $_.Current.Name -eq "$queryText : Application, 4 Appended controls available" })
if($matches.Count -ne 1) { throw 'Exact fixture application result absent or ambiguous' }
$matches[0].GetCurrentPattern([Windows.Automation.SelectionItemPattern]::Pattern).Select()
$edit=$root.FindFirst([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::AutomationIdProperty,'QueryTextBox'))
if($edit.GetCurrentPattern([Windows.Automation.ValuePattern]::Pattern).Current.Value -ne $queryText) { throw 'Query changed' }
$edit.SetFocus()
if([RunWindowProbe]::GetForegroundWindow().ToInt64() -ne $visible[0].hwnd -or !$edit.Current.HasKeyboardFocus) { throw 'Run no longer has foreground/query focus' }
$capturePath="C:/WinmakaseP0/data/$fixtureActionId.json"
if(Test-Path $capturePath) { throw 'Fixture was already invoked; use a fresh test state' }
Add-Type -AssemblyName System.Windows.Forms
[Windows.Forms.SendKeys]::SendWait('{ENTER}')
$timer=[Diagnostics.Stopwatch]::StartNew()
while(!(Test-Path $capturePath) -and $timer.ElapsedMilliseconds -lt 5000) { Start-Sleep -Milliseconds 50 }
if(!(Test-Path $capturePath)) { throw 'Fixture did not create its exact argument capture' }
$capture=Get-Content $capturePath -Raw -Encoding UTF8 | ConvertFrom-Json
$actualArgs=$capture.arguments | ConvertTo-Json -Compress
$expectedArgs=$actionDef.arguments | ConvertTo-Json -Compress
if($actualArgs -cne $expectedArgs) { throw 'Argument capture differs from fixture manifest' }
Copy-Item -LiteralPath $capturePath -Destination "C:/WinmakaseEvidence/launcher-$fixtureActionId.json"
$registration="C:/WinmakaseP0/data/$fixtureActionId-windows.json"
if(Test-Path $registration) { Copy-Item $registration "C:/WinmakaseEvidence/launcher-$fixtureActionId-windows.json" }
[ordered]@{action=$fixtureActionId;query=$queryText;argumentsExact=$true;elapsedMs=$timer.Elapsed.TotalMilliseconds;runAfter=(Read-RunState)} | ConvertTo-Json -Depth 7 | Set-Content "C:/WinmakaseEvidence/invoke-$fixtureActionId-result.json" -Encoding UTF8
