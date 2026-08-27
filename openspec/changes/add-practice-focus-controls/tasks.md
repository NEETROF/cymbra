## 1. The expected-notes mute (independent — ships first)

- [x] 1.1 `player_preferences.dart`: a persisted `scoreAudioMuted` flag beside
  `instrumentSoundsItself`, with its serialisation and default (off)
- [x] 1.2 `player_data.dart`: the flag in player state, hydrated from prefs on
  load like its neighbours
- [x] 1.3 `player_notifier.dart` `_applyScoreAudio` (`:299`): honour the mute on
  **both** branches. The `_sounding` bookkeeping is skipped with the attack, so
  no release is ever owed for a voice that was never started (the shape of the
  `add-drum-audio-channel` 10.3 bug)
- [x] 1.4 Toggling the mute **on** mid-playback releases what is currently
  sounding and clears `_sounding`; toggling it **off** starts sounding again from
  the next onset. Tested in both directions, including mid-sustain on a keyboard
  note where a missing release would hang a voice
- [x] 1.5 Assert the mute changes nothing else: with it on, the playhead, the
  drawing, the Wait Mode gate, the scorer and the metronome behave identically
  (notifier tests with a mocked `AudioService`, per the `flutter-testing` skill)
- [x] 1.6 All four combinations with `instrumentSoundsItself` behave as their two
  parts describe
- [x] 1.7 UI: a toggle in the settings modal beside "my instrument produces its
  own sound", and a direct toggle whose state is visible, so a silent score is
  never mistaken for a broken one. **Not the transport rail** as D5 assumed: its
  seven controls already fill a phone-landscape viewport to the pixel, and an
  eighth overflowed a short desktop window by 34 px too. It sits in the top-bar
  trailing cluster instead, beside the metronome chip — they answer the same
  question ("what am I hearing while I play"), the cluster scales down as one
  block on a narrow window, and it is still the right-hand bar the tester asked
  for. `design.md` D5 updated
- [x] 1.8 Localised fr/en/es/it — every locale aligned with the template (the
  no-translation-drift rule)
- [x] 1.9 Widget test: toggling from the transport takes effect without pausing
  or restarting the run

## 2. Focus state (spec: `music-kit-piece-focus`)

- [ ] 2.1 A piece identity to key focus on, derived from the existing kit model —
  reuse `DrumLane` identity and the kick rather than inventing a second notion of
  "one piece" (design D1). Pure, in `drum_kit.dart`
- [ ] 2.2 `player_data.dart`: the focus set in state, defaulting to every piece of
  the loaded score's kit, session-only, reset on load
- [ ] 2.3 Pure predicate: is this note's piece in focus? Unit-tested including the
  collapsed cases the kit model already decided (closed/open hi-hat, acoustic/
  electric snare, side stick) — one pad, one answer
- [ ] 2.4 Notifier actions: mute, solo (additive), unmute, clear. Muting the last
  remaining piece restores everything rather than emptying the session (design
  D2) — tested explicitly
- [ ] 2.5 Test the state is session-only: a selection does not survive loading
  another score, and does not survive a relaunch

## 3. Focus governs drawing, gating and scoring — from one source

- [ ] 3.1 `player_data.dart`: replace the percussion branches of `_showsNote`,
  `_showsRest`, `visibleTieContinuations` and `playableDrumSurfaces` with the
  focus predicate. `selectedHands` becomes keyboard-only; `hasHandsAndFeet` and
  the `hands`/`feet`/`handsAndFeet` summary branches go
- [ ] 3.2 Test the three-way agreement the limb filter established and this must
  keep: a piece out of focus is not drawn, not awaited by the Wait Mode gate, and
  not judged by the scorer — asserted from the one `visibleNotes` source
- [ ] 3.3 Muting the kick hides its **bar** (design D3) — the cascade and the pad
  strip both, since the bar is drawn by the cascade and the pedal by the strip
