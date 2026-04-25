$ErrorActionPreference = "Stop"

Get-Process -Name "takt" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "electron" -ErrorAction SilentlyContinue | Where-Object { $_.Path -like "*\Takt\*" } | Stop-Process -Force
Get-Process -Name "keyme" -ErrorAction SilentlyContinue | Stop-Process -Force
Get-Process -Name "keeby_windows" -ErrorAction SilentlyContinue | Stop-Process -Force
