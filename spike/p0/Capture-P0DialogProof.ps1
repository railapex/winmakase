[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $FixtureRoot,

    [Parameter(Mandatory)]
    [ValidateSet('main-spaces', 'owned-dialog', 'unicode-modal', 'utility-popup')]
    [string] $ActionId,

    [Parameter(Mandatory)]
    [ValidatePattern('^[a-z0-9][a-z0-9-]{0,31}$')]
    [string] $Phase,

    [Parameter(Mandatory)]
    [string] $GlazeExecutablePath,

    [Parameter(Mandatory)]
    [string] $ConsentToken
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Import-Module (Join-Path $PSScriptRoot 'Fixture.Common.psm1') -Force
Import-Module (Join-Path $PSScriptRoot 'DialogProof.Common.psm1') -Force

# Keep this guard before loading interop or invoking Glaze. Host tests depend on
# rejection happening before any native/UI-facing operation.
$root = Assert-WindowsSandboxGuest -FixtureRoot $FixtureRoot -ConsentToken $ConsentToken
$glaze = Get-CanonicalPath $GlazeExecutablePath
if ([System.IO.Path]::GetFileName($glaze) -cne 'glazewm.exe' -or
    -not (Test-Path -LiteralPath $glaze -PathType Leaf)) {
    throw 'GlazeExecutablePath must identify an existing glazewm.exe.'
}

$manifest = Read-FixtureManifest $root
$action = @($manifest.actions | Where-Object { $_.id -ceq $ActionId })
if ($action.Count -ne 1) {
    throw "Fixture action was not found exactly once: $ActionId"
}
$action = $action[0]

$launchPath = Join-Path $root "data/$ActionId.json"
$registrationPath = Join-Path $root "data/$ActionId-windows.json"
foreach ($required in @($launchPath, $registrationPath)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
        throw "Required fixture capture is missing: $required"
    }
}
$launch = Get-Content -LiteralPath $launchPath -Raw -Encoding UTF8 | ConvertFrom-Json
$registration = Get-Content -LiteralPath $registrationPath -Raw -Encoding UTF8 | ConvertFrom-Json
if ([int] $launch.schemaVersion -ne 1 -or [int] $registration.schemaVersion -ne 2 -or
    [string] $launch.fixtureId -cne $ActionId -or [string] $registration.fixtureId -cne $ActionId -or
    [int] $registration.processId -le 0 -or [long] $registration.processCreationTimeUtcTicks -le 0) {
    throw 'Fixture launch or window registration capture is invalid.'
}

$expectedArguments = @($action.arguments | ForEach-Object { [string] $_ })
foreach ($capturedArguments in @(@($launch.arguments), @($registration.arguments))) {
    if ($capturedArguments.Count -ne $expectedArguments.Count) {
        throw 'Fixture capture arguments do not match the checked-in action tuple.'
    }
    for ($index = 0; $index -lt $expectedArguments.Count; $index++) {
        if ([string] $capturedArguments[$index] -cne $expectedArguments[$index]) {
            throw 'Fixture capture arguments do not match the checked-in action tuple.'
        }
    }
}

$expectedRoles = @(Get-P0ExpectedRoles -ActionId $ActionId)
$registrations = @($registration.windows)
if ($registrations.Count -ne $expectedRoles.Count) {
    throw "Action $ActionId must have active registrations for: $($expectedRoles -join ', ')"
}
for ($index = 0; $index -lt $expectedRoles.Count; $index++) {
    if ([string] $registrations[$index].role -cne $expectedRoles[$index]) {
        throw "Action $ActionId registration order/roles are invalid."
    }
    if ([long] $registrations[$index].handle -eq 0 -or
        [long] $registrations[$index].windowGeneration -le 0) {
        throw "Action $ActionId registration lifetime identity is invalid."
    }
    $titleProperty = $manifest.fixedTitles.PSObject.Properties[[string] $registrations[$index].role]
    if ($null -eq $titleProperty -or [string] $registrations[$index].title -cne [string] $titleProperty.Value) {
        throw "Action $ActionId registration title is invalid."
    }
}

