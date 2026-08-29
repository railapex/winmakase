# One-time (elevated): register the WinmakaseKanata and WinmakasePanic on-demand tasks.
# v5: cmd_allowed build (direct-drive — kanata executes glazewm CLI commands itself).
Get-Process | Where-Object { $_.ProcessName -like 'kanata*' } | Stop-Process -Force -ErrorAction SilentlyContinue
$kanata = New-ScheduledTaskAction -Execute 'D:\dev\winmakase\spike\tools\kanata\kanata_windows_gui_winIOv2_cmd_allowed_x64.exe' -Argument '--cfg D:\dev\winmakase\spike\caps.kbd'
$panic  = New-ScheduledTaskAction -Execute 'pwsh' -Argument '-NoProfile -WindowStyle Hidden -File D:\dev\winmakase\spike\panic.ps1'
$principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -RunLevel Highest
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit (New-TimeSpan -Hours 0)
Register-ScheduledTask -TaskName 'WinmakaseKanata' -Action $kanata -Principal $principal -Settings $settings -Force | Out-Null
Register-ScheduledTask -TaskName 'WinmakasePanic' -Action $panic -Principal $principal -Settings $settings -Force | Out-Null
Write-Output 'tasks registered: WinmakaseKanata, WinmakasePanic (on-demand, elevated, UAC-free to trigger)'
Start-Sleep -Seconds 3
