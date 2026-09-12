Set-StrictMode -Version Latest

function Get-ObjectProperty {
    param($InputObject, [Parameter(Mandatory)][string] $Name)

    if ($null -eq $InputObject) {
        return $null
    }
    $property = $InputObject.PSObject.Properties[$Name]
    if ($null -eq $property) {
        return $null
    }
    return $property.Value
}

function Get-StateType {
    param($State)

    if ($null -eq $State) {
        return $null
    }
    if ($State -is [string]) {
        return [string] $State
    }
    return [string] (Get-ObjectProperty $State 'type')
}

function Add-FixtureGlazeNodes {
    param(
        [Parameter(Mandatory)] $Node,
        [AllowNull()][string] $WorkspaceName,
        [Parameter(Mandatory)] $RegistrationsByHandle,
        [Parameter(Mandatory)] $Matches
    )

    if ($null -eq $Node) {
        return
    }

    $nodeType = [string] (Get-ObjectProperty $Node 'type')
    $currentWorkspace = $WorkspaceName
    if ($nodeType -ceq 'workspace') {
        $currentWorkspace = [string] (Get-ObjectProperty $Node 'name')
    }

    if ($nodeType -ceq 'window') {
        $rawHandle = Get-ObjectProperty $Node 'handle'
        if ($null -ne $rawHandle) {
            $handle = [Convert]::ToInt64($rawHandle, [Globalization.CultureInfo]::InvariantCulture)
            $handleKey = $handle.ToString([Globalization.CultureInfo]::InvariantCulture)
            if ($RegistrationsByHandle.ContainsKey($handleKey)) {
                if ($Matches.ContainsKey($handleKey)) {
                    throw "Glaze state contains fixture HWND $handle more than once."
                }
                $registration = $RegistrationsByHandle[$handleKey]
                $title = [string] (Get-ObjectProperty $Node 'title')
                $processName = [string] (Get-ObjectProperty $Node 'processName')
                if ($title -cne [string] $registration.title -or
                    $processName -cne 'WinmakaseP0Fixture') {
                    throw "Glaze HWND $handle does not match its registered fixture identity."
                }

                $placement = Get-ObjectProperty $Node 'floatingPlacement'
                $Matches[$handleKey] = [ordered]@{
                    role = [string] $registration.role
                    handle = $handle
                    managed = $true
                    id = [string] (Get-ObjectProperty $Node 'id')
                    parentId = [string] (Get-ObjectProperty $Node 'parentId')
                    workspaceName = $currentWorkspace
                    title = $title
                    processName = $processName
                    className = [string] (Get-ObjectProperty $Node 'className')
                    appUserModelId = [string] (Get-ObjectProperty $Node 'appUserModelId')
                    stateType = Get-StateType (Get-ObjectProperty $Node 'state')
                    previousStateType = Get-StateType (Get-ObjectProperty $Node 'prevState')
                    displayState = [string] (Get-ObjectProperty $Node 'displayState')
                    hasFocus = [bool] (Get-ObjectProperty $Node 'hasFocus')
                    x = [int] (Get-ObjectProperty $Node 'x')
                    y = [int] (Get-ObjectProperty $Node 'y')
                    width = [int] (Get-ObjectProperty $Node 'width')
                    height = [int] (Get-ObjectProperty $Node 'height')
                    floatingPlacement = if ($null -eq $placement) { $null } else { [ordered]@{
                        x = [int] (Get-ObjectProperty $placement 'x')
                        y = [int] (Get-ObjectProperty $placement 'y')
                        width = [int] (Get-ObjectProperty $placement 'width')
                        height = [int] (Get-ObjectProperty $placement 'height')
                    }}
                }
            }
        }
    }

    $children = Get-ObjectProperty $Node 'children'
    if ($null -ne $children) {
        foreach ($child in @($children)) {
            Add-FixtureGlazeNodes `
                -Node $child `
                -WorkspaceName $currentWorkspace `
                -RegistrationsByHandle $RegistrationsByHandle `
                -Matches $Matches
        }
    }
}

