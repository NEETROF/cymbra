## RENAMED Requirements

- FROM: `### Requirement: On-Screen Key Input`
  TO: `### Requirement: On-Screen Controller Input`
- FROM: `### Requirement: Three-State Key Feedback`
  TO: `### Requirement: Key And Kit Feedback States`

## MODIFIED Requirements

### Requirement: On-Screen Controller Input

For a **keyboard** score the on-screen keyboard SHALL be playable by mouse and
touch at all times — **while playback is running and while it is stopped** —
in every render mode. A pointer-down on a key SHALL produce a note-on for
that key's pitch and the matching pointer-up (or pointer-cancel) SHALL
produce a note-off. On-screen play SHALL be handled identically to a physical
MIDI key — internally it reuses the player's note-on/note-off entry points —
so it drives the pressed-key feedback and the Wait Mode gate the same way,
including while Wait Mode is blocking.

For a **percussion** score the drawn kit SHALL be playable the same way, at
all the same times: a pointer-down on a drawn piece SHALL produce a note-on for
one of the General MIDI numbers its lane collapses, and the matching pointer-up
(or pointer-cancel) the corresponding note-off, through the same entry points
(see `music-drum-input`) — so striking a drum drives feedback, and, once
`add-drum-scoring` arms them, the gate and the scorer, exactly like a key
press. This lifts the display-only interim `add-drum-kit-view` stated here:
the kit now emits, and the former "a tap on a drum does nothing yet" scenario
is deliberately replaced by the input scenarios below.

The number a piece emits SHALL be deterministic: each named piece's members
carry a canonical order pinned in the kit-piece table — for the hi-hat the
closed 42 before the open 46; for the snare the acoustic 38, then the
electric 40, then the side stick 37; for the kick Bass Drum 1 (36) before
Acoustic Bass Drum (35) — and a tap SHALL emit the **first member the loaded
score actually uses**; a generic piece emits its single number. Emitting
inside the score's own vocabulary sounds the stroke with the score's own
piece and spares the future matcher avoidable same-piece mismatches.

The **bass drum** SHALL be playable like any piece and SHALL emit the kick by
the same rule: General MIDI **36** — Bass Drum 1, the number virtually every
notation export and e-kit default map uses for the kick — or 35 when the
score writes its kick only as 35.

The kit SHALL offer **no gesture that distinguishes an open from a closed
hi-hat stroke**: it draws one hi-hat, emitting its lane's first present
member — the closed 42 whenever the score uses it; only a score whose hi-hat
lane holds solely the open 46 emits 46. Open versus closed is
pedal-controlled on the instrument, not a second aim point — a split cymbal
would teach a kit that does not exist and shrink both touch targets, and a
modifier gesture is unplayable at tempo. An external kit produces the open
stroke (46) naturally, through the same entry points.

#### Scenario: Tap plays a note
- **WHEN** the user presses a displayed key with mouse or finger
- **THEN** a note-on for that key's pitch is emitted and the key shows the pressed
  state

#### Scenario: Release stops the note
- **WHEN** the user lifts the pointer that pressed a key
- **THEN** a note-off for that pitch is emitted and the key leaves the pressed
  state

#### Scenario: On-screen play satisfies the gate
- **WHEN** Wait Mode awaits a note and the user presses that note on the on-screen
  keyboard
- **THEN** the gate is satisfied exactly as if the note arrived from MIDI

#### Scenario: Playable in every mode
- **WHEN** the player is in Synthesia, Staff, or Partition mode
- **THEN** the on-screen keys respond to pointer input

#### Scenario: Playable while stopped
- **WHEN** playback is stopped (the playhead is not advancing)
- **THEN** pressing on-screen keys still emits note-on/note-off and shows the
  pressed state, so the user can play freely

#### Scenario: Striking a drawn drum emits its lane's stroke
- **WHEN** a percussion score using the acoustic snare (38) is shown and the
  user taps the drawn snare
- **THEN** a note-on for 38 is emitted through the player's entry points, the
  stroke sounds, and the snare flashes

#### Scenario: The emitted number stays inside the score's vocabulary
- **WHEN** the loaded score writes its snare only as the electric 40 and the
  user taps the drawn snare
- **THEN** the emitted number is 40, not the absent canonical 38

#### Scenario: The bass drum emits the kick
- **WHEN** the user taps the drawn bass drum
- **THEN** a note-on for General MIDI 36 is emitted — or 35 when the score
  writes its kick only as 35 — and it flashes

