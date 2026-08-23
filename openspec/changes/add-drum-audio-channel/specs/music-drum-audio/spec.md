## ADDED Requirements

### Requirement: Percussion sounds on the drum channel in every synthesizer site

The system SHALL sound percussion notes on the reserved drum channel — MIDI
channel 10, index 9 as rustysynth counts channels, where preset lookup resolves
in the SoundFont's bank 128 — in **every** synthesizer site: the app engine, the
console's wasm renderer, and the backend's headless preview renderer. Keyboard
notes SHALL keep sounding on the melodic channel 0, byte-for-byte as today. The
two channel numbers SHALL be defined **once**, in the shared MusicXML crate
beside its existing playback constants, and consumed from there — never
re-declared per site: the wasm renderer's "matches the app" comment copy is
retired by this rule, and the app engine's local `DEFAULT_VELOCITY` copy is
unified onto the crate's constant, so no synth site keeps a local playback
constant beside the shared ones.

Which channel a note takes SHALL be decided by the score's instrument
classification, never by the caller's context or a client-supplied claim: the
document-holding renderers (wasm, backend) key the whole render on the parsed
document's classification, and the event-driven app engine exposes distinct
percussion entry points that the player routes to by the loaded score's family.
The playback schedule already emits unpitched notes only for
percussion-classified scores, so a mixed score cannot reach the drum channel
through any renderer.

The metronome click and the WAV preview-clip player are synthesized outside the
SoundFont and SHALL be unaffected by the channel routing and by which family's
font is loaded.

#### Scenario: A percussion score sounds kit voices

- **WHEN** a percussion-classified score is played or rendered in any
  synthesizer site with a percussion-family font loaded
- **THEN** its General MIDI numbers sound on the drum channel as drum-kit
  voices, and the render is not silent

#### Scenario: A keyboard score is unaffected

- **WHEN** a keyboard score is played or rendered in any synthesizer site
- **THEN** every note sounds on the melodic channel exactly as before this
  change

#### Scenario: The channel constants have one definition

- **WHEN** any synthesizer site refers to the melodic or drum channel
- **THEN** it consumes the shared crate's constant rather than a local copy

#### Scenario: A release lands on the channel where the note sounded

- **WHEN** a percussion note is released while keyboard-channel voices are also
  tracked
- **THEN** the release is issued on the drum channel and no melodic voice is
  disturbed

#### Scenario: The metronome ignores the kit

- **WHEN** the metronome clicks while a percussion score and its kit font are
  loaded
- **THEN** the click sounds exactly as it does for a keyboard score

### Requirement: The active SoundFont follows the loaded score's family

The app SHALL key the synthesizer's loaded SoundFont on the **loaded score's**
instrument family: opening a percussion score SHALL swap to the remembered
percussion-family font (the bundled kit by default), and returning to a keyboard
score or to free play SHALL restore the remembered keyboard font. The home
instrument context SHALL never participate in this decision — the score carries
its own instrument (`music-instrument-context`), so a keyboard score opened
under a drums context loads a keyboard font.

Because the font swap parses off-thread and keeps the outgoing font until the
incoming one is ready, the player SHALL NOT sound a percussion score's notes
before the kit font is installed: the swap SHALL expose a completion the player
waits on, so a drum part is never sounded through a keyboard font. A
percussion-family font that cannot be resolved or loaded SHALL fall back to the
bundled kit; if even the bundled kit cannot be loaded, playback degrades to
silence without crashing, exactly as the keyboard path degrades today.

#### Scenario: Opening a drum score loads the kit

- **WHEN** a percussion score is opened while a piano font is active
- **THEN** the remembered kit font is swapped in before any of the score's notes
  sound

#### Scenario: Returning to keyboard restores the piano

- **WHEN** the user leaves a percussion score and opens a keyboard score
- **THEN** the remembered keyboard font is active for its playback

#### Scenario: The context does not choose the font

- **WHEN** the home instrument context is drums and a keyboard score is opened
- **THEN** a keyboard-family font is loaded, not a kit

#### Scenario: A missing chosen kit falls back to the bundled kit

