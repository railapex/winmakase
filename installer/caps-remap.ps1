# Caps -> F13 and ScrLk -> Caps through the registry Scancode Map: the remap
# that sits below every keyboard hook (input recut decision 3). Windows reads
# it at boot, for every user and at the sign-in screen. Writing needs admin;
# showing does not.
#
#   pwsh -File caps-remap.ps1                   show the current map
#   pwsh -File caps-remap.ps1 -Apply [-DryRun]  add our two entries, keep any others
#   pwsh -File caps-remap.ps1 -Remove [-DryRun] take our two entries out again
#
# Every write first saves the previous value (or its absence) under
# ~/.winmakase/backups/. A change takes effect at the next boot.
#
# PowerToys Keyboard Manager must stop remapping Caps and ScrLk once this map
# is active. With both, ScrLk becomes Caps here and then F13 in Keyboard
# Manager, and the raw-CapsLock escape hatch is gone.
param(
    [switch]$Apply,
    [switch]$Remove,
    [switch]$DryRun
)
$ErrorActionPreference = 'Stop'

$keyPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\Keyboard Layout'
$valueName = 'Scancode Map'
# Source scancode -> target scancode. A plain hashtable on purpose: an
# [ordered] one reads an integer index as a position, not a key, so
# $ours[0x3A] came back $null and wrote both keys as disabled.
$ours = @{ 0x003A = 0x0064; 0x0046 = 0x003A }
$names = @{ 0x003A = 'Caps'; 0x0046 = 'ScrLk'; 0x0064 = 'F13'; 0x0000 = 'disabled' }

function Get-KeyName([int]$code) {
    if ($names.ContainsKey($code)) { return $names[$code] }
    return ('0x{0:X4}' -f $code)
}

function Read-ScancodeMap {
    $item = Get-ItemProperty -Path $keyPath -Name $valueName -ErrorAction SilentlyContinue
    if ($null -eq $item) { return $null }
    [byte[]]$bytes = $item.$valueName
    if ($bytes.Length -lt 12) { throw "$valueName is $($bytes.Length) bytes, shorter than its header" }
    $count = [BitConverter]::ToUInt32($bytes, 8)
    if ($count -lt 1 -or $bytes.Length -lt 12 + 4 * $count) {
        throw "$valueName claims $count entries but holds $($bytes.Length - 12) entry bytes"
    }
    $entries = [System.Collections.Generic.List[object]]::new()
    for ($i = 0; $i -lt $count; $i++) {
        $to = [BitConverter]::ToUInt16($bytes, 12 + 4 * $i)
        $from = [BitConverter]::ToUInt16($bytes, 14 + 4 * $i)
        if ($to -eq 0 -and $from -eq 0) { break }
        $entries.Add([pscustomobject]@{ From = [int]$from; To = [int]$to })
    }
    return , $entries
}

function ConvertTo-ScancodeBytes($entries) {
    $bytes = [System.Collections.Generic.List[byte]]::new()
    $bytes.AddRange([byte[]]::new(8))
    $bytes.AddRange([BitConverter]::GetBytes([uint32]($entries.Count + 1)))
    foreach ($e in $entries) {
        $bytes.AddRange([BitConverter]::GetBytes([uint16]$e.To))
        $bytes.AddRange([BitConverter]::GetBytes([uint16]$e.From))
    }
    $bytes.AddRange([byte[]]::new(4))
    return , $bytes.ToArray()
}

function Show-Entries([string]$title, $entries) {
    if ($null -eq $entries -or $entries.Count -eq 0) {
        Write-Output "${title}: no Scancode Map"
        return
    }
    Write-Output "${title}:"
    foreach ($e in $entries) {
        Write-Output ('  {0} -> {1}' -f (Get-KeyName $e.From), (Get-KeyName $e.To))
    }
}

$current = Read-ScancodeMap
if (-not $Apply -and -not $Remove) {
    Show-Entries 'Current map' $current
    exit 0
}
if ($Apply -and $Remove) { throw 'Choose -Apply or -Remove, not both.' }

$next = [System.Collections.Generic.List[object]]::new()
if ($null -ne $current) {
    foreach ($e in $current) {
        $isOurs = $ours.Contains($e.From) -and $ours[$e.From] -eq $e.To
        if ($Apply -and $ours.Contains($e.From) -and -not $isOurs) {
            throw ('The map already sends {0} to {1}. Resolve that entry by hand; this script only adds or removes its own.' -f (Get-KeyName $e.From), (Get-KeyName $e.To))
        }
        if (-not $isOurs) { $next.Add($e) }
    }
}
if ($Apply) {
    foreach ($from in ($ours.Keys | Sort-Object)) {
        $next.Add([pscustomobject]@{ From = $from; To = $ours[$from] })
    }
}

Show-Entries 'Current map' $current
Show-Entries 'New map' $next
if ($DryRun) {
    if ($next.Count -gt 0) {
        Write-Output ('Bytes: ' + (((ConvertTo-ScancodeBytes $next) | ForEach-Object { '{0:X2}' -f $_ }) -join ' '))
    }
    Write-Output 'Dry run: nothing written.'
    exit 0
}

$elevated = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $elevated) { throw 'Writing the Scancode Map needs an elevated PowerShell.' }

$backupDir = Join-Path $env:USERPROFILE '.winmakase\backups'
New-Item -ItemType Directory -Force $backupDir | Out-Null
$backup = Join-Path $backupDir ('scancode-map-{0}.txt' -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
$raw = (Get-ItemProperty -Path $keyPath -Name $valueName -ErrorAction SilentlyContinue).$valueName
if ($null -eq $raw) {
    Set-Content -Path $backup -Value 'absent'
} else {
    Set-Content -Path $backup -Value (($raw | ForEach-Object { '{0:X2}' -f $_ }) -join ' ')
}
Write-Output "Previous value saved to $backup"

if ($next.Count -eq 0) {
    Remove-ItemProperty -Path $keyPath -Name $valueName -ErrorAction SilentlyContinue
    Write-Output "$valueName removed."
} else {
    Set-ItemProperty -Path $keyPath -Name $valueName -Type Binary -Value (ConvertTo-ScancodeBytes $next)
    Write-Output "$valueName written."
}
Write-Output 'Takes effect at the next boot. Then remove the Caps and ScrLk rules from PowerToys Keyboard Manager.'
