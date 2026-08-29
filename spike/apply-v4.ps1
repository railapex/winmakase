# Apply v4: restart kanata (pure-modifier config) + bounce PowerToys (PT Run hotkey -> Win+Space)
Get-Process | Where-Object { $_.ProcessName -like 'kanata*' } | Stop-Process -Force
Get-Process PowerToys -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 2
Start-Process -FilePath 'D:\dev\winmakase\spike\tools\kanata\kanata_windows_gui_winIOv2_x64.exe' -ArgumentList '--cfg','D:\dev\winmakase\spike\caps.kbd'
Start-Process 'C:\Program Files\PowerToys\PowerToys.exe'
Write-Output 'kanata v4 + PowerToys restarted'
Start-Sleep -Seconds 3
