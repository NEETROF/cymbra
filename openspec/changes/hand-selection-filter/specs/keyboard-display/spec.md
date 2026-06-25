## MODIFIED Requirements

### Requirement: Three-State Key Feedback

The keyboard SHALL render three visual states per key derived from the notes
required at the current playhead and the notes currently held: a key that is
required but not held SHALL show an "expected / press-this" color; a key that is
required and held SHALL show a distinct "correct" color; a key that is held but
not required SHALL show the "pressed" color. The required set SHALL be the same
gate used by Wait Mode at the current playback position, and SHALL include only
notes belonging to the selected hand(s): when a single hand is selected, notes of
the unselected hand SHALL NOT appear in the required set and SHALL NOT be shown in
the expected or correct state.

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
