# Keyme

Mechanical keyboard sounds for Windows.

This implementation is intentionally privacy-preserving:

- It installs a Windows low-level keyboard hook.
- It only observes virtual-key codes so it can choose pan/pitch.
- It does not store text, reconstruct words, transmit data, or use the network.

## Current Features

- Global keyboard sounds across apps.
- Low-latency audio playback through the default Windows audio output.
- Per-key stereo pan based on physical keyboard position.
- Small pitch variation per key so repeated typing is less robotic.
- Synthesized switch profiles: `red`, `holy-panda`, `alps-blue`, `box-navy`, `topre`.

## Run

```powershell
cargo run --release -- --profile holy-panda --volume 75
```

Press `Ctrl+C` in the terminal to quit.

## Easy Install

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\install.ps1
```

This creates:

- `Start Keeby Windows` on the Desktop.
- `Stop Keeby Windows` on the Desktop.
- `Keeby Windows` in the Windows Startup folder so it starts after login.
- Start Menu shortcuts under `Keeby Windows`.

To remove the shortcuts and stop the app:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\uninstall.ps1
```

## Build

```powershell
cargo build --release
```

The binary will be at:

```text
target\release\keeby_windows.exe
```

## What Comes Next

To reach Keeby-level polish on Windows, the next milestones are:

- Add licensed recorded switch sample packs instead of runtime synthesis.
- Add a tray app with toggle, volume, and profile selection.
- Persist settings in `%APPDATA%`.
- Add optional mouse click and wheel sounds.
- Add a visualizer overlay.
- Package as an installer with startup-at-login support.
