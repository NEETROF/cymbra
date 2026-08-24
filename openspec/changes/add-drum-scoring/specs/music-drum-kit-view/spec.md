## RENAMED Requirements

- FROM: `### Requirement: Wait Mode is not offered for a percussion score, for now`
  TO: `### Requirement: Wait Mode is offered for a percussion score`

## MODIFIED Requirements

### Requirement: Wait Mode is offered for a percussion score

Wait Mode SHALL be offered for a percussion score exactly as for a keyboard one.
The interim restriction is lifted on the condition it named: a percussion input
path exists (`add-drum-input-mapping`) and instrument-aware scoring exists
(`music-drum-scoring`), so the gate can be satisfied and judged. The gate's
percussion semantics — freeze at each onset, release on the required strokes,
stroke identity at the kit piece's grain — are stated in `wait-mode` and
`music-drum-scoring`, not re-derived here. The timed modes are unchanged, and a
keyboard score's Wait Mode is untouched.

The interim requirement's "Wait Mode absent" scenario is inverted here **by
design**: the restriction was interim by construction, and this requirement is
its stated successor.

#### Scenario: Wait Mode is offered and gates

- **WHEN** a percussion score is loaded
- **THEN** Wait Mode can be enabled, and with it on, playback freezes at each
  onset until the required strokes are struck

#### Scenario: Keyboard scores keep Wait Mode

- **WHEN** a keyboard score is loaded
- **THEN** Wait Mode is offered exactly as before

## ADDED Requirements

### Requirement: A note lights when it is struck, never when it arrives

On both play surfaces, the **only** thing that lights a note SHALL be the player
striking that note's piece within the stroke tolerance window this change
defines (`wait-mode`). A note SHALL NOT light itself as its instant comes.

This is what lifts the interim of `add-drum-input-mapping`, which kept every
reaction on the controller because no matcher existed to say whether a stroke
had answered anything. The window is that answer, and it is the same number on
both sides: a stroke that lights a note is exactly a stroke that would satisfy
that onset's gate.

A note that announces its own moment teaches the player to read the surface
instead of hearing the beat, and it makes a good hit and a lucky one look
identical — the surface would be playing the piece, and the player following it.
The hit line, the metre and the falling note say *when*; only the player's own
stroke says *landed*.

A note whose instant has passed SHALL leave the surface rather than come to rest
on the hit line: a note parked where it landed reads as still owed.

#### Scenario: An arriving note does not light

- **WHEN** a note reaches the hit line and the player strikes nothing
- **THEN** the note is drawn no differently than it was a moment earlier

#### Scenario: A stroke on time lights the note it answered

- **WHEN** the player strikes a piece while one of its notes is inside the
  tolerance window
- **THEN** that note lights, and the pieces of every other lane do not

#### Scenario: A passed note leaves

- **WHEN** the playhead has crossed a note's instant by more than the tolerance
  window
- **THEN** that note is no longer drawn on the surface
