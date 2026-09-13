$ErrorActionPreference='Stop'
Import-Module C:/WinmakaseSource/Fixture.Common.psm1 -Force
Assert-WindowsSandboxGuest -FixtureRoot C:/WinmakaseP0 -ConsentToken DISPOSABLE-WINDOWS-GUEST | Out-Null
. C:/WinmakaseI1Probe/Probe-Launcher.ps1 -FunctionsOnly

Ensure-Hidden
$opened=Invoke-LauncherCli @()
$state=Wait-Launcher $true
$launcher=@($state.launcher | Where-Object { $_.visible -and $_.cloaked -eq 0 })[0]
$query=Query-Value ([long]$launcher.hwnd)
$query.pattern.SetValue('winmakase-unfocused-proof')
$query.element.SetFocus()
$focusControl=Start-Process powershell.exe -ArgumentList '-NoProfile','-ExecutionPolicy','Bypass','-File','C:/WinmakaseI1Probe/Focus-Control.ps1' -PassThru
$timer=[Diagnostics.Stopwatch]::StartNew()
do { Start-Sleep -Milliseconds 25; $focusControl.Refresh() } while (!$focusControl.MainWindowHandle -and $timer.ElapsedMilliseconds -lt 5000)
if (!$focusControl.MainWindowHandle) { throw 'Focus control window did not appear' }
[ordered]@{ pid=$focusControl.Id; hwnd=$focusControl.MainWindowHandle.ToInt64(); launcherHwnd=$launcher.hwnd } | ConvertTo-Json | Set-Content C:/WinmakaseI1Evidence/unfocused-ready.json -Encoding UTF8
$timer.Restart()
while (!(Test-Path C:/WinmakaseI1Evidence/unfocused-continue.flag) -and $timer.ElapsedMilliseconds -lt 180000) { Start-Sleep -Milliseconds 50 }
if (!(Test-Path C:/WinmakaseI1Evidence/unfocused-continue.flag)) { [void]$focusControl.CloseMainWindow(); throw 'External focus proof timed out' }
$before=Read-RunState
$result=Invoke-LauncherCli @()
$after=Read-RunState
$afterQuery=(Query-Value ([long]$launcher.hwnd)).value
[void]$focusControl.CloseMainWindow()
$verdict=($before.foregroundHwnd -eq $focusControl.MainWindowHandle.ToInt64() -and $result.exitCode -eq 0 -and $result.json.outcome -eq 'focused_existing' -and !$result.json.signalled -and $after.foregroundHwnd -eq $launcher.hwnd -and $afterQuery -eq 'winmakase-unfocused-proof')
[ordered]@{ at=[DateTimeOffset]::UtcNow.ToString('o'); before=$before; command=$result; after=$after; query=$afterQuery; verdict=if($verdict){'pass'}else{'fail'} } | ConvertTo-Json -Depth 9 | Set-Content C:/WinmakaseI1Evidence/visible-unfocused-proof.json -Encoding UTF8
if(!$verdict){throw 'visible-unfocused focus proof failed'}
