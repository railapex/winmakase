# Bring up the Winsome dogfood stack, UAC-free (kanata via pre-registered elevated task).
# Run NON-elevated so GlazeWM stays user-level.
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
$d = New-Object AppBar+APPBARDATA; $d.cbSize = [System.Runtime.InteropServices.Marshal]::SizeOf($d); $d.lParam = 0x1
[AppBar]::SHAppBarMessage(0x0000000A, [ref]$d) | Out-Null
Start-Process 'C:\Program Files\glzr.io\GlazeWM\glazewm.exe'
schtasks /run /tn WinsomeKanata | Out-Null
Write-Output 'GlazeWM up (user-level), kanata task triggered (elevated), taskbar auto-hidden'
