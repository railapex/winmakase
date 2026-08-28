' Winsome shim: move focused window to workspace N and follow it (two ordered commands, no console).
Dim ws, sh
ws = WScript.Arguments(0)
Set sh = CreateObject("WScript.Shell")
sh.Run """C:\Program Files\glzr.io\GlazeWM\cli\glazewm.exe"" command move --workspace " & ws, 0, True
sh.Run """C:\Program Files\glzr.io\GlazeWM\cli\glazewm.exe"" command focus --workspace " & ws, 0, False
