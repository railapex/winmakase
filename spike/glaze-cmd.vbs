' Winmakase shim: run a glazewm command with NO console window (wscript = GUI subsystem).
' Usage: wscript //B glaze-cmd.vbs <command args...>   e.g. ... glaze-cmd.vbs focus --direction left
Dim i, cmdline
cmdline = """C:\Program Files\glzr.io\GlazeWM\cli\glazewm.exe"" command"
For i = 0 To WScript.Arguments.Count - 1
  cmdline = cmdline & " " & WScript.Arguments(i)
Next
CreateObject("WScript.Shell").Run cmdline, 0, False
