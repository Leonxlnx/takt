param(
    [string]$Profile = "holy-panda",
    [int]$Volume = 75
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root "target\release\keyme.exe"

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
