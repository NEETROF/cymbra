## Context

The audio engine is a single cpal stream opened once, on the system default:

```rust
let host = cpal::default_host();
let device = host.default_output_device()…;   // api/audio.rs:234
```

Everything the app emits is mixed into that one stream's callback — synthesized
notes, metronome clicks (`ClickVoice`), and SoundFont preview clips
(`AudioCommand::PlayClip` / `ClipVoice`). That is convenient: **one routing
decision moves all app audio at once**, and the preview clips do *not* escape
through a separate platform player. The stream is driven by an `AudioCommand`
channel, so a rebuild can be commanded without restarting the app.

On the Flutter side, every input source converges early:

```dart
case MidiEventKind.noteOn: noteOn(event.pitch);   // player_notifier.dart:247
…
void noteOn(int pitch) { … _audio.noteOn(pitch); … }   // :279-283
```

`noteOn` is called by the MIDI stream, the on-screen keyboard and the
computer-keyboard fallback alike — **the source is discarded at the call site**.
Score playback sounds separately (`_audio.noteOn(p)` at `:162`, driven by the
playhead), which is why it is naturally unaffected by a source rule.

A graph pass over the code (`graphify`) bounds the work precisely:

- Exactly **three sources and six call sites** feed the notifier — MIDI
  (`player_notifier.dart:247/249`), the on-screen keyboard pointer handlers
  (`player_screen.dart:209/215`) and the computer-keyboard assist
  (`player_screen.dart:158/165`).
- On the Rust side `audio_init` (degree 3) → `run_audio_thread` → `build_stream`
  (degree 11) is a closed chain, and `AudioCommand` is referenced only by those
  two functions. Device selection is contained; nothing else in the workspace
  depends on how the stream is opened.
- **`state/performance_scoring_core.dart` imports only `dart:math`**: it is a pure
  module of functions over numbers, with no knowledge of the playhead or app
  state. The offset therefore never needs to touch it — see D4.

Platform reality for device selection splits in two:

- **Desktop** (macOS/Windows/Linux): cpal enumerates real output devices
  (`host.output_devices()`), and a chosen device can be opened by name. A genuine
  in-app picker is possible.
- **Mobile** (iOS/Android): there is no device to pick. iOS routes through
  `AVAudioSession`, and the sanctioned UI is `AVRoutePickerView`; Android routes
  through `AudioManager`/MediaRouter. The app can *present* the OS picker and
  *report* the active route, not choose a device itself.

## Goals / Non-Goals

**Goals:**

- Stop double-sounding notes the connected instrument already played, without
  silencing anything else.
- Let a desktop user send Cymbra's audio to a specific output (piano in USB-audio
  mode, interface, headset) and have it persist.
- Make the active route visible on every platform, and name it when it is
  wireless.
- Keep a delayed route usable for *listening* by aligning the playhead and the
  scoring reference with what is actually heard.
- Preserve the existing graceful-degradation contract: no audio device must never
  mean a crash or a dead player.

**Non-Goals:**

- Per-sound routing (notes to one device, metronome to another). One output for
  all app audio; the only split is *whether* a sound is produced at all.
- Sending audio *to* the instrument over Bluetooth A2DP as a recommended setup —
  it is reachable through the OS route picker, and it is warned about, not
  blocked.
- Fixing the latency of the user's *own* notes on a delayed route. If the output
  is 200 ms late, an on-screen key press is heard 200 ms late; no offset can undo
  that. Only the *reference* can be moved.
- Sample-accurate output-latency measurement (loopback calibration). The offset is
  a user-set number with a suggested starting value.
- MIDI *output* to the instrument (Cymbra driving the piano's own voices for score
  playback) — attractive, but a different capability.

## Decisions

### D1 — Source-tagged note entry, gate only the instrument's own notes

`noteOn`/`noteOff` gain a source (`midiDevice` | `onScreen` | `computerKeyboard`),
defaulting to `onScreen` so existing call sites and tests keep compiling. The
audio call becomes conditional on exactly one predicate:

