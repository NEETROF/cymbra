## MODIFIED Requirements

### Requirement: Sync Gauge Display

While a scored run is active, the system SHALL display a compact score chip in
the player's top bar that reflects the live synchronization percentage, the
current feedback tier (via the tier colour) and the combo counter. The chip
SHALL live in the top bar — outside the play surface — so it can never occlude
the falling notes, the note-hit line, the on-screen keyboard, or the engraved
notation, and SHALL be shown in **every render mode the loaded score offers**:
for a keyboard score Synthesia, the scrolling staff, and the Partition; for a
percussion score the cascade and the percussion notation modes
(`add-drum-notation-render`). No floating score display SHALL be drawn over the
play surface, and no engraving width SHALL be reserved for one. When no scored
run is active the chip SHALL be hidden.

#### Scenario: Chip visible during a scored run

- **WHEN** a scored run is active in any render mode (Synthesia, scrolling
  staff, or Partition)
- **THEN** the top-bar score chip is shown and reflects the current
  synchronization percentage, tier colour and combo

#### Scenario: Chip visible in the cascade

- **WHEN** a scored percussion run is active in the cascade
- **THEN** the top-bar score chip is shown, reflecting the percussion run's
  percentage, tier colour and combo, and nothing floats over the lanes or the
  kick bar

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

### Requirement: Per-Note Hit Feedback

On each judged onset the system SHALL show brief, Guitar-Hero–style visual feedback at
the note's location (e.g. a spark/flash whose intensity tracks the timing verdict) and
SHALL maintain and surface a combo/streak counter that increments on consecutive
non-missed onsets and resets on a miss or wrong note. The feedback SHALL be
transient and SHALL NOT leave persistent clutter over the play surface.

For a **percussion** score the feedback is the surface's own: the note the
stroke answered LIGHTS, and the piece that was struck flashes
(`music-drum-kit-view`, `keyboard-display`). No separate spark layer SHALL be
drawn over the play surface. Two surfaces drawing the same event twice is the
defect this replaces — the layer anchored its sparks to the bottom of the
screen, which stopped being where anything happens the day the surfaces drew
the kit there, so a clean hit lit the note in one place and threw a half-clipped
circle in another. The timing VERDICT is carried by the sync gauge and the
score, not by the kit: a stroke either answered its note or did not.

#### Scenario: Perfect hit shows the strongest spark

- **WHEN** an onset is judged `perfect`
- **THEN** a brief high-intensity hit effect is shown at that note's position

#### Scenario: Combo increments then resets

- **WHEN** the player lands consecutive non-missed onsets and then misses one (or plays a
  wrong note)
- **THEN** the combo counter increments on each landed onset and resets to zero on the miss

#### Scenario: Feedback is transient

- **WHEN** a hit effect has been shown
- **THEN** it fades within a short time and does not persist over subsequent notes

#### Scenario: Sparks are hidden when the keyboard is hidden

- **WHEN** the on-screen keyboard is hidden (a notation mode with the keyboard off)
- **THEN** the hit sparks — which anchor to the keyboard/note-hit line — are not drawn,
  while the sync gauge still shows

#### Scenario: A percussion run draws no spark layer

- **WHEN** a scored percussion run is played on either play surface
- **THEN** no spark overlay is drawn over it — the answered note lighting and
  the struck piece's flash are the whole of the per-note feedback