function Select-P0FixtureGlazeState {
    param(
        [Parameter(Mandatory)] $GlazeResponse,
        [Parameter(Mandatory)] $Registrations
    )

    $registrationsByHandle = @{}
    foreach ($registration in @($Registrations)) {
        $handle = [Convert]::ToInt64($registration.handle, [Globalization.CultureInfo]::InvariantCulture)
        $key = $handle.ToString([Globalization.CultureInfo]::InvariantCulture)
        if ($registrationsByHandle.ContainsKey($key)) {
            throw "Fixture registration contains HWND $handle more than once."
        }
        $registrationsByHandle[$key] = $registration
    }

    $data = Get-ObjectProperty $GlazeResponse 'data'
    if ($null -eq $data) {
        $data = $GlazeResponse
    }
    $nodes = Get-ObjectProperty $data 'workspaces'
    if ($null -eq $nodes) {
        if ($data -is [System.Array]) {
            $nodes = $data
        }
        else {
            throw 'Glaze response does not contain data.workspaces or a workspace array.'
        }
    }

    $matches = @{}
    foreach ($node in @($nodes)) {
        Add-FixtureGlazeNodes `
            -Node $node `
            -WorkspaceName $null `
            -RegistrationsByHandle $registrationsByHandle `
            -Matches $matches
    }

    $result = @()
    foreach ($registration in @($Registrations)) {
        $handle = [Convert]::ToInt64($registration.handle, [Globalization.CultureInfo]::InvariantCulture)
        $key = $handle.ToString([Globalization.CultureInfo]::InvariantCulture)
        if ($matches.ContainsKey($key)) {
            $result += $matches[$key]
        }
        else {
            $result += [ordered]@{
                role = [string] $registration.role
                handle = $handle
                managed = $false
            }
        }
    }
    return $result
}

function Find-ProofWindow {
    param($Windows, [Parameter(Mandatory)][string] $Role)

    $matches = @($Windows | Where-Object { [string] $_.role -ceq $Role })
    if ($matches.Count -ne 1) {
        throw "Proof must contain role $Role exactly once."
    }
    return $matches[0]
}