```
synthesize = !(instrumentSoundsItself && source == midiDevice)
```

Everything else in `noteOn` — scoring, key feedback, Wait Mode release — runs
unchanged for every source. Score playback, metronome and preview clips never
consult the predicate, because they do not come from an input source at all.

*Alternative considered:* a global "mute the synth" switch. Rejected outright by
the requirement: the on-screen keyboard must still sound, or the app becomes
silent for anyone without an instrument in front of them.

*Alternative considered:* muting at the engine level (a synth gain of zero).
Rejected: the engine cannot see where a note came from, and it would also kill
score playback and previews.

*Consequence:* the mode is only meaningful while an instrument is connected. When
no MIDI port is connected, the toggle is shown disabled with the reason, rather
than silently doing nothing.

### D2 — Desktop: enumerate and select by name, fall back to default

New engine surface, all commanded through the existing `AudioCommand` channel so
the audio thread owns the stream lifecycle:

| Function | Behaviour |
| --- | --- |
| `list_audio_outputs()` | names of the host's output devices, default first |
| `set_audio_output(name: Option<String>)` | `None` = follow the system default; otherwise rebuild the stream on the named device |
| `active_audio_output()` | name actually in use (may differ from the request after a fallback) |

Selection is persisted **by name**, which is the only stable-ish handle cpal
offers. A remembered name that is absent at startup falls back to the system
default and reports the actual device, so the user sees reality rather than a
stale preference. Rebuilding recreates the `Synthesizer` on the new sample rate;
an `all-notes-off` is issued first so no voice is stranded across the swap.

*Alternative considered:* rebuilding the whole engine (re-reading the SoundFont
from disk). Rejected — the SoundFont is multi-MB and the swap must feel instant;
only the stream and synthesizer are rebuilt.

*Failure policy:* if the requested device fails to open, keep the current stream
alive and report the failure. Never tear down working audio to chase a broken
device.

### D3 — Mobile: present the OS picker, report the route, never fake a choice

iOS presents `AVRoutePickerView` and reads
`AVAudioSession.currentRoute.outputs.first` (portType + portName) for display;
Android presents the system output switcher and reads `AudioManager`'s
communication/output device. Both report a `{name, kind}` where `kind` ∈
`{builtin, headphones, bluetooth, usb, other}` — the *kind* is what drives the
wireless warning, not string-matching the name.

The desktop picker and the mobile route button are the same UI slot with two
implementations, behind one Dart seam, so the settings section has a single
shape.

*Alternative considered:* `AudioTrack.setPreferredDevice` on Android API 28+ for a
real in-app choice. Rejected: cpal does not expose the underlying `AudioTrack`,
so it would mean forking the audio backend for one platform.

### D4 — Latency: classify the route, warn, and move the *reference*

A route whose kind is `bluetooth` is flagged wireless. The UI states plainly that
it suits listening but not playing, and does not block it.

The offset is one number, `outputOffsetMs`, defaulting to 0 and suggested (not
imposed) at a typical value when a wireless route becomes active. It shifts two
things and nothing else:

- the **visual playhead**, rendered `outputOffsetMs` behind the wall clock, so
  what is highlighted matches what is heard;
- the **scoring reference**, so a user playing along with delayed audio is judged
  against the audio they heard rather than the clock.

Both derive from the same value, so they cannot drift apart. Wait Mode already
freezes on the playhead, so it inherits the shift with no separate handling —
this is asserted by a test rather than assumed.

**Where the shift is applied matters more than the arithmetic.**
`performance_scoring_core.dart` is a pure module over numbers (it imports nothing
but `dart:math`), and the notifier already hands it an explicit time reference:

```dart
_scorer.noteOn(pitch, state.elapsedMs, waitMode: state.waitMode);  // :309
```

So the offset is applied **at that call site**, to the reference the notifier
passes in — the scoring core is not modified at all. This keeps the most
delicate subsystem in the app untouched, turns the change into arithmetic on one
value in one layer, and means the existing scoring tests keep their meaning.

