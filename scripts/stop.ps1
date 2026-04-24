$ErrorActionPreference = "Stop"

Get-Process -Name "keyme" -ErrorAction SilentlyContinue | Stop-Process -Force
