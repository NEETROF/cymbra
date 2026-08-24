## ADDED Requirements

### Requirement: Percussion onsets gate on expected strokes

The Wait Mode gate SHALL, for a percussion score, freeze at each onset and
release when every stroke that onset requires — restricted to the selected
hands/feet (`hand-selection`) — has been **struck while the gate is active**,
with stroke identity decided by `music-drum-scoring` (kit-piece grain, the
open/closed hi-hat shading rule) in place of pitch equality. The kick's
full-width bar is a note in a different shape: a kick required at the onset
gates like any stroke.

A stroke is an attack with no hold, so the keyboard gate's held-pitch
satisfaction has no percussion counterpart and none SHALL be invented: nothing
a drummer does is "still holding it" at the onset. What the kit needs instead
is a **tolerance window**: a stroke that lands within a bounded interval
**before** the onset SHALL satisfy it when the playhead arrives, rather than
being discarded and demanded again.

Without it the gate accepts only a stroke landing after the playhead does,
which is not a timing exercise but a reaction test: a kit is played by feel,
the hand leaves before the ear checks, and a window narrower than the player's
own anticipation makes a correct groove read as a failure. The window is the
percussion half of the same tolerance the keyboard has always had, stated in
the terms a kit can express.

Three properties bound it, and SHALL hold together:

1. it is **one number**, shared by the gate and by every surface that lights a
   stroke, so what is accepted and what is shown cannot drift apart;
2. it is measured on the **playhead's** clock, not the wall clock — a musical
   window has to be measured where the onset lives, and a frozen or
   speed-scaled playhead would otherwise skew it;
3. a stroke is **spent** when credited, so one stroke never satisfies two
   onsets — the percussion counterpart of the keyboard's "each press counts
   toward at most one onset".

#### Scenario: A strike releases the gate

- **WHEN** Wait Mode is on, the playhead is frozen at a percussion onset, and
  the player strikes the required piece
- **THEN** the gate releases and the playhead advances to the next onset

#### Scenario: A stroke a hair early satisfies the onset

- **WHEN** the player strikes the upcoming onset's piece within the tolerance
  window before the playhead reaches that onset
- **THEN** the playhead crosses the onset without freezing — the stroke is
  credited to it, and nothing is demanded a second time

#### Scenario: A stroke long before the onset does not

- **WHEN** the player strikes the upcoming onset's piece further ahead of it
  than the tolerance window
- **THEN** the gate is not satisfied; when the playhead arrives it still waits
  for a fresh strike

#### Scenario: One stroke never validates two onsets

- **WHEN** the same piece is required at two onsets close enough together that
  a single stroke falls inside both tolerance windows
- **THEN** the stroke is credited to the first onset only, and the second waits
  for its own

#### Scenario: A multi-piece onset requires every stroke

- **WHEN** an onset requires a snare and a crash together and only the snare has
  been struck while the gate is active
- **THEN** the gate stays frozen until the crash is also struck, in any order

#### Scenario: A kick coincidence gates the foot too

- **WHEN** an onset requires a hand stroke and a kick, with both hands and feet
  selected
- **THEN** the gate releases only when both the hand stroke and the kick have
  been struck

#### Scenario: The wrong piece keeps blocking

- **WHEN** the gate awaits a tom and the player strikes the snare
- **THEN** the gate stays frozen and the stroke is handled as an extra note by
  the scorer

## MODIFIED Requirements

### Requirement: Validation At The Right Moment

For a **keyboard** score, a pitch SHALL count toward the current onset gate
when it is down at the moment the playhead has reached that onset — whether the
player presses it while the gate is active OR was already holding it
continuously when the gate became active. A press that occurred **and was
released** before the playhead reached the onset SHALL NOT pre-satisfy it.

Each press SHALL count toward at most one onset: once a held pitch has satisfied
an onset, that same hold SHALL NOT satisfy a later onset of the same pitch — a
repeated pitch requires a fresh attack (release and re-press, or a new press).
This keeps Wait Mode a timing exercise while tolerating a sustained/tied note
carried into the onset where it first sounds; it does not let a single held key
auto-advance through repeated notes.

For a **percussion** score, onset satisfaction is defined by the
percussion-gate requirement ("Percussion onsets gate on expected strokes"): a
stroke is an attack with no hold, so the held-pitch branch above has no
percussion counterpart — its role is played by the tolerance window stated
there, under the same "at most one onset" rule.

