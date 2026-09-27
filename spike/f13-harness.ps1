# Injected-input check of stock Glaze f13+X chords against a disposable console probe.
# Every injection is gated on the probe being the foreground window.
$ErrorActionPreference = 'Stop'
$S = $PSScriptRoot
$cli = "$env:USERPROFILE/.winmakase/bin/glazewm-cli.exe"

Add-Type @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class K {
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, string title);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h, StringBuilder sb, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder sb, int n);
  public static string Cls(IntPtr h) { var sb = new StringBuilder(256); GetClassName(h, sb, 256); return sb.ToString(); }
  public static string Title(IntPtr h) { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); return sb.ToString(); }
}
'@

$VK = @{ F13 = 0x7C; SHIFT = 0xA0; CTRL = 0xA2; ALT = 0xA4; T = 0x54; L = 0x4C }
function Down($k) { [K]::keybd_event([byte]$k, 0, 0, [UIntPtr]::Zero) }
function Up($k) { [K]::keybd_event([byte]$k, 0, 2, [UIntPtr]::Zero) }
function Digit($n) { 0x30 + $n }

function Windows {
    $tree = (& $cli query monitors | ConvertFrom-Json).data.monitors
    foreach ($m in $tree) { foreach ($ws in $m.children) {
        foreach ($w in @($ws.children)) { if ($w.type -eq 'window') { [pscustomobject]@{ ws = $ws.name; handle = [int64]$w.handle; state = $w.state.type; focus = $w.hasFocus } } }
    } }
}
function FocusedWs { ((& $cli query monitors | ConvertFrom-Json).data.monitors.children | Where-Object hasFocus).name }

$results = [System.Collections.Generic.List[object]]::new()
function Record($name, $pass, $detail) {
    $results.Add([pscustomobject]@{ test = $name; pass = $pass; detail = $detail }) | Out-Null
    '{0,-34} {1,-5} {2}' -f $name, ($(if ($pass) { 'PASS' } else { 'FAIL' })), $detail
}

