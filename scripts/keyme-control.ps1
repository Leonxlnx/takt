$ErrorActionPreference = "Stop"

Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$root = Split-Path -Parent $PSScriptRoot
$sourceRoot = Split-Path -Parent $root
$configDir = Join-Path $env:APPDATA "Keyme"
$configPath = Join-Path $configDir "config.json"
$startupShortcut = Join-Path ([Environment]::GetFolderPath("Startup")) "Keyme.lnk"
$launcher = Join-Path $PSScriptRoot "launch-hidden.vbs"
$profiles = @(
    "holy-panda",
    "red",
    "alps-blue",
    "box-navy",
    "topre",
    "nk-cream",
    "buckling-spring",
    "ink-black",
    "turquoise-tealios",
    "alpaca",
    "typewriter"
)

function Ensure-Config {
    if (-not (Test-Path $configPath)) {
        New-Item -ItemType Directory -Path $configDir -Force | Out-Null
        $defaultConfig = Join-Path $root "config\default.json"
        if (-not (Test-Path $defaultConfig)) {
            $defaultConfig = Join-Path $sourceRoot "config\default.json"
        }
        Copy-Item $defaultConfig $configPath -Force
    }
}

function Read-KeymeConfig {
    Ensure-Config
    Get-Content $configPath -Raw | ConvertFrom-Json
}

function Write-KeymeConfig {
    param([string]$Profile, [int]$Volume, [bool]$Autostart)

    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    [pscustomobject]@{
        profile = $Profile
        volume = $Volume
        autostart = $Autostart
    } | ConvertTo-Json | Set-Content $configPath -Encoding UTF8
}

function Get-KeymeRunning {
    [bool](Get-Process -Name "keyme" -ErrorAction SilentlyContinue)
}

