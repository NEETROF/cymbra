## Context

The MIDI engine is a Rust crate (`rust_lib_music`, using `midir 0.11`) exposed to
Flutter via `flutter_rust_bridge 2.12.0`. A single background watcher thread polls
ports every ~700 ms, (re)connects to the desired port, and streams NoteOn/NoteOff
events into a `StreamSink`. Each platform needs specific native wiring for hot-plug
and device access. This change documents that behavior and makes it testable to
satisfy the project's new 80% coverage gate.

## Goals / Non-Goals

**Goals:**
- Capture current MIDI behavior as the baseline `midi` capability spec.
- Make the genuinely testable logic unit-testable on headless CI hosts.
- Make `PlayerState` and the player UI testable without a native library.

**Non-Goals:**
- No change to the public FFI surface (no `flutter_rust_bridge` regeneration).
- No new MIDI features (output, CC, program change remain out of scope).
- Player-UI and score-model are not specified here (future capabilities).

## Decisions

- **CoreMIDI refresher client on the main run loop (macOS/iOS).** CoreMIDI only
  delivers configuration-change (hot-plug) notifications to the main run loop, so
  an otherwise-empty `MIDIClientCreateWithBlock` is created in `AppDelegate`.
  Alternative (polling from the Rust thread alone) does not observe post-startup
  device changes.
- **`System.loadLibrary` + `JNI_OnLoad` (Android).** `flutter_rust_bridge` loads
  the lib via `dlopen`, which does not trigger `JNI_OnLoad`; an explicit
  `System.loadLibrary` in `MainActivity` initializes `ndk_context` for midir's
  AMidi backend (minSdk 29).
- **700 ms watcher poll + virtual-port-last ordering.** Simple, dependency-free
  hot-plug detection; auto mode skips virtual "Through"/rtpMIDI/network ports.
- **Extract pure logic to `api/midi_core.rs`.** `parse_midi`, `is_virtual_port`
  and port ordering move to a sibling module (private `pub(crate)`, so the FFI
  surface is unchanged) where they can be unit-tested and measured. The thread/IO
  glue in `midi.rs` is excluded from the coverage gate as it needs hardware.
- **Injectable Dart seam (`MidiService` / `ScoreSource`).** `PlayerState` takes
  these via its constructor (defaulting to the real FFI-backed implementations),
  so unit/widget tests inject fakes and run on the Dart VM with no native lib.
  `PlayerScreen` accepts an optional injected `PlayerState` for widget tests.

## Risks / Trade-offs

- Golden tests are platform-sensitive → tagged `golden` and excluded from the
  cross-platform CI gate; painters are still covered by widget tests.
- The MIDI thread/IO code cannot run on headless CI → excluded from the Rust
  coverage measurement (documented in `rust.yml`), with all pure logic measured.
- Integration tests need a real native build under Xvfb → run as a separate,
  slower CI concern; locally on desktop (`-d macos`).
