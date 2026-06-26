# Bundled SoundFont credits

The app ships a single piano **SoundFont** (`.sf2`) used by the Rust audio
synthesizer to render a real piano sound. It is in the **public domain** (CC0),
so it carries no attribution requirement and no runtime dependency on its
source — the file is **vendored** (copied) into this repository.

| File | Instrument | Source | Recorded by | License |
|------|------------|--------|-------------|---------|
| `UprightPianoKW-20220221.sf2` | Kawai upright piano (version 2022-02-21) | [FreePats — Upright Piano KW](https://freepats.zenvoid.org/Piano/acoustic-grand-piano.html#UprightKW) | Gonzalo & Roberto (Jan 2017) | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) |

The full CC0 dedication and the upstream readme are preserved alongside the
font as `UprightPianoKW-LICENSE-CC0.txt` and `UprightPianoKW-README.txt`.

Multiple-piano selection/management (importing your own `.sf2`, switching
instruments) is a separate change (`piano-sound-selection`).