function Set-KeymeAutostart {
    param([bool]$Enabled)

    if ($Enabled) {
        $shell = New-Object -ComObject WScript.Shell
        $shortcut = $shell.CreateShortcut($startupShortcut)
        $shortcut.TargetPath = "$env:WINDIR\System32\wscript.exe"
        $shortcut.Arguments = "`"$launcher`""
        $shortcut.WorkingDirectory = $root
        $shortcut.Description = "Start Keyme at login"
        $shortcut.Save()
    }
    else {
        Remove-Item $startupShortcut -ErrorAction SilentlyContinue
    }
}

function Start-Keyme {
    & (Join-Path $PSScriptRoot "run.ps1")
}

function Stop-Keyme {
    & (Join-Path $PSScriptRoot "stop.ps1")
}

$config = Read-KeymeConfig

$form = New-Object Windows.Forms.Form
$form.Text = "Keyme"
$form.Width = 460
$form.Height = 430
$form.StartPosition = "CenterScreen"
$form.FormBorderStyle = "FixedSingle"
$form.MaximizeBox = $false
$form.BackColor = [Drawing.Color]::FromArgb(18, 22, 25)
$form.ForeColor = [Drawing.Color]::White

$title = New-Object Windows.Forms.Label
$title.Text = "Keyme"
$title.Font = New-Object Drawing.Font("Segoe UI Variable Display", 24, [Drawing.FontStyle]::Bold)
$title.Location = New-Object Drawing.Point(28, 22)
$title.Size = New-Object Drawing.Size(390, 44)
$form.Controls.Add($title)

$subtitle = New-Object Windows.Forms.Label
$subtitle.Text = "Mechanical keyboard sounds for Windows"
$subtitle.Font = New-Object Drawing.Font("Segoe UI", 10)
$subtitle.ForeColor = [Drawing.Color]::FromArgb(174, 186, 194)
$subtitle.Location = New-Object Drawing.Point(31, 67)
$subtitle.Size = New-Object Drawing.Size(390, 24)
$form.Controls.Add($subtitle)

$status = New-Object Windows.Forms.Label
$status.Font = New-Object Drawing.Font("Segoe UI", 10, [Drawing.FontStyle]::Bold)
$status.Location = New-Object Drawing.Point(31, 105)
$status.Size = New-Object Drawing.Size(390, 24)
$form.Controls.Add($status)

$profileLabel = New-Object Windows.Forms.Label
$profileLabel.Text = "Sound profile"
$profileLabel.Location = New-Object Drawing.Point(31, 148)
$profileLabel.Size = New-Object Drawing.Size(160, 22)
$form.Controls.Add($profileLabel)

$profileBox = New-Object Windows.Forms.ComboBox
$profileBox.DropDownStyle = "DropDownList"
$profileBox.Location = New-Object Drawing.Point(200, 145)
$profileBox.Size = New-Object Drawing.Size(205, 24)
[void]$profileBox.Items.AddRange($profiles)
$profileBox.SelectedItem = if ($profiles -contains $config.profile) { $config.profile } else { "holy-panda" }
$form.Controls.Add($profileBox)

$volumeLabel = New-Object Windows.Forms.Label
$volumeLabel.Text = "Volume: $($config.volume)%"
$volumeLabel.Location = New-Object Drawing.Point(31, 196)
$volumeLabel.Size = New-Object Drawing.Size(160, 22)
$form.Controls.Add($volumeLabel)

$volumeSlider = New-Object Windows.Forms.TrackBar
$volumeSlider.Minimum = 0
$volumeSlider.Maximum = 100
$volumeSlider.TickFrequency = 10
$volumeSlider.Value = [Math]::Max(0, [Math]::Min(100, [int]$config.volume))
$volumeSlider.Location = New-Object Drawing.Point(194, 187)
$volumeSlider.Size = New-Object Drawing.Size(220, 45)
$volumeSlider.Add_ValueChanged({ $volumeLabel.Text = "Volume: $($volumeSlider.Value)%" })
$form.Controls.Add($volumeSlider)

$autostart = New-Object Windows.Forms.CheckBox
$autostart.Text = "Start Keyme automatically after login"
$autostart.Checked = [bool]$config.autostart
$autostart.Location = New-Object Drawing.Point(31, 245)
$autostart.Size = New-Object Drawing.Size(360, 26)
$autostart.ForeColor = [Drawing.Color]::White
$form.Controls.Add($autostart)

$saveButton = New-Object Windows.Forms.Button
$saveButton.Text = "Save"
$saveButton.Location = New-Object Drawing.Point(31, 298)
$saveButton.Size = New-Object Drawing.Size(82, 36)
$form.Controls.Add($saveButton)

$startButton = New-Object Windows.Forms.Button
$startButton.Text = "Start / Restart"
$startButton.Location = New-Object Drawing.Point(126, 298)
$startButton.Size = New-Object Drawing.Size(116, 36)
$form.Controls.Add($startButton)

$stopButton = New-Object Windows.Forms.Button
$stopButton.Text = "Stop"
$stopButton.Location = New-Object Drawing.Point(255, 298)
$stopButton.Size = New-Object Drawing.Size(72, 36)
$form.Controls.Add($stopButton)

$openConfigButton = New-Object Windows.Forms.Button
$openConfigButton.Text = "Config"
$openConfigButton.Location = New-Object Drawing.Point(340, 298)
$openConfigButton.Size = New-Object Drawing.Size(72, 36)
$form.Controls.Add($openConfigButton)

$privacy = New-Object Windows.Forms.Label
$privacy.Text = "Privacy: Keyme observes virtual-key codes only. It does not store typed text or use the network."
$privacy.Font = New-Object Drawing.Font("Segoe UI", 8.5)
$privacy.ForeColor = [Drawing.Color]::FromArgb(140, 152, 160)
$privacy.Location = New-Object Drawing.Point(31, 352)
$privacy.Size = New-Object Drawing.Size(390, 34)
$form.Controls.Add($privacy)

function Refresh-Status {
    if (Get-KeymeRunning) {
        $status.Text = "Status: running"
        $status.ForeColor = [Drawing.Color]::FromArgb(80, 220, 150)
    }
    else {
        $status.Text = "Status: stopped"
        $status.ForeColor = [Drawing.Color]::FromArgb(255, 190, 90)
    }
}

function Save-CurrentSettings {
    Write-KeymeConfig -Profile $profileBox.SelectedItem -Volume $volumeSlider.Value -Autostart $autostart.Checked
    Set-KeymeAutostart -Enabled $autostart.Checked
}

$saveButton.Add_Click({
    Save-CurrentSettings
    Refresh-Status
})

$startButton.Add_Click({
    Save-CurrentSettings
    Stop-Keyme
    Start-Sleep -Milliseconds 250
    Start-Keyme
    Start-Sleep -Milliseconds 350
    Refresh-Status
})

$stopButton.Add_Click({
    Stop-Keyme
    Start-Sleep -Milliseconds 250
    Refresh-Status
})

$openConfigButton.Add_Click({
    Ensure-Config
    Start-Process notepad.exe $configPath
})

Refresh-Status
[void]$form.ShowDialog()
