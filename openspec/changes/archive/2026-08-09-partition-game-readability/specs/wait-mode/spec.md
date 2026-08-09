# wait-mode delta — partition-game-readability

## ADDED Requirements

### Requirement: Non-Intrusive Wait Indicator

The player SHALL indicate that Wait Mode is holding playback without covering
the play surface or breaking immersion: while the onset gate is blocked, the
expected keys highlighted on the on-screen keyboard SHALL pulse gently
(a slow breathing of their highlight), and no text banner or box SHALL be
displayed over the play surface. The pulse SHALL stop as soon as the gate
releases (the highlight returns to its steady state). When the on-screen
keyboard is hidden, the existing expected-note emphasis in the notation views
remains the indicator; no overlay SHALL be added.

#### Scenario: Blocked gate pulses the expected keys
- **WHEN** Wait Mode freezes playback at an onset with the keyboard shown
- **THEN** the expected keys' highlight pulses gently and no text banner is
  shown over the play surface

#### Scenario: Release restores the steady highlight
- **WHEN** the player satisfies the gate and playback resumes
- **THEN** the pulse stops and the keyboard highlight returns to its steady
  rendering

#### Scenario: Nothing is added when the keyboard is hidden
- **WHEN** Wait Mode blocks in a notation mode with the keyboard hidden
- **THEN** no overlay or banner appears (the notation's expected-note emphasis
  is the only indicator)