*Why not shift the audio instead:* the sound is late because the transport made it
late; the engine cannot emit earlier than real time. Only the references are
movable.

*Sequencing:* this is the riskiest piece and the only one that touches scoring, so
it lands last, behind the rest, and defaults to 0 (exactly today's behaviour)
until the user changes it.

### D5 — One settings section, two homes, existing seams

A "Sound output" section joins the MIDI device section in both the player
settings drawer and `pre_play_setup_modal.dart`: active route (+ picker), the
instrument-sounds-itself toggle, and the offset (revealed only when a wireless
route is active, so it does not clutter the common case).

Per the repo's Riverpod rules: widgets call notifier methods only, never the
routing service; the routing notifier does not imperatively invalidate the audio
or MIDI providers; and the side effects (snackbar on a failed device open,
re-reading the route after the OS picker is dismissed) live in a dedicated
listener widget near the section root.

## Risks / Trade-offs

- **R1 — Rebuilding the stream mid-session can glitch or fail** (device busy,
  exclusive mode on Windows) → `all-notes-off` before the swap, keep the old
  stream if the new device will not open, and surface a localized failure rather
  than a silent app. Covered by `audio_core.rs` tests on the selection/fallback
  logic, which is pure.
- **R2 — Device names are not stable identifiers** (same model twice, renamed
  interfaces, USB re-enumeration) → treat a missing name as "fall back to default
  and report it", never as an error state, and always display the *active*
  device rather than the *requested* one.
- **R3 — The offset affects scoring**, the one subsystem where a regression is
  both subtle and damaging (a wrong reference silently degrades everyone's
  scores) → largely defused by applying the shift at the notifier's call site
  rather than inside the pure scoring core, which stays byte-for-byte unchanged.
  Remaining mitigations: default 0, a single value feeding both the playhead and
  the scorer, landed last, and a test asserting offset 0 reproduces today's
  verdicts exactly.
- **R4 — Instrument-sounds-itself is confusing when nothing is connected**
  (the user enables it and the app goes quiet for MIDI notes that never arrive)
  → the toggle is disabled with an explicit reason while no MIDI port is
  connected.
- **R5 — Users may enable it with a *silent* instrument** (a controller with no
  speakers, or local-control off) and hear nothing at all → the setting's copy
  says it is for instruments that produce their own sound, and the toggle is
  reachable from the same panel so it is easy to undo.
- **R6 — No CI coverage for real devices**: no audio hardware, no BT speaker, no
  route changes in CI → all decision logic is pure and host-tested
  (`audio_core.rs`, the source predicate, offset arithmetic); the native pieces
  stay thin and are covered by an explicit manual matrix.
- **R7 — Four platforms' route reporting to maintain** for one settings row →
  the `{name, kind}` shape is deliberately minimal, and `kind` is what the UI
  reasons about, so a platform reporting an unknown port type degrades to `other`
  rather than breaking the section.
- **R8 — Android's AAudio does not reliably clock a USB-audio output**
  (measured, not theorised). On a Galaxy Tab S6 Lite (Android 13) with a Yamaha
  P-145 in USB-audio mode, the route is reported correctly, the stream is
  attached to the USB output thread, no error is ever raised — and the callback
  is pulled at the wrong rate: **+45%** of the stream's sample rate in one
  session, **−16%** in the next, correct in a third. Over-pull makes what is
  heard fall tens of seconds behind (indistinguishable from silence); under-pull
  starves the output. On the built-in speaker the same engine is pulled at
  *exactly* the stream rate, and another app (YouTube, which uses the Java
  `AudioTrack` path rather than AAudio) plays through the same piano correctly —
  so this is the platform's AAudio+USB path, not the engine.
  - Neither lever available through `cpal` changes it: matching the device's
    native 44.1 kHz, and bounding the AAudio buffer with
    `BufferSize::Fixed`, both leave the mis-pacing untouched (the bounded buffer
    additionally destabilised the app).
  - **Mitigation: on Android the platform owns the stream.** The `AudioTrack`
    path (the one YouTube uses) keeps perfect time on the same route — measured
    +15 ms over 24 s, the producer held a constant ~120 ms ahead — and its
    blocking `write` paces the producer, which is exactly what AAudio failed to
    provide. So `EngineOutput.kt` runs an `AudioTrack` that **pulls** rendered
    samples from the engine (`android_output.rs`), the inverse of the `cpal`
    model used everywhere else. It also solves the picker: `AudioManager`
    enumerates every output where `cpal`'s Android enumeration fails (its JNI
    call receives no usable Context from our `JNI_OnLoad` and falls back to a
    single placeholder device), and `setPreferredDevice` pins playback to the
    user's choice.
  - The drift monitor stays as the tripwire: the engine detects a sustained
    mismatch between the rate it is pulled at and the stream's sample rate and
    says so in the platform log — on the cpal path and the Android pull path
    alike. The Kotlin writer carries two more: a clock watchdog (a route that
    consumes off-clock without erroring — the Tab S6 Lite zombie — is rebuilt),
    and a rate-snap correction (a sink clocking at the *other* standard rate,
    measured 44.1 k consumed 1:1 on a 48 k clock, reopens the track and the
    synth at the measured rate).
  - **USB-audio out on Android is an experimental, informed opt-in** (settled
    after on-device testing on two machines). The Tab S6 Lite's USB HAL clocks
    the route at random per session (+87% measured on a fresh AudioTrack,
    mirroring the +45%/−16% AAudio measurements), and a Galaxy A53 crackles
    into the same piano from *every* app, YouTube included — both failures
    live below anything an app can configure, while the app-side pipeline
    measures clean (exact clock, zero client underruns, render 20× under
    budget). The policy that is NOT negotiable is the default: the app never
    *lands* on USB by itself — "system default" is policy-resolved to the best
    non-USB output whenever the platform would route media to USB. USB devices
    stay selectable but labelled "(experimental)" in the picker, and a USB
    route that keeps killing tracks is abandoned by the reopen strike ladder
    (same device → non-USB fallback → stop) rather than hammered — the rebuild
    churn is what used to knock the composite instrument, MIDI included, off
    the bus. The recommended Android setup remains instrument-sounds-itself:
    the piano sounds itself over MIDI, the app plays on the device. iOS
    (AVAudioSession/CoreAudio) and desktop keep full USB routing — they work.

