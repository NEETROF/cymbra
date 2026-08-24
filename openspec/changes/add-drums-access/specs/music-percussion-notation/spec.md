## RENAMED Requirements

- FROM: `### Requirement: The validation gate stays closed in this change`
  TO: `### Requirement: The validation gate admits percussion`

## MODIFIED Requirements

### Requirement: The validation gate admits percussion

The shared validation gate SHALL count unpitched notes as playable notes, so a
percussion score passes validation instead of being rejected as containing none.
This opens the admission the companion change `add-unpitched-notation` deliberately
left closed; it is safe to open here because this change also delivers the access
controls that decide who may submit and read such a score (see
`music-drums-visibility`).

A score containing neither pitched nor unpitched notes SHALL still be rejected,
with the same reason as before.

#### Scenario: A percussion score passes the gate

- **WHEN** a score whose only notes are unpitched is validated
- **THEN** it passes, and its summary reports a non-zero playable-note count

#### Scenario: A rest-only score is still rejected

- **WHEN** a score containing only rests is validated
- **THEN** it is rejected for containing no playable notes

#### Scenario: Opening the gate does not itself grant access

- **WHEN** the gate accepts a percussion score
- **THEN** whether that score may be submitted or read is still decided by the drum
  audience enforcement, not by the gate
