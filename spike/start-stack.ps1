# Winmakase dogfood stack — run after reboot/logoff to bring the spike back.
# GlazeWM starts user-level; kanata elevated (one UAC) so chords work in admin windows.
# PowerToys KBM must stay disabled while kanata owns Caps.
Start-Process 'C:\Program Files\glzr.io\GlazeWM\glazewm.exe'
Start-Process -Verb RunAs -FilePath 'D:\dev\winmakase\spike\tools\kanata\kanata_windows_gui_winIOv2_x64.exe' -ArgumentList '--cfg','D:\dev\winmakase\spike\caps.kbd'
Write-Output 'GlazeWM launched; approve the UAC for elevated kanata.'
