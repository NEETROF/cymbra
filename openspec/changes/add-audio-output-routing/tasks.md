## 1. Source-aware sounding (independently shippable)

- [x] 1.1 Add a `NoteSource` enum (`midiDevice` | `onScreen` | `computerKeyboard`)
  and thread it through `Player.noteOn`/`noteOff` with a default that keeps
  existing call sites compiling
- [x] 1.2 Tag the six call sites: MIDI stream
  ([player_notifier.dart:247/249](apps/music/lib/state/player_notifier.dart#L247)),
  on-screen keyboard
  ([player_screen.dart:209/215](apps/music/lib/screens/player_screen.dart#L209)),
  computer-keyboard assist
  ([player_screen.dart:158/165](apps/music/lib/screens/player_screen.dart#L158))
- [x] 1.3 Add the `instrumentSoundsItself` preference (Freezed state + persisted
  through the existing preferences store)
- [x] 1.4 Gate only the synth call:
  `synthesize = !(instrumentSoundsItself && source == midiDevice)` — scoring, key
  feedback and Wait Mode stay unconditional for every source
- [x] 1.5 Verify score playback (`_audio.noteOn` at `:162`), metronome clicks and
  preview clips are untouched by the gate
- [x] 1.6 Disable the setting with a stated reason while no MIDI port is connected

## 2. Engine: selectable output device

- [x] 2.1 Move the device choice out of `run_audio_thread`
  ([audio.rs:234](apps/music/rust/src/api/audio.rs#L234)): resolve a requested
  name, fall back to the system default, and report what is actually in use
- [x] 2.2 Put the pure resolution/fallback logic in `audio_core.rs` (requested
  name → available names → chosen name) so it is host-testable
- [x] 2.3 Add an `AudioCommand` variant to rebuild the stream on a new device:
  all-notes-off first, rebuild `Stream` + `Synthesizer` on the new sample rate,
  keep the SoundFont in memory (no reload)
- [x] 2.4 Add the public API — `list_audio_outputs()`,
  `set_audio_output(name: Option<String>)`, `active_audio_output()` — and keep
  the failure policy: a device that will not open leaves the current stream alive
- [x] 2.5 Run `flutter_rust_bridge_codegen generate` (public API changed)
- [x] 2.6 Rust tests on `audio_core.rs`: absent remembered device → default;
  unknown name → default; empty device list; default-follows-`None`
- [x] 2.7 Keep `cargo llvm-cov --workspace --fail-under-lines 80` green

## 3. Platform route reporting (mobile)

- [x] 3.1 Define the `{name, kind}` route shape with `kind` ∈ `builtin |
  headphones | bluetooth | usb | other`, unknown port types mapping to `other`
- [x] 3.2 iOS: read `AVAudioSession.currentRoute` for name + port type, present
  `AVRoutePickerView`, and push route-change notifications to Dart
- [x] 3.3 macOS/desktop: report the active cpal device as the route (kind derived
  from the device, `other` when unknown) so the UI has one shape everywhere
- [x] 3.4 Android: read the active output device from `AudioManager`, present the
  system output switcher, and observe route changes
- [x] 3.5 Confirm the AVAudioSession category still fits (playback, and that a
  route change does not stop the stream)

## 4. Dart seam and state

- [x] 4.1 Add `lib/services/audio_routing_service.dart`: abstract seam over
  `listOutputs` / `selectOutput` / `activeRoute` / `presentRoutePicker` /
  `supportsDeviceSelection`, with the frb + method-channel implementation
- [x] 4.2 Expose it as a `@riverpod` provider; add a Freezed routing state
  (available outputs, active route, selection failure)
- [x] 4.3 Persist the selected output **by name** and restore it at startup,
  displaying the *active* device rather than the requested one
- [x] 4.4 Persist `outputOffsetMs` (default 0) alongside `instrumentSoundsItself`

## 5. UI

- [x] 5.1 Add a "Sound output" section to the player settings drawer and
  [pre_play_setup_modal.dart](apps/music/lib/screens/pre_play_setup_modal.dart):
  device list on desktop, active route + picker button on mobile
- [x] 5.2 Add the instrument-sounds-itself toggle to that section, with copy
  explaining it is for instruments that produce their own sound
- [x] 5.3 Show the wireless warning when the active route's kind is `bluetooth` —
  driven by `kind`, never by matching the route name
- [x] 5.4 Reveal the offset control only when a wireless route is active; suggest
  a starting value without applying it
- [x] 5.5 Isolate side effects (snackbar on a failed device open, re-reading the
  route after the OS picker is dismissed) in a dedicated listener widget near the
  section root — no `ref.listen` in `build`, no awaiting a notifier action
- [x] 5.6 Add the fr/en l10n strings — no raw platform or engine error strings

## 6. Offset applied to the reference (land last)

- [x] 6.1 Apply `outputOffsetMs` to the time reference the notifier passes to the
  scorer ([player_notifier.dart:309](apps/music/lib/state/player_notifier.dart#L309))
  — **do not modify `performance_scoring_core.dart`**, which is a pure module
- [x] 6.2 Apply the same value to the rendered playhead so highlight and judgment
  derive from one number
- [x] 6.3 Confirm Wait Mode inherits the shift through the playhead, with no
  separate handling
- [x] 6.4 Test: offset 0 reproduces today's verdicts exactly (guard against a
  silent scoring regression)
- [x] 6.5 Test: with a non-zero offset, an attack aligned to the delayed audio is
  judged on time rather than late

## 7. Tests

- [x] 7.1 Unit tests for the source gate: MIDI note not synthesized when the
  setting is on; on-screen and computer-keyboard notes still synthesized; scoring
  and Wait Mode identical either way
- [x] 7.2 Unit test that score playback, metronome and preview clips are
  unaffected by the setting
- [x] 7.3 Unit tests for the routing notifier with a mockito mock: device list,
  selection, failed open, absent remembered device, route change
- [x] 7.4 Widget tests: desktop shows a device list, mobile shows the active route
  + picker button (override the platform seam); wireless warning shown for a
  `bluetooth` kind and not for `usb`/`headphones`
- [x] 7.5 Widget test that the toggle is disabled with its reason when no MIDI
  port is connected
- [x] 7.6 Keep Flutter line coverage ≥ 80%

## 8. Gates and manual verification

- [x] 8.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [x] 8.2 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --
  -D warnings`
- [ ] 8.3 Manual: with a digital piano over USB MIDI, enable the setting and
  confirm the flam disappears while the on-screen keyboard still sounds
- [x] 8.4 Manual (desktop): switch output to a piano in USB-audio mode / an
  interface and confirm notes, metronome and a SoundFont preview all follow
  (macOS + Yamaha P-145: selection, fallback on unplug and automatic
  re-adoption on replug all verified on device)
- [x] 8.5 Manual: unplug the selected device mid-session and confirm
  fallback to the default without silence or crash
- [x] 8.6 Manual (mobile): iOS (iPad + USB piano) routes via the system
  picker; Android validated on Tab S6 Lite + Galaxy A53 — device speakers by
  default, USB selectable as "(experimental)", MIDI stable while playing
- [ ] 8.7 Manual: measure a Bluetooth route's delay, set the offset to it, and
  confirm the playhead visually matches what is heard
- [x] 8.8 Restore the iOS/macOS project files if a macOS test run rewrote them
  (SPM/icon churn) before committing
