Set-StrictMode -Version Latest

$script:FixtureMarkerName = '.winmakase-p0-fixture-root'
$script:FixtureMarkerValue = 'winmakase-p0-disposable-fixture-v1'
$script:GuestConsent = 'DISPOSABLE-WINDOWS-GUEST'

function Get-CanonicalPath {
    param([Parameter(Mandatory)][string] $Path)

    $fullPath = [System.IO.Path]::GetFullPath($Path)
    $volumeRoot = [System.IO.Path]::GetPathRoot($fullPath)
    $trimmedPath = $fullPath.TrimEnd([char[]]@('\', '/'))
    if ($trimmedPath.Equals(
        $volumeRoot.TrimEnd([char[]]@('\', '/')),
        [System.StringComparison]::OrdinalIgnoreCase)) {
        return $volumeRoot
    }
    return $trimmedPath
}

function Test-PathInside {
    param(
        [Parameter(Mandatory)][string] $Parent,
        [Parameter(Mandatory)][string] $Child
    )

    $parentPath = Get-CanonicalPath $Parent
    $childPath = Get-CanonicalPath $Child
    if ($childPath.Equals($parentPath, [System.StringComparison]::OrdinalIgnoreCase)) {
        return $true
    }

    $parentPrefix = $parentPath
    if (-not $parentPrefix.EndsWith([System.IO.Path]::DirectorySeparatorChar.ToString())) {
        $parentPrefix += [System.IO.Path]::DirectorySeparatorChar
    }
    return $childPath.StartsWith(
        $parentPrefix,
        [System.StringComparison]::OrdinalIgnoreCase)
}

function Assert-SafeFixtureRoot {
    param([Parameter(Mandatory)][string] $FixtureRoot)

    $root = Get-CanonicalPath $FixtureRoot
    if (-not (Test-Path -LiteralPath $root -PathType Container)) {
        throw "Fixture root does not exist: $root"
    }
    if ($root.Equals(
        (Get-CanonicalPath ([System.IO.Path]::GetPathRoot($root))),
        [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Fixture root cannot be a volume root: $root"
    }

    $rootItem = Get-Item -LiteralPath $root -Force
    if (($rootItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
        throw "Fixture root cannot be a reparse point: $root"
    }

    $blocked = @(
        [Environment]::GetFolderPath('Desktop'),
        [Environment]::GetFolderPath('StartMenu'),
        [Environment]::GetFolderPath('Programs'),
        $env:WINDIR,
        $env:ProgramFiles,
        ${env:ProgramFiles(x86)}
    ) | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }

    foreach ($blockedPath in $blocked) {
        $canonicalBlocked = Get-CanonicalPath $blockedPath
        if ($root.Equals($canonicalBlocked, [System.StringComparison]::OrdinalIgnoreCase) -or
            (Test-PathInside -Parent $canonicalBlocked -Child $root) -or
            (Test-PathInside -Parent $root -Child $canonicalBlocked)) {
            throw "Fixture root cannot be inside a protected desktop/system location: $root"
        }
    }

    $userProfile = [Environment]::GetFolderPath('UserProfile')
    if (-not [string]::IsNullOrWhiteSpace($userProfile) -and
        $root.Equals((Get-CanonicalPath $userProfile), [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Fixture root cannot be the user profile root: $root"
    }

    $markerPath = Join-Path $root $script:FixtureMarkerName
    if (-not (Test-Path -LiteralPath $markerPath -PathType Leaf)) {
        throw "Fixture marker is missing: $markerPath"
    }

    $marker = (Get-Content -LiteralPath $markerPath -Raw).Trim()
    if ($marker -cne $script:FixtureMarkerValue) {
        throw "Fixture marker is invalid: $markerPath"
    }

    return $root
}

function Assert-WindowsSandboxGuest {
    param(
        [Parameter(Mandatory)][string] $FixtureRoot,
        [Parameter(Mandatory)][string] $ConsentToken
    )

    if ($ConsentToken -cne $script:GuestConsent) {
        throw 'The disposable guest consent token is missing or invalid.'
    }
    if ([Environment]::UserName -cne 'WDAGUtilityAccount') {
        throw 'This operation is restricted to a Windows Sandbox guest (WDAGUtilityAccount).'
    }

    $expectedRoot = Get-CanonicalPath 'C:\WinmakaseP0'
    $actualRoot = Get-CanonicalPath $FixtureRoot
    if (-not $actualRoot.Equals($expectedRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Guest fixture root must be exactly $expectedRoot"
    }

    return Assert-SafeFixtureRoot $actualRoot
}

function ConvertTo-WindowsCommandLineArgument {
    param([AllowEmptyString()][Parameter(Mandatory)][string] $Argument)

    if ($Argument.Length -gt 0 -and $Argument -notmatch '[\s"]') {
        return $Argument
    }

    $builder = New-Object System.Text.StringBuilder
    [void] $builder.Append('"')
    $backslashes = 0
    foreach ($character in $Argument.ToCharArray()) {
        if ($character -eq '\') {
            $backslashes++
            continue
        }

        if ($character -eq '"') {
            [void] $builder.Append(('\' * (($backslashes * 2) + 1)))
            [void] $builder.Append('"')
            $backslashes = 0
            continue
        }

        if ($backslashes -gt 0) {
            [void] $builder.Append(('\' * $backslashes))
            $backslashes = 0
        }
        [void] $builder.Append($character)
    }

    if ($backslashes -gt 0) {
        [void] $builder.Append(('\' * ($backslashes * 2)))
    }
    [void] $builder.Append('"')
    return $builder.ToString()
}

function Join-WindowsCommandLine {
    param([Parameter(Mandatory)][AllowEmptyCollection()][string[]] $Arguments)

    return (($Arguments | ForEach-Object { ConvertTo-WindowsCommandLineArgument $_ }) -join ' ')
}

function Import-ShortcutInterop {
    if (-not ('Winmakase.P0.ShortcutFile' -as [type])) {
        $interopPath = Join-Path $PSScriptRoot 'ShortcutInterop.cs'
        if (-not (Test-Path -LiteralPath $interopPath -PathType Leaf)) {
            throw "Unicode shortcut interop source is missing: $interopPath"
        }
        Add-Type -Path $interopPath
    }
}

function New-UnicodeShortcut {
    param(
        [Parameter(Mandatory)][string] $ShortcutPath,
        [Parameter(Mandatory)][string] $TargetPath,
        [Parameter(Mandatory)][string] $Arguments,
        [Parameter(Mandatory)][string] $WorkingDirectory,
        [Parameter(Mandatory)][string] $Description,
        [Parameter(Mandatory)][string] $IconPath
    )

    Import-ShortcutInterop
    [Winmakase.P0.ShortcutFile]::Create(
        $ShortcutPath,
        $TargetPath,
        $Arguments,
        $WorkingDirectory,
        $Description,
        $IconPath)
}

function Read-UnicodeShortcut {
    param([Parameter(Mandatory)][string] $ShortcutPath)

    Import-ShortcutInterop
    return [Winmakase.P0.ShortcutFile]::Read($ShortcutPath)
}

function Read-FixtureManifest {
    param([Parameter(Mandatory)][string] $FixtureRoot)

    $root = Assert-SafeFixtureRoot $FixtureRoot
    $manifestPath = Join-Path $root 'fixture-actions.json'
    if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
        throw "Fixture action manifest is missing: $manifestPath"
    }

    $manifest = Get-Content -LiteralPath $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($manifest.schemaVersion -ne 1) {
        throw 'Fixture action manifest schemaVersion must be 1.'
    }
    if ($manifest.executableRelativePath -cne 'bin/WinmakaseP0Fixture.exe') {
        throw 'Fixture executable path is not the fixed expected path.'
    }
    if ($manifest.captureDirectoryRelativePath -cne 'data') {
        throw 'Fixture capture directory is not the fixed expected path.'
    }
    if (@($manifest.actions).Count -ne 4) {
        throw 'Fixture action manifest must contain exactly four actions.'
    }

    $ids = @{}
    $shortcutFiles = @{}
    $allowedWindows = @('main', 'owned', 'modal', 'utility')
    foreach ($action in @($manifest.actions)) {
        if ([string]::IsNullOrWhiteSpace($action.id) -or $action.id -cnotmatch '^[a-z0-9-]+$') {
            throw "Invalid fixture action id: $($action.id)"
        }
        if ($ids.ContainsKey($action.id)) {
            throw "Duplicate fixture action id: $($action.id)"
        }
        $ids[$action.id] = $true

        if ([string]::IsNullOrWhiteSpace($action.shortcutFile) -or
            [System.IO.Path]::GetFileName($action.shortcutFile) -cne $action.shortcutFile -or
            [System.IO.Path]::GetExtension($action.shortcutFile) -cne '.lnk') {
            throw "Invalid shortcut filename for action $($action.id)"
        }
        if ($shortcutFiles.ContainsKey($action.shortcutFile)) {
            throw "Duplicate shortcut filename: $($action.shortcutFile)"
        }
        $shortcutFiles[$action.shortcutFile] = $true

        $arguments = @($action.arguments)
        if ($arguments.Count -ne 8) {
            throw "Action $($action.id) must have exactly four option/value pairs."
        }
        $parsed = @{}
        for ($index = 0; $index -lt $arguments.Count; $index += 2) {
            $option = [string] $arguments[$index]
            $value = [string] $arguments[$index + 1]
            if (@('--guest-consent', '--fixture-id', '--window', '--profile-label') -cnotcontains $option -or
                $parsed.ContainsKey($option)) {
                throw "Action $($action.id) has an invalid or duplicate option: $option"
            }
            $parsed[$option] = $value
        }
        if ($parsed['--guest-consent'] -cne $script:GuestConsent -or
            $parsed['--fixture-id'] -cne $action.expectedCapture.fixtureId -or
            $parsed['--fixture-id'] -cne $action.id -or
            $parsed['--window'] -cne $action.expectedCapture.window -or
            $parsed['--profile-label'] -cne $action.expectedCapture.profileLabel -or
            $allowedWindows -cnotcontains $parsed['--window']) {
            throw "Action $($action.id) arguments do not match its expected capture."
        }
        if ($parsed['--profile-label'].Length -gt 64 -or $parsed['--profile-label'] -match '[\x00-\x1f]') {
            throw "Action $($action.id) has an unsafe profile label."
        }
    }

    return $manifest
}

Export-ModuleMember -Function @(
    'Assert-SafeFixtureRoot',
    'Assert-WindowsSandboxGuest',
    'ConvertTo-WindowsCommandLineArgument',
    'Get-CanonicalPath',
    'Import-ShortcutInterop',
    'Join-WindowsCommandLine',
    'New-UnicodeShortcut',
    'Read-UnicodeShortcut',
    'Read-FixtureManifest',
    'Test-PathInside'
)
