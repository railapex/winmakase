# Winsome M0 spike — elevated batch (run via UAC):
# 1. Chrome BrowserThemeColor via HKLM (HKCU Policies is ACL-locked on this box)
# 2. kanata elevated with the caps spike config (admin-window chord test)
New-Item -Path 'HKLM:\SOFTWARE\Policies\Google\Chrome' -Force | Out-Null
Set-ItemProperty -Path 'HKLM:\SOFTWARE\Policies\Google\Chrome' -Name 'BrowserThemeColor' -Value '#1a1b26'
Write-Output "Chrome policy set (HKLM): $((Get-ItemProperty 'HKLM:\SOFTWARE\Policies\Google\Chrome').BrowserThemeColor)"
Start-Process -FilePath 'D:\dev\winsome\spike\tools\kanata\kanata_windows_tty_winIOv2_x64.exe' -ArgumentList '--cfg','D:\dev\winsome\spike\caps.kbd'
Write-Output 'kanata launched elevated (its own console window)'
Start-Sleep -Seconds 4
