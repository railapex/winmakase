# Summon — pull the most recent window off the hidden 'scratch' workspace onto
# the focused workspace, floated + centered + focused. Bound to rwin+s; the
# banish half is pure GlazeWM (rwin+alt+s = move --workspace scratch).
# Spike-grade stand-in for the `winsome scratchpad` verb
# (docs/research/gap-features.md). No scratch windows = silent no-op.
$ErrorActionPreference = 'Stop'

$reply = glazewm query workspaces | ConvertFrom-Json
$workspaces = $reply.data.workspaces
$scratch = $workspaces | Where-Object { $_.name -eq 'scratch' }
if (-not $scratch) { exit 0 }

# Windows can nest under splits; collect descendants of type 'window'.
function Get-Windows($node) {
    foreach ($child in @($node.children)) {
        if ($child.type -eq 'window') { $child }
        elseif ($child.children) { Get-Windows $child }
    }
}
$windows = @(Get-Windows $scratch)
if ($windows.Count -eq 0) { exit 0 }

# Most recently focused window wins; focus order can name a split, so fall
# back to the first window found.
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
