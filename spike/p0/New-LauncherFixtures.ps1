[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $FixtureRoot,

    [Parameter(Mandatory)]
    [string] $ExecutablePath
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Import-Module (Join-Path $PSScriptRoot 'Fixture.Common.psm1') -Force

$root = Assert-SafeFixtureRoot $FixtureRoot
$executable = Get-CanonicalPath $ExecutablePath
$expectedBin = Join-Path $root 'bin'
if (-not (Test-PathInside -Parent $expectedBin -Child $executable) -or
    -not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw "Fixture executable must exist beneath $expectedBin"
}
$binItem = Get-Item -LiteralPath $expectedBin -Force
if (($binItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
    throw "Fixture bin directory cannot be a reparse point: $expectedBin"
}

$manifest = Read-FixtureManifest $root
$expectedExecutable = Get-CanonicalPath (Join-Path $root $manifest.executableRelativePath)
if (-not $executable.Equals($expectedExecutable, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Fixture executable must be exactly $expectedExecutable"
}

$shortcutDirectory = Join-Path $root 'shortcuts'
if (-not (Test-Path -LiteralPath $shortcutDirectory)) {
    [void] (New-Item -ItemType Directory -Path $shortcutDirectory)
}
$shortcutDirectoryItem = Get-Item -LiteralPath $shortcutDirectory -Force
if (($shortcutDirectoryItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
    throw "Fixture shortcut directory cannot be a reparse point: $shortcutDirectory"
}
$unexpectedShortcuts = @(Get-ChildItem -LiteralPath $shortcutDirectory -Filter '*.lnk' -File |
    Where-Object { @($manifest.actions.shortcutFile) -cnotcontains $_.Name })
if ($unexpectedShortcuts.Count -gt 0) {
    throw "Shortcut directory contains unmanaged .lnk files: $($unexpectedShortcuts.Name -join ', ')"
}

$expectations = @()
foreach ($action in @($manifest.actions)) {
    $shortcutPath = Join-Path $shortcutDirectory $action.shortcutFile
    $arguments = @($action.arguments | ForEach-Object { [string] $_ })
    $argumentsString = Join-WindowsCommandLine -Arguments $arguments
    New-UnicodeShortcut `
        -ShortcutPath $shortcutPath `
        -TargetPath $executable `
        -Arguments $argumentsString `
        -WorkingDirectory (Split-Path -Parent $executable) `
        -Description "Winmakase P0 disposable fixture: $($action.id)" `
        -IconPath $executable

    $expectations += [ordered]@{
        id = $action.id
        shortcutFile = $action.shortcutFile
        targetPath = $executable
        workingDirectory = Split-Path -Parent $executable
        arguments = $arguments
        argumentsString = $argumentsString
        expectedCapturePath = "data/$($action.id).json"
        expectedCapture = [ordered]@{
            schemaVersion = 1
            executableName = 'WinmakaseP0Fixture.exe'
            fixtureId = $action.expectedCapture.fixtureId
            window = $action.expectedCapture.window
            profileLabel = $action.expectedCapture.profileLabel
            arguments = $arguments
        }
    }
}

$export = [ordered]@{
    schemaVersion = 1
    fixtureRoot = $root
    fixedTitles = $manifest.fixedTitles
    actions = $expectations
}
$expectationPath = Join-Path $root 'expectations.json'
$export | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $expectationPath -Encoding UTF8

Write-Output $expectationPath
