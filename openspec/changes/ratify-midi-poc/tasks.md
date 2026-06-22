## 1. Rust: extract + test pure MIDI logic

- [x] 1.1 Create `api/midi_core.rs` with `parse_midi`, `is_virtual_port`, `sort_ports_virtual_last`
- [x] 1.2 Route `midi.rs` through `midi_core` (no public FFI change); register the module in `api/mod.rs`
- [x] 1.3 Unit-test `parse_midi` (note on/off, zero-velocity, CC, too-short), `is_virtual_port`, and port ordering
- [x] 1.4 Unit-test `score::demo_score` shape and timing, and `simple::greet`

## 2. Dart: testability seam

- [x] 2.1 Add `MidiService` / `ScoreSource` interfaces + FRB-backed implementations (`lib/services/midi_service.dart`)
- [x] 2.2 Inject the seam into `PlayerState` (default = real implementations)
- [x] 2.3 Accept an optional injected `PlayerState` in `PlayerScreen` for widget tests

## 3. Dart: tests

- [x] 3.1 Unit-test `PianoLayout` geometry
- [x] 3.2 Unit-test `PlayerState` (flatten/timing, note input, MIDI stream, status, wait-mode, controls) with a fake service
- [x] 3.3 Widget-test `PlayerScreen` (render, transport, mode toggle, keyboard fallback, indicator states, wait overlay)
- [x] 3.4 Golden-test the three painters + `shouldRepaint` (tagged `golden`, excluded from the cross-platform gate)
- [x] 3.5 Integration test driving the real app/FFI (`integration_test/app_test.dart`)

## 4. Coverage gate

- [x] 4.1 Wire `cargo-llvm-cov` (fail under 80%) into `rust.yml`; feed lcov to Sonar
- [x] 4.2 Wire `flutter test --coverage` + `very_good_coverage` (80%) and integration coverage into `flutter.yml`
- [x] 4.3 Uncomment Sonar lcov report paths; regenerate coverage in `sonar.yml`
- [ ] 4.4 Confirm both gates pass in CI (rust ≥80%, merged Flutter lcov ≥80%)

## 5. Validate the change

- [x] 5.1 `openspec validate ratify-midi-poc --strict`
- [ ] 5.2 After review, `openspec archive ratify-midi-poc` to fold the delta into `openspec/specs/midi/`
