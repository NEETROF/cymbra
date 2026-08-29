## ADDED Requirements

### Requirement: The Written Score Can Be Silenced

The player SHALL offer a control that stops the app sounding the notes of the
written score, for any instrument. When it is on, the transport SHALL continue
exactly as before — the playhead advances, the score is drawn, the Wait Mode gate
still holds and releases, the scorer still judges, and the metronome still
sounds if it is enabled. Only the score's own audio stops.

#### Scenario: The score goes quiet
- **WHEN** the mute is on and playback runs over a written note
- **THEN** nothing is sounded for that note

#### Scenario: Everything else is unaffected
- **WHEN** the mute is on during a scored run
- **THEN** the drawing, the gate, the judgment and the metronome behave exactly
  as they do with it off

#### Scenario: Unmuting resumes cleanly
- **WHEN** the mute is turned off mid-playback
- **THEN** subsequent notes sound, and no note that was skipped while muted is
  left hanging or retriggered

### Requirement: Distinct From Silencing The Player's Own Notes

This control SHALL be independent of the existing "my instrument sounds itself"
setting: one governs the notes the app *asks for*, the other the notes the player
*plays*. All four combinations SHALL be reachable and SHALL behave as their two
parts describe.

#### Scenario: Only the score is muted
- **WHEN** the score mute is on and "my instrument sounds itself" is off
- **THEN** the player's own strokes sound and the written score does not

#### Scenario: Only the player's notes are muted
- **WHEN** the score mute is off and "my instrument sounds itself" is on
- **THEN** the written score sounds and the player's MIDI notes are not doubled

#### Scenario: Both muted
- **WHEN** both are on
- **THEN** the app sounds neither, and the session still draws, gates and scores

### Requirement: Reachable Mid-Exercise And Remembered

The control SHALL be reachable during play without leaving the player — both from
the settings the top bar opens and as a direct toggle in the transport controls —
and its value SHALL be persisted across launches, like the other audio
preferences beside it.

#### Scenario: Toggled from the transport
- **WHEN** the player toggles it from the transport controls mid-exercise
- **THEN** it takes effect immediately, without pausing or restarting the run

#### Scenario: Remembered across launches
- **WHEN** the mute is on and the app is relaunched
- **THEN** it is still on

#### Scenario: Its state is visible
- **WHEN** the mute is on
- **THEN** the control shows it, so a silent score is never mistaken for a broken
  one
