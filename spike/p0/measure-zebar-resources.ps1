[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string] $OutputPath,

    [ValidateRange(2, 3601)]
    [int] $SampleCount = 31,

    [ValidateRange(100, 60000)]
    [int] $IntervalMilliseconds = 1000,

    [string] $RequestedModel = 'gpt-5.6-sol',

    [string] $RequestedEffort = 'high',

    [ValidateRange(1, 32)]
    [int] $ContextDisplayCount = 3,

    [string] $ContextObservedDate = '2026-09-08',

    [ValidateRange(48, 480)]
    [int] $ContextDpi = 96,

    [string] $ContextPrimarySource = 'DISPLAY2'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$backendName = 'zebar.exe'
$webViewName = 'msedgewebview2.exe'

function Get-ProcessCounter {
    param(
        [object] $Process,
        [ValidateSet('backend', 'webview')]
        [string] $Role,
        [AllowNull()]
        [Nullable[uint32]] $ParentProcessId
    )

    try {
        $expectedProcessName = if ($Role -eq 'backend') {
            [System.IO.Path]::GetFileNameWithoutExtension($backendName)
        } else {
            [System.IO.Path]::GetFileNameWithoutExtension($webViewName)
        }
        if ($Process.ProcessName -ne $expectedProcessName) {
            return $null
        }

        [pscustomobject][ordered]@{
            role = $Role
            pid = [uint32]$Process.Id
            parentPid = if ($Role -eq 'backend') { $null } else { $ParentProcessId }
            startUtc = $Process.StartTime.ToUniversalTime().ToString('o')
            cpuMs = [math]::Round($Process.TotalProcessorTime.TotalMilliseconds, 3)
            privateBytes = [int64]$Process.PrivateMemorySize64
            workingSetBytes = [int64]$Process.WorkingSet64
        }
    } catch {
        $null
    }
}

function Get-TargetTree {
    param(
        [uint32] $RootProcessId,
        [object[]] $Snapshot
    )

    $queue = [System.Collections.Generic.Queue[uint32]]::new()
    $seen = [System.Collections.Generic.HashSet[uint32]]::new()
    $descendants = [System.Collections.Generic.List[object]]::new()
    $byParent = @{}

    foreach ($item in $Snapshot) {
        $parentKey = [string][uint32]$item.ParentProcessId
        if (-not $byParent.ContainsKey($parentKey)) {
            $byParent[$parentKey] = [System.Collections.Generic.List[object]]::new()
        }
        $byParent[$parentKey].Add($item)
    }

    $queue.Enqueue($RootProcessId)
    [void]$seen.Add($RootProcessId)

    while ($queue.Count -gt 0) {
        $parentId = $queue.Dequeue()
        $parentKey = [string]$parentId
        if (-not $byParent.ContainsKey($parentKey)) {
            continue
        }
        foreach ($child in $byParent[$parentKey]) {
            $childId = [uint32]$child.ProcessId
            if ($seen.Add($childId)) {
                $queue.Enqueue($childId)
                $descendants.Add($child)
            }
        }
    }

    $descendants
}

function Get-Stats {
    param([double[]] $Values)

    if ($Values.Count -eq 0) {
        return $null
    }

    $sorted = @($Values | Sort-Object)
    $p95Index = [math]::Max(0, [math]::Ceiling($sorted.Count * 0.95) - 1)
    [pscustomobject][ordered]@{
        min = [math]::Round($sorted[0], 3)
        mean = [math]::Round(($Values | Measure-Object -Average).Average, 3)
        p95 = [math]::Round($sorted[$p95Index], 3)
        max = [math]::Round($sorted[-1], 3)
    }
}

$roots = @(Get-CimInstance Win32_Process -Filter "Name = '$backendName'" -Property ProcessId, ParentProcessId, Name)
if ($roots.Count -ne 1) {
    throw "Expected exactly one target backend; found $($roots.Count). No sample was taken."
}

$rootPid = [uint32]$roots[0].ProcessId
$initialRootProcess = Get-Process -Id $rootPid -ErrorAction Stop
if ($initialRootProcess.ProcessName -ne [System.IO.Path]::GetFileNameWithoutExtension($backendName)) {
    throw 'Target backend identity did not match the expected executable. No sample was taken.'
}
$rootStartUtc = $initialRootProcess.StartTime.ToUniversalTime().ToString('o')
$samples = [System.Collections.Generic.List[object]]::new()
$intervalDeltas = [System.Collections.Generic.List[object]]::new()
$timer = [System.Diagnostics.Stopwatch]::StartNew()
$startedUtc = [DateTimeOffset]::UtcNow
$previousByPid = @{}

