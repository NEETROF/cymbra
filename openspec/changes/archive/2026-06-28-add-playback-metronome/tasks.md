## 1. Rust audio engine — metronome click

- [x] 1.1 In `rust/src/api/audio_core.rs`, add host-testable click synthesis: a pure
  function/struct that produces a short, self-terminating click sample buffer
  (enveloped tone or noise burst) with an `accent` parameter (distinct pitch/level
  for accented vs. normal). No cpal/thread dependency.
- [x] 1.2 Unit-test the click logic in `audio_core.rs`: accented buffer differs from
  normal, buffer is bounded/finite (decays to silence), no panics on repeated calls.
- [x] 1.3 In `rust/src/api/audio.rs`, add a `metronome_click(accent: bool)` FFI entry
  point that pushes a lightweight click event onto the audio thread's channel and
  mixes the click into the output buffer alongside the synth, independent of the
  loaded SoundFont. Keep it allocation-free on the audio thread.
- [x] 1.4 Regenerate the bridge: `flutter_rust_bridge_codegen generate` (updates
  `rust/src/frb_generated.rs` and the Dart bindings).
- [x] 1.5 `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
  and `cargo llvm-cov` pass with coverage ≥ 80% (click logic in `audio_core.rs`).

## 2. Dart audio seam

- [x] 2.1 Add `void metronomeClick({required bool accent})` to the `AudioService`
  interface in `lib/services/audio_service.dart`.
- [x] 2.2 Implement it in `FrbAudioService` by calling the new Rust
  `metronomeClick` binding; no-op safely when audio is unavailable (graceful
  degradation).
- [x] 2.3 Update/extend the fake audio service used in tests to record
  `metronomeClick(accent)` calls.

## 3. Player state — flag, beat detection, pulse

- [x] 3.1 In `lib/state/player_data.dart`, add `metronomeEnabled` (bool, default
  false) and a beat-pulse signal (`beatCount` int + `lastBeatAccent` bool, or a
  small beat-tick value object) to `PlayerData`; regenerate Freezed.
- [x] 3.2 Add a host-testable helper that, given `measureStartMs`, the time
  signature, and a half-open span `[from, to)`, returns the beat boundaries crossed
  and whether each is an accent (measure start = accent). Keep it pure for testing.
- [x] 3.3 In `lib/state/player_notifier.dart`, add `toggleMetronome()` to flip the
  flag (and persist it with the other player settings).
- [x] 3.4 In `advance()`, when `metronomeEnabled` and `isPlaying`, run the beat
  helper over the crossed span; for each boundary call
  `audioService.metronomeClick(accent: …)` and advance the beat-pulse signal in the
  new state. Skip emission on loop/seek seams (reset the beat tracker, no spurious
  tick), mirroring the existing `_silenceAll` handling.
- [x] 3.5 Ensure paused/stopped state emits no clicks or pulses, and that the
  enabled flag survives pause; reset the tracker on stop/seek so the next genuine
  boundary fires correctly.

## 4. UI — Tempo chip toggle and pulse

- [x] 4.1 In `lib/screens/player_screen.dart`, make the Tempo `_Chip` in `_TopBar`
  a tap target that calls `toggleMetronome()`; watch `metronomeEnabled`.
- [x] 4.2 Show an active/inactive style on the chip based on `metronomeEnabled`,
  keeping the existing `Tempo: $bpm` label.
- [x] 4.3 Watch the beat-pulse signal and animate a short pulse on the chip per
  beat (stronger pulse on accent); confirm it renders in all three modes
  (synthesia/staff/partition) since the chip is in the shared header.

## 5. Tests

- [x] 5.1 Unit-test the beat-boundary helper (3.2): correct beats per measure,
  accent on downbeats, no boundary fired across a loop/seek seam, multiple
  boundaries in one span each fire once.
- [x] 5.2 Notifier tests (fake audio via `ProviderScope`/`ProviderContainer`
  override): advancing across beats with the metronome enabled records the expected
  `metronomeClick(accent)` calls and updates the pulse; paused/stopped records none;
  toggle flips the flag and persists.
- [x] 5.3 Widget test: tapping the Tempo chip toggles state and updates the chip's
  active style; chip pulses on a beat-pulse change.
- [x] 5.4 Confirm graceful degradation: with audio unavailable the visual pulse
  still updates and nothing crashes.

## 6. Validation

- [x] 6.1 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`,
  then `melos run analyze`, `dart format`, and `dart run custom_lint` clean.
- [x] 6.2 `flutter test --coverage --exclude-tags golden` passes with Flutter line
  coverage ≥ 80%.
- [x] 6.3 Manually verify on macOS in all three modes: tap Tempo → audible click +
  chip pulse on each beat, accent on the downbeat, in sync with the notes; pause →
  silent; resume → back in sync; speed change → beats track the notes.
- [x] 6.4 `openspec validate add-playback-metronome --strict` passes.
