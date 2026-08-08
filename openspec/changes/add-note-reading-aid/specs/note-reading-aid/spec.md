## ADDED Requirements

### Requirement: Opt-In Reading Aid With Three Levels

The player SHALL offer a note reading aid with exactly three levels: **off**,
**note name**, and **note name and rhythm**. The aid SHALL default to *off* so
that a player who has not asked for it is never given it. At the *note name*
level the aid SHALL name the awaited note(s); at the *note name and rhythm* level
it SHALL additionally show the rhythmic figure. The level SHALL be a play setting
like the others (hands, tempo, metronome, device) rather than a per-score choice.

#### Scenario: Aid is absent by default

- **WHEN** a player who has never changed the setting opens a score and Wait Mode
  blocks at an onset
- **THEN** no reading aid is displayed

#### Scenario: Note-name level names the note without the figure

- **WHEN** the aid is set to *note name* and Wait Mode blocks at an onset
- **THEN** the awaited note's name is displayed and no rhythmic figure is shown

#### Scenario: Rhythm level adds the figure

- **WHEN** the aid is set to *note name and rhythm* and Wait Mode blocks at an
  onset
- **THEN** both the awaited note's name and its rhythmic figure are displayed

### Requirement: Aid Is Shown Only While Wait Mode Blocks

The aid SHALL be displayed only while Wait Mode is actively blocking at an onset,
and SHALL be withdrawn as soon as the gate is satisfied and the playhead resumes.
It SHALL NOT be displayed while the playhead is travelling between onsets, nor
when Wait Mode is disabled, so that a large readable panel can never compete with
a moving score. When the aid level is *off*, nothing SHALL be displayed in any
state.

#### Scenario: Shown when the gate blocks

- **WHEN** Wait Mode is enabled, the aid is enabled, and the playhead has stopped
  at an onset awaiting input
- **THEN** the aid is displayed for the notes that onset requires

#### Scenario: Withdrawn when playback resumes

- **WHEN** the awaited onset is satisfied and the playhead resumes
- **THEN** the aid is withdrawn

#### Scenario: Not shown outside Wait Mode

- **WHEN** the aid is enabled but Wait Mode is disabled
- **THEN** no aid is displayed at any point during playback

### Requirement: The Aid Costs The Score No Layout Space

The aid SHALL take no layout space from the score at any level. Note names SHALL
be drawn on the awaited keys of the on-screen keyboard — the surface the player
is already looking at, and the very key the finger is going to — rather than in a
panel of their own. The rhythmic figure SHALL be presented as a transient overlay
over the score while the gate holds, not as a reserved band. Enabling the aid,
and the gate blocking or releasing, SHALL leave the score render area and the
keyboard at exactly the size and position they have with the aid off.

#### Scenario: Enabling the aid takes nothing from the score

- **WHEN** the reading aid is switched from *off* to either enabled level
- **THEN** the score render area and the keyboard keep the size and position they
  had with the aid off

#### Scenario: Render area does not shift when the aid appears

- **WHEN** the aid is enabled and the gate blocks, then resumes
- **THEN** the score render area keeps the same size and position throughout

### Requirement: A Key Label Never Overflows Its Key

A note name drawn on a key SHALL be rendered strictly inside that key's bounds.
It SHALL NOT be allowed to spill over the neighbouring keys, since a label
crossing a key boundary points at the wrong note. When the name does not fit
across a narrow key, the system SHALL use the key's other dimension — turning the
label a quarter turn so its length runs along the key's height — rather than
overflowing or shrinking the text below legibility. When no legible placement
exists inside the key, the label SHALL be omitted; the key's existing highlight
still identifies it.

#### Scenario: A wide key reads across

- **WHEN** the keyboard range is narrow enough that keys are wide
- **THEN** the label is drawn upright, inside the key

#### Scenario: A narrow key turns the label rather than overflowing

- **WHEN** the label is wider than the key it belongs to (a full 88-key range on
  a phone)
- **THEN** the label is turned a quarter turn and drawn within the key's bounds,
  and never crosses into the adjacent keys

#### Scenario: An impossible fit drops the label, not the cue

- **WHEN** no legible label placement fits inside the key
- **THEN** no label is drawn for that key, and the key's expected-note highlight
  is unaffected

### Requirement: One Naming Convention Across The Keyboard

The keyboard's octave anchors SHALL use the same naming convention as the reading
aid — those anchors being the marker on each C that lets a player orient their
hands — so the keyboard cannot label a key in one convention while the aid names
it in another.
These anchors SHALL keep their octave index — identifying *which* C is their
entire purpose — even though the reading aid's names carry none. An anchor SHALL
yield to a reading-aid name on the same key rather than both being drawn.

#### Scenario: Anchors follow the language

- **WHEN** the app language is French
- **THEN** the octave anchors read `Do3`, `Do4`… rather than `C3`, `C4`

#### Scenario: The aid supersedes the anchor on a shared key

- **WHEN** an awaited note falls on a key that carries an octave anchor
- **THEN** that key shows the reading-aid name and not the anchor

### Requirement: Names Follow The Written Spelling And Effective Alteration

A note SHALL be named from its **written** staff degree and its **effective**
alteration — the alteration actually in force, whether it comes from the key
signature, from an accidental engraved on the note, or from an accidental earlier
in the measure. The name SHALL NOT be derived from the sounding pitch alone, and
SHALL NOT omit an alteration merely because the score engraves no accidental
symbol on that note. Enharmonic spellings SHALL be preserved: a note written as
D♭ SHALL be named D♭ and never C♯.