for ($sampleIndex = 0; $sampleIndex -lt $SampleCount; $sampleIndex++) {
    $targetOffset = [int64]$sampleIndex * $IntervalMilliseconds
    $remaining = $targetOffset - $timer.ElapsedMilliseconds
    if ($remaining -gt 0) {
        Start-Sleep -Milliseconds $remaining
    }

    $capturedUtc = [DateTimeOffset]::UtcNow
    $processes = [System.Collections.Generic.List[object]]::new()
    $counterUnavailableCount = 0

    $targetSnapshot = @(Get-CimInstance Win32_Process -Filter "Name = '$backendName' OR Name = '$webViewName'" -Property ProcessId, ParentProcessId, Name)
    $descendants = @(Get-TargetTree -RootProcessId $rootPid -Snapshot $targetSnapshot | Where-Object Name -eq $webViewName)
    $targetPids = [uint32[]]@($rootPid) + [uint32[]]@($descendants | ForEach-Object ProcessId)
    $processByPid = @{}
    foreach ($process in @(Get-Process -Id $targetPids -ErrorAction SilentlyContinue)) {
        $processByPid[[string][uint32]$process.Id] = $process
    }

    $backend = if ($processByPid.ContainsKey([string]$rootPid)) {
        Get-ProcessCounter -Process $processByPid[[string]$rootPid] -Role backend -ParentProcessId $null
    } else {
        $null
    }
    if ($null -eq $backend -or $backend.startUtc -ne $rootStartUtc) {
        throw 'Target backend disappeared or changed identity during capture; no evidence was written.'
    }
    $processes.Add($backend)

    foreach ($descendant in $descendants) {
        $descendantKey = [string][uint32]$descendant.ProcessId
        $counter = if ($processByPid.ContainsKey($descendantKey)) {
            Get-ProcessCounter -Process $processByPid[$descendantKey] -Role webview -ParentProcessId ([uint32]$descendant.ParentProcessId)
        } else {
            $null
        }
        if ($null -ne $counter) {
            $processes.Add($counter)
        } else {
            $counterUnavailableCount++
        }
    }

    $orderedProcesses = @($processes | Sort-Object role, pid)
    $privateTotal = [int64](($orderedProcesses | Measure-Object privateBytes -Sum).Sum)
    $workingSetTotal = [int64](($orderedProcesses | Measure-Object workingSetBytes -Sum).Sum)
    $backendCount = @($orderedProcesses | Where-Object role -eq backend).Count
    $webViewCount = @($orderedProcesses | Where-Object role -eq webview).Count

    $currentByPid = @{}
    foreach ($process in $orderedProcesses) {
        $currentByPid[[string]$process.pid] = $process
    }

    if ($sampleIndex -gt 0) {
        $validCpuDeltaMs = 0.0
        $validDeltaCount = 0
        $invalid = [ordered]@{
            newIdentity = 0
            missingIdentity = 0
            pidStartChanged = 0
            counterReset = 0
        }

        foreach ($process in $orderedProcesses) {
            $pidKey = [string]$process.pid
            if (-not $previousByPid.ContainsKey($pidKey)) {
                $invalid.newIdentity++
                continue
            }

            $previous = $previousByPid[$pidKey]
            if ($previous.startUtc -ne $process.startUtc) {
                $invalid.pidStartChanged++
                continue
            }

            $delta = [double]$process.cpuMs - [double]$previous.cpuMs
            if ($delta -lt 0) {
                $invalid.counterReset++
                continue
            }

            $validCpuDeltaMs += $delta
            $validDeltaCount++
        }

        foreach ($pidKey in $previousByPid.Keys) {
            if (-not $currentByPid.ContainsKey($pidKey)) {
                $invalid.missingIdentity++
            }
        }

        $intervalDeltas.Add([pscustomobject][ordered]@{
            fromSample = $sampleIndex - 1
            toSample = $sampleIndex
            validCpuDeltaMs = [math]::Round($validCpuDeltaMs, 3)
            validDeltaCount = $validDeltaCount
            invalid = [pscustomobject]$invalid
        })
    }

    $samples.Add([pscustomobject][ordered]@{
        index = $sampleIndex
        capturedUtc = $capturedUtc.ToString('o')
        elapsedMs = $timer.ElapsedMilliseconds
        backendCount = $backendCount
        webViewCount = $webViewCount
        counterUnavailableCount = $counterUnavailableCount
        totalPrivateBytes = $privateTotal
        totalWorkingSetBytes = $workingSetTotal
        processes = $orderedProcesses
    })

    $previousByPid = $currentByPid
}

