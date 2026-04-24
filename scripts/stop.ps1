$ErrorActionPreference = "Stop"

Get-Process -Name "keyme" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "keeby_windows" -ErrorAction SilentlyContinue | Stop-Process -Force
