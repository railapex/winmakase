$host.UI.RawUI.WindowTitle = 'F13-PROBE'
$log = Join-Path $PSScriptRoot 'f13-keylog.txt'
Set-Content $log ''
Write-Host 'F13 probe: logging every key this console receives. Close the window to stop.'
while ($true) {
    $k = [Console]::ReadKey($true)
    $c = [int]$k.KeyChar
    $ch = if ($c -ge 32) { [string]$k.KeyChar } else { '\x{0:x2}' -f $c }
    $line = '{0:HH:mm:ss.fff} key={1} char=[{2}] mods={3}' -f (Get-Date), $k.Key, $ch, $k.Modifiers
    Add-Content $log $line
    Write-Host $line
}
