## 1. Flag and scaffolding

- [x] 1.1 Define the server flag (staff + beta campaign audience) and the
  default-`false` provider with the `main.dart` remote override, mirroring
  `drumsEnabled` (`apps/music/lib/state/drums_access.dart`)
- [x] 1.2 Scaffold `api/audio_input.rs` (glue) + `api/audio_input_core.rs`
  (pure core) with the `#[frb(ignore)]` internal-types pattern; add
  `/audio_input\.rs` to the coverage exclusion regex in
  `.github/workflows/rust.yml` and `sonar.yml`
- [x] 1.3 Run `flutter_rust_bridge_codegen generate` and wire the empty FFI
  surface end to end (a no-op capture start/stop reachable from Dart)

## 2. Capture foundation (Rust)

- [x] 2.1 Implement cpal input-device enumeration, capture start/stop, and
  the capture thread handing fixed-size frames to the core; lifecycle bound
  to explicit start/stop calls only
- [x] 2.2 Implement input route classification in the core (built-in / wired
  / USB / Bluetooth / other from platform descriptors, never display names)
  with unit tests, including the unknown-kind degrade case
- [x] 2.3 Implement the Bluetooth refusal in the core (route accepted /
  refused verdict + reason) with unit tests; glue surfaces the verdict to
  Dart
- [x] 2.4 Implement the calibration state machine in the core (armed → click
  emitted → detected(latency) | timeout) with unit tests over synthetic PCM;
  glue emits the reference click through the existing output path

## 3. Platform capture configuration

- [x] 3.1 iOS: `NSMicrophoneUsageDescription`, `AVAudioSession`
  `.measurement` mode scoped to capture sessions; verify play+record does not
  disturb the existing output route handling
- [x] 3.2 macOS: `com.apple.security.device.audio-input` entitlement +
  usage description; verify the entitlement survives the **store archive
  export re-signing** (memory: export previously stripped entitlements)
- [x] 3.3 Android: `RECORD_AUDIO` permission, probe
  `PROPERTY_SUPPORT_AUDIO_SOURCE_UNPROCESSED`, select `UNPROCESSED` else
  `VOICE_RECOGNITION`; record the obtained configuration and expose it to
  diagnostics
- [x] 3.4 Desktop Linux/Windows: default input device capture (no permission
  prompt); confirm graceful no-input-device behavior

## 4. Dart capture seam and state

- [x] 4.1 `AudioCaptureService` seam (permission state, routes, refusal,
  calibration, lifecycle) + mockito-generated mock; provider following the
  house pattern
- [x] 4.2 Permission flow state: contextual rationale before the OS prompt,
  denied → guidance with settings pointer; localized fr/en
- [x] 4.3 Calibration notifier + UI flow (run, result, re-run, per-route
  storage keyed like the output selection, invalidation on route change);
  localized failure guidance; widget tests over the mocked seam
- [x] 4.4 Ship calibration results through the existing diagnostics/usage
  path (D7 fleet probe)

## 5. Piano detection

- [x] 5.1 Onset detector in `audio_input_core.rs` (attack-transient
  timestamping) with unit tests over synthetic and recorded-fixture PCM
- [x] 5.2 Score-informed pitch-presence stage (expected-set evaluation,
  detuning-tolerant bands, evidence accumulation post-onset) with fixture
  tests: single notes, sparse chords, damper-sustained repeats
- [x] 5.3 Expected-set feed: the played score's active window (playhead
  vicinity / open Wait gate pitches) crossed into the engine; emitted events
  carry onset timestamps and enter `midi_event_stream()`
- [x] 5.4 Metronome-click rejection in the detector (known transient masked
  by construction); test with click overlaid on note fixtures

## 6. Source selection and no-echo

- [x] 6.1 `PlayerInputSource` (MIDI | microphone) state above the seam;
  microphone visible only under the flag; MIDI port memory untouched by
  switching; unit tests
- [x] 6.2 Input-status surfaces show the microphone source + route where the
  MIDI port shows today; localized fr/en
- [x] 6.3 Detected-note events bypass synthesis (no `MidiEcho` for the
  microphone source — inherent, independent of the MIDI-scoped setting);
  on-screen keyboard still sounds; test both
- [x] 6.4 Capture lifecycle bound to consuming features: starts with an
  audio-sourced session/calibration, stops on end/background; never runs
  under other sources; widget/notifier tests

## 7. Scoring integration

- [x] 7.1 Stamp the input source on the immutable session record; unit tests
- [x] 7.2 Apply the measured input offset to audio-sourced attack timestamps
  before judgment (`performance_scoring_core.dart`); MIDI runs bit-identical;
  unit tests both ways
- [x] 7.3 Exclude the sustain dimension for audio-sourced runs and
  redistribute its weight (percussion precedent); unit tests
- [x] 7.4 Free-run gate: scored free-run with the microphone requires a
  stored, window-compatible measurement; otherwise steer to Wait Mode /
  calibration with localized copy; widget tests of both branches
- [x] 7.5 Wait Mode path end-to-end with the mocked capture seam: gate
  satisfaction from detected events, uncalibrated route allowed

## 8. Verification

- [x] 8.1 `melos run analyze`, `dart run custom_lint`, `dart format` (repo
  root), `cargo fmt --all --check`, `cargo clippy --workspace --all-targets
  -- -D warnings`
- [x] 8.2 Coverage ≥ 80 % both ecosystems with the new exclusions in place
- [x] 8.3 `openspec validate add-acoustic-piano-input --strict` passes
- [ ] 8.4 Manual on-device pass (staff flag): calibration on iPhone + one
  Android device against a real acoustic piano; Wait Mode session plays; BT
  mic route correctly refused; free-run gate honest on both devices;
  store archive export keeps the macOS `audio-input` entitlement
