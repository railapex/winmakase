[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $FixtureRoot,

    [Parameter(Mandatory)]
    [string] $DestinationDirectory,

    [Parameter(Mandatory)]
    [string] $ConsentToken
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Import-Module (Join-Path $PSScriptRoot 'Fixture.Common.psm1') -Force

$root = Assert-WindowsSandboxGuest -FixtureRoot $FixtureRoot -ConsentToken $ConsentToken
$expectedDestination = Join-Path ([Environment]::GetFolderPath('Programs')) 'Winmakase P0 Fixtures'
$actualDestination = Get-CanonicalPath $DestinationDirectory
if (-not $actualDestination.Equals(
    (Get-CanonicalPath $expectedDestination),
    [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "DestinationDirectory must be exactly $expectedDestination"
}

if (Test-Path -LiteralPath $actualDestination) {
    $publicationMarker = Join-Path $actualDestination '.winmakase-p0-published'
    if (-not (Test-Path -LiteralPath $publicationMarker -PathType Leaf) -or
        (Get-Content -LiteralPath $publicationMarker -Raw).Trim() -cne 'winmakase-p0-disposable-publication-v1') {
        throw 'Existing destination is not owned by the P0 fixture publisher.'
    }
}
else {
    [void] (New-Item -ItemType Directory -Path $actualDestination)
    Set-Content -LiteralPath (Join-Path $actualDestination '.winmakase-p0-published') `
        -Value 'winmakase-p0-disposable-publication-v1' -NoNewline -Encoding ASCII
}

$manifest = Read-FixtureManifest $root
foreach ($action in @($manifest.actions)) {
    $source = Join-Path (Join-Path $root 'shortcuts') $action.shortcutFile
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "Generated shortcut is missing: $source"
    }
    Copy-Item -LiteralPath $source -Destination (Join-Path $actualDestination $action.shortcutFile) -Force
}

Write-Output "Published four fixture shortcuts to the disposable guest Start Menu: $actualDestination"
