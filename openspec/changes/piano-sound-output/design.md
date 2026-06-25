## Context

The engine is Rust (`apps/music/rust`) bridged to Flutter via flutter_rust_bridge.
Real-time MIDI already follows a proven pattern: hardware/thread glue in
`api/midi.rs` (excluded from coverage) over pure logic in `api/midi_core.rs`, with
a Flutter seam `services/midi_service.dart` exposed as `midiServiceProvider` and
faked in tests. All input converges on `Player.noteOn(pitch)`/`noteOff(pitch)`
(`state/player_notifier.dart:173`), and the score plays as `advance(dtMs)` moves
the playhead. The score derivation is currently declared "visual only — no audio".

This change adds a piano synthesizer the same way: a Rust audio module behind an
injectable Flutter seam, triggered from the existing convergence points. Because
on-screen taps (`playable-onscreen-keyboard`) and the computer keyboard already
land on `noteOn/noteOff`, hooking audio there sounds every source at once.

## Goals / Non-Goals

**Goals:**
- Polyphonic piano via a bundled SoundFont, rendered natively from Rust.
- Sound on every note-on/off from any source, during playback and when stopped.
- Audible score playback synced to the playhead (speed/tempo honored), with Wait
  Mode freezes not pre-sounding notes.
- Injectable, gracefully-degrading seam; no native dependency in unit tests.

**Non-Goals:**
- No MIDI **output** to external devices (internal synth only).
- No reverb/effects, multiple instruments, or per-note velocity curves beyond what
  the SoundFont provides (a future enhancement).
- No recording/export of audio.
- No synchronization scoring (owned by the future scoring capability).

## Decisions

### Decision: Rust-side synth — `rustysynth` + `cpal`
Use `rustysynth` (pure-Rust SoundFont synthesizer) rendering into a `cpal` output
stream on a dedicated audio thread. FFI exposes `audio_init(sf2_bytes)`,
`note_on(pitch, velocity)`, `note_off(pitch)`, `all_notes_off()`. **Why:** keeps
DSP in the engine (consistent with the architecture), one cross-platform code
path, authentic piano from an `.sf2`, and `rustysynth` is pure Rust (no C build).
**Alternatives considered:** Flutter `flutter_soloud`/`soundpool` (rejected: moves
DSP out of the engine); a hand-rolled oscillator (rejected: not a real piano).

### Decision: Lock-free event hand-off to the audio thread
FFI calls SHALL NOT touch the synth directly; they push events
(NoteOn/NoteOff/AllOff) onto a queue the audio callback drains each block. Keep
this queue/voice bookkeeping and the pitch/velocity mapping in `api/audio_core.rs`
(pure, host-testable); keep `cpal` stream setup and the FFI surface in
`api/audio.rs` (coverage-excluded, like `midi.rs`). **Why:** real-time audio
callbacks must not block or allocate; splitting pure logic out preserves the
≥80% coverage rule without testing the device.

### Decision: Single audio hook at the player's convergence points
Add `audioServiceProvider` and call `audioService.noteOn/noteOff` from
`Player.noteOn/noteOff`. For score playback, in `advance` detect onsets the
playhead crosses this tick → `audioService.noteOn`; track sounded notes and
`noteOff` when the playhead passes their end; issue `allNotesOff` on
stop/restart/seek. **Why:** one place sounds every input source and the score;
mirrors how feedback already derives from the same state. **Trade-off:** `advance`
gains note-edge bookkeeping; keep it in a small pure helper for testability.

### Decision: Score audio honors Wait Mode and visible hands
A note sounds only when the playhead actually advances past its onset, so a frozen
Wait Mode gate does not pre-sound the awaited note (it sounds when the user's press
releases the gate and time moves). Score auto-play SHALL sound only **visible**
notes — when a hand is hidden (`hand-selection-filter`), its notes are neither
shown nor sounded — keeping audio consistent with the practice model. **Why:**
matches the user's choice that a hidden hand is "not displayed and not awaited";
auto-accompanying the hidden hand is deferred (see Open Questions).

### Decision: Injectable seam + graceful degradation
`AudioService` abstract class with `FrbAudioService` (production) and a fake for
tests, exactly like `MidiService`. `audio_init` failures (no device, bad/missing
SoundFont) are caught; the service becomes a silent no-op and the app continues.
**Why:** CLAUDE.md testability rule and resilience — audio must never crash the
player.

## Risks / Trade-offs

- **Overlapping delta with `hand-selection-filter`** on `score-notation ›
  Derived Playback Timing` → both changes MODIFY this requirement. Mitigation:
  whichever is archived **second** must restate the merged requirement (drop
  "visual only" **and** keep the staff/hand content). Flag in both PRs; archive
  order: land one, rebase the other's delta before archiving.
- **Cross-platform audio backends** → `cpal` covers CoreAudio (macOS/iOS), WASAPI
  (Windows), ALSA (Linux); Android may need `oboe`/AAudio. Mitigation: gate the
  Android backend in `Cargo.toml` like the existing `midir` Android setup; verify
  on a device early. iOS audio session/category may need native config.
- **Latency** → on-screen/MIDI play must feel immediate. Mitigation: small `cpal`
  buffer, lock-free queue, no allocation in the callback; measure on device.
- **SoundFont asset size & license** → ship a **piano-only**, permissively-licensed
  `.sf2` (avoid 100 MB GM banks); document source/license. App-bundle size impact
  noted; consider compression/download-on-first-run only if too large (out of
  scope v1).
- **Hanging voices on hot paths** (seek/loop/restart) → always `all_notes_off` on
  those transitions; test the loop-at-end path.
- **Coverage** → pure logic in `audio_core.rs` and the `advance` note-edge helper
  are unit-tested; `audio.rs` added to the Rust ignore regex; Flutter tests use
  the fake `AudioService`.

## Migration Plan

Additive: new module, new seam, new asset, plus one requirement reworded. Default
build with audio available simply gains sound; if audio fails to init the app
behaves as today (silent). Public Rust API changes → run
`flutter_rust_bridge_codegen generate`. Rollback = remove the audio module/seam,
revert the `score-notation` wording, drop the asset and deps. Sequence after (or
alongside) `playable-onscreen-keyboard` so on-screen play is audible end-to-end.

## Open Questions

- **Hidden-hand accompaniment**: should score auto-play optionally sound the
  *hidden* hand as backing while the user plays the visible one? Deferred; current
  default sounds only visible notes. Revisit with product (ties into a future
  "play-along" mode).
- **Master volume / mute control** in the UI — likely wanted; add a transport
  control in a follow-up or a small task here. Decide during implementation.
- **Velocity from on-screen taps** (no pressure) — use a fixed sensible velocity;
  revisit if pointer pressure/Force is available.
- **SoundFont choice** (specific `.sf2`, license, size) — finalize in the asset
  task.
