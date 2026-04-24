@echo off
setlocal
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\install.ps1"
if errorlevel 1 (
  echo.
  echo Keyme setup failed.
  pause
  exit /b 1
)
echo.
echo Keyme is installed. You can open it from the Desktop shortcut.
pause
