# Takt

Mechanical keyboard sounds for Windows.

Takt is a small Windows utility that adds satisfying switch-style audio to every key press. It runs locally, starts with Windows if you want it to, and includes an Electron settings app for choosing sounds and volume.

## Features

- Global keyboard sounds across apps.
- Low-latency audio playback through your default Windows output.
- Stereo panning based on approximate key position.
- Per-key pitch variation so typing feels less repetitive.
- 17 built-in synthesized switch profiles.
- Multiple sound modes: clean key sounds plus generated piano, guitar, pop chords, and pop lead notes.
- Held keys play once until released, so holding a key does not spam sounds.
- Electron settings app. No PowerShell UI.
- Local-first privacy model with no telemetry and no network calls.

## Sound Profiles

Takt includes 17 procedural profiles from soft thock to loud vintage click. See [docs/profiles.md](docs/profiles.md).

## Install

Download the release ZIP, extract it, then double-click:

```text
Setup-Takt.cmd
```

For development installs, you can also run:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\install.ps1
```

The installer copies Takt to `%LOCALAPPDATA%\Takt` and creates:

- `Takt` desktop shortcut for the native settings app.
- Start Menu shortcuts under `Takt`.
- A Startup shortcut so Takt runs after login.

See [docs/windows-install.md](docs/windows-install.md) for details.

## Use

Open the `Takt` desktop shortcut to change the profile, volume, or autostart setting.

You can also run it directly:

```powershell
cargo run --release --bin takt -- --profile holy-panda --volume 75
```

## Uninstall

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\scripts\uninstall.ps1
```

This stops Takt and removes the installed app. It does not delete a cloned source folder.

## Privacy

Takt installs a Windows low-level keyboard hook so it can react to key presses. It only observes virtual-key codes, does not reconstruct typed text, does not store keystrokes, and does not use the network.

See [docs/privacy.md](docs/privacy.md) and [docs/architecture.md](docs/architecture.md) for more detail.

## Development

```powershell
cargo build --release
cargo run --release --bin takt -- --help
```

The sound engine is written in Rust. The settings app is Electron. Installer scripts are used only for copying files and creating Windows shortcuts.

## Roadmap

- Optional tray icon.
- Licensed recorded WAV sound packs.
- Import your own switch samples.
- Mouse click and scroll sounds.
- Visual keyboard overlay.
- Signed installer.

## License

MIT