$interopPath = Join-Path $PSScriptRoot 'DialogFactInterop.cs'
if (-not (Test-Path -LiteralPath $interopPath -PathType Leaf)) {
    throw "Dialog fact interop source is missing: $interopPath"
}
if (-not ('Winmakase.P0.DialogFactReader' -as [type])) {
    Add-Type -Path $interopPath
}

$nativeWindows = @()
foreach ($registered in $registrations) {
    $observed = [Winmakase.P0.DialogFactReader]::Snapshot(
        [long] $registered.handle,
        [int] $registration.processId,
        [long] $registration.processCreationTimeUtcTicks,
        [long] $registered.windowGeneration)
    if ([string] $observed.title -cne [string] $registered.title -or
        [long] $observed.handle -ne [long] $registered.handle) {
        throw "Registered fixture HWND identity changed for role $($registered.role)."
    }
    $nativeWindows += [ordered]@{
        role = [string] $registered.role
        title = [string] $registered.title
        intended = [ordered]@{
            classification = [string] $registered.intended.classification
            ownerRole = [string] $registered.intended.ownerRole
            modal = [bool] $registered.intended.modal
            resizable = [bool] $registered.intended.resizable
            showInTaskbar = [bool] $registered.intended.showInTaskbar
            formBorderStyle = [string] $registered.intended.formBorderStyle
        }
        observedBeforeGlaze = $observed
        observedAfterGlaze = $null
        observed = $null
    }
}

$glazeJson = Invoke-P0GlazeQueryProcess `
    -GlazeExecutablePath $glaze `
    -TimeoutMilliseconds 10000
try {
    $glazeResponse = $glazeJson | ConvertFrom-Json
}
catch {
    throw "glazewm query workspaces did not return valid JSON: $($_.Exception.Message)"
}
$glazeWindows = @(Select-P0FixtureGlazeState `
    -GlazeResponse $glazeResponse `
    -Registrations $registrations)

# Bracket the Glaze query with exact native lifetime checks. A destroyed and
# reused PID/HWND/title tuple has a different process creation time or native
# property generation and fails before evidence is written.
foreach ($nativeWindow in $nativeWindows) {
    $registered = @($registrations | Where-Object { $_.role -ceq $nativeWindow.role })
    if ($registered.Count -ne 1) {
        throw "Fixture registration changed during capture for role $($nativeWindow.role)."
    }
    $observedAfter = [Winmakase.P0.DialogFactReader]::Snapshot(
        [long] $registered[0].handle,
        [int] $registration.processId,
        [long] $registration.processCreationTimeUtcTicks,
        [long] $registered[0].windowGeneration)
    $observedBefore = $nativeWindow.observedBeforeGlaze
    if ([long] $observedBefore.handle -ne [long] $observedAfter.handle -or
        [int] $observedBefore.processId -ne [int] $observedAfter.processId -or
        [long] $observedBefore.processCreationTimeUtcTicks -ne [long] $observedAfter.processCreationTimeUtcTicks -or
        [long] $observedBefore.windowGeneration -ne [long] $observedAfter.windowGeneration) {
        throw "Fixture window lifetime changed during capture for role $($nativeWindow.role)."
    }
    $nativeWindow.observedAfterGlaze = $observedAfter
    $nativeWindow.observed = $observedAfter
}

$proof = [ordered]@{
    schemaVersion = 2
    fixtureId = $ActionId
    phase = $Phase
    processId = [int] $registration.processId
    processCreationTimeUtcTicks = [long] $registration.processCreationTimeUtcTicks
    arguments = $expectedArguments
    nativeWindows = $nativeWindows
    glazeQuery = 'query workspaces'
    glazeWindows = $glazeWindows
}

$proofDirectory = Join-Path $root 'data/dialog-proof'
if (-not (Test-Path -LiteralPath $proofDirectory)) {
    [void] (New-Item -ItemType Directory -Path $proofDirectory)
}
$proofDirectoryItem = Get-Item -LiteralPath $proofDirectory -Force
if (($proofDirectoryItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
    throw "Dialog proof directory cannot be a reparse point: $proofDirectory"
}
$proofPath = Join-Path $proofDirectory "$ActionId-$Phase.json"
$json = $proof | ConvertTo-Json -Depth 12
[System.IO.File]::WriteAllText($proofPath, $json, (New-Object System.Text.UTF8Encoding($false)))
Write-Output $proofPath
