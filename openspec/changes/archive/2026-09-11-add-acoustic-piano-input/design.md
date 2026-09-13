## Context

The player's only ear today is MIDI: `MidiService` (an injectable seam,
`apps/music/lib/services/midi_service.dart`) streams normalized
`MidiEvent { kind, pitch, velocity, timestamp_ms }` records
(`apps/music/rust/src/api/midi.rs`) to eleven consumers — the player
notifier, the status/monitor/calibration notifiers, and the four course
widgets. None of them knows where an event came from. The Rust audio engine
(`api/audio.rs`) is output-only: cpal + rustysynth, no capture anywhere, no
microphone permission declared on any platform.

The scoring layer defines the latency budget this design must live inside
(`performance_scoring_core.dart`): free-run binds an attack within ±160 ms
and calls ±40 ms perfect on the *score clock*; Wait Mode judges the reaction
after the gate opens (120/300 ms) and — per the `wait-mode` spec — already
scores no sustain. The output side already models route kinds and a wireless
warning (`audio_routing_service.dart`, `AudioRouteKind.isWireless`); the
`drumsEnabled` provider is the house pattern for a server flag defaulting to
`false` with a `main.dart` override.

Guitar was evaluated first and deferred (memory:
`guitar-deferred-acoustic-piano-first`); this change builds the
instrument-agnostic half (capture) so guitar can come back to it later.

## Goals / Non-Goals

**Goals:**

- A capture foundation (device, permissions, unprocessed configuration, route
  classification, calibration) that piano uses now and other instruments can
  use later, testable without hardware.
- Acoustic piano detection good enough for a staff/beta proof of concept:
  score-informed presence checking feeding the existing event stream.
- Wait Mode usable on any device from day one; free-run only where a measured
  round-trip fits the windows.
- Zero behavioral change for every existing input path and for MIDI-sourced
  scoring.

**Non-Goals:**

- Blind polyphonic transcription, velocity fidelity, or release detection.
- Guitar (tablature, fretboard, corpus) — deferred deliberately.
- A new `AppInstrument`: acoustic piano is the keyboard context.
- Leaderboard policy for audio-sourced runs — the run record is stamped with
  its source so policy can come later; the POC audience is staff/beta anyway.
- Bluetooth input support of any kind.

## Decisions

**D1 — Detection lives in Rust, behind the existing event stream.**
Capture and DSP run in the engine (real-time, SIMD-friendly, off the UI
thread) and emit the same normalized events `midi_event_stream()` carries.
The split copies the house pattern exactly: `api/audio_input.rs` holds the
cpal-input/thread/permission glue (excluded from the coverage gate — the
exclusion regex in `rust.yml` and `sonar.yml` gains `/audio_input\.rs`), and
`api/audio_input_core.rs` holds every testable piece — onset detection, pitch
presence scoring, the calibration state machine — covered like
`midi_core.rs`/`audio_core.rs`. Alternative considered: detection in Dart via
an FFI PCM stream — rejected: pushing raw audio across the bridge burns the
latency budget and puts DSP on the UI isolate.

**D2 — Source selection above the seam, not a second seam.**
A `PlayerInputSource` (MIDI | microphone) selector decides which backend
feeds the one event stream consumers already listen to. The `MidiService`
port-management surface (`listPorts`/`selectPort`) stays MIDI-only; the
microphone source exposes its own route/calibration state via the capture
seam. Rationale: eleven consumers stay untouched; the two sources have
genuinely different device semantics, so forcing microphone routes into
`listPorts` would corrupt the MIDI spec's port contract (stable-key matching,
auto-connect). The remembered MIDI port survives a source switch by
construction, since its state is never rewritten.

**D3 — Score-informed presence detection, onset first.**
Two stages: (1) a broadband onset detector stamps attack times within a few
milliseconds of the transient; (2) a pitch-presence stage confirms which of
the *expected* pitches (the score window around the playhead / the open Wait
gate) are present, using spectral evidence accumulated after the onset. The
emitted event carries the onset timestamp (spec: onset decoupled from
confirmation). Wrong/extra notes are reported only when evidence is
unambiguous; otherwise they are simply not emitted — scoring judges only what
arrives. Alternative considered: an ML transcription model (Basic Pitch /
Onsets & Frames) — rejected for the POC: model shipping/licensing weight and
per-platform inference cost, when the score prior makes classical DSP
sufficient for presence checking. The stage-2 interface is deliberately
narrow so a model can replace it later without touching stage 1 or the seam.