- **WHEN** a percussion score is opened and the remembered kit font cannot be
  resolved or loaded
- **THEN** the bundled kit is loaded instead and playback proceeds

#### Scenario: No note sounds through the wrong family

- **WHEN** playback of a percussion score is started while the kit swap is still
  in flight
- **THEN** no note sounds until the kit is installed, rather than the first
  notes sounding through the outgoing keyboard font

### Requirement: A drum kit SoundFont ships bundled with the app

The app SHALL bundle a drum-kit SoundFont beside the bundled piano, subject to
the same settled constraints: `.sf2` only (the synthesizer rejects compressed
SF3), and a licence a human has verified as permitting redistribution **before**
the bytes enter the repository, recorded in the assets' credits file with the
licence text vendored alongside. The bundled kit SHALL be available offline with
no account, and SHALL be the default and the fallback for the percussion family.
The same kit SHALL be seeded into the server SoundFont catalog (family
`percussion`, accepted) under a stable id, so the console's audition and the
backend's preview renders have a kit that provably exists — mirroring how the
bundled piano is also served.

#### Scenario: The drum path works out of the box

- **WHEN** a fresh install opens a percussion score with no network and no
  account
- **THEN** the bundled kit is loaded and the score sounds

#### Scenario: The kit's licence is recorded

- **WHEN** the bundled kit asset is present in the repository
- **THEN** the credits file names its source, author and licence, and the
  licence text ships beside the font

#### Scenario: The server catalog offers the same kit

- **WHEN** the console or the preview job needs a percussion-family font
- **THEN** the seeded kit is present in the catalog as an accepted
  `percussion`-family font

### Requirement: A one-shot percussion entry point exists for input mapping

The engine SHALL expose, through the injectable audio seam, an entry point that
sounds a single General MIDI percussion number on the drum channel now
(`drum_on`, with its paired `drum_off`), independent of any loaded score's
schedule. Wiring pad taps and MIDI drum-pad input to it is
`add-drum-input-mapping`'s, which SHALL NOT need to modify the engine to do so.

#### Scenario: The verb is callable without a schedule

- **WHEN** the seam's percussion entry point is invoked with a General MIDI
  number outside any playback
- **THEN** that kit voice sounds on the drum channel

### Requirement: A font's declared family is verified against its preset banks

The backend SHALL verify a SoundFont's declared instrument family against the
file's actual preset banks on **every** write path — the admin catalog upload,
the private-library import sync, and the proposal of a private font to the
catalog. The check is asymmetric, because a font may legitimately hold both
banks: declaring `percussion` SHALL require at least one bank-128 preset;
declaring `keyboard` SHALL require at least one melodic-bank preset; a font
holding both passes either declaration, which then decides its single recorded
family. A mismatch SHALL be refused with a typed, localisable reason — never
recorded on trust, and never silently corrected, since a both-banks font has no
bank-derived answer to correct to.

The app's import flow SHALL **detect** the family from the file's preset banks
(only bank-128 presets → `percussion`; otherwise `keyboard`) rather than ask the
user, and the server SHALL re-verify the synced declaration — the client is
never trusted. Catalog rows recorded before verification existed SHALL be
checked by a one-shot ops pass that re-reads each stored object and reports
mismatches rather than rewriting them.

#### Scenario: A kit-less font cannot claim percussion

- **WHEN** a font with no bank-128 preset is uploaded, imported or proposed with
  the family `percussion`
- **THEN** it is refused with a typed reason, and nothing is stored or recorded

#### Scenario: A kit-only font cannot claim keyboard

- **WHEN** a font whose presets are all in bank 128 is submitted with the family
  `keyboard`
- **THEN** it is refused with a typed reason

#### Scenario: A both-banks font passes either declaration

- **WHEN** a font holding melodic and bank-128 presets is submitted with either
  family
- **THEN** it is accepted and recorded with the declared family

#### Scenario: An import's family is detected, then re-verified

- **WHEN** a user imports a kit-only `.sf2` and it syncs to their private
  library
- **THEN** the app records it as `percussion` without asking, and the server's
  verification accepts it against its banks
