## ADDED Requirements

### Requirement: Lanes are derived from the pieces present

The cascade SHALL show one lane per kit piece that the loaded score actually uses,
derived from the General MIDI percussion numbers its notes resolve to — never a
fixed kit layout. A piece absent from the score SHALL NOT occupy a lane, so a
three-element groove is drawn as three wide lanes rather than as a full kit with
empty columns.

Several General MIDI numbers may denote one physical piece (the acoustic and
electric snares, for instance); numbers that map to the same piece SHALL share one
lane rather than splitting it.

#### Scenario: A sparse groove gets wide lanes

- **WHEN** a score uses only hi-hat, snare and kick
- **THEN** the cascade shows two lanes (hi-hat, snare) at full available width, and
  no lane for any absent piece

#### Scenario: A large kit gets its pieces

- **WHEN** a score uses hi-hat, snare, three toms, ride and crash
- **THEN** each of those pieces has its own lane

#### Scenario: Equivalent numbers share a lane

- **WHEN** a score uses two General MIDI numbers that denote the same physical piece
- **THEN** both are drawn in the same lane

### Requirement: Lane order follows a fixed rule

The lane order SHALL be produced by sorting the pieces present, in this order:

1. the hi-hat — or, when the score has no hi-hat, the ride;
2. the snare;
3. the toms, highest to lowest;
4. the remaining cymbals (the ride when a hi-hat already took position 1, then the
   crashes and other accent cymbals).

The order SHALL be a rule applied to the pieces present, not a fixed list, so that
adding or removing a piece never reorders the others.

The rule protects one invariant: **position 1 is whatever the player strikes
continuously and position 2 is the snare.** Those two carry the overwhelming
majority of a groove's notes, so they must sit adjacent — inside a single eye
fixation — and any further piece must be appended to their right rather than
inserted between them. A player moving from a sparse score to a dense one therefore
never has to relearn where to look.

#### Scenario: The core of the groove is leftmost and adjacent

- **WHEN** any percussion score is loaded
- **THEN** the continuously-struck piece is in position 1 and the snare in position
  2, whatever else the score contains

#### Scenario: The ride takes position 1 when there is no hi-hat

- **WHEN** a score keeps time on the ride and uses no hi-hat
- **THEN** the ride occupies position 1

#### Scenario: The ride joins the cymbals when a hi-hat is present

- **WHEN** a score uses both hi-hat and ride
- **THEN** the hi-hat is in position 1 and the ride is ordered with the cymbals,
  after the toms

#### Scenario: Added pieces do not disturb the core

- **WHEN** two scores are compared, one using three pieces and one using seven
- **THEN** positions 1 and 2 hold the same roles in both, and the extra pieces appear
  to their right

### Requirement: The kick is a full-width bar, not a lane

The kick SHALL be drawn as a bar spanning the full width of the cascade at its
onset, and SHALL NOT occupy a lane.

A lane encodes *where to aim*. The foot does not aim — it is always on the same
pedal — so giving the kick a lane spends horizontal width on information that does
not exist, and forces the eye to compare two horizontally distant positions to
answer the question that actually matters in drumming: does the foot land **with**
a hand stroke or **between** hand strokes? A full-width bar answers it by
intersection, without moving the gaze.

#### Scenario: A kick coinciding with a hand stroke reads as simultaneous

- **WHEN** a kick and a hand stroke share an onset
- **THEN** the bar passes through that note, and both are legible

#### Scenario: A kick between hand strokes reads as offset

- **WHEN** a kick falls between two hand strokes
- **THEN** the bar sits clear of every note

#### Scenario: The kick never consumes lane width

- **WHEN** the lanes are derived
- **THEN** the kick has no lane, and the available width is divided among the
  hand-struck pieces only

### Requirement: Foot bars are drawn beneath the note layer

The foot bars SHALL be painted **before** — therefore beneath — the hand notes, and
SHALL be thinner than a note and attenuated relative to one.

Painted above, a full-width bar covers the hand note precisely on a coincidence,
hiding the very information the bar exists to convey. Attenuation alone does not
fix it: a bar faint enough to reveal a note behind it becomes invisible across the
rest of the width, where it has nothing behind it. The three properties — order,
thickness, attenuation — SHALL hold together.

This defect is only observable on coincidences, so a score in which the foot and
the hands never align does not reveal it; the tests SHALL therefore exercise a
coinciding onset explicitly.

#### Scenario: The hand note stays visible on a coincidence

- **WHEN** a kick and a hand stroke share an onset
- **THEN** the note is drawn over the bar and remains fully legible

#### Scenario: The bar stays visible away from the notes

- **WHEN** a kick falls where no lane has a note
- **THEN** the bar is clearly legible across the full width

### Requirement: The pad strip replaces the on-screen keyboard

For a percussion score the on-screen controller SHALL be a strip of pads, one per
lane, **in the same order as the lanes**, with the kick as a single wide pedal
beneath the pads rather than one pad among them.

Sharing the order is what makes the two surfaces one mapping: a player who learns
where a piece is in the cascade finds it in the same place on the strip, and never
translates between two layouts.

#### Scenario: Pads and lanes share one order

- **WHEN** the cascade lanes are derived for a score
- **THEN** the pad strip presents the same pieces in the same left-to-right order

#### Scenario: The kick sits apart, as a pedal

- **WHEN** the pad strip is drawn
- **THEN** the kick is a wide pedal beneath the pads, not one of them

### Requirement: Open and closed hi-hat are note variants

An open hi-hat SHALL be drawn as a visual variant of the note **within the hi-hat
lane**, distinct from a closed one, and SHALL NOT produce a bar or a lane of its
own.

Open versus closed is not a foot event in the score: it is a different General MIDI
number on the **hand** stroke. Treating it as a foot event would invent a note the
file does not contain.

#### Scenario: Open and closed are distinguishable

- **WHEN** a score alternates open and closed hi-hat strokes
- **THEN** both appear in the hi-hat lane, visually distinct from one another

#### Scenario: An open hi-hat creates no foot event

- **WHEN** an open hi-hat stroke is shown
- **THEN** no bar and no additional lane is drawn for it

### Requirement: The inverted-kit setting mirrors the controller only

The player SHALL be able to enable an **inverted kit** layout, which reverses the
lane order and the pad strip **together**. It SHALL NOT reverse the notation modes:
drum notation is a fixed convention, and a player whose kit is mirrored reads the
same score as anyone else. It SHALL NOT affect how incoming notes are interpreted:
a hi-hat reports the same General MIDI number wherever it physically stands.

The setting SHALL be persisted, SHALL default to the standard layout, and SHALL NOT
be inferred — nothing in a score or a device reveals how a player's kit is set up.

Its label SHALL describe the **kit's layout**, not the player: a left-handed drummer
may well play a standard kit, and a label naming handedness would invite them to
enable a setting that does not apply to them.

#### Scenario: Both controller surfaces mirror together

- **WHEN** the inverted-kit setting is enabled
- **THEN** the lane order and the pad strip are both reversed, and they still match
  each other

#### Scenario: Notation is unaffected

- **WHEN** the setting is enabled and a notation mode is shown
- **THEN** the score is engraved exactly as with the standard layout

#### Scenario: Input is unaffected

- **WHEN** the setting is enabled and a pad is struck
- **THEN** the note is interpreted from its General MIDI number, unchanged

#### Scenario: The setting persists and is never guessed

- **WHEN** a player enables the setting and returns later
- **THEN** it is still enabled, and it was never enabled on their behalf
