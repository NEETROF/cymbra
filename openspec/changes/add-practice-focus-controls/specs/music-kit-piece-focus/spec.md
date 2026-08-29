## ADDED Requirements

### Requirement: Focus Is Held Per Kit Piece

The player SHALL hold, for a percussion score, the set of kit pieces currently
**in focus** — the pieces the session asks for. Every piece of the loaded score's
kit, the kick included, SHALL be individually addressable. The default SHALL be
every piece in focus, and the state SHALL be session-only: it describes a passage
being worked on, not a preference, and SHALL reset to everything on relaunch and
on loading another score.

#### Scenario: Everything is in focus by default
- **WHEN** a percussion score is loaded
- **THEN** every piece of its kit is in focus

#### Scenario: The kick is addressable like any other piece
- **WHEN** the kick is muted
- **THEN** it leaves the focus set exactly as a lane piece would

#### Scenario: Focus does not survive a new score
- **WHEN** a focus selection is made and another score is loaded
- **THEN** every piece of the new score's kit is in focus

### Requirement: Mute And Solo

A piece SHALL be muteable (removed from focus) and soloable (the only piece, or
one of the only pieces, in focus). Soloing a second piece SHALL add it to the
soloed set rather than replacing it. Clearing the focus selection SHALL restore
every piece.

#### Scenario: Muting one piece
- **WHEN** the player mutes the crash
- **THEN** every other piece stays in focus and the crash does not

#### Scenario: Soloing one piece
- **WHEN** the player solos the hi-hat
- **THEN** the hi-hat is the only piece in focus

#### Scenario: Soloing a second piece adds it
- **WHEN** the hi-hat is soloed and the player solos the snare
- **THEN** both are in focus and nothing else is

#### Scenario: Clearing restores everything
- **WHEN** the player clears the selection
- **THEN** every piece of the kit is in focus again

#### Scenario: Nothing in focus is not a reachable state
- **WHEN** the player mutes the last remaining piece
- **THEN** the selection returns to every piece rather than leaving a session
  that asks for nothing

### Requirement: Focus Governs Drawing, Gating And Scoring Together

A piece out of focus SHALL be excluded from all three of: what is drawn (the
cascade, the pad strip's expected outline, the staff and the engraved score,
including the kick's full-width bar), what the Wait Mode gate waits for, and what
the scorer judges. The three SHALL be driven from the one focus set, so a session
can never draw a note it does not ask for, or judge one it never drew.

#### Scenario: A muted piece is not drawn
- **WHEN** the crash is out of focus
- **THEN** no crash note is drawn in any render mode

#### Scenario: Muting the kick hides its bar
- **WHEN** the kick is out of focus
- **THEN** no kick bar is drawn — the bar is a note in another shape, and hiding
  the note while keeping the bar would hide nothing

#### Scenario: A muted piece does not gate
- **WHEN** the playhead reaches an onset whose only note is an out-of-focus
  piece's
- **THEN** the required set at that position is empty and Wait Mode advances

#### Scenario: A muted piece is not judged
- **WHEN** a scored run completes with a piece out of focus
- **THEN** that piece's written notes count neither as hit nor as missed

### Requirement: Focus Never Filters Input

A stroke on an out-of-focus piece SHALL still sound and SHALL still flash its
pad. Focus states what the session asks of the player; it SHALL NOT change what
the player hears from their own instrument.

#### Scenario: A muted piece still sounds when struck
- **WHEN** the crash is out of focus and the player strikes it
- **THEN** it sounds

#### Scenario: A muted piece still flashes
- **WHEN** the crash is out of focus and the player strikes it
- **THEN** its pad flashes exactly as an in-focus piece's would

#### Scenario: A muted piece's stroke is not a mistake
- **WHEN** a stroke lands on an out-of-focus piece during a scored run
- **THEN** it is not counted against the player

### Requirement: Focus Control

The player screen SHALL offer the focus control from the settings surface its top
bar opens, available in every render mode, listing the pieces of the loaded
score's kit in the same order the pad strip draws them — so the control and the
instrument read left to right the same way. It SHALL be offered only for a
percussion score, and only when the kit has more than one piece to choose between.

#### Scenario: Reachable in every mode
- **WHEN** a percussion score is shown in any render mode
- **THEN** the focus control is reachable from the settings and reflects the
  current selection

#### Scenario: Ordered like the instrument
- **WHEN** the focus control is opened
- **THEN** the pieces appear in the pad strip's order

#### Scenario: Not offered where there is nothing to choose
- **WHEN** the loaded percussion score's kit holds a single piece
- **THEN** the focus control is not offered

#### Scenario: Not offered for a keyboard score
- **WHEN** a keyboard score is loaded
- **THEN** the focus control is not offered and the hand selector is
