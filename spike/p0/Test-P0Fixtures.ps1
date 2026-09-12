[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Import-Module (Join-Path $PSScriptRoot 'Fixture.Common.psm1') -Force
Import-Module (Join-Path $PSScriptRoot 'DialogProof.Common.psm1') -Force

$script:Assertions = 0
function Assert-True {
    param([Parameter(Mandatory)][bool] $Condition, [Parameter(Mandatory)][string] $Message)
    $script:Assertions++
    if (-not $Condition) {
        throw "Assertion failed: $Message"
    }
}

function Assert-Equal {
    param($Expected, $Actual, [Parameter(Mandatory)][string] $Message)
    $script:Assertions++
    if ($Expected -cne $Actual) {
        throw "Assertion failed: $Message`nExpected: <$Expected>`nActual:   <$Actual>"
    }
}

$canonicalTempRoot = Get-CanonicalPath ([System.IO.Path]::GetTempPath())
$testRootName = "winmakase-p0-test-" + [Guid]::NewGuid().ToString('N')
$testRoot = Join-Path $canonicalTempRoot $testRootName
try {
    [void] (New-Item -ItemType Directory -Path $testRoot)
    Set-Content -LiteralPath (Join-Path $testRoot '.winmakase-p0-fixture-root') `
        -Value 'winmakase-p0-disposable-fixture-v1' -NoNewline -Encoding ASCII
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'fixture-actions.json') -Destination $testRoot
    [void] (New-Item -ItemType Directory -Path (Join-Path $testRoot 'bin'))

    $unicodeLabel = 'Caf' + [char]0x00E9 + ' ' + [char]0x6771 + [char]0x4EAC
    Assert-Equal 'plain' (ConvertTo-WindowsCommandLineArgument 'plain') 'plain argument serialization'
    Assert-Equal '"Work Profile"' (ConvertTo-WindowsCommandLineArgument 'Work Profile') 'space argument serialization'
    Assert-Equal ('"' + $unicodeLabel + '"') (ConvertTo-WindowsCommandLineArgument $unicodeLabel) 'Unicode argument serialization'
    Assert-Equal '"a\\\"b"' (ConvertTo-WindowsCommandLineArgument 'a\"b') 'quote and slash serialization'
    Assert-Equal '"tail slash\\"' (ConvertTo-WindowsCommandLineArgument 'tail slash\') 'quoted trailing slash serialization'
    Assert-Equal '""' (ConvertTo-WindowsCommandLineArgument '') 'empty argument serialization'

    $manifest = Read-FixtureManifest $testRoot
    Assert-Equal 4 @($manifest.actions).Count 'manifest action count'
    Assert-Equal 'Work Profile' $manifest.actions[0].expectedCapture.profileLabel 'space fixture expectation'
    Assert-Equal $unicodeLabel $manifest.actions[2].expectedCapture.profileLabel 'Unicode fixture expectation'
    Assert-Equal 'utility' $manifest.actions[3].expectedCapture.window 'utility fixture expectation'

    [xml] $sandboxTemplate = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'WinmakaseP0.wsb.template') -Raw
    Assert-Equal 'Disable' $sandboxTemplate.Configuration.Networking 'sandbox networking is disabled'
    Assert-Equal 'Disable' $sandboxTemplate.Configuration.ClipboardRedirection 'sandbox clipboard is disabled'
    Assert-Equal 'true' $sandboxTemplate.Configuration.MappedFolders.MappedFolder.ReadOnly 'source mapping is read-only'
    Assert-Equal 1 @($sandboxTemplate.Configuration.MappedFolders.MappedFolder).Count 'only fixture source is mapped'

    $compilerCandidates = @(
        "$env:WINDIR\Microsoft.NET\Framework64\v4.0.30319\csc.exe",
        "$env:WINDIR\Microsoft.NET\Framework\v4.0.30319\csc.exe"
    )
    $compiler = $compilerCandidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
    Assert-True ($null -ne $compiler) 'inbox .NET Framework compiler exists'
    $executable = Join-Path $testRoot 'bin/WinmakaseP0Fixture.exe'
    & $compiler @(
        '/nologo',
        '/target:winexe',
        '/reference:System.dll',
        '/reference:System.Drawing.dll',
        '/reference:System.Windows.Forms.dll',
        '/reference:System.Web.Extensions.dll',
        "/out:$executable",
        (Join-Path $PSScriptRoot 'FixtureApp.cs')
    )
    Assert-Equal 0 $LASTEXITCODE 'fixture app compiles with inbox compiler'
    Assert-True (Test-Path -LiteralPath $executable -PathType Leaf) 'fixture executable was produced'

    $dialogInterop = Join-Path $testRoot 'bin/DialogFactInterop.dll'
    & $compiler @(
        '/nologo',
        '/target:library',
        '/reference:System.dll',
        "/out:$dialogInterop",
        (Join-Path $PSScriptRoot 'DialogFactInterop.cs')
    )
    Assert-Equal 0 $LASTEXITCODE 'dialog fact interop compiles with inbox compiler'
    Assert-True (Test-Path -LiteralPath $dialogInterop -PathType Leaf) 'dialog fact interop assembly was produced'
    Add-Type -TypeDefinition (Get-Content -LiteralPath (Join-Path $PSScriptRoot 'DialogFactInterop.cs') -Raw)
    $invalidLifetimeRejected = $false
    try {
        [void] [Winmakase.P0.DialogFactReader]::Snapshot(0, 1, 1, 1)
    }
    catch {
        $invalidLifetimeRejected = $_.Exception.Message -match 'no longer valid'
    }
    Assert-True $invalidLifetimeRejected 'native fact reader rejects an invalid HWND before reading facts'

    $expectationPath = & (Join-Path $PSScriptRoot 'New-LauncherFixtures.ps1') `
        -FixtureRoot $testRoot `
        -ExecutablePath $executable
    Assert-Equal (Join-Path $testRoot 'expectations.json') $expectationPath 'expectation export path'
    $expectations = Get-Content -LiteralPath $expectationPath -Raw -Encoding UTF8 | ConvertFrom-Json
    Assert-Equal 4 @($expectations.actions).Count 'exported expectation count'

    foreach ($action in @($expectations.actions)) {
        $shortcutPath = Join-Path (Join-Path $testRoot 'shortcuts') $action.shortcutFile
        Assert-True (Test-Path -LiteralPath $shortcutPath -PathType Leaf) "shortcut exists: $($action.id)"
        $shortcut = Read-UnicodeShortcut $shortcutPath
        Assert-True ((Get-CanonicalPath $executable).Equals(
            (Get-CanonicalPath $shortcut.TargetPath),
            [System.StringComparison]::OrdinalIgnoreCase)) "shortcut target: $($action.id)"
        Assert-Equal $action.argumentsString $shortcut.Arguments "shortcut arguments: $($action.id)"
        Assert-True ((Get-CanonicalPath (Split-Path -Parent $executable)).Equals(
            (Get-CanonicalPath $shortcut.WorkingDirectory),
            [System.StringComparison]::OrdinalIgnoreCase)) "shortcut working directory: $($action.id)"
    }

    $rejectedOutsideExecutable = $false
    try {
        & (Join-Path $PSScriptRoot 'New-LauncherFixtures.ps1') `
            -FixtureRoot $testRoot `
            -ExecutablePath $compiler
    }
    catch {
        $rejectedOutsideExecutable = $true
    }
    Assert-True $rejectedOutsideExecutable 'generator rejects executables outside the fixture root'

    $process = Start-Process -FilePath $executable `
        -ArgumentList '--guest-consent DISPOSABLE-WINDOWS-GUEST --fixture-id main-spaces --window main --profile-label "Work Profile"' `
        -WindowStyle Hidden `
        -PassThru
    if (-not $process.WaitForExit(5000)) {
        Stop-Process -InputObject $process -Force
        [void] $process.WaitForExit(5000)
        throw 'Compiled fixture app did not complete its non-Sandbox guard within five seconds.'
    }
    Assert-Equal 41 $process.ExitCode 'compiled app rejects a non-Sandbox host before UI or writes'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $testRoot 'data'))) 'host guard produced no capture data'

    $captureRejectedHost = $false
    try {
        & (Join-Path $PSScriptRoot 'Capture-P0DialogProof.ps1') `
            -FixtureRoot $testRoot `
            -ActionId owned-dialog `
            -Phase initial `
            -GlazeExecutablePath $executable `
            -ConsentToken DISPOSABLE-WINDOWS-GUEST
    }
    catch {
        $captureRejectedHost = $_.Exception.Message -match 'Windows Sandbox guest'
    }
    Assert-True $captureRejectedHost 'dialog capture rejects host before HWND or Glaze access'
    Assert-True (-not (Test-Path -LiteralPath (Join-Path $testRoot 'data/dialog-proof'))) 'host capture rejection writes no proof data'

    $compareRejectedHost = $false
    try {
        & (Join-Path $PSScriptRoot 'Compare-P0DialogProof.ps1') `
            -FixtureRoot $testRoot `
            -ActionId owned-dialog `
            -BeforePhase initial `
            -AfterPhase after-reload `
            -ConsentToken DISPOSABLE-WINDOWS-GUEST
    }
    catch {
        $compareRejectedHost = $_.Exception.Message -match 'Windows Sandbox guest'
    }
    Assert-True $compareRejectedHost 'dialog comparison rejects host before file access'

    $registrations = @(
        [pscustomobject]@{ role = 'main'; title = 'Winmakase P0 Fixture - Main'; handle = 101 },
        [pscustomobject]@{ role = 'owned'; title = 'Winmakase P0 Fixture - Owned Dialog'; handle = 202 }
    )
    $glazeResponse = @'
{
  "messageType": "client_response",
  "data": {
    "workspaces": [
      {
        "type": "workspace",
        "id": "workspace-2",
        "name": "2",
        "children": [
          {
            "type": "window",
            "id": "main-id",
            "parentId": "workspace-2",
            "handle": 101,
            "title": "Winmakase P0 Fixture - Main",
            "processName": "WinmakaseP0Fixture",
            "className": "FixtureClass",
            "appUserModelId": "",
            "state": { "type": "tiling" },
            "prevState": null,
            "displayState": "shown",
            "hasFocus": true,
            "x": 10, "y": 20, "width": 700, "height": 400,
            "floatingPlacement": { "x": 40, "y": 50, "width": 700, "height": 400 }
          },
          {
            "type": "window",
            "id": "owned-id",
            "parentId": "workspace-2",
            "handle": 202,
            "title": "Winmakase P0 Fixture - Owned Dialog",
            "processName": "WinmakaseP0Fixture",
            "className": "FixtureClass",
            "appUserModelId": "",
            "state": { "type": "floating" },
            "prevState": null,
            "displayState": "shown",
            "hasFocus": false,
            "x": 120, "y": 100, "width": 380, "height": 150,
            "floatingPlacement": { "x": 120, "y": 100, "width": 380, "height": 150 }
          },
          {
            "type": "window",
            "id": "unrelated-id",
            "parentId": "workspace-2",
            "handle": 303,
            "title": "Unrelated",
            "processName": "other",
            "state": { "type": "tiling" },
            "displayState": "shown",
            "hasFocus": false,
            "x": 0, "y": 0, "width": 1, "height": 1
          }
        ]
      }
    ]
  },
  "success": true
}
'@ | ConvertFrom-Json
    $fixtureGlaze = @(Select-P0FixtureGlazeState -GlazeResponse $glazeResponse -Registrations $registrations)
    Assert-Equal 2 $fixtureGlaze.Count 'Glaze readout contains only registered fixture HWNDs'
    Assert-Equal '2' $fixtureGlaze[0].workspaceName 'main workspace is derived from recursive Glaze state'
    Assert-Equal 'floating' $fixtureGlaze[1].stateType 'dialog state is retained in fixture-only Glaze data'
    Assert-True $fixtureGlaze[1].managed 'registered dialog is marked managed'

    $mainObserved = [pscustomobject]@{
        handle = 101; ownerHandle = 0; isResizable = $true
        processId = 9001; processCreationTimeUtcTicks = 638000000000000000; windowGeneration = 1
        className = 'FixtureClass'; styleHex = '0x1'; extendedStyleHex = '0x0'
    }
    $ownedObserved = [pscustomobject]@{
        handle = 202; ownerHandle = 101; isResizable = $false
        processId = 9001; processCreationTimeUtcTicks = 638000000000000000; windowGeneration = 2
        className = 'FixtureClass'; styleHex = '0x2'; extendedStyleHex = '0x1'
    }
    $beforeProof = [pscustomobject]@{
        schemaVersion = 2
        fixtureId = 'owned-dialog'
        phase = 'initial'
        processId = 9001
        processCreationTimeUtcTicks = 638000000000000000
        nativeWindows = @(
            [pscustomobject]@{
                role = 'main'
                intended = [pscustomobject]@{ ownerRole = ''; resizable = $true }
                observedBeforeGlaze = $mainObserved
                observedAfterGlaze = $mainObserved
                observed = $mainObserved
            },
            [pscustomobject]@{
                role = 'owned'
                intended = [pscustomobject]@{ ownerRole = 'main'; resizable = $false }
                observedBeforeGlaze = $ownedObserved
                observedAfterGlaze = $ownedObserved
                observed = $ownedObserved
            }
        )
        glazeWindows = $fixtureGlaze
    }
    $afterProof = $beforeProof | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    $afterProof.phase = 'after-reload'
    $comparison = Compare-P0DialogProofObjects -Before $beforeProof -After $afterProof
    Assert-Equal 'pass' $comparison.verdict 'unchanged fixture state passes reload comparison'
    Assert-True $comparison.windows[1].sameWorkspaceAsNativeOwner 'owned dialog remains with main workspace'

    $changedProof = $afterProof | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    $changedProof.glazeWindows[1].stateType = 'tiling'
    $failedComparison = Compare-P0DialogProofObjects -Before $beforeProof -After $changedProof
    Assert-Equal 'fail' $failedComparison.verdict 'dialog state change fails reload comparison'
    Assert-True (@($failedComparison.failures) -contains 'owned reload state') 'reload failure names changed dialog state'

    $changedPidProof = $afterProof | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    $changedPidProof.processId = 9002
    foreach ($window in @($changedPidProof.nativeWindows)) {
        $window.observedBeforeGlaze.processId = 9002
        $window.observedAfterGlaze.processId = 9002
        $window.observed.processId = 9002
    }
    $changedPidComparison = Compare-P0DialogProofObjects -Before $beforeProof -After $changedPidProof
    Assert-Equal 'fail' $changedPidComparison.verdict 'replacement PID fails preservation comparison'
    Assert-True (@($changedPidComparison.failures) -contains 'proof process identity') 'replacement PID names proof process identity failure'
    Assert-True (@($changedPidComparison.failures) -contains 'main native lifetime') 'replacement PID names native window lifetime failure'

    $reusedIdentityProof = $afterProof | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    $reusedIdentityProof.processCreationTimeUtcTicks = 638000000000000100
    foreach ($window in @($reusedIdentityProof.nativeWindows)) {
        $window.observedBeforeGlaze.processCreationTimeUtcTicks = 638000000000000100
        $window.observedAfterGlaze.processCreationTimeUtcTicks = 638000000000000100
        $window.observed.processCreationTimeUtcTicks = 638000000000000100
        $window.observedBeforeGlaze.windowGeneration = [long] $window.observedBeforeGlaze.windowGeneration + 10
        $window.observedAfterGlaze.windowGeneration = [long] $window.observedAfterGlaze.windowGeneration + 10
        $window.observed.windowGeneration = [long] $window.observed.windowGeneration + 10
    }
    $reusedIdentityComparison = Compare-P0DialogProofObjects -Before $beforeProof -After $reusedIdentityProof
    Assert-Equal 'fail' $reusedIdentityComparison.verdict 'reused PID/HWND/title with new lifetimes fails comparison'
    Assert-True (@($reusedIdentityComparison.failures) -contains 'proof process identity') 'PID reuse names process creation identity failure'
    Assert-True (@($reusedIdentityComparison.failures) -contains 'owned native lifetime') 'HWND reuse names window generation failure'

    $remanagedProof = $afterProof | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    $remanagedProof.glazeWindows[1].id = 'owned-remanaged-id'
    $remanagedComparison = Compare-P0DialogProofObjects -Before $beforeProof -After $remanagedProof
    Assert-Equal 'fail' $remanagedComparison.verdict 'new Glaze container ID fails preservation comparison'
    Assert-True (@($remanagedComparison.failures) -contains 'owned reload Glaze ID') 'remanagement names changed Glaze ID'

    $unmanagedProof = $afterProof | ConvertTo-Json -Depth 10 | ConvertFrom-Json
    $unmanagedProof.glazeWindows[1] = [pscustomobject]@{
        role = 'owned'; handle = 202; managed = $false
    }
    $unmanagedComparison = Compare-P0DialogProofObjects -Before $beforeProof -After $unmanagedProof
    Assert-Equal 'fail' $unmanagedComparison.verdict 'changed Glaze management fails preservation comparison'
    Assert-True (@($unmanagedComparison.failures) -contains 'owned reload Glaze management') 'management loss names Glaze preservation failure'

    Write-Output "PASS: $script:Assertions assertions; no fixture UI launched."
}
finally {
    if (Test-Path -LiteralPath $testRoot) {
        $canonicalTestRoot = Get-CanonicalPath $testRoot
        if (-not (Test-PathInside -Parent $canonicalTempRoot -Child $canonicalTestRoot) -or
            $canonicalTestRoot.Equals($canonicalTempRoot, [System.StringComparison]::OrdinalIgnoreCase) -or
            [System.IO.Path]::GetFileName($canonicalTestRoot) -cnotmatch '^winmakase-p0-test-[0-9a-f]{32}$') {
            throw "Refusing to clean an unexpected test path: $canonicalTestRoot"
        }
        [void] (Assert-SafeFixtureRoot $canonicalTestRoot)
        $reparsePoints = @(Get-ChildItem -LiteralPath $canonicalTestRoot -Recurse -Force |
            Where-Object { ($_.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0 })
        if ($reparsePoints.Count -gt 0) {
            throw "Refusing to clean a fixture containing reparse points: $canonicalTestRoot"
        }
        Remove-Item -LiteralPath $canonicalTestRoot -Recurse -Force
    }
}
