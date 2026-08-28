# Diagnose: does a held modifier chord re-fire GlazeWM bindings on repeated key presses?
# Synthesizes Ctrl+Alt+Win held + two discrete OEM_PLUS presses (below kanata), measures width.
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class KB {
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public const uint KEYUP = 0x0002;
}
'@
$cli = 'C:\Program Files\glzr.io\GlazeWM\cli\glazewm.exe'
function Get-FocusedWidth { (& $cli query focused | ConvertFrom-Json).data.focused.width }
$before = Get-FocusedWidth
# VK: CONTROL 0x11, MENU(alt) 0x12, LWIN 0x5B, OEM_PLUS 0xBB
[KB]::keybd_event(0x11,0,0,[UIntPtr]::Zero); [KB]::keybd_event(0x12,0,0,[UIntPtr]::Zero); [KB]::keybd_event(0x5B,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 120
[KB]::keybd_event(0xBB,0,0,[UIntPtr]::Zero); Start-Sleep -Milliseconds 60; [KB]::keybd_event(0xBB,0,[KB]::KEYUP,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 250
$mid = Get-FocusedWidth
[KB]::keybd_event(0xBB,0,0,[UIntPtr]::Zero); Start-Sleep -Milliseconds 60; [KB]::keybd_event(0xBB,0,[KB]::KEYUP,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 250
$after = Get-FocusedWidth
[KB]::keybd_event(0x5B,0,[KB]::KEYUP,[UIntPtr]::Zero); [KB]::keybd_event(0x12,0,[KB]::KEYUP,[UIntPtr]::Zero); [KB]::keybd_event(0x11,0,[KB]::KEYUP,[UIntPtr]::Zero)
"width before=$before afterFirst=$mid afterSecond=$after"
if ($mid -ne $before -and $after -ne $mid) { "VERDICT: both presses fired - repeat works below kanata; suspect kanata layer" }
elseif ($mid -ne $before -and $after -eq $mid) { "VERDICT: only FIRST press fired - GlazeWM edge-triggers per chord formation" }
else { "VERDICT: nothing fired - synthetic chord not caught; inconclusive" }
