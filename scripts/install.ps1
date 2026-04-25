$ErrorActionPreference = "Stop"

$sourceRoot = Split-Path -Parent $PSScriptRoot
$installDir = Join-Path $env:LOCALAPPDATA "Takt"
$installScripts = Join-Path $installDir "scripts"
$installConfig = Join-Path $installDir "config"
$installAssets = Join-Path $installDir "assets"
$sourceExe = Join-Path $sourceRoot "target\release\takt.exe"
$sourceControlExe = Join-Path $sourceRoot "target\release\takt-control.exe"

if (-not (Test-Path $sourceExe) -or -not (Test-Path $sourceControlExe)) {
    $bundledExe = Join-Path $sourceRoot "takt.exe"
    $bundledControlExe = Join-Path $sourceRoot "takt-control.exe"
    if ((Test-Path $bundledExe) -and (Test-Path $bundledControlExe)) {
        $sourceExe = $bundledExe
        $sourceControlExe = $bundledControlExe
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
Start-Sleep -Seconds 2

New-Item -ItemType Directory -Path $installDir, $installScripts, $installConfig, $installAssets -Force | Out-Null
Copy-Item $sourceExe (Join-Path $installDir "takt.exe") -Force
Copy-Item $sourceControlExe (Join-Path $installDir "takt-control.exe") -Force
Copy-Item (Join-Path $PSScriptRoot "*") $installScripts -Recurse -Force
Copy-Item (Join-Path $sourceRoot "config\*") $installConfig -Recurse -Force
Copy-Item (Join-Path $sourceRoot "assets\*") $installAssets -Recurse -Force
Copy-Item (Join-Path $sourceRoot "README.md") $installDir -Force
Copy-Item (Join-Path $sourceRoot "LICENSE") $installDir -Force

$startVbs = Join-Path $installScripts "launch-hidden.vbs"
$stopVbs = Join-Path $installScripts "stop-hidden.vbs"

$shell = New-Object -ComObject WScript.Shell
$desktop = [Environment]::GetFolderPath("Desktop")
$startup = [Environment]::GetFolderPath("Startup")
$programs = [Environment]::GetFolderPath("Programs")
$startMenuFolder = Join-Path $programs "Takt"
$legacyKeymeStartMenuFolder = Join-Path $programs "Keyme"
$legacyStartMenuFolder = Join-Path $programs "Keeby Windows"
New-Item -ItemType Directory -Path $startMenuFolder -Force | Out-Null

Remove-Item (Join-Path $desktop "Keyme.lnk") -ErrorAction SilentlyContinue
Remove-Item (Join-Path $startup "Keyme.lnk") -ErrorAction SilentlyContinue
Remove-Item (Join-Path $desktop "Start Keeby Windows.lnk") -ErrorAction SilentlyContinue
Remove-Item (Join-Path $desktop "Stop Keeby Windows.lnk") -ErrorAction SilentlyContinue
Remove-Item (Join-Path $startup "Keeby Windows.lnk") -ErrorAction SilentlyContinue
Remove-Item $legacyKeymeStartMenuFolder -Recurse -ErrorAction SilentlyContinue
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
    $iconPath = Join-Path $installAssets "takt.ico"
    if (Test-Path $iconPath) {
        $shortcut.IconLocation = $iconPath
    }
    $shortcut.Save()
}

function New-ExeShortcut {
    param(
        [string]$Path,
        [string]$ExePath,
        [string]$Description
    )

    $shortcut = $shell.CreateShortcut($Path)
    $shortcut.TargetPath = $ExePath
    $shortcut.WorkingDirectory = $installDir
    $shortcut.Description = $Description
    $iconPath = Join-Path $installAssets "takt.ico"
    if (Test-Path $iconPath) {
        $shortcut.IconLocation = $iconPath
    }
    $shortcut.Save()
}

New-ExeShortcut `
    -Path (Join-Path $desktop "Takt.lnk") `
    -ExePath (Join-Path $installDir "takt-control.exe") `
    -Description "Open Takt"

New-VbsShortcut `
    -Path (Join-Path $startup "Takt.lnk") `
    -VbsPath $startVbs `
    -Description "Start mechanical keyboard sounds at login"

New-ExeShortcut `
    -Path (Join-Path $startMenuFolder "Takt.lnk") `
    -ExePath (Join-Path $installDir "takt-control.exe") `
    -Description "Open Takt"

New-VbsShortcut `
    -Path (Join-Path $startMenuFolder "Start Takt.lnk") `
    -VbsPath $startVbs `
    -Description "Start mechanical keyboard sounds"

New-VbsShortcut `
    -Path (Join-Path $startMenuFolder "Stop Takt.lnk") `
    -VbsPath $stopVbs `
    -Description "Stop mechanical keyboard sounds"

& (Join-Path $installScripts "run.ps1")
for ($i = 0; $i -lt 10 -and -not (Get-Process -Name "takt" -ErrorAction SilentlyContinue); $i++) {
    Start-Sleep -Milliseconds 500
}

if (-not (Get-Process -Name "takt" -ErrorAction SilentlyContinue)) {
    $configPath = Join-Path $env:APPDATA "Takt\config.json"
    $config = Get-Content $configPath -Raw | ConvertFrom-Json
    Start-Process `
        -FilePath (Join-Path $installDir "takt.exe") `
        -ArgumentList @("--profile", $config.profile, "--volume", "$($config.volume)") `
        -WindowStyle Hidden
    for ($i = 0; $i -lt 10 -and -not (Get-Process -Name "takt" -ErrorAction SilentlyContinue); $i++) {
        Start-Sleep -Milliseconds 500
    }
}

if (-not (Get-Process -Name "takt" -ErrorAction SilentlyContinue)) {
    Start-Process -FilePath "$env:WINDIR\System32\wscript.exe" -ArgumentList "`"$startVbs`""
}

if (-not (Get-Process -Name "takt" -ErrorAction SilentlyContinue)) {
    $delayedLaunch = "timeout /t 2 /nobreak >nul & wscript.exe `"$startVbs`""
    Start-Process -FilePath $env:ComSpec -ArgumentList @("/c", $delayedLaunch) -WindowStyle Hidden
}

Write-Host "Installed Takt."
Write-Host "Install folder: $installDir"
Write-Host "Desktop shortcut: Takt"
Write-Host "Startup shortcut: Takt"
