# Architecture

Takt has two layers:

- `src/main.rs`: the Rust background engine that listens for global key presses and plays audio.
- `electron/`: the Electron settings app that edits user configuration and starts or stops the engine.

## Keyboard Input

The engine uses a Windows low-level keyboard hook. It reads virtual-key codes only and forwards them to the audio thread. It does not record text or inspect application content.

## Audio

The audio engine uses `rodio` and generates short stereo switch sounds at runtime. Each key press creates a small source with:

- switch profile parameters
- approximate stereo pan
- deterministic pitch variation
- short decaying body, click, and noise components

## Settings

User settings live at:

```text
%APPDATA%\Takt\config.json
```

The settings app writes this file. The launcher script reads it before starting the engine.
