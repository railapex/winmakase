[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $FixtureRoot,

    [Parameter(Mandatory)]
    [ValidateSet('main-spaces', 'owned-dialog', 'unicode-modal', 'utility-popup')]
    [string] $ActionId,

    [Parameter(Mandatory)]
    [ValidatePattern('^[a-z0-9][a-z0-9-]{0,31}$')]
    [string] $BeforePhase,

    [Parameter(Mandatory)]
    [ValidatePattern('^[a-z0-9][a-z0-9-]{0,31}$')]
    [string] $AfterPhase,

    [Parameter(Mandatory)]
    [string] $ConsentToken
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Import-Module (Join-Path $PSScriptRoot 'Fixture.Common.psm1') -Force
Import-Module (Join-Path $PSScriptRoot 'DialogProof.Common.psm1') -Force

$root = Assert-WindowsSandboxGuest -FixtureRoot $FixtureRoot -ConsentToken $ConsentToken
$proofDirectory = Join-Path $root 'data/dialog-proof'
$beforePath = Join-Path $proofDirectory "$ActionId-$BeforePhase.json"
$afterPath = Join-Path $proofDirectory "$ActionId-$AfterPhase.json"
foreach ($required in @($beforePath, $afterPath)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
        throw "Required dialog proof is missing: $required"
    }
}

$before = Get-Content -LiteralPath $beforePath -Raw -Encoding UTF8 | ConvertFrom-Json
$after = Get-Content -LiteralPath $afterPath -Raw -Encoding UTF8 | ConvertFrom-Json
if ([string] $before.fixtureId -cne $ActionId -or [string] $after.fixtureId -cne $ActionId -or
    [string] $before.phase -cne $BeforePhase -or [string] $after.phase -cne $AfterPhase) {
    throw 'Dialog proof identity does not match the requested action/phases.'
}

$comparison = Compare-P0DialogProofObjects -Before $before -After $after
$comparisonPath = Join-Path $proofDirectory "$ActionId-$BeforePhase-to-$AfterPhase-comparison.json"
$json = $comparison | ConvertTo-Json -Depth 10
[System.IO.File]::WriteAllText($comparisonPath, $json, (New-Object System.Text.UTF8Encoding($false)))
Write-Output $comparisonPath
