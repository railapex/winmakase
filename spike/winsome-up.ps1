# Bring up the Winsome dogfood stack after a panic or manual stop. The
# supervisor owns everything now — components AND taskbar state
# (hide_taskbar in ~/.winsome/config.toml) — so this is just the trigger.
schtasks /run /tn WinsomeSupervisor | Out-Null
Write-Output 'WinsomeSupervisor triggered (it adopts or starts kanata + GlazeWM and hides the taskbar)'
