# score-conversion Specification

## Purpose
TBD - created by archiving change add-score-crawler. Update Purpose after archive.
## Requirements
### Requirement: Native MusicXML validation

The system SHALL validate that a file already in MusicXML (`.xml`/`.musicxml`)
is well-formed and genuinely MusicXML (not arbitrary XML) before accepting it,
using the shared parser/validator, then compress it to `.mxl`.

#### Scenario: Valid MusicXML accepted and compressed
- **WHEN** an input `.musicxml` parses as valid MusicXML
- **THEN** it is compressed to a spec-compliant `.mxl` and marked converted

#### Scenario: Non-MusicXML XML rejected
- **WHEN** an input `.xml` is well-formed XML but is not MusicXML
- **THEN** it is not written to the corpus and is recorded as a conversion
  failure

### Requirement: External converter invocation

The system SHALL convert non-MusicXML score formats by invoking external
binaries as subprocesses with exit-code checking and timeouts: MuseScore
(`.mscx`/`.mscz`) via the MuseScore CLI producing `.mxl` directly (headless,
e.g. `QT_QPA_PLATFORM=offscreen` / `xvfb-run`); MEI via Verovio
(`-t musicxml`); LilyPond (`.ly`) via `python-ly` (`ly musicxml`). MIDI SHALL
never be used as a score source.

#### Scenario: MuseScore file converted headless
- **WHEN** a `.mscx`/`.mscz` input is processed and the MuseScore CLI is
  available
- **THEN** the CLI is invoked headless to emit `.mxl`, and a non-zero exit or
  timeout is recorded as a conversion failure rather than propagated as a panic

#### Scenario: MEI converted via Verovio
- **WHEN** an MEI input is processed and Verovio is available
- **THEN** Verovio converts it to MusicXML which is then compressed to `.mxl`

#### Scenario: MIDI never converted
- **WHEN** the only available payload for an item is MIDI
- **THEN** the item is not converted to MusicXML and is recorded as skipped

### Requirement: LilyPond degraded fallback

The system SHALL, when LilyPond-to-MusicXML conversion fails or is unavailable,
keep the original `.ly` (and any produced PDF) and mark the item's conversion
status `failed_kept_source` rather than discarding the score or emitting dubious
MusicXML.

#### Scenario: Failed LilyPond conversion keeps source
- **WHEN** `ly musicxml` fails or is not installed for a `.ly` item
- **THEN** the original `.ly` (plus PDF if produced) is retained and the manifest
  records conversion status `failed_kept_source`

### Requirement: Spec-compliant .mxl container

The system SHALL, when compressing raw `.musicxml` itself, produce a `.mxl` that
follows the MusicXML container spec — a ZIP containing `META-INF/container.xml`
that points to the internal `.musicxml` — not an arbitrary ZIP. Native `.mxl`
output from MuseScore/Verovio SHALL be preferred when available.

#### Scenario: Container structure written correctly
- **WHEN** the tool compresses a raw `.musicxml` into `.mxl`
- **THEN** the archive contains a valid `META-INF/container.xml` referencing the
  internal score file

### Requirement: Output verification

The system SHALL verify each produced `.mxl` by re-opening it and confirming it
parses as MusicXML; a `.mxl` that fails re-parsing SHALL NOT be counted as a
successful conversion.

#### Scenario: Produced .mxl re-parses
- **WHEN** an `.mxl` is generated
- **THEN** the tool re-opens and parses it, and only marks the item converted if
  parsing succeeds

