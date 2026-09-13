$ErrorActionPreference = 'Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes
. C:/WinmakaseI1Probe/Probe-Launcher.ps1 -FunctionsOnly

trap {
  [ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o'); error=$_.Exception.ToString(); zebar=@(Get-Process zebar -ErrorAction SilentlyContinue | Select-Object Id,MainWindowHandle,MainWindowTitle); run=(Read-RunState)} | ConvertTo-Json -Depth 9 | Set-Content C:/WinmakaseI1Evidence/bar-launcher-proof-error.json -Encoding UTF8
  exit 1
}

# Use the same clean start regardless of prior probe state.
& C:/WinmakaseI1Probe/Start-GuestRun.ps1
$runner=Get-Process PowerToys -ErrorAction Stop | Select-Object -First 1
Ensure-Hidden

$root=[Windows.Automation.AutomationElement]::RootElement
$buttonCondition=[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,'Open or focus PowerToys Run')
$buttons=@($root.FindAll([Windows.Automation.TreeScope]::Descendants,$buttonCondition))
if ($buttons.Count -ne 1) { throw "Expected one accessible launcher button, found $($buttons.Count)" }
$button=$buttons[0]
$zebarPids=@(Get-Process zebar -ErrorAction Stop | ForEach-Object Id)
if ($zebarPids -notcontains $button.Current.ProcessId) { throw 'Launcher button is not owned by Zebar' }
$invoke=[Windows.Automation.InvokePattern]$button.GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern)
$before=Read-RunState
$timer=[Diagnostics.Stopwatch]::StartNew()
$invoke.Invoke()
$after=Wait-Launcher $true
$visible=@($after.launcher | Where-Object { $_.visible -and $_.cloaked -eq 0 })
if ($visible.Count -ne 1 -or $after.foregroundHwnd -ne $visible[0].hwnd -or !$after.focusedRunElement.keyboard) { throw 'Bar action did not leave one focused Run launcher with keyboard focus' }
[ordered]@{
  at=[DateTimeOffset]::UtcNow.ToString('o')
  runnerPid=$runner.Id
  button=[ordered]@{ name=$button.Current.Name; processId=$button.Current.ProcessId; enabled=$button.Current.IsEnabled; controlType=$button.Current.ControlType.ProgrammaticName }
  elapsedMs=$timer.Elapsed.TotalMilliseconds
  before=$before
  after=$after
  verdict='pass'
} | ConvertTo-Json -Depth 9 | Set-Content C:/WinmakaseI1Evidence/bar-launcher-proof.json -Encoding UTF8
