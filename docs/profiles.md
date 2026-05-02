# Profiles

Takt's built-in profiles are procedural approximations. They are designed to create different typing textures without bundling third-party recordings.

| Profile | Character |
| --- | --- |
| `holy-panda` | rounded tactile thock |
| `clean-muted` | short muted tactile |
| `clean-thock` | low clean thock |
| `soft-linear` | light smooth linear |
| `studio-pop` | brighter clean key pop |
| `red` | light linear clack |
| `alps-blue` | crisp click |
| `box-navy` | heavy click |
| `topre` | soft low dome |
| `nk-cream` | smooth creamy pop |
| `buckling-spring` | loud vintage snap |
| `ink-black` | deep muted linear |
| `turquoise-tealios` | clean polished linear |
| `alpaca` | soft pop |
| `typewriter` | sharp retro strike |
| `oil-king` | deep linear thock |
| `mx-black` | classic weighted linear |
| `box-jade` | sharp clickbar-style click |
| `silent-tactile` | quiet muted tactile |
| `ceramic` | clean bright clack |
| `terminal` | retro terminal board |

These profiles are synthesized, not sampled. That keeps the project redistributable and avoids copying sounds from commercial apps. Future recorded packs should live in `sounds/` with clear licensing.

## Instrument Modes

Instrument modes ignore the switch profile.

- `piano` plays generated piano-like notes from changing original pop progressions.
- `guitar` plays generated guitar-style plucks from changing original pop progressions.
- `chords` plays generated original pop chords with bass and upper tones.
- `melody` plays generated original pop-style lead notes with randomized key, scale, and chord tones.

Held keys trigger once until released, so Windows key-repeat does not create sound spam.
