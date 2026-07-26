## Why

Opening a score drops the user straight into the play surface with no chance to
set up how they want to play it. The hand choice, MIDI device, tempo and
metronome all live behind the settings gear, and the score's own details
(composer, difficulty, key/time/tempo) aren't shown anywhere. A short pre-play
setup step — shown each time a score opens — lets the user confirm the piece and
tune the essentials before the first note.

## What Changes

- Add a **pre-play setup modal**, centered over the player, shown **each time**
  the player opens a score (once the notation has loaded).
- The modal presents, in one place:
  - **Score information** (title, composer, difficulty, key / time signature /
    tempo when known) — read-only.
  - **Hands** to play (left / right / both), offered only for multi-staff pieces.
  - **Tempo** (playback speed) and **metronome** on/off.
  - **MIDI device**: the connected piano and a picker to select one (with the
    Android no-device guidance reused).
- A **Validate** button at the bottom applies the chosen settings and closes the
  modal; a **close (X)** dismisses it keeping the current settings. Either way the
  user stays on the player.
- Extract the small shared controls (an option/radio row, the hand labels, the
  difficulty badge) so the modal and the existing settings drawer share them
  instead of duplicating.

## Capabilities

### New Capabilities
- `pre-play-setup`: the pre-play setup modal — when it appears, what it shows, and
  how validate/close behave.

### Modified Capabilities
<!-- None: hand-selection, midi, score-facets and keyboard-display are surfaced in
     a new place but their requirements are unchanged. -->

## Impact

- **Flutter app** (`apps/music`):
  - New `lib/screens/pre_play_setup_modal.dart` (the modal + `showPrePlaySetup`).
  - New shared widgets: `lib/widgets/setting_option_row.dart` (option row +
    `handLabel`), `lib/widgets/difficulty_badge.dart` (extracted public badge).
  - `lib/screens/player_screen.dart`: trigger the modal once per opened score;
    reuse the shared controls in `_SettingsDrawer`.
  - `lib/widgets/score_card.dart`: use the extracted difficulty badge.
  - New l10n strings (modal title, section labels, validate) in
    `app_{en,fr,es,it}.arb`.
- Reuses existing state/mutators only (no new player state, no backend change):
  `selectedHands`/`setSelectedHands`, `speed`/`setSpeed`, `metronomeEnabled`/
  `toggleMetronome`, `midiPorts`/`connectedDevice`/`selectMidiPort`.
