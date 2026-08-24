## MODIFIED Requirements

### Requirement: Scored Run Activation

Performance scoring SHALL be active in **every** render mode — Synthesia, the horizontal
scrolling staff, and the engraved vertical Partition — with Wait Mode **either on or off**.
A scored run SHALL begin **only for a full run** — playback starting from the beginning of the
**whole piece** (the active measure range is the whole piece) — and SHALL end when the playhead
reaches the end of the piece. A **selective run** (a practice run whose active measure range is
narrower than the whole piece) SHALL NOT begin a scored run. Switching the render mode during a
run SHALL keep the run active (the scored note set does not depend on the render mode).

A scored run SHALL judge only the notes of the currently selected hand(s) and SHALL record
which hand(s) were played, so the result reflects a single hand selection. Changing the
selected hand(s) SHALL restart the piece from the beginning with a fresh scored run for the
new selection, so the score stays coherent over the whole piece and still ends in a summary
(rather than leaving the piece playing unscored).

A **percussion** score SHALL NOT begin a scored run until instrument-aware
judgment exists (`add-drum-scoring`): the scorer never arms — the same
never-arms mechanism a selective run uses — so a percussion stroke fed to the
scoring entry points is a no-op by construction, and no session result exists
to feed the run summary, the persisted history, or the backend ingest sites
that `music-drums-visibility` already requires to fail closed. The judge is
keyboard-shaped — exact-pitch matching against numbers a drum lane
deliberately collapses, sustain judgment against one-shots that have no
sustain — so a percussion "score" would be confidently wrong. Strokes stay
audible and visible (`music-drum-input`); they are not judged.

#### Scenario: Scoring runs in Synthesia with Wait Mode off
- **WHEN** Wait Mode is off, the render mode is Synthesia or scrolling staff, and the
  player starts a full run from the beginning
- **THEN** a scored run begins and note judgments accumulate as the playhead advances

#### Scenario: Scoring runs with Wait Mode on
- **WHEN** Wait Mode is on, the render mode is Synthesia or scrolling staff, and the
  player starts a full run from the beginning
- **THEN** a scored run begins and note judgments accumulate at each gated onset, using the
  Wait-Mode timing model, without altering the Wait-Mode gating behavior

#### Scenario: The Partition view is scored
- **WHEN** the render mode is the engraved Partition view and the player starts a full run from
  the beginning
- **THEN** a scored run begins and note judgments accumulate

#### Scenario: A selective run is not scored
- **WHEN** the active measure range is narrower than the whole piece and the player starts the
  run
- **THEN** no scored run begins, no judgments accumulate, and the run stays unscored

#### Scenario: Switching render mode keeps the run
- **WHEN** the render mode changes (e.g. Synthesia → Partition) during an active scored run
- **THEN** the run stays active with its accumulated judgments and its gauge/effects

#### Scenario: Scoring is scoped to the selected hand(s)
- **WHEN** the player has selected a single hand and starts a scored run
- **THEN** only that hand's notes are judged and the result records the hand selection

#### Scenario: Changing hands restarts scoring from the top
- **WHEN** the selected hand(s) change while a scored run is active
- **THEN** the playhead returns to the start and a fresh scored run begins for the new
  selection, keeping the gauge/effects active and still ending in a summary

#### Scenario: A percussion run is not scored
- **WHEN** the player starts a full run on a percussion score
- **THEN** no scored run begins, strokes accumulate no judgments, and no
  session result, summary or gauge is produced

#### Scenario: Keyboard runs are unaffected by the percussion carve-out
- **WHEN** the player starts a full run on a keyboard score
- **THEN** a scored run begins exactly as before
