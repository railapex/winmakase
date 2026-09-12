[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $SourceRoot,

    [Parameter(Mandatory)]
    [string] $GuestRoot,

    [Parameter(Mandatory)]
    [string] $ConsentToken
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$modulePath = Join-Path $SourceRoot 'Fixture.Common.psm1'
if (-not (Test-Path -LiteralPath $modulePath -PathType Leaf)) {
    throw "Fixture source root is invalid: $SourceRoot"
}
Import-Module $modulePath -Force

if ($ConsentToken -cne 'DISPOSABLE-WINDOWS-GUEST' -or
    [Environment]::UserName -cne 'WDAGUtilityAccount') {
    throw 'Initialization is restricted to an explicitly authorized Windows Sandbox guest.'
}
$canonicalGuestRoot = Get-CanonicalPath $GuestRoot
if (-not $canonicalGuestRoot.Equals('C:\WinmakaseP0', [System.StringComparison]::OrdinalIgnoreCase)) {
    throw 'GuestRoot must be exactly C:\WinmakaseP0.'
}

$allowedSources = @(
    'Fixture.Common.psm1',
    'FixtureApp.cs',
    'DialogFactInterop.cs',
    'DialogProof.Common.psm1',
    'ShortcutInterop.cs',
    'fixture-actions.json',
    'Capture-P0DialogProof.ps1',
    'Compare-P0DialogProof.ps1',
    'New-LauncherFixtures.ps1',
    'Start-P0Fixture.ps1',
    'Publish-P0GuestShortcuts.ps1'
)
foreach ($name in $allowedSources) {
    if (-not (Test-Path -LiteralPath (Join-Path $SourceRoot $name) -PathType Leaf)) {
        throw "Required fixture source is missing: $name"
    }
}

if (Test-Path -LiteralPath $canonicalGuestRoot) {
    $markerPath = Join-Path $canonicalGuestRoot '.winmakase-p0-fixture-root'
    if (-not (Test-Path -LiteralPath $markerPath -PathType Leaf) -or
        (Get-Content -LiteralPath $markerPath -Raw).Trim() -cne 'winmakase-p0-disposable-fixture-v1') {
        throw 'Existing guest root is not an owned Winmakase P0 fixture directory.'
    }
}
else {
    [void] (New-Item -ItemType Directory -Path $canonicalGuestRoot)
    Set-Content -LiteralPath (Join-Path $canonicalGuestRoot '.winmakase-p0-fixture-root') `
        -Value 'winmakase-p0-disposable-fixture-v1' -NoNewline -Encoding ASCII
}
$canonicalGuestRoot = Assert-WindowsSandboxGuest `
    -FixtureRoot $canonicalGuestRoot `
    -ConsentToken $ConsentToken

$sourceDestination = Join-Path $canonicalGuestRoot 'source'
$binDirectory = Join-Path $canonicalGuestRoot 'bin'
foreach ($directory in @($sourceDestination, $binDirectory)) {
    if (-not (Test-Path -LiteralPath $directory)) {
        [void] (New-Item -ItemType Directory -Path $directory)
    }
    $directoryItem = Get-Item -LiteralPath $directory -Force
    if (($directoryItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Guest fixture child directory cannot be a reparse point: $directory"
    }
}
foreach ($name in $allowedSources) {
    Copy-Item -LiteralPath (Join-Path $SourceRoot $name) -Destination (Join-Path $sourceDestination $name) -Force
}
Copy-Item -LiteralPath (Join-Path $SourceRoot 'Fixture.Common.psm1') -Destination $canonicalGuestRoot -Force
Copy-Item -LiteralPath (Join-Path $SourceRoot 'ShortcutInterop.cs') -Destination $canonicalGuestRoot -Force
Copy-Item -LiteralPath (Join-Path $SourceRoot 'fixture-actions.json') -Destination $canonicalGuestRoot -Force

$compilerCandidates = @(
    "$env:WINDIR\Microsoft.NET\Framework64\v4.0.30319\csc.exe",
    "$env:WINDIR\Microsoft.NET\Framework\v4.0.30319\csc.exe"
)
$compiler = $compilerCandidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
if (-not $compiler) {
    throw 'The inbox .NET Framework C# compiler was not found.'
}

$outputPath = Join-Path $binDirectory 'WinmakaseP0Fixture.exe'
$compileArguments = @(
    '/nologo',
    '/target:winexe',
    '/optimize+',
    '/reference:System.dll',
    '/reference:System.Drawing.dll',
    '/reference:System.Windows.Forms.dll',
    '/reference:System.Web.Extensions.dll',
    "/out:$outputPath",
    (Join-Path $sourceDestination 'FixtureApp.cs')
)
& $compiler $compileArguments
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $outputPath -PathType Leaf)) {
    throw "Fixture compilation failed with exit code $LASTEXITCODE"
}

& (Join-Path $sourceDestination 'New-LauncherFixtures.ps1') `
    -FixtureRoot $canonicalGuestRoot `
    -ExecutablePath $outputPath

Write-Output "Prepared disposable fixture at $canonicalGuestRoot"
Write-Output 'No fixture UI was launched and no Start Menu shortcut was published.'
