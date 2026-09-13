param(
  [string]$Winmakase = 'C:/WinmakaseBuild/winmakase.exe',
  [string]$Helper = 'C:/WinmakaseBuild/winmakase-run.exe',
  [switch]$FunctionsOnly
)
$ErrorActionPreference = 'Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class I1RunProbe {
  public delegate bool EnumProc(IntPtr h, IntPtr p);
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int left,top,right,bottom; }
  public sealed class Window { public long hwnd; public uint pid; public string title; public bool visible; public int cloaked; public Rect rect; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc fn, IntPtr p);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out Rect r);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h,StringBuilder text,int n);
  [DllImport("dwmapi.dll")] static extern int DwmGetWindowAttribute(IntPtr h,int attr,out int value,int size);
  public static Window[] Windows(uint[] pids) {
    var accepted=new HashSet<uint>(pids); var result=new List<Window>();
    EnumWindows(delegate(IntPtr h,IntPtr p) {
      uint pid; GetWindowThreadProcessId(h,out pid); if(!accepted.Contains(pid)) return true;
      var text=new StringBuilder(256); GetWindowText(h,text,text.Capacity);
      Rect rect; GetWindowRect(h,out rect); int cloaked; if(DwmGetWindowAttribute(h,14,out cloaked,4)!=0) cloaked=-1;
      result.Add(new Window{hwnd=h.ToInt64(),pid=pid,title=text.ToString(),visible=IsWindowVisible(h),cloaked=cloaked,rect=rect}); return true;
    },IntPtr.Zero); return result.ToArray();
  }
}
'@

