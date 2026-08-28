# Emergency stop: kill any elevated kanata instance (gui or tty build)
Get-Process | Where-Object { $_.ProcessName -like 'kanata*' } | Stop-Process -Force
Write-Output 'kanata stopped'
Start-Sleep -Seconds 2
