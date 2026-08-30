## Why

Every input path into the player today is MIDI: a digital piano, an e-kit, the
on-screen keys. A family upright, a school piano, a rehearsal-room grand — the
instruments most beginners actually sit at — cannot drive the app at all,
because nothing in it can hear. This change gives the app ears: microphone
acquisition (built-in mic, or a wired/USB-C external mic) feeding acoustic
piano note detection, as a **proof of concept** gated to staff/beta.

Piano is deliberately the first audio instrument because it is the cheapest
possible client of the investment: detection has only to produce the existing
normalized note events behind the existing injectable input seam, and the
whole downstream stack — player, Wait Mode, scoring, courses, leaderboards —
runs unchanged. Guitar was evaluated and is intentionally **out** of this
change (deferred, not abandoned): it would additionally require tablature
parsing, a fretboard view, a new instrument context and a corpus, none of
which piano needs.

## What Changes

**A microphone capture foundation** (reusable beyond piano)

- Audio input acquisition in the engine, split like MIDI already is: thin
  device/thread glue plus a pure, host-testable DSP core.
- Platform microphone permissions and — critically — **voice processing
  disabled**: the OS default AGC/noise-suppression/echo-cancellation chains
  are tuned for speech and destroy sustained musical notes.
- Input route classification mirroring the output side's `AudioRouteKind`,
  with one harder conclusion: **Bluetooth microphones are refused** for
  acquisition (input BT is the HFP/SCO voice profile — 8–16 kHz and
  100–300 ms of jitter against a ±160 ms binding window). USB-C wireless
  receivers present as class-compliant USB audio and are first-class.
- A **measured input-offset calibration loop**: the app emits a click,
  detects it at the mic, and measures the device's real round-trip. Unlike
  the output offset — a guessed, hand-tuned setting — the input offset is
  closed-loop and measured.

**Acoustic piano note detection** (the proof of concept)

- Detected notes enter the player as the same normalized note events the MIDI
  path produces, through the same seam. No downstream change.
- Detection is **score-informed**: the expected notes are known, so the
  problem is presence-checking against the score, not blind polyphonic
  transcription. Onset detection (timing) and pitch confirmation (identity)
  are decoupled so the attack timestamp is not delayed by pitch analysis.
- Detected notes are **never synthesized** — the acoustic piano sounds
  itself. This is inherent to the source, not a setting (the existing
  "Instrument Sounds Itself" setting stays scoped to MIDI sources).
- **Wait-Mode-first**: Wait Mode judges reaction after the gate opens
  (120/300 ms windows) and already scores no sustain, so detection latency is
  absorbed and the damper pedal's unreliable note-offs cost nothing. The POC
  ships usable there with zero scoring change.
- **Free-run is gated on measured latency**: it opens only where the
  calibration loop shows the device's round-trip fits the free-run windows
  (±40/±90/±160 ms), and its sustain dimension is reweighted for
  audio-sourced runs (attack-weighted — the drum precedent), because the
  damper pedal makes releases undetectable.

**Gating and non-goals**

- Behind a runtime feature flag following the drums pattern: default `false`,
  server-evaluated flag overrides at boot, staff/beta audience first.
- **No new instrument context**: an acoustic piano is the keyboard context.
  No catalog, notation, back-office or site change. Guitar explicitly out.

## Capabilities

### New Capabilities

- `music-audio-capture`: microphone acquisition for the Music app — device
  enumeration and input route classification (Bluetooth refused), platform
  permissions, voice-processing bypass, capture lifecycle, and the measured
  input-offset calibration loop. Instrument-agnostic by design.
- `music-acoustic-piano-input`: acoustic piano note detection over captured
  audio — score-informed presence detection, onset/pitch decoupling, the
  no-synthesis rule for detected notes, Wait-Mode-first availability, the
  measured-latency gate for free-run, and the feature-flag audience.

### Modified Capabilities

- `performance-scoring`: two requirement-level changes for **audio-sourced
  runs only** — the timing reference incorporates the measured input offset
  (analog of the existing output-offset requirement), and sustain judgment is
  reweighted to attack-only because releases are masked by the damper pedal.
  MIDI-sourced runs are untouched.

## Impact

**Products**

| Product | Consumes | New |
|---|---|---|
| **Music** (`apps/music`) | the `MidiService` seam and its normalized note events, the flag client (`drumsEnabled` pattern), the `AudioRouteKind` classification model, Wait Mode as-is | capture engine (Rust glue + pure DSP core), mic permissions on all platforms, input routing + calibration UI, detection core, source selection (MIDI vs microphone), the feature flag |
| **Platform** (backend) | the existing feature-flags capability | one flag definition + beta audience — no new backend code |
| **ID / Live / Back-office / Site** | — | untouched |

**Code**

- `apps/music/rust/src/api/`: new input glue (excluded from the coverage gate
  like `midi.rs`/`audio.rs` — the exclusion regex in `rust.yml` and
  `sonar.yml` gains the new glue file) + new pure core (covered, like
  `midi_core.rs`/`audio_core.rs`); `flutter_rust_bridge_codegen` regenerated.
- `apps/music/lib/services/` + `lib/state/`: the input-source seam beside
  `midi_service.dart`, capture/calibration state, flag wiring in `main.dart`.
- `apps/music/ios|macos|android/`: `NSMicrophoneUsageDescription`,
  `com.apple.security.device.audio-input` (⚠️ store re-signing implications),
  `RECORD_AUDIO`; session/source configuration per platform.
- `apps/music/lib/l10n/`: permission rationale, calibration flow, refused-
  Bluetooth explanation, latency-gate copy (fr/en).

**Risks.** The load-bearing unknown is Android capture (device-dependent
`UNPROCESSED` support, the input twin of the audio-output routing scars);
the calibration loop in phase 1 is designed to surface exactly that, per
device, before any detection work depends on it.
