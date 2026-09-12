param([ValidateSet('Observe','Signal')][string]$Action='Observe',[string]$Label='run')
$ErrorActionPreference='Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
if ($Label -notmatch '^[a-z0-9-]+$') { throw 'Invalid evidence label' }
Add-Type -AssemblyName UIAutomationClient,UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class RunWindowProbe {
  public delegate bool EnumProc(IntPtr h, IntPtr p);
  [StructLayout(LayoutKind.Sequential)] public struct Rect { public int left,top,right,bottom; }
  public sealed class Window { public long hwnd; public uint pid; public string title; public bool visible; public int cloaked; public Rect rect; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc fn, IntPtr p);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
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
$runProcesses=@(Get-Process -Name PowerToys.PowerLauncher -ErrorAction Stop)
foreach($run in $runProcesses) {
  if((Get-FileHash -LiteralPath $run.Path -Algorithm SHA256).Hash -ne '062ED25AA8F343C83641277111BCCAC366B25EBF80021B1E4D783EEF25D2E688') { throw 'Unexpected Run process image' }
}
[uint32[]]$runIds=@($runProcesses | ForEach-Object Id)
function Read-RunState {
  $foreground=[RunWindowProbe]::GetForegroundWindow()
  [uint32]$foregroundPid=0
  [void][RunWindowProbe]::GetWindowThreadProcessId($foreground,[ref]$foregroundPid)
  $focus=$null
  try {
    $focused=[Windows.Automation.AutomationElement]::FocusedElement
    if($focused -and $runIds -contains $focused.Current.ProcessId) {
      $focus=[ordered]@{processId=$focused.Current.ProcessId; name=$focused.Current.Name; type=$focused.Current.ControlType.ProgrammaticName; isKeyboardFocused=$focused.Current.HasKeyboardFocus}
    }
  } catch {}
  [ordered]@{windows=@([RunWindowProbe]::Windows($runIds)); foregroundHwnd=$foreground.ToInt64(); foregroundPid=$foregroundPid; focusedRunElement=$focus}
}
$before=Read-RunState
$event=[Threading.EventWaitHandle]::OpenExisting('Local\PowerToysRunInvokeEvent-30f26ad7-d36d-4c0e-ab02-68bb5ff3c4ab')
$timer=[Diagnostics.Stopwatch]::StartNew()
try {
  if($Action -eq 'Signal') { [void]$event.Set() }
} finally { $event.Dispose() }
$after=Read-RunState
if($Action -eq 'Signal') {
  $beforeVisible=@($before.windows | Where-Object { $_.visible -and $_.cloaked -eq 0 }).Count -gt 0
  do {
    $after=Read-RunState
    $afterVisible=@($after.windows | Where-Object { $_.visible -and $_.cloaked -eq 0 }).Count -gt 0
    if($beforeVisible -and !$afterVisible) { break }
    if(!$beforeVisible -and $afterVisible -and $after.focusedRunElement -and $after.focusedRunElement.isKeyboardFocused) { break }
    Start-Sleep -Milliseconds 10
  } while($timer.ElapsedMilliseconds -lt 3000)
}
$record=[ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o'); action=$Action; elapsedMs=$timer.Elapsed.TotalMilliseconds; before=$before; after=$after; note='Timing includes UIA observation; not a pure activation benchmark.'}
$record | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath "C:/WinmakaseEvidence/$Label.json" -Encoding UTF8
