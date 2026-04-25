$ErrorActionPreference = "Stop"

$sourceRoot = Split-Path -Parent $PSScriptRoot
$installDir = Join-Path $env:LOCALAPPDATA "Keyme"
$installScripts = Join-Path $installDir "scripts"
$installConfig = Join-Path $installDir "config"
$installAssets = Join-Path $installDir "assets"
$sourceExe = Join-Path $sourceRoot "target\release\keyme.exe"

if (-not (Test-Path $sourceExe)) {
    $bundledExe = Join-Path $sourceRoot "keyme.exe"
    if (Test-Path $bundledExe) {
        $sourceExe = $bundledExe
    }
    else {
        Push-Location $sourceRoot
        try {
            cargo build --release
        }
        finally {
            Pop-Location
        }
    }
}

& (Join-Path $PSScriptRoot "stop.ps1")

New-Item -ItemType Directory -Path $installDir, $installScripts, $installConfig, $installAssets -Force | Out-Null
Copy-Item $sourceExe (Join-Path $installDir "keyme.exe") -Force
Copy-Item (Join-Path $PSScriptRoot "*") $installScripts -Recurse -Force
Copy-Item (Join-Path $sourceRoot "config\*") $installConfig -Recurse -Force
Copy-Item (Join-Path $sourceRoot "assets\*") $installAssets -Recurse -Force
Copy-Item (Join-Path $sourceRoot "README.md") $installDir -Force
Copy-Item (Join-Path $sourceRoot "LICENSE") $installDir -Force

$startVbs = Join-Path $installScripts "launch-hidden.vbs"
$stopVbs = Join-Path $installScripts "stop-hidden.vbs"
$controlVbs = Join-Path $installScripts "control-hidden.vbs"

$shell = New-Object -ComObject WScript.Shell
$desktop = [Environment]::GetFolderPath("Desktop")
$startup = [Environment]::GetFolderPath("Startup")
$programs = [Environment]::GetFolderPath("Programs")
$startMenuFolder = Join-Path $programs "Keyme"
$legacyStartMenuFolder = Join-Path $programs "Keeby Windows"
New-Item -ItemType Directory -Path $startMenuFolder -Force | Out-Null

Remove-Item (Join-Path $desktop "Start Keeby Windows.lnk") -ErrorAction SilentlyContinue
Remove-Item (Join-Path $desktop "Stop Keeby Windows.lnk") -ErrorAction SilentlyContinue
Remove-Item (Join-Path $startup "Keeby Windows.lnk") -ErrorAction SilentlyContinue
Remove-Item $legacyStartMenuFolder -Recurse -ErrorAction SilentlyContinue

function New-VbsShortcut {
    param(
        [string]$Path,
        [string]$VbsPath,
        [string]$Description
    )

    $shortcut = $shell.CreateShortcut($Path)
    $shortcut.TargetPath = "$env:WINDIR\System32\wscript.exe"
    $shortcut.Arguments = "`"$VbsPath`""
    $shortcut.WorkingDirectory = $installDir
    $shortcut.Description = $Description
    $iconPath = Join-Path $installAssets "keyme.ico"
    if (Test-Path $iconPath) {
        $shortcut.IconLocation = $iconPath
    }
    $shortcut.Save()
}

New-VbsShortcut `
    -Path (Join-Path $desktop "Keyme.lnk") `
    -VbsPath $controlVbs `
    -Description "Open Keyme settings"

New-VbsShortcut `
    -Path (Join-Path $startup "Keyme.lnk") `
    -VbsPath $startVbs `
    -Description "Start mechanical keyboard sounds at login"

New-VbsShortcut `
    -Path (Join-Path $startMenuFolder "Keyme.lnk") `
    -VbsPath $controlVbs `
    -Description "Open Keyme settings"

New-VbsShortcut `
    -Path (Join-Path $startMenuFolder "Start Keyme.lnk") `
    -VbsPath $startVbs `
    -Description "Start mechanical keyboard sounds"

New-VbsShortcut `
    -Path (Join-Path $startMenuFolder "Stop Keyme.lnk") `
    -VbsPath $stopVbs `
    -Description "Stop mechanical keyboard sounds"

& (Join-Path $installScripts "run.ps1")

Write-Host "Installed Keyme."
Write-Host "Install folder: $installDir"
Write-Host "Desktop shortcut: Keyme"
Write-Host "Startup shortcut: Keyme"
