# Privacy

Takt is intentionally local-only.

## What Takt Reads

Takt reads Windows virtual-key codes such as `A`, `Space`, or `Enter`. These codes are used to choose sound position and pitch variation.

## What Takt Does Not Do

- It does not store key presses.
- It does not reconstruct typed words.
- It does not read passwords.
- It does not read clipboard data.
- It does not send network requests.
- It does not include analytics or telemetry.

## Why a Keyboard Hook Is Needed

Windows requires a global keyboard hook for an app to react to keystrokes outside its own window. Takt uses that hook only to trigger sound playback.
