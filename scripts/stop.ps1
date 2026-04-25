$ErrorActionPreference = "Stop"

Get-Process -Name "takt" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "takt-control" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "keyme" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "keeby_windows" -ErrorAction SilentlyContinue | Stop-Process -Force
