# WinsomePanic — back to stock in seconds. Runs elevated via the WinsomePanic scheduled task.
Get-Process | Where-Object { $_.ProcessName -like 'kanata*' } | Stop-Process -Force -ErrorAction SilentlyContinue
& 'C:\Program Files\glzr.io\GlazeWM\cli\glazewm.exe' command wm-exit 2>$null
Start-Sleep -Seconds 2
Get-Process glazewm* -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
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
Write-Output 'stock restored: kanata dead, GlazeWM exited (windows restored), taskbar visible'
