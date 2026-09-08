[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $FixtureRoot,

    [Parameter(Mandatory)]
    [ValidateSet('main-spaces', 'owned-dialog', 'unicode-modal', 'utility-popup')]
    [string] $ActionId,

    [Parameter(Mandatory)]
    [string] $ConsentToken
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Import-Module (Join-Path $PSScriptRoot 'Fixture.Common.psm1') -Force

$root = Assert-WindowsSandboxGuest -FixtureRoot $FixtureRoot -ConsentToken $ConsentToken
$expectationsPath = Join-Path $root 'expectations.json'
if (-not (Test-Path -LiteralPath $expectationsPath -PathType Leaf)) {
    throw 'Fixture expectations are missing. Run Initialize-P0Guest.ps1 first.'
}
$expectations = Get-Content -LiteralPath $expectationsPath -Raw -Encoding UTF8 | ConvertFrom-Json
$action = @($expectations.actions | Where-Object { $_.id -ceq $ActionId })
if ($action.Count -ne 1) {
    throw "Fixture action was not found exactly once: $ActionId"
}

Start-Process -FilePath $action[0].targetPath `
    -WorkingDirectory $action[0].workingDirectory `
    -ArgumentList $action[0].argumentsString
