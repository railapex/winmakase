$ErrorActionPreference='Stop'
. C:/WinmakaseProbe/Probe-RunEvent.ps1 -Action Observe -Label benchmark-before
$event=[Threading.EventWaitHandle]::OpenExisting('Local\PowerToysRunInvokeEvent-30f26ad7-d36d-4c0e-ab02-68bb5ff3c4ab')
function Wait-RunVisibility([bool]$Visible,[Diagnostics.Stopwatch]$Timer) {
  do {
    $state=Read-RunState
    $shown=@($state.windows | Where-Object { $_.visible -and $_.cloaked -eq 0 }).Count -eq 1
    if($shown -eq $Visible -and (!$Visible -or ($state.foregroundPid -in $runIds -and $state.focusedRunElement.isKeyboardFocused -and $state.focusedRunElement.name -eq 'Query'))) { return $true }
    Start-Sleep -Milliseconds 5
  } while($Timer.ElapsedMilliseconds -lt 3000)
  return $false
}
$samples=@()
try {
  $initial=Read-RunState
  if(@($initial.windows | Where-Object visible).Count -gt 0) { [void]$event.Set(); if(!(Wait-RunVisibility $false ([Diagnostics.Stopwatch]::StartNew()))) { throw 'Cannot hide initial Run' } }
  foreach($i in 1..30) {
    $timer=[Diagnostics.Stopwatch]::StartNew()
    [void]$event.Set()
    $ready=Wait-RunVisibility $true $timer
    $samples+=[ordered]@{sample=$i;ready=$ready;inputReadyMs=$timer.Elapsed.TotalMilliseconds}
    if(!$ready) { break }
    [void]$event.Set()
    if(!(Wait-RunVisibility $false ([Diagnostics.Stopwatch]::StartNew()))) { throw 'Run failed to hide between samples' }
    Start-Sleep -Milliseconds 50
  }
} finally { $event.Dispose() }
[ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o');method='30 warm event activations; until visible/uncloaked, foreground and UIA Query keyboard focus. Timing includes repeated native/UIA observation, excludes launching the observer. Guest without Glaze management; software rendering and one RDP display. Not physical-input or cold OS benchmark.';samples=$samples} | ConvertTo-Json -Depth 6 | Set-Content C:/WinmakaseEvidence/run-warm-benchmark.json -Encoding UTF8
