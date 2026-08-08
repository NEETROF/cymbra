# gamified-feedback delta — partition-game-readability

## MODIFIED Requirements

### Requirement: Sync Gauge Display

While a scored run is active, the system SHALL display a compact score chip in
the player's top bar that reflects the live synchronization percentage, the
current feedback tier (via the tier colour) and the combo counter. The chip
SHALL live in the top bar — outside the play surface — so it can never occlude
the falling notes, the note-hit line, the on-screen keyboard, or the engraved
notation, and SHALL be shown in every render mode (Synthesia, the scrolling
staff, and the Partition). No floating score display SHALL be drawn over the
play surface, and no engraving width SHALL be reserved for one. When no scored
run is active the chip SHALL be hidden.

#### Scenario: Chip visible during a scored run
- **WHEN** a scored run is active in any render mode (Synthesia, scrolling
  staff, or Partition)
- **THEN** the top-bar score chip is shown and reflects the current
  synchronization percentage, tier colour and combo

#### Scenario: Chip hidden when not scoring
- **WHEN** no scored run is active (playback stopped before a run, or the run
  was cancelled)
- **THEN** the score chip is not shown

#### Scenario: Chip visible in Wait Mode
- **WHEN** a scored run is active with Wait Mode on
- **THEN** the score chip is shown and reflects the current synchronization
  percentage

#### Scenario: Nothing floats over the play surface
- **WHEN** a scored run is active in the Partition render mode
- **THEN** no score box is drawn over the engraved notation and the engraving
  uses the full available width (no reserved gutter)

### Requirement: Tiered Feedback At 20% Thresholds

The system SHALL define visual feedback tiers keyed to 20% bands of the live
synchronization percentage (0–20, 20–40, 40–60, 60–80, 80–100). Crossing upward
into a higher tier SHALL trigger a distinct, celebratory escalation of the
top-bar score chip — at minimum the chip SHALL shake and a brief
firework/particle burst SHALL play within it, in the tier colour. The tier SHALL
also visibly de-escalate (quietly, no celebration) when the percentage falls
back into a lower band. Tier changes SHALL be derived purely from the
percentage so they are host-testable without rendering.

#### Scenario: Crossing up a tier escalates feedback
- **WHEN** the live synchronization percentage rises from one 20% band into the
  next higher band
- **THEN** the feedback tier increases and a celebratory escalation is signaled
  (the score chip shakes and a brief firework burst plays within it)

#### Scenario: Dropping a tier de-escalates
- **WHEN** the live synchronization percentage falls back from a band into a
  lower band
- **THEN** the feedback tier decreases accordingly

#### Scenario: Tier derived purely from percentage
- **WHEN** a synchronization percentage value is supplied
- **THEN** the corresponding tier is computed deterministically from that value
  alone