trap {
  if ($focusControl -and !$focusControl.HasExited) { [void]$focusControl.CloseMainWindow() }
  $failure=[ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o'); error=$_.Exception.ToString()}
  try { $failure.state=Read-RunState } catch {}
  $failure | ConvertTo-Json -Depth 9 | Set-Content C:/WinmakaseI1Evidence/launcher-proof-error.json -Encoding UTF8
  exit 1
}

function Read-RunState {
  $runs = @(Get-Process -Name PowerToys.PowerLauncher -ErrorAction SilentlyContinue)
  [uint32[]]$pids = @($runs | ForEach-Object Id)
  $windows = if ($pids.Count) { @([I1RunProbe]::Windows($pids)) } else { @() }
  $launcher = @($windows | Where-Object title -eq 'PowerToys.PowerLauncher')
  $foreground = [I1RunProbe]::GetForegroundWindow()
  $focus = $null
  try {
    $focused = [Windows.Automation.AutomationElement]::FocusedElement
    if ($focused -and $pids -contains $focused.Current.ProcessId) {
      $focus = [ordered]@{ processId=$focused.Current.ProcessId; name=$focused.Current.Name; automationId=$focused.Current.AutomationId; type=$focused.Current.ControlType.ProgrammaticName; keyboard=$focused.Current.HasKeyboardFocus }
    }
  } catch {}
  [ordered]@{ launcher=$launcher; allWindows=$windows; foregroundHwnd=$foreground.ToInt64(); focusedRunElement=$focus }
}

function Get-VisibleLauncher {
  $state = Read-RunState
  @($state.launcher | Where-Object { $_.visible -and $_.cloaked -eq 0 })
}

function Wait-Launcher([bool]$Visible, [int]$TimeoutMs = 3000) {
  $timer = [Diagnostics.Stopwatch]::StartNew()
  do {
    $state = Read-RunState
    $isVisible = @( $state.launcher | Where-Object { $_.visible -and $_.cloaked -eq 0 } ).Count -eq 1
    if ($isVisible -eq $Visible) { return $state }
    Start-Sleep -Milliseconds 10
  } while ($timer.ElapsedMilliseconds -lt $TimeoutMs)
  throw "Run visibility did not become $Visible"
}

function Signal-RawToggle {
  $event = [Threading.EventWaitHandle]::OpenExisting('Local\PowerToysRunInvokeEvent-30f26ad7-d36d-4c0e-ab02-68bb5ff3c4ab')
  try { [void]$event.Set() } finally { $event.Dispose() }
}

function Ensure-Hidden {
  if ((Get-VisibleLauncher).Count) {
    Signal-RawToggle
    [void](Wait-Launcher $false)
  }
}

function Invoke-LauncherCli([string[]]$Arguments) {
  $lines = @(& $Winmakase launcher open --json @Arguments 2>&1)
  $code = $LASTEXITCODE
  $text = $lines -join "`n"
  $json = $null
  try { $json = $text | ConvertFrom-Json } catch {}
  [ordered]@{ exitCode=$code; text=$text; json=$json }
}

function Query-Value([long]$Hwnd) {
  $root = [Windows.Automation.AutomationElement]::FromHandle([IntPtr]$Hwnd)
  $edit = $root.FindFirst([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::AutomationIdProperty,'QueryTextBox'))
  if (!$edit) { throw 'Run QueryTextBox not found' }
  $pattern = [Windows.Automation.ValuePattern]$edit.GetCurrentPattern([Windows.Automation.ValuePattern]::Pattern)
  [ordered]@{ element=$edit; pattern=$pattern; value=$pattern.Current.Value }
}

if ($FunctionsOnly) { return }

$record = [ordered]@{ at=[DateTimeOffset]::UtcNow.ToString('o'); machine=$env:COMPUTERNAME; user=$env:USERNAME }
Ensure-Hidden
$record.hiddenBefore = Read-RunState

$record.hiddenOpen = Invoke-LauncherCli @()
$record.afterHiddenOpen = Wait-Launcher $true
if ($record.hiddenOpen.exitCode -ne 0 -or $record.hiddenOpen.json.outcome -ne 'opened' -or !$record.hiddenOpen.json.signalled) { throw 'hidden CLI open did not report one successful signal' }
$visible = @(Get-VisibleLauncher)
$firstHwnd = [long]$visible[0].hwnd
$query = Query-Value $firstHwnd
$query.pattern.SetValue('winmakase-proof')
$query.element.SetFocus()

$record.repeatedFocused = Invoke-LauncherCli @()
$record.afterRepeatedFocused = Read-RunState
$afterQuery = Query-Value $firstHwnd
$record.repeatedQuery = $afterQuery.value
if ($record.repeatedFocused.exitCode -ne 0 -or $record.repeatedFocused.json.outcome -ne 'already_focused' -or $record.repeatedFocused.json.signalled) { throw 'focused repeat was not idempotent' }
if ($record.repeatedQuery -ne 'winmakase-proof' -or [long](Get-VisibleLauncher)[0].hwnd -ne $firstHwnd) { throw 'focused repeat changed query or HWND' }

Ensure-Hidden
$out1='C:/WinmakaseI1Evidence/concurrent-1.stdout.txt'; $err1='C:/WinmakaseI1Evidence/concurrent-1.stderr.txt'
$out2='C:/WinmakaseI1Evidence/concurrent-2.stdout.txt'; $err2='C:/WinmakaseI1Evidence/concurrent-2.stderr.txt'
$one=Start-Process $Winmakase -ArgumentList 'launcher','open','--json' -RedirectStandardOutput $out1 -RedirectStandardError $err1 -PassThru
$two=Start-Process $Winmakase -ArgumentList 'launcher','open','--json' -RedirectStandardOutput $out2 -RedirectStandardError $err2 -PassThru
$one.WaitForExit(); $two.WaitForExit()
$concurrent=@(@($one,$two) | ForEach-Object { $_.Refresh(); [ordered]@{ pid=$_.Id; exitCode=$_.ExitCode } })
$json1=Get-Content $out1 -Raw | ConvertFrom-Json; $json2=Get-Content $out2 -Raw | ConvertFrom-Json
$reports=@($json1,$json2)
$record.concurrent=[ordered]@{ processes=$concurrent; reports=$reports; state=(Wait-Launcher $true) }
if (@($reports | Where-Object { $_.signalled }).Count -ne 1 -or @($reports | Where-Object { !$_.outcome }).Count) { throw 'concurrent callers did not serialize to one signal' }

Ensure-Hidden
$helperProcess=Start-Process $Helper -PassThru
$helperProcess.WaitForExit(); $helperProcess.Refresh()
$record.helper=[ordered]@{ exitCode=$helperProcess.ExitCode; state=(Wait-Launcher $true) }
if ($helperProcess.ExitCode -ne 0) { throw 'GUI-subsystem helper failed' }

Get-Process PowerToys.PowerLauncher -ErrorAction Stop | Stop-Process -Force
Start-Sleep -Milliseconds 200
$record.missingProcess = Invoke-LauncherCli @('--timeout-ms','100')
if ($record.missingProcess.exitCode -eq 0 -or $record.missingProcess.json.kind -ne 'not_running') { throw 'missing process did not report not_running' }

$record.verdict='pass'
$record | ConvertTo-Json -Depth 10 | Set-Content C:/WinmakaseI1Evidence/launcher-proof.json -Encoding UTF8
