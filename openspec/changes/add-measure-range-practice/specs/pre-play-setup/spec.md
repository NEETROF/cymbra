## ADDED Requirements

### Requirement: Full-run vs selective-practice choice in setup

The pre-play setup modal SHALL let the user choose to start a **full run** (the whole piece,
scored as usual) or a **selective (practice) run** (a chosen measure range, unscored). When
selective is chosen, the modal SHALL let the user pick the **measure range** (a from-measure and
a to-measure) and the **loop settings** (loop on/off, loop count, tempo ramp), pre-filled from
any per-score saved settings. The chosen run type and settings SHALL be applied to the session
that begins when the modal is dismissed. The default choice SHALL be a full run.

#### Scenario: Full run is the default
- **WHEN** the setup modal opens and the user makes no run-type change
- **THEN** dismissing it starts a full run of the whole piece

#### Scenario: Selective run applies the chosen range
- **WHEN** the user chooses a selective run and picks a measure range in the setup modal
- **THEN** dismissing it starts a selective (unscored) run over that range

#### Scenario: Selective run applies loop settings
- **WHEN** the user enables looping (with a loop count and/or tempo ramp) for a selective run
- **THEN** the run loops the range according to those settings

#### Scenario: Saved settings pre-fill the picker
- **WHEN** the score has per-score saved practice settings
- **THEN** the setup modal pre-fills the range and loop settings from them
