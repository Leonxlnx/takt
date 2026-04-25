@echo off
setlocal
cd /d "%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0scripts\install.ps1"
if errorlevel 1 (
  echo.
  echo Takt setup failed.
  pause
  exit /b 1
)
echo.
echo Takt is installed. Open it from the Desktop shortcut.
pause
