# Remove the BrowserThemeColor spike test policy (HKLM, needs elevation)
Remove-Item -Path 'HKLM:\SOFTWARE\Policies\Google\Chrome' -Recurse -Force -ErrorAction SilentlyContinue
Write-Output "Chrome policy key removed: $(-not (Test-Path 'HKLM:\SOFTWARE\Policies\Google\Chrome'))"
Start-Sleep -Seconds 3