**D4 — Wait-Mode-first is a spec-level gate, not a suggestion.**
Wait Mode's reaction windows (120/300 ms) minus a realistic detection chain
(~30–60 ms) leave most of the budget intact, and Wait Mode scores no sustain
— so the POC is *fully honest* there with zero scoring change. Free-run
instead depends on the **measured** round-trip: the calibration loop (emit a
click through the existing output path, detect it with stage 1) yields the
per-route input offset that free-run judgment subtracts (delta spec on
`performance-scoring`), and the gate compares that measurement against the
free-run windows. No measurement → no scored free-run with the microphone.

**D5 — No synthesis of detected notes; the echo problem is dissolved, not
solved.** The acoustic piano sounds itself, so the detected-note path never
calls the synth (the `MidiEcho` concept in `audio_core.rs` is bypassed for
the microphone source — inherent, not the MIDI-scoped setting). Remaining
mic pollution is score playback and the metronome; the score prior makes the
detector robust to the metronome click (a known, broadband, short transient
it can ignore by construction). Full playback-while-detecting (play-along
with accompaniment) is accepted as degraded in the POC and measured rather
than pre-engineered around.

**D6 — Bluetooth input is refused, USB is first-class.** Input BT means
HFP/SCO (8–16 kHz, 100–300 ms jitter — beyond the ±160 ms bind window even
before detection), so the route classifier hard-refuses it with copy, unlike
the output side's compensable warning. Wireless mics with USB-C receivers
enumerate as class-compliant USB audio and pass. Android capture uses
`UNPROCESSED` when `PROPERTY_SUPPORT_AUDIO_SOURCE_UNPROCESSED` says so,
falling back to `VOICE_RECOGNITION` (the least-processed guaranteed source);
iOS/macOS use `AVAudioSession` `.measurement` / the input-configured
AVAudioEngine equivalent. The obtained configuration is recorded for
diagnostics (D7).

**D7 — Calibration is also the fleet probe.** The measured round-trip is
what gates free-run, but it equally answers the open fleet question (which
Android devices are usable at all) from staff/beta hardware before any
detection tuning depends on it. Calibration results ride the existing
diagnostics/analytics path so the answer arrives as data.

**D8 — Flag: one server-evaluated key, `drumsEnabled` pattern.** Default
`false` provider, `main.dart` overrides with the remote flag, staff + beta
campaign audience server-side. Flag off ⇒ no surface, no permission prompt,
nothing new reachable. No backend code beyond the flag definition.

## Risks / Trade-offs

- **[Android capture fragmentation]** — the input twin of the audio-output
  scars: `UNPROCESSED` unsupported, vendor DSP that cannot be bypassed, buggy
  route reporting. → Mitigation: D7 measures per device before anything
  depends on it; the fallback ladder is explicit; free-run simply stays
  gated shut on unfit devices while Wait Mode still works.
- **[Detection quality on real uprights]** (detuned instruments, low-octave
  weak fundamentals, room noise) → score-informed matching tolerates cents-
  level detuning by construction (presence bands, not exact bins); the low
  two octaves are accepted as weak on built-in mics; staff/beta feedback
  decides whether a mic-placement guide is enough.
- **[Metronome/playback bleed into the mic]** → D5: no synthesis of detected
  notes removes the loudest source; the metronome transient is masked by the
  score prior; play-along quality is measured in beta, with headphones as
  the documented workaround if needed.
- **[Latency budget in free-run]** → the gate (D4) refuses rather than
  degrades; the worst outcome is a device honestly limited to Wait Mode.
- **[macOS entitlement / store signing]** — `audio-input` entitlement must
  survive the export re-signing path that previously stripped entitlements
  (memory: `macos-app-store-publishing`). → Task-level verification on the
  archive, not just the dev build.
- **[Scope creep toward transcription]** → the specs pin presence-checking
  and best-effort extras; anything more is a future change.

## Open Questions

- Whether stage-2 presence evidence suffices for *chords* on hard surfaces
  (close attacks, shared partials) or the POC limits itself to the sparse
  textures of the learning repertoire — answered by beta data, not upfront.
- Where calibration UI lives (player settings drawer vs. a first-use flow
  when selecting the microphone source) — decided at mockup time inside the
  tasks, no spec impact.
- Leaderboard eligibility of audio-sourced free-runs once the flag widens —
  deliberately deferred; the source stamp keeps every option open.
