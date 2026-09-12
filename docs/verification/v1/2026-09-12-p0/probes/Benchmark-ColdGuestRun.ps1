$ErrorActionPreference='Stop'
trap { ($_ | Out-String) | Set-Content C:/WinmakaseEvidence/run-cold-error.txt -Encoding UTF8; exit 1 }
. C:/WinmakaseProbe/Probe-RunEvent.ps1 -Action Observe -Label cold-before
$installRoot=Join-Path $env:LOCALAPPDATA PowerToys
$runnerPath=Join-Path $installRoot PowerToys.exe
$samples=@()
foreach($sample in 1..5) {
  $owned=@(Get-Process -Name 'PowerToys*' -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($installRoot+'\',[StringComparison]::OrdinalIgnoreCase) })
  foreach($proc in $owned) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
  Start-Sleep -Milliseconds 300
  $timer=[Diagnostics.Stopwatch]::StartNew()
  $runner=Start-Process -FilePath $runnerPath -WindowStyle Hidden -PassThru
  $event=$null
  do {
    $newRun=@(Get-Process -Name PowerToys.PowerLauncher -ErrorAction SilentlyContinue)
    if($newRun.Count -eq 1) {
      try { $event=[Threading.EventWaitHandle]::OpenExisting('Local\PowerToysRunInvokeEvent-30f26ad7-d36d-4c0e-ab02-68bb5ff3c4ab') } catch [Threading.WaitHandleCannotBeOpenedException] {}
    }
    if(!$event) { Start-Sleep -Milliseconds 20 }
  } while(!$event -and $timer.ElapsedMilliseconds -lt 20000)
  if(!$event) { throw 'Run activation event did not appear after restart' }
  if((Get-FileHash $newRun[0].Path -Algorithm SHA256).Hash -ne '062ED25AA8F343C83641277111BCCAC366B25EBF80021B1E4D783EEF25D2E688') { throw 'Unexpected Run image' }
  [uint32[]]$runIds=@($newRun | ForEach-Object Id)
  $ready=$false
  $eventReadyMs=$timer.Elapsed.TotalMilliseconds
  try {
    [void]$event.Set()
    do {
      $state=Read-RunState
      $shown=@($state.windows | Where-Object { $_.visible -and $_.cloaked -eq 0 }).Count -eq 1
      if($shown -and $state.foregroundPid -in $runIds -and $state.focusedRunElement.isKeyboardFocused -and $state.focusedRunElement.name -eq 'Query') { $ready=$true; break }
      Start-Sleep -Milliseconds 10
    } while($timer.ElapsedMilliseconds -lt 20000)
    $samples+=[ordered]@{sample=$sample;ready=$ready;eventAvailableMs=$eventReadyMs;restartToInputReadyMs=$timer.Elapsed.TotalMilliseconds;runPid=$runIds[0]}
    if($ready) { [void]$event.Set() }
  } finally { $event.Dispose() }
  if(!$ready) { break }
  Start-Sleep -Milliseconds 100
}
[ordered]@{at=[DateTimeOffset]::UtcNow.ToString('o');method='Five complete guest PowerToys runner/Run process restarts with warm OS/file caches. From Start-Process until event exists, signal, visible/uncloaked Run foreground and UIA Query focus. Observer polling included. Pinned guest Glaze running; nonfixture windows ignored. Not cold boot, physical key or catalog readiness.';samples=$samples} | ConvertTo-Json -Depth 6 | Set-Content C:/WinmakaseEvidence/run-cold-benchmark.json -Encoding UTF8
