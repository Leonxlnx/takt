$ErrorActionPreference = "Stop"

& (Join-Path $PSScriptRoot "stop.ps1")

$desktop = [Environment]::GetFolderPath("Desktop")
$startup = [Environment]::GetFolderPath("Startup")
$programs = [Environment]::GetFolderPath("Programs")
$startMenuFolder = Join-Path $programs "Keyme"

$paths = @(
    (Join-Path $desktop "Start Keyme.lnk"),
    (Join-Path $desktop "Stop Keyme.lnk"),
    (Join-Path $startup "Keyme.lnk"),
    (Join-Path $startMenuFolder "Start Keyme.lnk"),
    (Join-Path $startMenuFolder "Stop Keyme.lnk")
)

foreach ($path in $paths) {
    Remove-Item $path -ErrorAction SilentlyContinue
}

Remove-Item $startMenuFolder -ErrorAction SilentlyContinue

Write-Host "Removed Keyme shortcuts and stopped the app."