#### Scenario: No gesture distinguishes open from closed
- **WHEN** a score alternates closed (42) and open (46) hi-hat and the user
  taps the drawn hi-hat
- **THEN** the emitted number is the closed 42 — the lane's first present
  member — and no on-screen gesture selects the open 46 over it

#### Scenario: The kit is playable while stopped
- **WHEN** playback is stopped and the user taps a drawn piece
- **THEN** the stroke is emitted, sounds, and flashes, exactly as during
  playback

### Requirement: Pointer Pitch Hit-Testing

Mapping a pointer position to a pitch SHALL use the shared keyboard layout so a
tap selects the key actually drawn under the pointer. Black keys SHALL take
priority over white keys within their overlapping region (the upper part of the
keyboard where a black key is painted over the gap between whites); a pointer
outside the displayed key range SHALL map to no pitch and produce no note.

For a **percussion** score the same mapping SHALL run against **the geometry
that drew the kit** — never a second, parallel computation of where the pieces
are (see `music-drum-kit-view`). The whole band SHALL be live: a pointer on a
drawn piece hits it, a pointer between two pieces hits the nearer of the two,
and a pointer in the bass drum's band hits the kick — the space between the
drawn shapes is styling, not a hit boundary. A drummer at tempo aims at a
region, not at an outline; a tap swallowed by a decorative gap is a ghost
stroke, which reads as broken input. A piece drawn over the bass drum's band
SHALL win it: the nearer object is the one the eye aimed at.

#### Scenario: White key hit
- **WHEN** the pointer is over the lower part of a white key, away from any black
  key
- **THEN** the hit-test returns that white key's pitch

#### Scenario: Black key priority in overlap
- **WHEN** the pointer is in the upper region where a black key overlaps the
  boundary between two white keys
- **THEN** the hit-test returns the black key's pitch, not the white key's

#### Scenario: Out of range is ignored
- **WHEN** the pointer is beyond the lowest or highest displayed key
- **THEN** the hit-test returns no pitch and no note-on is emitted

#### Scenario: A tap between two drums still strikes
- **WHEN** the pointer lands in the space between two drawn pieces
- **THEN** the hit-test returns the nearer piece — no dead zone exists in the
  kit's band

#### Scenario: The bass drum's band is all kick
- **WHEN** the pointer lands in the bass drum's band, clear of any piece
- **THEN** the hit-test returns the kick

#### Scenario: A piece drawn over the bass drum wins its own taps
- **WHEN** a drawn piece overlaps the bass drum's band and the pointer lands on
  that piece
- **THEN** the hit-test returns the piece, not the kick

### Requirement: Key And Kit Feedback States

The keyboard SHALL render three visual states per key derived from the notes
required at the current playhead and the notes currently held: a key that is
required but not held SHALL show an "expected / press-this" color; a key that is
required and held SHALL show a distinct "correct" color; a key that is held but
not required SHALL show the "pressed" color. The required set SHALL be the same
gate used by Wait Mode at the current playback position, and SHALL include only
notes belonging to the selected hand(s): when a single hand is selected, notes of
the unselected hand SHALL NOT appear in the required set and SHALL NOT be shown in
the expected or correct state.

For a **percussion** score the drawn pieces and the bass drum SHALL carry
exactly one feedback state: a brief, time-based **struck** flash on the piece
that was struck — by an on-screen tap or by an external stroke resolving to
it — replacing the no-feedback interim `add-drum-kit-view` stated here. The
flash SHALL claim nothing about correctness: with no percussion matcher,
expected/correct/incorrect states would be a judgment the product cannot
honestly make, and they arrive with `add-drum-scoring` together with the
matcher whose verdicts they would reflect. The flash SHALL decay on its own
short fixed duration rather than lasting while the note is held — percussion
releases arrive within milliseconds (see `music-drum-input`), so a
hold-driven highlight would be an invisible flicker. Feedback SHALL live on
the controller only **in this change**: a falling note reacting to a stroke
would be claiming the stroke answered *it*, which is the judgment this change
has no matcher to make. Notes light with `add-drum-scoring`, whose tolerance
window defines what "answered" means (see `music-drum-kit-view`).

#### Scenario: Expected key highlighted
- **WHEN** a note is required at the playhead and not currently held
- **THEN** its key shows the expected/press-this color

#### Scenario: Correct key highlighted distinctly
- **WHEN** a required note is currently held
- **THEN** its key shows the correct color, distinct from the expected color

