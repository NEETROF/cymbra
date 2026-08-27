## Why

The app plays back **exactly the General MIDI number an instrument sends**, and
resolves a stroke to a kit piece by that same number. There is no input mapping
anywhere: the engine echo sounds `event.pitch` raw
(`api/audio_core.rs::echo_event`), the pad and cascade flash comes from
`laneIndexOf(pitch)`, and the scorer matches on the piece that number belongs to.

That is a correct default for a module on the General MIDI drum map and silently
wrong for anything else — which is most e-kits, because a module lets its owner
reassign every pad, and because head/rim/bell zones have no GM numbers to be
assigned to. The drum beta's first tester reported a tom that produces **no sound
at all** and asks which MIDI code the app expects. The shipped kit font answers
half of it: measured through the engine, every number from 27 to 87 sounds, toms
among the loudest in the kit. So a stroke that is inaudible is a stroke arriving
on a number outside that window — and it is not just silent, it is *invisible*:
no pad flash, no Wait-Mode gate release, no scoring credit. The player has no way
to find that out, because nothing in the app ever shows what the instrument sent.

Two gaps, one after the other: the player cannot **see** what their kit emits,
and cannot **tell the app** what it means.

## What Changes

- **A MIDI input monitor** (Music app): a live read-out of incoming events —
  note number, the General MIDI name the app resolves it to, velocity, channel,
  and whether it resolves to a piece of the loaded score's kit or to nothing at
  all. Reachable from the MIDI device section of the settings, works with no
  score loaded, and is instrument-agnostic (a keyboard shows note names). This is
  a diagnostic, not a mapping: it makes the invisible visible and nothing else,
  and it ships on its own.
- **A calibration pass** (Music app): a guided sequence — "hit your snare",
  "hit your hi-hat" — that records the number each pad sends and stores a
  **mapping for that device**. Skippable per piece (a kit with no China does not
  own one), re-runnable, and reviewable/editable afterwards as a plain table.
- **The mapping is applied at exactly one seam**, so what sounds, what flashes
  and what scores can never disagree: an incoming number is translated once, on
  the way in, and everything downstream keeps working on General MIDI numbers.
  That includes the engine's own echo, which sounds a stroke from the MIDI
  callback and therefore has to learn the mapping too.
- **Per-device, persisted locally**: keyed by MIDI port name, so a player with a
  kit at home and a practice pad elsewhere keeps both. An unknown device has no
  mapping and behaves exactly as today (identity translation).
- **The monitor reports the mapping it applied**: an event shows both the raw
  number and what it was translated to, so a wrong mapping is as visible as a
  missing one.
- Not in scope: velocity curves and dynamics (a stroke is still sounded at the
  schedule's own loudness — the uniform-loudness interim pinned by
  `add-drum-input-mapping` task 2.4), multi-zone pads resolving to *different*
  pieces by the same physical strike, and any server-side storage of the mapping.

## Capabilities

### New Capabilities
- `music-midi-input-monitor`: a live, human-readable view of the MIDI events the
  app is receiving, with the resolution the app performs on each one.
- `music-drum-input-mapping`: a per-device translation from the numbers an
  instrument sends to the General MIDI numbers the app reasons in, learned by a
  guided calibration pass, editable, and applied at a single input seam.

### Modified Capabilities
- `midi`: the real-time event stream gains an observable form (the monitor reads
  it without competing with the player for it) and the requirement that a live
  event is translated before it is interpreted.
- `local-preferences`: the calibrated mapping joins the locally persisted
  preferences, keyed per MIDI device.

## Impact

**Product: Cymbra Music only.** No backend, no back office, no site, no
identity. Nothing is sent to a server and no gRPC surface changes.

Consumed, unchanged:
- the platform's `local-preferences` store (the mapping is one more persisted
  key) and the injectable preferences seam;
- the existing `midi` port enumeration, selection and event stream;
- the kit model (`drum_kit.dart`) — lanes, canonical emission order, piece
  resolution — which keeps working on General MIDI numbers throughout.

New or modified in the app + engine:
- `apps/music/rust/src/api/midi.rs` / `audio_core.rs`: the engine echo consults
  the mapping (it sounds a stroke before the event crosses the bridge, so it
  cannot be corrected on the Dart side);
- `flutter_rust_bridge_codegen generate` — the engine gains a way to be told the
  mapping, so the public API changes;
- `player_notifier.dart`: one translation point on the way in;
- a monitor surface and a calibration flow (screens + a notifier), plus fr/en/
  es/it strings;
- `player_preferences.dart` + the preferences seam: the persisted mapping.
