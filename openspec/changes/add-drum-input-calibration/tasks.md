## 1. One MIDI stream, many readers (prerequisite — D1)

- [x] 1.1 The fan-out lives in **`FrbMidiService.events`**, not in a new
  provider: one process-wide lazily-created broadcast stream over the single
  `midi_event_stream()` subscription. Better than the planned provider — no
  consumer changes at all, no provider-shape change, and the fix sits exactly
  where the fault is (the adapter treating a single-sink engine as
  multi-subscriber). Test fakes are unaffected; theirs is already broadcast
- [x] 1.2 ~~Convert every consumer~~ — unnecessary under 1.1's shape: all six
  keep calling `events()` and are simply no longer in competition
- [ ] 1.3 Test the caching directly. `events()` reaches the bridge, so this needs
  the opener behind a seam (a `@visibleForTesting` injection point) before it can
  be asserted on the VM — currently covered only by 3.7 at the consumer level and
  by the on-device pass
- [x] 1.4 Regression guard at the consumer level: the monitor and the player both
  receive the same event (`midi_monitor_screen_test.dart`, "observing does not
  disturb the player") — the property that was impossible before

## 2. Channel on the wire (spec: `midi`)

- [x] 2.1 `api/midi.rs`: `MidiEvent` carries the transmitting `channel`.
  Interpretation stays channel-agnostic — reported, never used to accept or
  reject (the property `midi_core.rs:312` already pins for channel 6 and
  `add-drum-input-mapping` 5.1 pins for channel 10)
- [x] 2.2 `midi_core.rs`: decode the channel from the status byte; unit-test it
  across channels 1, 7 and 10, asserting the rest of the event is byte-identical
  to what it is today
- [x] 2.3 `flutter_rust_bridge_codegen generate` (public API change) and update
  the Dart-side `MidiEvent` construction in test fakes

## 3. The monitor (spec: `music-midi-input-monitor`)

- [x] 3.1 A monitor notifier over the shared stream: a bounded ring of the last N
  events (fixed, small), each carrying the raw number, velocity, channel, kind
  and arrival time. Pure enough to unit-test without a screen
- [x] 3.2 Resolution, as a pure function beside the kit model: number → General
  MIDI percussion name (or note name for a keyboard score) → the loaded score's
  kit piece, or explicitly nothing. Unit-tested including the "matches nothing"
  case, which is the one the beta report is about
- [ ] 3.3 "Will not sound": the resolution reports when a number falls outside
  the loaded SoundFont's **sampled range**. Deferred — it needs the range out of
  the engine, and hard-coding the shipped kit's measured 27–87 is exactly the
  guess this task exists to avoid. What shipped instead is a statement about the
  **standard**: a number outside the General MIDI percussion map (35–81) is
  reported as unknown to the app, which is true, useful, and the actual shape of
  the beta report
- [x] 3.4 The monitor screen: a live list, newest first, each row showing raw
  number / resolved piece / velocity / channel, with unmatched rows visually
  distinct. Reachable from the settings modal's MIDI section, as its **own
  screen** (the modal pauses playback, which is wrong for a surface whose point
  is watching live input — design open question, settle here)
- [x] 3.5 Empty and no-device states, localised (fr/en/es/it — every locale
  aligned with the template, per the `flutter-riverpod-architecture` skill)
- [x] 3.6 Widget tests: an event appears; a stroke on an unmatched number is
  shown as unmatched; the ring drops the oldest; opening with no score works;
  opening with no device says so
- [x] 3.7 Assert the monitor changes nothing: with the monitor mounted, a stroke
  still sounds, flashes and scores exactly as it does without it (the property
  §1 exists to make true)
- [x] 3.8 Riverpod layering: the screen reads notifier state only, no service
  call from a widget, no provider invalidating a sibling

## 4. The mapping model (spec: `music-drum-input-mapping`)

- [x] 4.1 A pure `DrumInputMapping` value: piece → number, with the identity
  translation for anything unmapped. `translate(int) -> int`, total and
  order-independent. Unit-tested, including that an empty mapping is exactly the
  identity
- [x] 4.2 Conflict detection as a pure predicate: a number already claimed by
  another piece. Tested against the reassignment case (the same piece being
  re-recorded is not a conflict with itself)
- [x] 4.3 Serialisation to/from the preferences store, keyed by port name; an
  unreadable or absent value is "no mapping", never an error and never another
  device's mapping (spec: `local-preferences`)

## 5. One translation seam (D2)

- [x] 5.1 `player_notifier.dart`: translate a live event **once**, where it
  enters, before anything interprets it. Nothing downstream learns the mapping
  exists — assert that by leaving every existing input test unchanged
- [x] 5.2 `api/midi.rs` + `audio_core.rs`: the engine holds the table (behind
  the lock the echo mode already uses) and the MIDI callback translates before
  it echoes. Pure decision unit-tested in `audio_core.rs` beside `echo_event`
- [x] 5.3 `set_midi_mapping` on the bridge, plus `flutter_rust_bridge_codegen
  generate`. The app owns the policy and **pushes** it on every input that can
  change it — device change, calibration completing, an edited entry, leaving
  the player — idempotent, so over-calling is safe (the `_applyEcho` shape)
- [x] 5.4 Test source convergence under a mapping: the same physical stroke
  produces one interpretation across sound, flash, gate and score. Specifically
  assert the echo sounds the *translated* number — the failure D2 exists to
  prevent
- [x] 5.5 Test the no-mapping path is byte-identical to today for a percussion
  score AND a keyboard score — the keyboard half **found a bug**: the first
  version translated every live event, so a calibrated kit transposed a piano
  score's pitches. The seam is percussion-only now (design D8), and the engine is
  pushed the same answer so the two cannot drift. The percussion half is asserted
  by the 2019 pre-existing tests continuing to pass unchanged, which is also what
  5.1 asks for

## 6. The calibration pass (spec: `music-drum-input-mapping`)

- [x] 6.1 A calibration notifier: an ordered list of pieces to learn, the current
  step, the recorded numbers so far, and terminal states (completed / abandoned).
  Pure state machine, unit-tested
- [x] 6.2 Each step records the **next** stroke: a stroke stamped before the step
  armed is discarded (D4 — the same stale-stamp lesson as the Wait-Mode fix).
  Test explicitly with a stroke delivered before the step begins
- [x] 6.3 A conflict is surfaced, not silently resolved: the step reports it and
  offers strike-again or reassign. Tested both ways
- [x] 6.4 Skipping a step records nothing and moves on; abandoning the pass
  leaves the stored mapping exactly as it was (tested — nothing is written until
  the pass completes)
- [x] 6.5 Strokes stay audible throughout the pass (the player must hear that the
  instrument reaches the app at all) — made **explicit** rather than inherited:
  the pass keeps the player alive for its duration (`ref.listen(playerProvider)`,
  a dependency without a rebuild), so audibility does not depend on which route
  happens to be underneath. Writing the test is what exposed the difference
- [x] 6.6 The calibration screen: one piece at a time, named and illustrated,
  with skip / back / abandon. Localised fr/en/es/it
- [x] 6.7 Which pieces to offer: the fixed standard kit rather than the loaded
  score's (design open question — a mapping calibrated once should serve every
  score). Settle and record the decision in `design.md` — **settled as design
  D7**: `kCalibrationPieceOrder` in `drum_kit.dart`, round the kit as a drummer
  sits at it, reached from the settings rather than from a score
