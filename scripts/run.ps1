param(
    [string]$Profile,
    [int]$Volume = -1
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$sourceRoot = Split-Path -Parent $root
$exe = Join-Path $root "takt.exe"
if (-not (Test-Path $exe)) {
    $exe = Join-Path $root "target\release\takt.exe"
}
$configDir = Join-Path $env:APPDATA "Takt"
$configPath = Join-Path $configDir "config.json"

if (-not (Test-Path $configPath)) {
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    $defaultConfig = Join-Path $root "config\default.json"
    if (-not (Test-Path $defaultConfig)) {
        $defaultConfig = Join-Path $sourceRoot "config\default.json"
    }
    Copy-Item $defaultConfig $configPath -Force
}

$config = Get-Content $configPath -Raw | ConvertFrom-Json
if ([string]::IsNullOrWhiteSpace($Profile)) {
    $Profile = $config.profile
}
if ($Volume -lt 0) {
    $Volume = [int]$config.volume
}

if (-not (Test-Path $exe)) {
    Push-Location $sourceRoot
    try {
        cargo build --release
    }
    finally {
        Pop-Location
    }
}

$existing = Get-Process -Name "takt" -ErrorAction SilentlyContinue
if ($existing) {
    exit 0
}

Start-Process -FilePath $exe -ArgumentList @("--profile", $Profile, "--volume", "$Volume") -WindowStyle Hidden
