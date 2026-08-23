## 1. Kit table — canonical emission order

- [ ] 1.1 `drum_kit.dart`: give each named piece's General MIDI members a **canonical order** (hi-hat 42, 46; snare 38, 40, 37; ride 51, 59, 53; kick 36, 35) — the members are collapse-equal for lanes but not for emission, so the table gains order without changing lane derivation
- [ ] 1.2 Pure emission helper: lane (or the kick) → the first canonical member **the loaded score actually uses**; a generic piece returns its single number. Test the present-member fallback explicitly: a score writing its snare only as the electric 40 emits 40, a score writing its kick only as 35 emits 35, an open-only hi-hat (46 without 42) emits 46
- [ ] 1.3 Test that every emitted number is a member of the lane it came from (the property that keeps the struck-flash lookup and the future matcher coherent), and that emission is deterministic for a given score
- [ ] 1.4 Resolution helper for feedback: General MIDI number → the pad to flash (`laneIndexOf`) or the pedal (`kKickGmNumbers`), or nothing for a number outside the score's kit

## 2. Stroke path in the player

- [ ] 2.1 `player_notifier.dart` `noteOn` (`:425`): for a percussion score, sound through the seam's percussion one-shot — `drum_on(key, velocity)`, per `add-drum-audio-channel`, which exercises it from scheduled playback only and leaves live wiring here with no engine change — never `_audio.noteOn(pitch)` (the pitched piano voice a live stroke wrongly gets today). The `state.synthesizes(source)` guard is consulted unchanged, so a module that sounds itself is not doubled while on-screen strokes always sound
- [ ] 2.2 `noteOff` (`:458`): for percussion, bookkeeping first — clear the held entry; forward the release to the paired `drum_off(key)` only if that does not clip the sounding one-shot (the binding scenario: an immediate note-off leaves a cymbal ringing to its natural end — check the engine's actual `drum_off` semantics before wiring it). Test that an immediate note-off after a stroke changes nothing audible, and that a missing note-off leaves no sounding voice
- [ ] 2.3 Test source convergence: the same GM number arriving as an on-screen stroke and as a MIDI-device stroke produces identical state below the sounding decision (mockito on the audio/midi seams per the `flutter-testing` skill)
- [ ] 2.4 Velocity stays unconsumed: `drum_on`'s velocity parameter is filled with the schedule's own `DEFAULT_VELOCITY`, never the stroke's — assert the one-shot is invoked identically whatever velocity the event carried (the uniform-loudness interim, pinned so it cannot drift silently)

## 3. Pad strip input

- [ ] 3.1 `player_screen.dart`: wrap the pad-strip `CustomPaint` (`:432` — remove the "no Listener, display-only" comment and construction) in a `Listener` mirroring the keyboard's (`_onKeyboardPointerDown`, `:234`): pointer-down → emission helper → `noteOn(gm, source: NoteSource.onScreen)`; pointer-up/cancel → the matching `noteOff`
- [ ] 3.2 Hit testing: pads band → the lane whose horizontal span contains the pointer, pedal band → the kick; **the whole strip is live** — assert a tap landing in the 3 px inset between two drawn pads still strikes the lane whose span contains it (no dead gutters), and a tap anywhere in the pedal band strikes the kick
- [ ] 3.3 Multi-touch: per-pointer tracking like `_keyboardPointers`; test two pads struck simultaneously both emit, and a **two-finger roll on one pad** — each pointer-down emits a fresh stroke while the other finger is still down (no keyboard-style retrigger exclusivity)
- [ ] 3.4 Widget test that a pad tap works while playback is stopped and in the cascade during playback — the same at-all-times contract the keyboard has

## 4. Struck feedback

- [ ] 4.1 Player state: short-lived struck timestamps per controller surface (each lane + the pedal), set from `noteOn` for any source via the resolution helper (1.4); a number resolving to no surface sets nothing — audible free play, no flash, no error
- [ ] 4.2 `drum_pad_strip_painter.dart`: render the flash from the struck timestamps — one state, intensity decaying over a short fixed duration, on pads and pedal alike; no expected/correct/incorrect states exist anywhere in the painter
- [ ] 4.3 Drive repaints for the flash while playback is stopped (the strip must animate on the flash's own clock, not only on playhead frames), and assert the decay completes independently of note-off timing
- [ ] 4.4 Test the honest-feedback boundary: a stroke on an onset and a stroke far from any onset produce the identical flash; the cascade painter receives no stroke state at all
- [ ] 4.5 Input never suppressed: with hands-only selected, a kick stroke still sounds and flashes the pedal; with feet-only, a snare stroke still sounds and flashes its pad — asserted against `visibleNotes` filtering staying presentation-only

## 5. External MIDI and guards

- [ ] 5.1 `apps/music/rust/src/api/midi_core.rs`: add the e-kit-shaped decode test — NoteOn status `0x99` (channel 10), a GM percussion number, velocity > 0 → a NoteOn event exactly as from any other channel (the property exists and is pinned for channel 6 at `:312`; the percussion case is the one a channel-10 "fix" would break). No engine behavior change, no public API change, no frb regeneration
- [ ] 5.2 App-level test through the midi seam: a device stroke reaches `noteOn` with `NoteSource.midiDevice`, sounds via the one-shot, and flashes the resolved pad — and is not synthesized when instrument-sounds-itself is on
- [ ] 5.3 `_maybeStartRun` (`player_notifier.dart:138`): never arm the scorer for a percussion score — the same never-arms mechanism as a selective run. Test that a full percussion run opens no scored run, produces no session result and no summary, and that a keyboard run still scores exactly as before
- [ ] 5.4 Test Wait Mode is still not offered for a percussion score (the restriction now survives on the matcher rationale, not the no-input one — `add-drum-kit-view`'s existing assertion keeps passing unchanged)
- [ ] 5.5 Riverpod layering: the screen calls notifier methods only; the painter reads no service; no provider invalidates a sibling (per the `flutter-riverpod-architecture` skill)

## 6. Gates

- [ ] 6.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [ ] 6.2 `cd apps/music && flutter test --coverage --exclude-tags golden`, coverage ≥ 80%
- [ ] 6.3 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 6.4 `cargo llvm-cov --workspace --fail-under-lines 80` with the repo's usual ignore regex (the new test lives in host-testable `midi_core.rs`)
- [ ] 6.5 `openspec validate add-drum-input-mapping --strict`

## 7. Manual verification — device and feel

- [ ] 7.1 Real e-kit on a device: strokes sound the right pieces with no perceptible extra latency over the module's own sound; then flip instrument-sounds-itself on and confirm the module is not doubled while pad taps still sound
- [ ] 7.2 Reconfigure the kit's transmit channel away from 10 and confirm nothing changes — the channel-agnostic property, verified on hardware where the misconfiguration actually happens
- [ ] 7.3 Play a roll with two fingers on one pad at tempo: every stroke sounds and the flash stays legible rather than smearing into a solid glow — tune the flash duration here and record the chosen value in `design.md`
- [ ] 7.4 Aim deliberately at the gutters between pads at tempo: no ghost strokes
- [ ] 7.5 Hands-only practice with feet strokes (and the reverse): input audibly never suppressed, cascade filtering unchanged
- [ ] 7.6 Confirm a full percussion run ends with no score summary and no post-run judgment of any kind, and that a keyboard score's run, feedback and assist keys behave exactly as before