- [x] 6.8 Widget tests for the flow end to end: calibrate three pieces, skip one,
  complete, and assert the stored mapping is exactly what was played

## 7. Reviewing and editing the mapping

- [x] 7.1 A table view of the current device's mapping: piece → number, with the
  General MIDI name of what that number would otherwise have meant
- [x] 7.2 Edit one entry (pick a piece, strike a pad or type a number) and clear
  one entry, without re-running the pass
- [x] 7.3 Clear the whole mapping — the device returns to uncalibrated behaviour,
  asserted by an input test rather than by the absence of a row
- [x] 7.4 The monitor shows the applied translation: raw → mapped when they
  differ, one number when they do not (spec requirement, widget-tested)

## 8. Gates

- [x] 8.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [x] 8.2 `cd apps/music && flutter test --coverage --exclude-tags golden`,
  coverage ≥ 80% — 2061 tests green; the new files are 94–100 %
  (`drum_input_mapping` 95.8, its store 93.9, `drum_calibration` 96.5, its
  notifier 95.8, the screen 96.2, `midi_monitor` 100, `player_notifier` 99.2).
  The 80 % gate itself is CI's merged unit+integration lcov
- [x] 8.3 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets
  -- -D warnings`
- [x] 8.4 `cargo llvm-cov --workspace --fail-under-lines 80` with the repo's
  usual ignore regex — **88.70 %** overall, 79 engine tests green
- [x] 8.5 `flutter_rust_bridge_codegen generate` ran and the bridge is in sync
- [x] 8.6 `openspec validate add-drum-input-calibration --strict`

## 9. Manual verification — on the tester's kit

- [ ] 9.1 Open the monitor and strike every pad: record the numbers the module
  actually sends, and identify the one behind the "tom is not audible" report.
  Write the observed map into `design.md` — it is the first real e-kit we have
  numbers for
- [ ] 9.2 Confirm the unmatched and will-not-sound markers are accurate against
  what the player hears
- [ ] 9.3 Run the calibration pass on the kit; confirm each step records the pad
  actually struck (watch for cross-triggering on rim shots — tune here and record
  what was needed)
- [ ] 9.4 After calibrating: the previously silent piece sounds, flashes its pad,
  releases the Wait-Mode gate and scores — all four, on the same stroke
- [ ] 9.5 Disconnect the kit and connect a keyboard: the keyboard is untouched by
  the mapping, and the monitor reports note names
- [ ] 9.6 Relaunch with the kit connected: the mapping is still in force with no
  recalibration
- [ ] 9.7 Confirm input latency is unchanged from the §8 baseline of
  `add-drum-input-mapping` — the translation runs in the MIDI callback and must
  not be perceptible
