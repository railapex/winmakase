# Repeat diagnosis v2: workspace switches under a continuously-held synthetic chord.
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class KB {
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public const uint KEYUP = 0x0002;
}
'@
$cli = 'C:\Program Files\glzr.io\GlazeWM\cli\glazewm.exe'
function Get-FocusedWs { ((& $cli query workspaces | ConvertFrom-Json).data.workspaces | Where-Object { $_.hasFocus }).name }
$orig = Get-FocusedWs
"start ws=$orig"
[KB]::keybd_event(0x11,0,0,[UIntPtr]::Zero); [KB]::keybd_event(0x12,0,0,[UIntPtr]::Zero); [KB]::keybd_event(0x5B,0,0,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 120
[KB]::keybd_event(0x31,0,0,[UIntPtr]::Zero); Start-Sleep -Milliseconds 60; [KB]::keybd_event(0x31,0,[KB]::KEYUP,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 300
$q1 = Get-FocusedWs
[KB]::keybd_event(0x33,0,0,[UIntPtr]::Zero); Start-Sleep -Milliseconds 60; [KB]::keybd_event(0x33,0,[KB]::KEYUP,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 300
$q2 = Get-FocusedWs
[KB]::keybd_event(0x5B,0,[KB]::KEYUP,[UIntPtr]::Zero); [KB]::keybd_event(0x12,0,[KB]::KEYUP,[UIntPtr]::Zero); [KB]::keybd_event(0x11,0,[KB]::KEYUP,[UIntPtr]::Zero)
Start-Sleep -Milliseconds 200
& $cli command "focus --workspace $orig" | Out-Null
"after first press (1): ws=$q1 · after second press (3, mods never released): ws=$q2"
if ($q1 -eq '1' -and $q2 -eq '3') { "VERDICT: both fired under held mods - GlazeWM repeats fine; suspect kanata layer" }
elseif ($q1 -eq '1' -and $q2 -eq '1') { "VERDICT: only first fired - GlazeWM edge-triggers per chord formation" }
else { "VERDICT: inconclusive (q1=$q1 q2=$q2)" }
