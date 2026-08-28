# Restart elevated kanata with the current caps.kbd (run via UAC — stops the old elevated instance first)
Get-Process kanata_windows_tty_winIOv2_x64 -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Seconds 1
Start-Process -FilePath 'D:\dev\winsome\spike\tools\kanata\kanata_windows_gui_winIOv2_x64.exe' -ArgumentList '--cfg','D:\dev\winsome\spike\caps.kbd'
Write-Output 'kanata restarted elevated (gui build - tray icon, no console)'
Start-Sleep -Seconds 2
