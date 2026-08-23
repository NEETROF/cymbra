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

The **cascade SHALL remain the default** presentation when a percussion score
is loaded: it is the designed reading surface for playing (`add-drum-kit-view`
settled it against a drummer), and re-offering notation must not change what a
returning tester sees on load. The notation modes are an explicit choice, and
switching between modes SHALL behave as it does for a keyboard score.

The inverted-kit setting keeps its existing contract untouched: it reaches the
cascade and the pad strip only, and has no effect on the notation modes now
that they exist — drum notation is a fixed convention.

#### Scenario: The mode toggle offers the full set

- **WHEN** a percussion score is loaded on a device where a keyboard score
  gets Synthesia, Staff and Partition
- **THEN** the toggle offers the cascade, Staff and Partition, and each is
  selectable

#### Scenario: The cascade is still the default

- **WHEN** a percussion score is loaded
- **THEN** the player opens in the cascade, and a notation mode is entered
  only by the player's choice

#### Scenario: The notation modes engrave the percussion rules

- **WHEN** Staff or Partition is selected for a percussion score
- **THEN** the score is drawn per `music-percussion-engraving` — percussion
  clef, written positions, x-form cymbal heads, two voices — not with
  keyboard-staff assumptions

#### Scenario: Keyboard scores keep the three modes

- **WHEN** a keyboard score is loaded
- **THEN** Synthesia, Staff and Partition are all offered, exactly as before
