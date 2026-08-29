# WinsomePanic — back to stock in seconds. Runs elevated via the WinsomePanic scheduled task.
# The supervisor restarts whatever dies, so panic stops the SUPERVISOR first —
# killing components while it lives means the desktop refuses to die.
# `winsome down` returns only when the shutdown has FINISHED (or failed); the
# sweep below is for a supervisor that is absent, hung, or died mid-shutdown.
Start-Transcript -Path (Join-Path $env:USERPROFILE '.winsome\logs\panic-last.log') -Force
$winsome = Join-Path $env:USERPROFILE '.winsome\bin\winsome.exe'
if (Test-Path $winsome) { & $winsome down --timeout 15 }
# Sweep: kill anything still standing, supervisor first so nothing respawns.
Get-Process winsomed -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction Continue
schtasks /end /tn WinsomeKanata 2>$null | Out-Null
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
$survivors = Get-Process winsomed,glazewm,kanata* -ErrorAction SilentlyContinue
if ($survivors) { Write-Output "SURVIVORS: $($survivors.ProcessName -join ', ')" }
else { Write-Output 'stock restored: supervisor down, kanata dead, GlazeWM exited, taskbar visible' }
Stop-Transcript
