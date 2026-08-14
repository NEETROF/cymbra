## Why

Cymbra has no notion of *where* its sound goes. The engine opens
`host.default_output_device()` once at startup and everything the app produces —
played notes, score playback, metronome clicks, SoundFont preview clips — leaves
through that one stream, on whatever the OS happens to call the default.

That is wrong in two different ways, and the second one is the expensive one.

**The user cannot choose.** Someone with a digital piano that accepts USB audio,
an audio interface, or simply a headset that is not the system default has no way
to say so from inside Cymbra. On mobile they cannot even see which route is
active.

**The app duplicates a sound the instrument already made.** When a real digital
piano is connected over USB MIDI, it produces its own sound the instant a key is
pressed — with its own samples and zero latency. Cymbra then synthesizes the
*same* note a second time, through its own SoundFont, arriving 10–30 ms later.
The user hears a flam. Today the only workaround is turning the piano's volume
down, which throws away the better sound to keep the worse one.

Routing also determines whether the app is playable at all. A Bluetooth speaker
adds 100–300 ms; that is fine for listening to a piece and unusable for playing
one. The app currently offers no warning and no compensation, so a wireless route
reads as "Cymbra feels broken".

## What Changes

- **Instrument-sounds-itself mode**: when enabled, notes arriving **from the
  connected MIDI instrument** are no longer synthesized by the app — the
  instrument's own sound is the sound. This is a **per-source rule, not a mute**:
  - notes from the MIDI device → **not** synthesized (the piano already sounded them)
  - notes from the **on-screen keyboard** and the computer-keyboard fallback →
    **synthesized as today** (the instrument cannot know about them)
  - score playback, metronome clicks, SoundFont preview clips and any other app
    audio → **unchanged**, always through the app's output
- **Explicit output selection on desktop**: enumerate the host's audio outputs
  (macOS/Windows/Linux), let the user pick one, persist the choice, and rebuild
  the audio stream on it. This is what routes Cymbra to a piano in USB-audio
  mode, an interface, or a specific headset.
- **Active route on mobile**: on iOS/Android the app cannot pick a device, so it
  surfaces the operating system's route picker and displays the currently active
  route (device speaker, headphones, Bluetooth, USB).
- **Latency awareness**: detect and label a wireless route, warn that it is
  unsuitable for playing (as opposed to listening), and offer an adjustable
  **output offset** that shifts the visual playhead and the scoring reference so
  they line up with the sound the user actually hears.
- **New "Sound output" section** in the player settings drawer and the pre-play
  setup modal, grouping the route, the instrument-sounds-itself toggle and the
  offset.
- Graceful degradation is preserved: a chosen device that has disappeared falls
  back to the system default rather than leaving the app silent.

## Capabilities

### New Capabilities

- `audio-output-routing`: choosing and reporting where the app's audio goes —
  desktop output-device enumeration and selection, mobile system-route
  presentation and reporting, persistence, fallback when a device disappears,
  wireless-route detection and the output offset applied to the playhead and
  scoring reference.

### Modified Capabilities

- `audio-output`: the engine gains an output-device selection surface instead of
  binding to the system default forever, and live note sounding becomes
  source-aware — the "every note-on sounds, regardless of input source"
  requirement is narrowed by the instrument-sounds-itself rule, while score
  playback, metronome and preview clips stay unconditional.
- `performance-scoring`: the scoring reference is shifted by the configured
  output offset, so a user following delayed audio is not scored as late.
- `pre-play-setup`: the setup modal gains the sound-output section (route,
  instrument-sounds-itself, offset) next to the existing MIDI device section.

## Impact

- **Rust engine** (`apps/music/rust`): `api/audio.rs` stops hard-coding
  `default_output_device()`; new public API to list outputs, select one and
  report the active one, with the stream rebuilt through the existing
  `AudioCommand` channel. Selection/fallback logic goes in host-testable
  `audio_core.rs`. **Public API change → `flutter_rust_bridge_codegen`.**
- **Flutter app** (`apps/music`): source-tagged note entry points in
  `player_notifier.dart` (today `noteOn`/`noteOff` converge and lose the source),
  a new audio-routing service seam + notifier, the settings section in the player
  drawer and `pre_play_setup_modal.dart`, persistence through the existing
  preferences store, fr/en strings.
- **Native platform code**: iOS/macOS route picker presentation and route
  reporting; Android `AudioManager` route reporting and picker.
- **Scoring**: the offset shifts the time reference the notifier passes to the
  scorer. `performance_scoring_core.dart` is a pure module over numbers and is
  **not** modified.
- **No backend change**: entirely device-local, nothing crosses gRPC.
