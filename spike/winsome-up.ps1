# Bring up the Winsome dogfood stack. SUPERSEDED as the bring-up path by the
# supervisor (WinsomeSupervisor runs at logon and owns the components); this
# script remains for after a panic or a manual stop: it hides the taskbar
# (which the supervisor does not own — that is Zebar-milestone territory) and
# starts the supervisor task, which adopts or starts kanata + GlazeWM itself.
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
schtasks /run /tn WinsomeSupervisor | Out-Null
Write-Output 'taskbar auto-hidden, WinsomeSupervisor triggered (it adopts or starts kanata + GlazeWM)'
