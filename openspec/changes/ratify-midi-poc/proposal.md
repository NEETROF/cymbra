## Why

The cross-platform MIDI engine that powers the Cymbra music POC (real-time
NoteOn/NoteOff streaming, hot-plug, per-platform native wiring) has been
validated on macOS, iOS, Android, Linux and Windows but its behavior lives only
in code and comments. Capturing it as the first OpenSpec capability gives the
project a documented baseline that future changes diff against — and lets us
introduce the testability seam needed to meet the new 80% coverage gate.

## What Changes

- Ratify the existing **MIDI multi-platform** behavior as the baseline `midi`
  capability spec (no runtime behavior change).
- Extract the pure MIDI logic (`parse_midi`, `is_virtual_port`, port ordering)
  into a host-testable Rust module (`api/midi_core.rs`) and unit-test it.
- Introduce an injectable Dart seam (`MidiService` / `ScoreSource`) so
  `PlayerState` and the player UI can be tested without the native library.
- Add Rust unit tests, Flutter unit/widget tests, painter golden tests, and an
  end-to-end integration test so both ecosystems clear the 80% coverage gate.

## Capabilities

### New Capabilities
- `midi`: cross-platform MIDI input — port enumeration & ordering, port
  selection (auto/manual), connection-status reporting, real-time NoteOn/NoteOff
  streaming, hot-plug/unplug handling, and per-platform native integration.

### Modified Capabilities
<!-- None: this is the project's first capability. -->

## Impact

- Rust: `apps/music/rust/src/api/{midi.rs,midi_core.rs,score.rs,simple.rs}` —
  pure-logic extraction + tests; no public FFI surface change (no bridge regen).
- Dart: `apps/music/lib/services/midi_service.dart` (new seam),
  `lib/state/player_state.dart` and `lib/screens/player_screen.dart`
  (constructor injection); new `test/` suite + `integration_test/app_test.dart`.
- CI: 80% coverage gate (`rust.yml`, `flutter.yml`, `sonar.yml`,
  `sonar-project.properties`, `melos.yaml`). No behavior change for end users.