function Compare-P0DialogProofObjects {
    param(
        [Parameter(Mandatory)] $Before,
        [Parameter(Mandatory)] $After
    )

    if ([int] $Before.schemaVersion -ne 1 -or [int] $After.schemaVersion -ne 1) {
        throw 'Dialog proof schemaVersion must be 1.'
    }
    if ([string] $Before.fixtureId -cne [string] $After.fixtureId) {
        throw 'Dialog proofs belong to different fixture actions.'
    }
    if (@($Before.nativeWindows).Count -ne @($After.nativeWindows).Count -or
        @($Before.glazeWindows).Count -ne @($After.glazeWindows).Count) {
        throw 'Dialog proofs contain different window-role sets.'
    }

    $rows = @()
    $failures = @()
    $incomplete = @()
    foreach ($beforeNative in @($Before.nativeWindows)) {
        $role = [string] $beforeNative.role
        $afterNative = Find-ProofWindow $After.nativeWindows $role
        $beforeGlaze = Find-ProofWindow $Before.glazeWindows $role
        $afterGlaze = Find-ProofWindow $After.glazeWindows $role
        $ownerRole = [string] $beforeNative.intended.ownerRole

        $beforeOwnerMatches = if ([string]::IsNullOrEmpty($ownerRole)) {
            [long] $beforeNative.observed.ownerHandle -eq 0
        }
        else {
            $owner = Find-ProofWindow $Before.nativeWindows $ownerRole
            [long] $beforeNative.observed.ownerHandle -eq [long] $owner.observed.handle
        }
        $afterOwnerMatches = if ([string]::IsNullOrEmpty($ownerRole)) {
            [long] $afterNative.observed.ownerHandle -eq 0
        }
        else {
            $owner = Find-ProofWindow $After.nativeWindows $ownerRole
            [long] $afterNative.observed.ownerHandle -eq [long] $owner.observed.handle
        }
        $nativeStyleMatches =
            [bool] $beforeNative.intended.resizable -eq [bool] $beforeNative.observed.isResizable -and
            [bool] $afterNative.intended.resizable -eq [bool] $afterNative.observed.isResizable
        $nativeHandlePreserved = [long] $beforeNative.observed.handle -eq [long] $afterNative.observed.handle
        $nativeStylePreserved =
            [string] $beforeNative.observed.className -ceq [string] $afterNative.observed.className -and
            [string] $beforeNative.observed.styleHex -ceq [string] $afterNative.observed.styleHex -and
            [string] $beforeNative.observed.extendedStyleHex -ceq [string] $afterNative.observed.extendedStyleHex

        $managedBoth = [bool] $beforeGlaze.managed -and [bool] $afterGlaze.managed
        $statePreserved = $null
        $workspacePreserved = $null
        $geometryPreserved = $null
        $glazeParentPreserved = $null
        $displayStatePreserved = $null
        $focusPreserved = $null
        $sameOwnerWorkspace = $null
        if ($managedBoth) {
            $statePreserved = [string] $beforeGlaze.stateType -ceq [string] $afterGlaze.stateType
            $workspacePreserved = [string] $beforeGlaze.workspaceName -ceq [string] $afterGlaze.workspaceName
            $geometryPreserved =
                [int] $beforeGlaze.x -eq [int] $afterGlaze.x -and
                [int] $beforeGlaze.y -eq [int] $afterGlaze.y -and
                [int] $beforeGlaze.width -eq [int] $afterGlaze.width -and
                [int] $beforeGlaze.height -eq [int] $afterGlaze.height
            $glazeParentPreserved = [string] $beforeGlaze.parentId -ceq [string] $afterGlaze.parentId
            $displayStatePreserved = [string] $beforeGlaze.displayState -ceq [string] $afterGlaze.displayState
            $focusPreserved = [bool] $beforeGlaze.hasFocus -eq [bool] $afterGlaze.hasFocus
            if (-not [string]::IsNullOrEmpty($ownerRole)) {
                $beforeOwnerGlaze = Find-ProofWindow $Before.glazeWindows $ownerRole
                $afterOwnerGlaze = Find-ProofWindow $After.glazeWindows $ownerRole
                if ([bool] $beforeOwnerGlaze.managed -and [bool] $afterOwnerGlaze.managed) {
                    $sameOwnerWorkspace =
                        [string] $beforeGlaze.workspaceName -ceq [string] $beforeOwnerGlaze.workspaceName -and
                        [string] $afterGlaze.workspaceName -ceq [string] $afterOwnerGlaze.workspaceName
                }
            }
        }
        else {
            $incomplete += "$role was not represented in both Glaze captures"
        }

        foreach ($check in @(
            @{ name = "$role native owner"; value = $beforeOwnerMatches -and $afterOwnerMatches },
            @{ name = "$role native resizable style"; value = $nativeStyleMatches },
            @{ name = "$role native HWND"; value = $nativeHandlePreserved },
            @{ name = "$role native styles"; value = $nativeStylePreserved },
            @{ name = "$role reload state"; value = $statePreserved },
            @{ name = "$role reload workspace"; value = $workspacePreserved },
            @{ name = "$role reload geometry"; value = $geometryPreserved },
            @{ name = "$role reload parent"; value = $glazeParentPreserved },
            @{ name = "$role reload display state"; value = $displayStatePreserved },
            @{ name = "$role reload focus"; value = $focusPreserved },
            @{ name = "$role owner workspace"; value = $sameOwnerWorkspace }
        )) {
            if ($null -ne $check.value -and -not [bool] $check.value) {
                $failures += [string] $check.name
            }
        }

        $rows += [ordered]@{
            role = $role
            nativeOwnerMatchesIntended = $beforeOwnerMatches -and $afterOwnerMatches
            nativeResizableMatchesIntended = $nativeStyleMatches
            nativeHandlePreserved = $nativeHandlePreserved
            nativeStylesPreserved = $nativeStylePreserved
            glazeManagedBefore = [bool] $beforeGlaze.managed
            glazeManagedAfter = [bool] $afterGlaze.managed
            beforeStateType = if ([bool] $beforeGlaze.managed) { [string] $beforeGlaze.stateType } else { $null }
            afterStateType = if ([bool] $afterGlaze.managed) { [string] $afterGlaze.stateType } else { $null }
            reloadStatePreserved = $statePreserved
            reloadWorkspacePreserved = $workspacePreserved
            reloadGeometryPreserved = $geometryPreserved
            reloadParentIdPreserved = $glazeParentPreserved
            reloadDisplayStatePreserved = $displayStatePreserved
            reloadFocusPreserved = $focusPreserved
            sameWorkspaceAsNativeOwner = $sameOwnerWorkspace
        }
    }

    $verdict = if ($failures.Count -gt 0) { 'fail' } elseif ($incomplete.Count -gt 0) { 'incomplete' } else { 'pass' }
    return [ordered]@{
        schemaVersion = 1
        fixtureId = [string] $Before.fixtureId
        beforePhase = [string] $Before.phase
        afterPhase = [string] $After.phase
        verdict = $verdict
        windows = $rows
        failures = @($failures)
        incomplete = @($incomplete)
    }
}

Export-ModuleMember -Function @(
    'Compare-P0DialogProofObjects',
    'Select-P0FixtureGlazeState'
)
