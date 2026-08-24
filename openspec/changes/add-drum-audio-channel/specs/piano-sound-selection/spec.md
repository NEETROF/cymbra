## ADDED Requirements

### Requirement: The sound catalog and picker are family-scoped

The app's sound catalog SHALL record each font's instrument family (`keyboard`
or `percussion`, the score vocabulary), sourced from the server listing for
catalog fonts and detected from preset banks for imports, and the sound picker
SHALL offer only the fonts of the **loaded score's** family — a keyboard score
lists keyboard fonts, a percussion score lists kit fonts, whatever the home
instrument context says. The selection SHALL be remembered **separately per
family** (the existing persisted piano choice becomes the keyboard family's
memory; the kit choice is a second persisted selection defaulting to the bundled
kit), so choosing a kit never disturbs the chosen piano and vice versa. The
soundfonts management screen, which is not scoped to a score, SHALL show every
font with its family visible rather than filtering.

#### Scenario: The picker follows the score's family

- **WHEN** the sound picker is opened while a percussion score is loaded
- **THEN** it lists percussion-family fonts with the current kit selection
  marked, and no keyboard font

#### Scenario: A keyboard score under a drums context lists pianos

- **WHEN** the home instrument context is drums and the picker is opened on a
  keyboard score
- **THEN** keyboard-family fonts are listed — the context never decides the
  family

#### Scenario: Each family remembers its own choice

- **WHEN** the user selects a kit for a drum score, then opens a keyboard score
- **THEN** the previously chosen piano is active, and reopening a drum score
  restores the chosen kit

#### Scenario: The kit selection survives a restart

- **WHEN** the user selects a kit and relaunches the app
- **THEN** the next percussion score loads that kit, and an unknown persisted
  kit id falls back to the bundled kit and is re-persisted

## MODIFIED Requirements

### Requirement: Selectable Piano SoundFonts

The app SHALL offer a catalog of two or more piano SoundFonts — including at least
one bundled, cleanly-licensed default available without a network — and SHALL let
the user select which one the synthesizer uses. The catalog MAY include bundled,
download-on-first-use, and user-imported pianos. Selecting a piano SHALL apply at
runtime so that subsequent notes — from the on-screen keyboard, the computer
keyboard, a MIDI device, or score playback — sound with the chosen timbre, without
restarting the app. The picker SHALL list the fonts of the **active family**
("The sound catalog and picker are family-scoped" below): on a keyboard score
that is every piano in the catalog; on a percussion score it is the
percussion-family fonts instead. The catalog SHALL be exposed through an
injectable provider so tests can supply a fake set of pianos.

#### Scenario: Choosing a piano changes the sound
- **WHEN** the user selects a different piano from the catalog
- **THEN** notes played afterward sound with the newly selected piano

#### Scenario: Catalog lists the available pianos
- **WHEN** the picker is shown
- **THEN** it lists every font of the active family — every piano in the catalog
  when a keyboard score is loaded — with the current selection marked

#### Scenario: Selection applies to every input source
- **WHEN** a piano is selected and a note is then triggered from any source
- **THEN** that note sounds with the selected piano's timbre

### Requirement: SoundFont Source And Graceful Fallback

The bytes for a selected font SHALL be obtained through an injectable source:
bundled fonts load from the app's asset bundle, download-on-first-use fonts are
fetched once, cached, and loaded from cache thereafter, and user-imported fonts
load from their copied file in app storage. If a selected font's bytes cannot be
obtained (e.g. a download fails or is offline, or an imported file is missing),
the app SHALL fall back to the bundled default **of the selected font's family**
— the bundled piano for a keyboard font, the bundled kit for a percussion font —
and SHALL NOT crash or interrupt playback. Each family's bundled default SHALL
always be available without a network.

#### Scenario: Bundled piano loads from assets
- **WHEN** a bundled piano is selected
- **THEN** its SoundFont is loaded from the asset bundle and used by the synthesizer

#### Scenario: Failed download falls back to the default
- **WHEN** a download-on-first-use font is selected but its bytes cannot be fetched
- **THEN** the app falls back to that family's bundled default and continues
  without crashing

#### Scenario: Selection persists even when audio is unavailable
- **WHEN** audio is unavailable and the user selects a piano
- **THEN** the choice is still persisted and applies once audio becomes available

### Requirement: User-Imported SoundFonts

The app SHALL let the user import a SoundFont (`.sf2`) from their device so it
becomes a selectable font in the catalog, under the instrument family
**detected from its preset banks** — a font whose presets are all in bank 128 is
recorded as `percussion`, any other loadable font as `keyboard` — never asked of
the user. The app SHALL validate that an imported file is a loadable SoundFont
before accepting it and SHALL reject an invalid file without crashing. An
accepted file SHALL be copied into the app's own storage so the imported font
remains available across launches independent of the original file, and SHALL be
added to a persisted registry of imported fonts that records the detected
family. The user SHALL be able to remove an imported font; removing the
currently selected font SHALL fall back to that family's default (the bundled
piano for keyboard, the bundled kit for percussion). Imported SoundFonts SHALL
remain on the device and SHALL NOT be redistributed by the app.

#### Scenario: Importing a SoundFont adds it to the catalog

- **WHEN** the user imports a valid `.sf2` file
- **THEN** it appears in the catalog as a selectable font of its detected family
  and can be selected to play with that sound

#### Scenario: An imported kit is offered for drum scores

- **WHEN** the user imports a kit-only `.sf2` and then opens a percussion score's
  sound picker
- **THEN** the import is listed among the percussion-family fonts, and it is
  absent from a keyboard score's picker

#### Scenario: Imported font survives a restart

- **WHEN** the user imports a SoundFont and later relaunches the app
- **THEN** the imported font is still listed with its detected family and
  selectable

#### Scenario: Invalid file is rejected gracefully

- **WHEN** the user picks a file that is not a loadable SoundFont
- **THEN** the app rejects it with a non-fatal message and the catalog is unchanged

#### Scenario: Removing an imported font

- **WHEN** the user removes an imported font that is currently selected
- **THEN** it is deleted from the catalog and the app falls back to that
  family's bundled default
