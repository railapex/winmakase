# WinmakasePanic — back to stock in seconds. Runs elevated via the WinmakasePanic scheduled task.
# The supervisor restarts whatever dies, so panic stops the SUPERVISOR first —
# killing components while it lives means the desktop refuses to die.
# `winmakase down` returns only when the shutdown has FINISHED (or failed); the
# sweep below is for a supervisor that is absent, hung, or died mid-shutdown.
Start-Transcript -Path (Join-Path $env:USERPROFILE '.winmakase\logs\panic-last.log') -Force
$winmakase = Join-Path $env:USERPROFILE '.winmakase\bin\winmakase.exe'
if (Test-Path $winmakase) { & $winmakase down --timeout 15 }
# Sweep: kill anything still standing, supervisor first so nothing respawns.
Get-Process winmakased -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction Continue
schtasks /end /tn WinmakaseKanata 2>$null | Out-Null
Get-Process | Where-Object { $_.ProcessName -like 'kanata*' } | Stop-Process -Force -ErrorAction Continue
& 'C:\Program Files\glzr.io\GlazeWM\cli\glazewm.exe' command wm-exit 2>$null
Start-Sleep -Seconds 2
Get-Process glazewm* -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction Continue
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class AppBar {
  [StructLayout(LayoutKind.Sequential)]
  public struct APPBARDATA { public uint cbSize; public IntPtr hWnd; public uint uCallbackMessage; public uint uEdge; public RECT rc; public int lParam; }
  [StructLayout(LayoutKind.Sequential)]
  public struct RECT { public int left, top, right, bottom; }
  [DllImport("shell32.dll")] public static extern uint SHAppBarMessage(uint dwMessage, ref APPBARDATA pData);
}
'@
$d = New-Object AppBar+APPBARDATA; $d.cbSize = [System.Runtime.InteropServices.Marshal]::SizeOf($d); $d.lParam = 0
[AppBar]::SHAppBarMessage(0x0000000A, [ref]$d) | Out-Null
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class TrayShow {
  public delegate bool CB(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(CB cb, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Auto)] public static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
}
'@
$cb = [TrayShow+CB]{
  param($h, $l)
  $sb = New-Object System.Text.StringBuilder 256
  [void][TrayShow]::GetClassName($h, $sb, 256)
  $cls = $sb.ToString()
  if ($cls -eq 'Shell_TrayWnd' -or $cls -eq 'Shell_SecondaryTrayWnd') {
    [void][TrayShow]::ShowWindow($h, 5) # SW_SHOW
  }
  $true
}
[void][TrayShow]::EnumWindows($cb, [IntPtr]::Zero)
$survivors = Get-Process winmakased,glazewm,kanata* -ErrorAction SilentlyContinue
if ($survivors) { Write-Output "SURVIVORS: $($survivors.ProcessName -join ', ')" }
else { Write-Output 'stock restored: supervisor down, kanata dead, GlazeWM exited, taskbar visible' }
Stop-Transcript