#### Scenario: Extra pressed key
- **WHEN** a key is held that is not required
- **THEN** it shows the pressed color

#### Scenario: Narrow black key remains visible
- **WHEN** a required or correct key is a black key at a small on-screen width
- **THEN** its highlight remains visible (e.g. via an outline/cap)

#### Scenario: Unselected hand never expected
- **WHEN** a single hand is selected and a note of the other hand falls at the
  playhead
- **THEN** that note's key is not shown in the expected or correct state and is
  absent from the required set

#### Scenario: A struck piece flashes
- **WHEN** a drawn piece is tapped, or an external stroke resolves to its lane
- **THEN** that piece shows the struck flash, which decays on its own

#### Scenario: The flash claims no verdict
- **WHEN** one stroke lands on an onset and another lands far from any onset
- **THEN** both produce the identical struck flash — no correct or incorrect
  distinction is shown

#### Scenario: An immediate release does not clip the flash
- **WHEN** a stroke's note-off arrives within milliseconds of its note-on
- **THEN** the flash still runs its full duration

#### Scenario: The bass drum flashes on kick strokes
- **WHEN** a kick stroke arrives from the drawn bass drum or an external kit
- **THEN** the bass drum shows the struck flash

### Requirement: Multi-Touch Polyphony

The on-screen keyboard SHALL support multiple simultaneous pointers so that
chords can be played where the platform reports multi-touch. Each pointer SHALL
track its own pressed pitch independently, so releasing one finger SHALL note-off
only that finger's pitch and leave the others sounding.

The drawn kit SHALL support the same simultaneity: each pointer maps
independently to the piece it lands on, and simultaneous strokes on different
pieces each emit. **Every pointer-down on a piece SHALL be a fresh stroke, even
while another pointer still rests on the same piece** — alternating two fingers
on one drum is how a roll is played; one-shots have no sustained voice an extra
attack could steal.

#### Scenario: Two keys held at once
- **WHEN** two pointers press two different keys simultaneously
- **THEN** both pitches are note-on and held together

#### Scenario: Independent release
- **WHEN** two keys are held by two pointers and one pointer lifts
- **THEN** only that pointer's pitch is note-off and the other remains held

#### Scenario: Two drums struck together
- **WHEN** two pointers strike two different drawn pieces at the same time
- **THEN** both strokes are emitted and both pieces sound

#### Scenario: A two-finger roll on one drum
- **WHEN** two fingers alternate rapidly on a single drawn piece, each pressing
  while the other is still down
- **THEN** every pointer-down emits a stroke — none is swallowed by the other
  finger's contact

### Requirement: Assisted Correct-Hand Keys

For a **keyboard** score the desktop keyboard SHALL provide two assist keys that
play the notes expected at the current playhead for one hand: the **left-hand**
key plays all expected staff-2 notes and the **right-hand** key plays all
expected staff-1 notes. The expected set is the same gate used by Wait Mode at
the playhead. Pressing the key SHALL note-on every expected pitch for that hand
and releasing it SHALL note-off those pitches, through the same note-on/off path
as MIDI, so playing the correct hand key satisfies the gate. When no note is
expected for that hand, the key SHALL do nothing.

For a **percussion** score the assist keys SHALL remain not offered: their
expected set is the same gate Wait Mode uses, and no percussion gate exists
until `add-drum-scoring` — the input path now exists
(`add-drum-input-mapping`), but the judgment the assist keys shortcut does
not. When that gate lands, any percussion assist scheme SHALL key on the
hands/feet voice convention stated in `hand-color-coding`, not on staves.

#### Scenario: Right-hand key plays the expected right-hand notes
- **WHEN** staff-1 notes are expected at the playhead and the right-hand assist
  key is pressed
- **THEN** those pitches are note-on (and note-off on release), satisfying the
  Wait Mode gate for the right hand

#### Scenario: Left-hand key plays the expected left-hand notes
- **WHEN** staff-2 notes are expected and the left-hand assist key is pressed
- **THEN** those pitches are note-on and note-off on release

#### Scenario: Chord of expected notes
- **WHEN** a hand has several notes expected at the same playhead
- **THEN** pressing that hand's key plays all of them together

#### Scenario: Nothing expected for the hand
- **WHEN** no note is expected for a hand at the playhead
- **THEN** pressing that hand's assist key produces no note

#### Scenario: No assist keys for a percussion score
- **WHEN** a percussion score is loaded on desktop
- **THEN** the assist keys produce no notes
