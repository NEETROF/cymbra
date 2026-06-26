## Why

Cymbra is silent: the score derivation is explicitly "visual only — no audio
synthesis or MIDI output", and no input source makes a sound. A learner pressing
keys (on-screen, computer keyboard, or MIDI) hears nothing, and the piece never
plays back audibly. Adding a real piano sound — triggered both by the keys the
user plays and by the score during playback — makes practice far more useful and
musical.

## What Changes

- Add **audio output** to the Rust engine: a polyphonic **SoundFont piano
  synthesizer** (`rustysynth`) rendered to the system audio device via `cpal`,
  exposed through a small FFI seam (`note_on`/`note_off`/`all_notes_off`).
- **Every key press makes a sound**: because all input converges on the player's
  note-on/note-off, a single hook there sounds a piano note for **any** source —
  on-screen keyboard, computer keyboard, or MIDI — both during playback and when
  stopped.
- **The score plays audibly**: during playback the app sounds each note as the
  playhead reaches its onset and releases it at its end, honoring the speed/tempo
  and the Wait Mode freeze (frozen notes do not pre-sound).
- Expose an injectable `AudioService` (Riverpod provider) so state/widgets are
  testable with a fake, and **degrade gracefully** — if audio init or the
  SoundFont fails, the app keeps working silently rather than crashing.
- Bundle a small, permissively-licensed piano **SoundFont (.sf2)** asset.

This **lifts the prior "visual only" limitation** of the derived playback timing.

## Capabilities

### New Capabilities
- `audio-output`: polyphonic SoundFont piano synthesis in the Rust engine,
  triggered by every note-on/note-off (any input source) and by the score during
  playback, behind an injectable, gracefully-degrading audio seam.

### Modified Capabilities
- `score-notation`: the **Derived Playback Timing** requirement no longer states
  the derivation is "visual only / no audio" — the same derived timing now also
  feeds the audio synthesizer (which renders the sound).

## Impact

- **Rust engine** (public API → re-run `flutter_rust_bridge_codegen`):
  - new `apps/music/rust/src/api/audio.rs` — `cpal` output + `rustysynth`
    synth on the audio thread, FFI `audio_init`/`note_on`/`note_off`/
    `all_notes_off` (hardware/thread glue, coverage-excluded like `api/midi.rs`).
  - new `apps/music/rust/src/api/audio_core.rs` — pure, host-testable logic
    (event queue, MIDI pitch/velocity mapping, voice bookkeeping), mirroring
    `midi_core.rs`.
  - `Cargo.toml`: add `rustysynth` + `cpal` (Android may need `oboe`).
- **Flutter**:
  - new `apps/music/lib/services/audio_service.dart` — `audioServiceProvider`
    + abstract `AudioService` seam + `FrbAudioService` production impl + fake for
    tests (mirrors `midi_service.dart`).
  - `state/player_notifier.dart` — call `audioService.noteOn/noteOff` from the
    player's `noteOn`/`noteOff`; in `advance`, sound/release score notes as the
    playhead crosses their onsets/ends; `all_notes_off` on stop/restart/seek.
- **Assets**: bundle a piano `.sf2` (size/licensing noted in design);
  `pubspec.yaml` asset entry.
- **CI/coverage**: add `audio.rs` to the Rust coverage ignore regex; Flutter
  coverage already excludes `lib/src/rust/**`; keep audio behind the seam so unit
  tests run without the native lib.
- **Interactions**: composes with `playable-onscreen-keyboard` (its taps now
  sound) and `wait-on-keypress` (frozen notes stay silent until played). Overlaps
  `hand-selection-filter` on the `score-notation` Derived-Playback-Timing
  requirement — reconcile at archive (see design Risks).