- [ ] 3.4 Test input is never filtered (the `add-drum-input-mapping` 4.5
  property, restated at the new grain): a stroke on an out-of-focus piece sounds,
  flashes its pad, and is not counted against the player in a scored run
- [ ] 3.5 Confirm the painters need no change beyond the note set they already
  receive — if one reads the hand selection directly, that is the bug to fix here

## 4. Focus control (UI)

- [ ] 4.1 `pre_play_setup_modal.dart`: replace the percussion limb section with
  the focus control, listing the loaded kit's pieces **in pad-strip order**;
  offered only for percussion and only when there is more than one piece
- [ ] 4.2 Mute and solo gestures; settle whether solo is a distinct gesture or a
  second control (design open question) and record the decision in `design.md`
- [ ] 4.3 Keyboard scores keep the hand selector untouched — asserted by a test,
  not by inspection
- [ ] 4.4 Localised fr/en/es/it, reusing the existing `kitPiece*` labels the pad
  strip already has
- [ ] 4.5 Riverpod layering: the modal calls notifier methods only; no service
  read from a widget; no provider invalidating a sibling

## 5. Scoring posture for a focus-restricted run

- [ ] 5.1 Settle the open question: is a run with pieces muted scored, and does it
  reach the leaderboards and the play rewards? Leaning — scored locally, not
  submitted, the posture practice sessions already have. Record the decision and
  its rationale in `design.md` before implementing
- [ ] 5.2 Implement the decision and test it from both sides: a full-kit run
  behaves exactly as today, and a restricted run does what §5.1 decided —
  asserted on the submission seam, not on a UI absence

## 6. Removing the limb selector

- [ ] 6.1 Delete the percussion reading of `Hand` and everything that served it:
  `handHands` / `handFeet` / the `hands` / `feet` / `handsAndFeet` summary
  branches / `coachPlayerLimbsTitle` / `coachPlayerLimbsBody`, in every locale
- [ ] 6.2 `setting_option_row.dart`: drop the `percussion` branch
- [ ] 6.3 `coach_copy.dart` + the coaching sequence: the limb step is replaced by
  the focus step, or removed if the control is self-evident — decide by looking
  at the finished control, not before
- [ ] 6.4 Update the tests that assert the hands/feet behaviour: they become the
  focus tests, not deletions — each existing assertion has a counterpart at the
  new grain, and any that does not is a behaviour being dropped on purpose and
  should be named here

## 7. Gates

- [ ] 7.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [ ] 7.2 `cd apps/music && flutter test --coverage --exclude-tags golden`,
  coverage ≥ 80%
- [ ] 7.3 Golden refresh if the settings surface or the pad strip moved
  (`flutter test --tags golden --update-goldens` on the pinned platform)
- [ ] 7.4 `openspec validate add-practice-focus-controls --strict`
- [ ] 7.5 **Ordering:** confirm `add-drum-kit-view` has been archived before this
  change is, or its percussion `hand-selection` text will never have reached the
  base spec this delta removes it from

## 8. Manual verification — with the tester

- [ ] 8.1 A groove with the crashes muted: they are not drawn, Wait Mode does not
  wait for them, and hitting one anyway sounds and flashes without penalty
- [ ] 8.2 Solo the hi-hat, then add the snare: exactly those two are asked for
- [ ] 8.3 Mute the kick: the bar is gone, and the groove is playable without it
- [ ] 8.4 Mute every piece in turn: the last one restores the full kit instead of
  leaving an empty session
- [ ] 8.5 The expected-notes mute during a Wait Mode session on the kit — the
  exercise is audible where it was masked, and the metronome still sounds
- [ ] 8.6 Confirm with him that focus reads the way he thinks about isolating a
  groove, and that nothing about the removed hands/feet control is missed
- [ ] 8.7 A keyboard score end to end: hand selection, gate and scoring all
  exactly as before