#### Scenario: Key-signature alteration is named

- **WHEN** the awaited note is written on the F degree under a key signature of
  one sharp, and the score engraves no accidental on it
- **THEN** the aid names it F♯ (`Fa♯` in solfège locales), not F

#### Scenario: Written enharmonic spelling is preserved

- **WHEN** the awaited note is written as D♭
- **THEN** the aid names it D♭ (`Ré♭`), not C♯ (`Do♯`)

#### Scenario: Natural cancelling the key signature is named

- **WHEN** the awaited note carries a natural that cancels the key signature's
  alteration
- **THEN** the aid names the natural degree rather than the key-signature
  alteration

### Requirement: Fallback Naming Without Written Spelling

The aid SHALL still name a note that carries no written spelling — as with the
built-in demo score and any MIDI-only source — deriving the name from the sounding
pitch and choosing sharp or flat spelling according to the key signature in force
at that point in the piece (sharps for sharp keys, flats for flat keys). The aid
SHALL never be blank or show a placeholder for a note it is displaying.

#### Scenario: MIDI-only note is named from the key signature

- **WHEN** the awaited note has no written spelling and the key signature in force
  is a flat key
- **THEN** the note is named with flat spelling rather than sharp spelling

#### Scenario: No spelling never yields an empty label

- **WHEN** the aid displays a note that has no written spelling
- **THEN** a note name is shown rather than an empty or placeholder label

### Requirement: Localized Naming Convention

Note names SHALL follow the convention of the app's active language: letter names
(C, D, E, F, G, A, B) in English, and solfège names (Do, Ré/Re, Mi, Fa, Sol, La,
Si) in the other supported languages, using `Ré` in French and `Re` in Spanish and
Italian. Alterations SHALL be rendered with the musical symbols (♯, ♭, ♮, and the
double alterations) appended to the degree name. Exactly one naming
implementation SHALL exist in the app, shared by the reading aid and by any other
surface that names a pitch or a key signature.

#### Scenario: English uses letter names

- **WHEN** the app language is English and the awaited note is the third degree of
  C major
- **THEN** the aid names it E

#### Scenario: French uses solfège with Ré

- **WHEN** the app language is French and the awaited note is the second degree of
  C major
- **THEN** the aid names it Ré

#### Scenario: Spanish and Italian use Re

- **WHEN** the app language is Spanish or Italian and the awaited note is the
  second degree of C major
- **THEN** the aid names it Re

#### Scenario: One shared naming implementation

- **WHEN** a pitch or key signature is named anywhere in the app
- **THEN** it is named through the shared naming module rather than a
  surface-local copy

### Requirement: No Octave Index In Note Names

The aid SHALL NOT include an octave number in the note name. It names the pitch
class with its alteration only, because the on-screen keyboard already identifies
which key is awaited and an octave index would add a second notation for the
learner to decode.

#### Scenario: Name carries no octave number

- **WHEN** the aid names any awaited note
- **THEN** the displayed name contains the degree and its alteration only, with no
  octave index

### Requirement: Rhythmic Figure Is Named And Quantified

At the *note name and rhythm* level the aid SHALL show, for the awaited note, the
name of its rhythmic figure including any augmentation dots, the corresponding
notation glyph, and the duration expressed in beats of the piece's current time
signature — so the player learns the vocabulary and simultaneously knows how long
to hold. When a note carries no notated figure, the figure SHALL be inferred from
its duration.

#### Scenario: Dotted figure is named and quantified

- **WHEN** the aid shows a dotted half note in a 4/4 piece
- **THEN** the figure is named as a dotted half note, its glyph is shown, and the
  duration is expressed as three beats

#### Scenario: Figure inferred when not notated

- **WHEN** the awaited note carries no notated figure (as in the demo score)
- **THEN** a figure is inferred from the note's duration and displayed

### Requirement: Every Awaited Note Is Named On Its Own Key

Each note of an awaited onset SHALL be named on the key it belongs to, however
many there are — because each name lives on its own key there is no shared line
to overflow and no need to truncate a chord. When every awaited note shares the
same rhythmic figure, that figure SHALL be shown once for the chord; when the
awaited notes carry different figures, the aid SHALL show no figure rather than a
misleading single one.

#### Scenario: Every note of a chord is named

- **WHEN** the awaited onset requires three pitches
- **THEN** each of the three keys carries its own name

#### Scenario: A large chord is not truncated

- **WHEN** the awaited onset requires more pitches than would fit on a single
  shared line
- **THEN** every awaited key is still named, none is dropped for lack of room

#### Scenario: Shared figure is shown once

- **WHEN** every note of the awaited onset carries the same rhythmic figure and
  the rhythm level is enabled
- **THEN** that figure is displayed once for the whole onset

#### Scenario: Mixed figures degrade

- **WHEN** the awaited onset's notes carry different rhythmic figures
- **THEN** the aid does not present a single figure as if it applied to all of them

### Requirement: Aid Follows The Selected Hands

The aid SHALL name only notes that the player is actually being asked to play:
when a hand is deselected, notes belonging to that hand SHALL NOT be named, in the
same way they are neither awaited by the gate nor highlighted on the keyboard.

#### Scenario: Muted hand is not named

- **WHEN** the player has selected the right hand only and an onset carries notes
  on both staves
- **THEN** the aid names only the right-hand notes
