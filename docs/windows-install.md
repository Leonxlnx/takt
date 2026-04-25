# Windows Install

Takt is Windows-only.

## Normal Install

1. Download the release ZIP from GitHub.
2. Extract the ZIP.
3. Double-click `Setup-Takt.cmd`.

Setup installs Takt to:

```text
%LOCALAPPDATA%\Takt
```

It creates one Desktop shortcut:

```text
Takt
```

It also creates a Startup shortcut so Takt launches after Windows login.

## Uninstall

Run:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%LOCALAPPDATA%\Takt\scripts\uninstall.ps1"
```

The uninstall script stops Takt, removes shortcuts, and removes the installed app folder.
