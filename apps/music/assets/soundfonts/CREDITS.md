# SoundFont credits

The app renders piano and **drum** sound from **SoundFonts** (`.sf2`) via the
Rust audio synthesizer. One CC0 piano and one MIT drum kit are **bundled**
(below); two CC-BY grands are offered as **download-on-first-use** (change
`piano-sound-selection`). Users may also import their own `.sf2`, which stays on
the device and is never redistributed.

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

### Drum kit (change `add-drum-audio-channel`)

The percussion channel needs a **bank-128** font: General MIDI resolves drum
kits there, which is what makes a kick a kick rather than a piano note. The
bundled kit is the **"Standard" kit (preset 0) extracted from FluidR3 GM's bank
128** — the upstream font is ~142 MiB of full General MIDI, so shipping it whole
to play drums would be indefensible; the extraction keeps only the presets,
instruments and samples that one kit reaches (10.6 MiB, 105 samples).

The MIT license explicitly permits modification and redistribution; its notice is
preserved verbatim beside the font as `FluidR3Drums-LICENSE-MIT.txt` (Debian's
vetted `copyright` file for `fluid-soundfont-gm`, which is also where the font
was obtained — upstream musescore.org no longer hosts it).

| File | Instrument | Source | Author | License |
|------|------------|--------|--------|---------|
| `FluidR3Drums-bank128.sf2` | GM "Standard" drum kit (FluidR3 GM, bank 128 preset 0) | [Debian `fluid-soundfont-gm` 3.1-6](https://packages.debian.org/source/stable/fluid-soundfont) | Frank Wen (2000–2002, 2008), Toby Smithe (2008) | [MIT](https://opensource.org/licenses/MIT) |

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