$timer.Stop()
$endedUtc = [DateTimeOffset]::UtcNow
$cpuDeltas = [double[]]@($intervalDeltas | ForEach-Object validCpuDeltaMs)
$privateTotals = [double[]]@($samples | ForEach-Object totalPrivateBytes)
$workingSetTotals = [double[]]@($samples | ForEach-Object totalWorkingSetBytes)
$processCounts = [double[]]@($samples | ForEach-Object { $_.backendCount + $_.webViewCount })
$webViewCounts = [double[]]@($samples | ForEach-Object webViewCount)

$invalidTotals = [ordered]@{
    newIdentity = [int](($intervalDeltas | ForEach-Object invalid | Measure-Object newIdentity -Sum).Sum)
    missingIdentity = [int](($intervalDeltas | ForEach-Object invalid | Measure-Object missingIdentity -Sum).Sum)
    pidStartChanged = [int](($intervalDeltas | ForEach-Object invalid | Measure-Object pidStartChanged -Sum).Sum)
    counterReset = [int](($intervalDeltas | ForEach-Object invalid | Measure-Object counterReset -Sum).Sum)
}

$document = [pscustomobject][ordered]@{
    schemaVersion = 1
    subject = 'zebar-backend-and-descendant-webviews'
    model = [pscustomobject][ordered]@{
        requested = $RequestedModel
        requestedEffort = $RequestedEffort
        actual = $null
        actualEffort = $null
        caveat = 'The execution surface did not independently expose actual model or effort; no substitution was reported.'
    }
    context = [pscustomobject][ordered]@{
        displayCount = $ContextDisplayCount
        displayEvidenceObservedDate = $ContextObservedDate
        dpi = $ContextDpi
        primarySource = $ContextPrimarySource
        evidence = 'docs/verification/v1/2026-09-08-p0/baseline.json on the integration branch'
        displayCaveat = 'Dated context only; this capture did not query display topology or change displays.'
    }
    capture = [pscustomobject][ordered]@{
        startedUtc = $startedUtc.ToString('o')
        endedUtc = $endedUtc.ToString('o')
        elapsedMs = $timer.ElapsedMilliseconds
        sampleCount = $samples.Count
        targetIntervalMs = $IntervalMilliseconds
        intervalDeltaCount = $intervalDeltas.Count
    }
    method = [pscustomobject][ordered]@{
        rootSelection = 'Exactly one running target backend, selected by exact executable name.'
        descendants = 'One exact-name Win32_Process snapshot per sample; recursive ParentProcessId graph from the fixed backend PID; only connected descendant WebView executables contribute counters.'
        counters = 'Get-Process cumulative CPU time, private bytes, working set, PID, and UTC start time.'
        deltaValidity = 'A delta requires the same PID and startUtc in adjacent samples and a nondecreasing CPU counter. New, missing, reused, or reset identities are excluded.'
        sanitation = 'No window titles, command lines, executable paths, or unrelated process names are emitted.'
        limitations = 'CIM ancestry and Get-Process counters are sequential, not an atomic snapshot. Processes wholly between samples are missed. Valid CPU total excludes new, missing, reused, reset, and missed lifetimes. Summed working sets can double-count shared pages.'
    }
    summary = [pscustomobject][ordered]@{
        cpuDeltaMs = Get-Stats -Values $cpuDeltas
        validCpuDeltaMsTotal = [math]::Round(($cpuDeltas | Measure-Object -Sum).Sum, 3)
        invalidCpuDeltas = [pscustomobject]$invalidTotals
        privateBytes = Get-Stats -Values $privateTotals
        workingSetBytes = Get-Stats -Values $workingSetTotals
        processCount = Get-Stats -Values $processCounts
        webViewCount = Get-Stats -Values $webViewCounts
    }
    intervalCpuDeltas = $intervalDeltas
    samples = $samples
}

$resolvedOutput = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputPath)
$outputDirectory = Split-Path -Parent $resolvedOutput
if (-not (Test-Path -LiteralPath $outputDirectory)) {
    New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
}
$document | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $resolvedOutput -Encoding utf8NoBOM

[pscustomobject]@{
    output = $resolvedOutput
    samples = $samples.Count
    elapsedMs = $timer.ElapsedMilliseconds
    minProcesses = [int]$document.summary.processCount.min
    maxProcesses = [int]$document.summary.processCount.max
    validCpuDeltaMsTotal = $document.summary.validCpuDeltaMsTotal
} | ConvertTo-Json -Compress
