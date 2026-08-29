# Bring up the Winmakase dogfood stack after a panic or manual stop. The
# supervisor owns everything now — components AND taskbar state
# (hide_taskbar in ~/.winmakase/config.toml) — so this is just the trigger.
schtasks /run /tn WinmakaseSupervisor | Out-Null
Write-Output 'WinmakaseSupervisor triggered (it adopts or starts kanata + GlazeWM and hides the taskbar)'