## Diagnosability note

Engine lifecycle messages used `eprintln!`, which on Android reaches nothing: a
process's stderr is not wired to logcat. The one subsystem that can only be
diagnosed on a real device was therefore the one with no visibility, which is
what made R8 take a full debugging session to pin down. Those messages now go
through liblog on Android (plain stderr elsewhere), and the engine reports which
device it opened, at what rate and buffer size, every stream error with its kind,
and every reopen.

## Migration Plan

Additive and default-neutral; no data migration, no backend deployment.

- With no preference stored, the engine follows the system default and the
  instrument-sounds-itself toggle is off — byte-for-byte today's behaviour.
- The offset defaults to 0, so the playhead and scoring are unchanged until a
  user opts in.
- Rollback of the UI leaves the engine's new selection API unused but harmless
  (`None` = system default). Rollback of the engine change requires reverting the
  frb-generated bindings alongside it.
- Ship order: D1 (source rule) is independently valuable and can land first; D2/D3
  next; D4 last.

## Open Questions

- Should the metronome follow the instrument-sounds-itself rule when the
  instrument has its own metronome? Assuming **no** for now — the click is app
  audio and stays on the app's output.
- Should the app offer to drive the instrument's voices over MIDI *out* for score
  playback (so the piece plays on the piano too)? Deliberately out of scope, but
  it is the natural sequel once the instrument is the sound source.
- Is a suggested non-zero default offset on a wireless route helpful or
  presumptuous? Starting with "suggested, not applied" and revisiting with real
  feedback.
