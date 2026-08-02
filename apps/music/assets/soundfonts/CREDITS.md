# SoundFont credits

The app renders piano sound from **SoundFonts** (`.sf2`) via the Rust audio
synthesizer. One CC0 piano is **bundled** (below); two CC-BY grands are offered
as **download-on-first-use** (change `piano-sound-selection`). Users may also
import their own `.sf2`, which stays on the device and is never redistributed.

Each font is **vendored / self-hosted** — copied once under our control; the
original source is provenance, not a runtime dependency. The licenses (CC0 /
CC-BY) explicitly permit redistribution.

## Bundled (default)

Public domain (CC0) — no attribution requirement, always present, no network.

| File | Instrument | Source | Recorded by | License |
|------|------------|--------|-------------|---------|
| `UprightPianoKW-20220221.sf2` | Kawai upright piano (version 2022-02-21) | [FreePats — Upright Piano KW](https://freepats.zenvoid.org/Piano/acoustic-grand-piano.html#UprightKW) | Gonzalo & Roberto (Jan 2017) | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) |

The full CC0 dedication and the upstream readme are preserved alongside the
font as `UprightPianoKW-LICENSE-CC0.txt` and `UprightPianoKW-README.txt`.

## Download-on-first-use (CC-BY — attribution required)

Served by the backend delivery route `GET /soundfonts/{id}` (change
`add-soundfont-delivery`) from the private `cymbra-soundfonts` bucket. CC-BY
requires **visible attribution**, surfaced in the app's piano picker (each font
shows its license and credit) and recorded here.

| Delivery id | Instrument | Source | Attribution (credit) | License |
|-------------|------------|--------|----------------------|---------|
| `ydp-grand` | Yamaha DP Grand piano | [FreePats — YDP Grand Piano](https://freepats.zenvoid.org/Piano/acoustic-grand-piano.html) | Roberto / Zenph Studios | [CC-BY 3.0](https://creativecommons.org/licenses/by/3.0/) |
| `salamander-grand` | Yamaha C5 Grand piano | [FreePats — Salamander Grand Piano](https://freepats.zenvoid.org/Piano/acoustic-grand-piano.html) | Alexander Holm | [CC-BY 3.0](https://creativecommons.org/licenses/by/3.0/) |

The bundled `upright-piano-kw` is also exposed through the same delivery route
(id `upright-piano-kw`) for the back-office preview; in the app it loads from the
bundle, never the network.