#### Scenario: Early press-and-release does not pre-satisfy
- **WHEN** the player presses the upcoming note's key and releases it before the
  playhead reaches that note's onset
- **THEN** the gate is not satisfied; when the playhead reaches the onset it still
  waits for the pitch to be down at that moment

#### Scenario: Pitch held through the onset satisfies
- **WHEN** the player presses a note's key before its onset, that pitch has not
  already satisfied an earlier onset, and it is still held when the playhead
  reaches the onset
- **THEN** the held pitch counts as satisfied at the onset with no re-press
  required, and the gate releases (subject to any other pitches the onset needs)

#### Scenario: Repeated pitch held across onsets requires a fresh attack
- **WHEN** the same pitch is required at two consecutive onsets, the player's hold
  satisfied the first onset, and the player keeps holding it into the second onset
  without re-pressing
- **THEN** the second onset is NOT satisfied by the sustained hold; it stays frozen
  until the player releases and re-presses (or otherwise re-attacks) that pitch

#### Scenario: Press at the onset satisfies
- **WHEN** the playhead has reached the onset and the player presses the required
  key
- **THEN** the press counts and the gate releases

### Requirement: Chord Onset Requires All Pitches

When multiple notes share an onset, the gate SHALL require every one of those
pitches to be down before releasing. For a **keyboard** score, each pitch MAY
be satisfied either by being already held continuously when the onset becomes
active or by being pressed while the gate is active; the pitches need not be
pressed simultaneously. The gate SHALL NOT release until the full set for that
onset is satisfied. For a **percussion** score, held satisfaction has no
counterpart — every required stroke must land while the gate is active, as the
percussion-gate requirement ("Percussion onsets gate on expected strokes")
defines.

#### Scenario: All chord notes pressed
- **WHEN** an onset has three pitches and the player presses all three (in any
  order) while the gate is active
- **THEN** the gate releases after the third press

#### Scenario: Mix of held and freshly pressed pitches
- **WHEN** an onset has three pitches, the player is already holding one of them
  from before the onset, and then presses the other two while the gate is active
- **THEN** the held pitch and the two presses together satisfy the onset and the
  gate releases

#### Scenario: Partial chord keeps blocking
- **WHEN** an onset has three pitches and only two of them are down (held or
  pressed)
- **THEN** the gate stays frozen until the remaining pitch is down

### Requirement: Non-Intrusive Wait Indicator

The player SHALL indicate that Wait Mode is holding playback without covering
the play surface or breaking immersion: while the onset gate is blocked, the
expected keys highlighted on the on-screen keyboard SHALL pulse gently
(a slow breathing of their highlight), and no text banner or box SHALL be
displayed over the play surface. The pulse SHALL stop as soon as the gate
releases (the highlight returns to its steady state). When the on-screen
keyboard is hidden, the existing expected-note emphasis in the notation views
remains the indicator; no overlay SHALL be added.

For a **percussion** score the same contract holds on the drawn kit: while the
gate is blocked, the expected pieces — and the bass drum when a kick is
required — SHALL pulse gently, and no overlay SHALL be added. The kit is always
drawn on a play surface (`keyboard-display`), so the indicator is always
available there; in a percussion notation mode with no kit drawn, the
notation's expected-note emphasis is the indicator, exactly as for keyboard.

#### Scenario: Blocked gate pulses the expected keys

- **WHEN** Wait Mode freezes playback at an onset with the keyboard shown
- **THEN** the expected keys' highlight pulses gently and no text banner is
  shown over the play surface

#### Scenario: Blocked gate pulses the expected pieces

- **WHEN** Wait Mode freezes playback at a percussion onset with the kit drawn
- **THEN** the expected pieces (and the bass drum, when a kick is required)
  pulse gently and no text banner is shown over the play surface

#### Scenario: Release restores the steady highlight

- **WHEN** the player satisfies the gate and playback resumes
- **THEN** the pulse stops and the keyboard highlight returns to its steady
  rendering

#### Scenario: Nothing is added when the keyboard is hidden

- **WHEN** Wait Mode blocks in a notation mode with the keyboard hidden
- **THEN** no overlay or banner appears (the notation's expected-note emphasis
  is the only indicator)
