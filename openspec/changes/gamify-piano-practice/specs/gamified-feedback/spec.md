## ADDED Requirements

### Requirement: Sync Gauge Display

While a scored run is active, the system SHALL display a compact synchronization gauge
that reflects the live synchronization percentage. The gauge SHALL be positioned so it
does not occlude the falling notes, the note-hit line, or the on-screen keyboard, and
SHALL be shown in every render mode (Synthesia, the scrolling staff, and the Partition).
When no scored run is active the gauge SHALL be hidden.

#### Scenario: Gauge visible during a scored run
- **WHEN** a scored run is active in any render mode (Synthesia, scrolling staff, or Partition)
- **THEN** the sync gauge is shown and reflects the current synchronization percentage

#### Scenario: Gauge hidden when not scoring
- **WHEN** no scored run is active (playback stopped before a run, or the run was cancelled)
- **THEN** the sync gauge is not shown

#### Scenario: Gauge visible in Wait Mode
- **WHEN** a scored run is active with Wait Mode on
- **THEN** the sync gauge is shown and reflects the current synchronization percentage

#### Scenario: Gauge does not occlude the play surface
- **WHEN** the gauge is shown
- **THEN** it is placed clear of the falling notes, the hit line, and the keyboard

### Requirement: Tiered Feedback At 20% Thresholds

The system SHALL define visual feedback tiers keyed to 20% bands of the live
synchronization percentage (0–20, 20–40, 40–60, 60–80, 80–100). Crossing upward into a
higher tier SHALL trigger a distinct, celebratory escalation of the gauge/effects
styling; the tier SHALL also visibly de-escalate when the percentage falls back into a
lower band. Tier changes SHALL be derived purely from the percentage so they are
host-testable without rendering.

#### Scenario: Crossing up a tier escalates feedback
- **WHEN** the live synchronization percentage rises from one 20% band into the next
  higher band
- **THEN** the feedback tier increases and a celebratory escalation is signaled

#### Scenario: Dropping a tier de-escalates
- **WHEN** the live synchronization percentage falls back from a band into a lower band
- **THEN** the feedback tier decreases accordingly

#### Scenario: Tier derived purely from percentage
- **WHEN** a synchronization percentage value is supplied
- **THEN** the corresponding tier is computed deterministically from that value alone

### Requirement: Per-Note Hit Feedback

On each judged onset the system SHALL show brief, Guitar-Hero–style visual feedback at
the note's location (e.g. a spark/flash whose intensity tracks the timing verdict) and
SHALL maintain and surface a combo/streak counter that increments on consecutive
non-missed onsets and resets on a miss or wrong note. The feedback SHALL be transient
and SHALL NOT leave persistent clutter over the play surface.

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

### Requirement: Learning-Safe Feedback Constraints

Gamified feedback SHALL be constrained so it never impairs learning: it SHALL NOT hide,
recolor, or displace the note the player must read next; SHALL NOT alter Wait Mode, the
keyboard highlight of expected keys, or note timing; and SHALL NOT present feedback that
contradicts the actual judgment (a missed note SHALL never show a success effect). The
gamified visuals SHALL be suppressible so the play surface remains fully legible.

#### Scenario: Feedback never obscures upcoming notes
- **WHEN** hit effects and the gauge are rendered
- **THEN** the upcoming falling notes and expected-key highlights remain fully visible and
  unaltered

#### Scenario: No misleading success on a miss
- **WHEN** an onset is judged `missed`
- **THEN** no success/celebration effect is shown for that onset

#### Scenario: Effects do not change learning behavior
- **WHEN** gamified feedback is active
- **THEN** Wait Mode gating, expected-key highlighting, and note timing behave exactly as
  they do without the feedback layer
