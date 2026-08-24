## RENAMED Requirements

- FROM: `### Requirement: The notation modes are not offered for a percussion score, for now`
  TO: `### Requirement: The notation modes are offered for a percussion score`

## MODIFIED Requirements

### Requirement: The notation modes are offered for a percussion score

The render-mode toggle SHALL offer a percussion score the **same set of render
modes a keyboard score gets on the same device** — the cascade in place of
Synthesia, plus the scrolling Staff and the engraved Partition, both drawn per
`music-percussion-engraving`. This lifts the interim installed by
`add-drum-kit-view`, whose stated condition ("until `add-drum-notation-render`
lands") is met by this change. The parity phrasing is deliberate: it inherits
existing device rules (a phone that remaps Partition to Staff for keyboard
scores does the same for percussion) instead of pinning an absolute list that
would contradict them.

A **play surface SHALL remain the default** presentation when a percussion
score is loaded and the player has never chosen otherwise: they are the
designed reading surfaces for playing (`add-drum-kit-view` settled them against
a drummer), and re-offering notation must not change what a returning tester
sees on load. The notation modes are an explicit choice, and switching between
modes SHALL behave as it does for a keyboard score.

That choice SHALL then be remembered **per instrument family** and re-applied
when a score of that family opens. A player reads drums and piano differently —
the stage for a groove, the staff for a piece — and one memory would make each
score undo the other's setting. A remembered mode is checked against the family
before it is applied: the stage exists only for percussion, so a record naming
it can never reach a keyboard score.

The **scrolling Staff SHALL carry the drawn kit** under it, the same one the
play surfaces draw: it engraves notes but offers nothing to aim at, and a second
different picture of the same instrument under the same score is exactly what
`add-drum-kit-view` removed. The **engraved Partition SHALL carry none** — it is
the mode that already refuses the on-screen keyboard, for the same reason: a
printed page is read, not aimed at, and the freed height is what keeps the
current and next systems on screen together.

The inverted-kit setting keeps its existing contract untouched: it reaches the
falling notes and the drawn kit only, and has no effect on the notation modes
now that they exist — drum notation is a fixed convention.

#### Scenario: The mode toggle offers the full set

- **WHEN** a percussion score is loaded on a device where a keyboard score
  gets Synthesia, Staff and Partition
- **THEN** the toggle offers the cascade, Staff and Partition, and each is
  selectable

#### Scenario: The cascade is still the default

- **WHEN** a percussion score is loaded and the player has never chosen a mode
  for percussion
- **THEN** the player opens in the cascade, and a notation mode is entered
  only by the player's choice

#### Scenario: The last mode chosen for this family comes back

- **WHEN** the player reads a drum score on the stage, then opens another drum
  score
- **THEN** it opens on the stage — and a piano score opened in between still
  opens on the mode last chosen for the keyboard

#### Scenario: The notation modes engrave the percussion rules

- **WHEN** Staff or Partition is selected for a percussion score
- **THEN** the score is drawn per `music-percussion-engraving` — percussion
  clef, written positions, x-form cymbal heads, two voices — not with
  keyboard-staff assumptions

#### Scenario: The Staff carries the kit, the Partition does not

- **WHEN** a percussion score is shown in the scrolling Staff, then in the
  engraved Partition
- **THEN** the Staff draws the same kit the play surfaces draw, struck the same
  way, and the Partition draws no kit at all

#### Scenario: Keyboard scores keep the three modes

- **WHEN** a keyboard score is loaded
- **THEN** Synthesia, Staff and Partition are all offered, exactly as before
