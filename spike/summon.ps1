# Rescue-or-summon — bound to rwin+s. Two behaviors, one key:
#
# 1. RESCUE: if the FOCUSED window is not presentable — DWM-cloaked (the
#    ApplicationFrameHost/Settings ghost), Win32-invisible, or parked on a
#    non-displayed workspace — pull it into view: float centered, shown on
#    top, focused. Alt+Tab to a ghost makes it the focused window; this key
#    then materializes it.
# 2. SUMMON: otherwise pull the most recently focused window off the hidden
#    'scratch' workspace (banish half: rwin+alt+s = move --workspace scratch).
#
# Nothing in scratch and nothing abnormal = silent no-op. Spike-grade
# stand-in for the `winsome scratchpad` verb (docs/research/gap-features.md).
$ErrorActionPreference = 'Stop'

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class Native {
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr hwnd, int attr, out int value, int size);
}
'@

function Get-Windows($node) {
    foreach ($child in @($node.children)) {
        if ($child.type -eq 'window') { $child }
        elseif ($child.children) { Get-Windows $child }
    }
}

$reply = glazewm query workspaces | ConvertFrom-Json
$workspaces = $reply.data.workspaces

# --- Rescue pass: is the focused window presentable? ---
$focused = (glazewm query focused | ConvertFrom-Json).data.focused
if ($focused.type -eq 'window' -and $focused.state.type -ne 'minimized') {
    $hwnd = [IntPtr][int64]$focused.handle
    $cloaked = 0
    [void][Native]::DwmGetWindowAttribute($hwnd, 14, [ref]$cloaked, 4)  # DWMWA_CLOAKED
    $visible = [Native]::IsWindowVisible($hwnd)
    $ownWs = $workspaces | Where-Object {
        (@(Get-Windows $_) | ForEach-Object { $_.id }) -contains $focused.id
    } | Select-Object -First 1

    if (($cloaked -ne 0) -or (-not $visible) -or ($ownWs -and -not $ownWs.isDisplayed)) {
        if ($ownWs -and -not $ownWs.isDisplayed) {
            # Pull to the displayed workspace on the same monitor.
            $here = ($workspaces | Where-Object {
                $_.parentId -eq $ownWs.parentId -and $_.isDisplayed
            } | Select-Object -First 1).name
            if ($here) {
                glazewm command --id $focused.id move --workspace $here | Out-Null
            }
        }
        glazewm command --id $focused.id set-floating --centered --shown-on-top | Out-Null
        glazewm command focus --container-id $focused.id | Out-Null
        exit 0
    }
}

# --- Summon pass: pocket recall from 'scratch'. ---
$scratch = $workspaces | Where-Object { $_.name -eq 'scratch' }
if (-not $scratch) { exit 0 }
$windows = @(Get-Windows $scratch)
if ($windows.Count -eq 0) { exit 0 }

$windowIds = $windows | ForEach-Object { $_.id }
$target = @($scratch.childFocusOrder | Where-Object { $windowIds -contains $_ })[0]
if (-not $target) { $target = $windowIds[0] }

$here = ($workspaces | Where-Object { $_.hasFocus } | Select-Object -First 1).name
if (-not $here) {
    $here = ($workspaces | Where-Object { $_.isDisplayed } | Select-Object -First 1).name
}
if (-not $here) { exit 0 }

glazewm command --id $target move --workspace $here | Out-Null
glazewm command --id $target set-floating --centered --shown-on-top | Out-Null
glazewm command focus --container-id $target | Out-Null
