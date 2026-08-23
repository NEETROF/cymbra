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
