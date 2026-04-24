param(
    [string]$Profile,
    [int]$Volume = -1
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root "target\release\keyme.exe"
$configDir = Join-Path $env:APPDATA "Keyme"
$configPath = Join-Path $configDir "config.json"

if (-not (Test-Path $configPath)) {
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    Copy-Item (Join-Path $root "config\default.json") $configPath -Force
}

$config = Get-Content $configPath -Raw | ConvertFrom-Json
if ([string]::IsNullOrWhiteSpace($Profile)) {
    $Profile = $config.profile
}
if ($Volume -lt 0) {
    $Volume = [int]$config.volume
}

if (-not (Test-Path $exe)) {
    Push-Location $root
    try {
        cargo build --release
    }
    finally {
        Pop-Location
    }
}

$existing = Get-Process -Name "keyme" -ErrorAction SilentlyContinue
if ($existing) {
    exit 0
}

Start-Process -FilePath $exe -ArgumentList @("--profile", $Profile, "--volume", "$Volume") -WindowStyle Hidden