# Reuse or launch the probe; find it by title (FindWindow misses it from this process).
function ProbeProc { Get-Process pwsh -ErrorAction SilentlyContinue | Where-Object MainWindowTitle -eq 'F13-PROBE' | Select-Object -First 1 }
if (-not (ProbeProc)) { Start-Process conhost.exe -ArgumentList "pwsh -NoProfile -File `"$S/f13-keylog.ps1`"" }
foreach ($i in 1..60) { if (ProbeProc) { break }; Start-Sleep -Milliseconds 250 }
if (-not (ProbeProc)) { throw 'probe window not found' }
$probe = (ProbeProc).MainWindowHandle
Start-Sleep -Seconds 1
# Ask Glaze to focus the probe (Glaze owns foreground rights), then verify.
$pid0 = ((& $cli query windows | ConvertFrom-Json).data.windows | Where-Object handle -eq $probe.ToInt64()).id
if (-not $pid0) { throw 'Glaze is not managing the probe' }
& $cli command focus --container-id $pid0 | Out-Null
Start-Sleep -Milliseconds 600
function Guard($step) {
    $fg = [K]::GetForegroundWindow()
    if ($fg -ne $probe) { throw "ABORT before '$step': foreground is '$([K]::Title($fg))' ($([K]::Cls($fg))), not the probe" }
}
function StartMenuUp { $fg = [K]::GetForegroundWindow(); ([K]::Cls($fg) -eq 'Windows.UI.Core.CoreWindow') }

$me = Windows | Where-Object handle -eq $probe.ToInt64()
if (-not $me) { throw 'Glaze is not managing the probe' }
$homeWs = $me.ws
"probe hwnd=$($probe.ToInt64()) managed on workspace $homeWs state=$($me.state)"
$target = if ($homeWs -eq '5') { 2 } else { 5 }

# 1. Move probe to another workspace and follow: F13+Shift+N.
Guard 'move'
Down $VK.F13; Down $VK.SHIFT; Down (Digit $target); Up (Digit $target); Up $VK.SHIFT; Up $VK.F13
Start-Sleep -Milliseconds 700
$me = Windows | Where-Object handle -eq $probe.ToInt64()
Record 'f13+shift+N move+follow' (($me.ws -eq "$target") -and ((FocusedWs) -eq "$target")) "probe ws=$($me.ws) focused ws=$(FocusedWs)"
Record 'no Start menu after chord' (-not (StartMenuUp)) "fg class=$([K]::Cls([K]::GetForegroundWindow()))"

# 2. Float toggle twice: F13+T.
Guard 'float'
Down $VK.F13; Down $VK.T; Up $VK.T; Up $VK.F13
Start-Sleep -Milliseconds 900
$s1 = (Windows | Where-Object handle -eq $probe.ToInt64()).state
Guard 'tile'
Down $VK.F13; Down $VK.T; Up $VK.T; Up $VK.F13
Start-Sleep -Milliseconds 900
$s2 = (Windows | Where-Object handle -eq $probe.ToInt64()).state
Record 'f13+t float then tile' (($s1 -eq 'floating') -and ($s2 -eq 'tiling')) "after1=$s1 after2=$s2"

# 3. F13 held with autorepeat: what does the app receive?
Guard 'hold'
$before = @(Get-Content "$S/f13-keylog.txt").Count
Down $VK.F13; foreach ($i in 1..8) { Start-Sleep -Milliseconds 40; Down $VK.F13 }; Up $VK.F13
Start-Sleep -Milliseconds 500
$new = @(Get-Content "$S/f13-keylog.txt") | Select-Object -Skip $before
Record 'bare F13 reaches app as key only' ((@($new | Where-Object { $_ -notmatch 'key=F13 char=\[\\x00\]' }).Count -eq 0)) "$(@($new).Count) events: $((@($new) | Select-Object -First 1) -replace '^\S+ ','')"

# 4. Unbound F13+L: must type 'l' and must not lock.
Guard 'f13+l'
$before = @(Get-Content "$S/f13-keylog.txt").Count
Down $VK.F13; Down $VK.L; Up $VK.L; Up $VK.F13
Start-Sleep -Milliseconds 700
$new = @(Get-Content "$S/f13-keylog.txt") | Select-Object -Skip $before
$locked = [bool](Get-Process LogonUI -ErrorAction SilentlyContinue)
Record 'f13+l unbound: no lock' (-not $locked) "LogonUI running=$locked"
Record 'f13+l unbound: letter typed' ([bool](@($new) -match 'key=L char=\[l\]')) "$((@($new) -replace '^\S+ ','') -join ' | ')"

# 5. Move back silently, then focus there: F13+Shift+Alt+home, F13+home.
Guard 'return'
Down $VK.F13; Down $VK.SHIFT; Down $VK.ALT; Down (Digit ([int]$homeWs)); Up (Digit ([int]$homeWs)); Up $VK.ALT; Up $VK.SHIFT; Up $VK.F13
Start-Sleep -Milliseconds 700
$me = Windows | Where-Object handle -eq $probe.ToInt64()
Record 'f13+shift+alt+N silent move' (($me.ws -eq "$homeWs") -and ((FocusedWs) -eq "$target")) "probe ws=$($me.ws) focused ws=$(FocusedWs)"
Down $VK.F13; Down (Digit ([int]$homeWs)); Up (Digit ([int]$homeWs)); Up $VK.F13
Start-Sleep -Milliseconds 700
Record 'f13+N focus workspace' ((FocusedWs) -eq "$homeWs") "focused ws=$(FocusedWs) fg=$([K]::Title([K]::GetForegroundWindow()))"
Record 'no Start menu at end' (-not (StartMenuUp)) "fg class=$([K]::Cls([K]::GetForegroundWindow()))"

"--- probe keylog ---"
Get-Content "$S/f13-keylog.txt"
$results | ConvertTo-Json | Set-Content "$S/f13-results.json"
