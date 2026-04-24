Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
scriptDir = fso.GetParentFolderName(WScript.ScriptFullName)
ps1 = scriptDir & "\stop.ps1"
shell.Run "powershell.exe -NoProfile -ExecutionPolicy Bypass -File " & Chr(34) & ps1 & Chr(34), 0, False
