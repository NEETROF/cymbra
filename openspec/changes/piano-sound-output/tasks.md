## 1. Asset & dependencies

- [x] 1.1 Vendor **Upright Piano KW** (CC0, ~27 MiB) as `apps/music/assets/`'s single bundled `.sf2`; copy the file into the repo (no runtime tie to its source) and record source + CC0 license. Register it in `pubspec.yaml` assets.
- [x] 1.2 Add `rustysynth` and `cpal` to `apps/music/rust/Cargo.toml`; gate the Android audio backend (`oboe`/AAudio) under `[target.'cfg(target_os = "android")']` like the existing `midir` setup.

## 2. Rust engine: synth + output

- [x] 2.1 `api/audio_core.rs` (pure, host-testable): event types (NoteOn/NoteOff/AllOff), a lock-free-friendly event queue model, MIDI pitch/velocity mapping, and voice bookkeeping.
- [x] 2.2 `api/audio.rs` (FFI + thread glue, coverage-excluded): create a `cpal` output stream, instantiate the `rustysynth` synthesizer from the SoundFont bytes, drain the event queue in the audio callback (no allocation/locks in the callback).
- [x] 2.3 FFI surface: `audio_init(sf2_bytes)`, `note_on(pitch, velocity)`, `note_off(pitch)`, `all_notes_off()`. Register in `api/mod.rs`.
- [x] 2.4 `flutter_rust_bridge_codegen generate` (public API changed); commit generated Dart bridge is gitignored — ensure it builds.
- [x] 2.5 Unit-test `audio_core.rs`: pitch/velocity mapping, queue ordering, voice add/remove, all-notes-off clears voices.

## 3. Flutter seam

- [x] 3.1 `services/audio_service.dart`: `audioServiceProvider` (`@riverpod`), abstract `AudioService` (`init`, `noteOn`, `noteOff`, `allNotesOff`), `FrbAudioService` production impl forwarding to the bridge, mirroring `midi_service.dart`.
- [x] 3.2 Graceful degradation: catch init/SoundFont failures; the service becomes a silent no-op; surface nothing fatal to the UI.
- [x] 3.3 Load the SoundFont asset bytes and call `init` at startup (through an injectable source so tests don't touch the bundle).

## 4. Player wiring

- [x] 4.1 Call `audioService.noteOn/noteOff` from `Player.noteOn`/`noteOff` so every input source sounds (during playback and when stopped).
- [x] 4.2 In `advance`, add a pure note-edge helper: detect onsets crossed this tick → `noteOn`; track sounded notes and `noteOff` at their end; honor the speed multiplier.
- [x] 4.3 Wait Mode: ensure a frozen onset does not pre-sound; the note sounds only once the playhead advances past it.
- [x] 4.4 Sound only **visible** notes when a hand is hidden (compose with `hand-selection-filter`); issue `allNotesOff` on stop/restart/seek/loop.

## 5. Tests

- [x] 5.1 Override `audioServiceProvider` with a recording fake; assert player `noteOn/noteOff` (all sources) drive `audioService` calls.
- [x] 5.2 Score playback: advancing across onsets calls `noteOn`/`noteOff` at the right times; speed changes spacing; stop/restart triggers `allNotesOff`.
- [x] 5.3 Wait Mode frozen does not sound the awaited note until released; hidden-hand notes are not sounded.
- [x] 5.4 Graceful degradation: a failing audio service leaves visuals/feedback/Wait Mode working and does not crash.

## 6. Verify & gate

- [x] 6.1 Add `audio.rs` to the Rust coverage ignore regex (alongside `midi.rs`/`musicxml.rs`); `cargo llvm-cov --workspace --fail-under-lines 80` passes; `cargo fmt`/`clippy` clean.
- [x] 6.2 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`; `melos run analyze`, `dart format`, `dart run custom_lint` clean.
- [x] 6.3 `flutter test --coverage --exclude-tags golden` green and Flutter line coverage ≥ 80%.
- [x] 6.4 Manually confirm on macOS + one mobile device (and Linux desktop): keys (on-screen/MIDI) sound immediately; the score plays audibly; speed and stop behave; no hanging voices.
- [x] 6.5 `openspec validate piano-sound-output --strict` passes.
- [x] 6.6 Manually confirm on **Windows desktop**: keys (on-screen/MIDI, incl. hot-plug after launch and song re-entry) sound immediately; the score plays audibly; speed and stop behave; no hanging voices.
