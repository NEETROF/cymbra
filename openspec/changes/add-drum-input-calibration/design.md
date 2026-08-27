## Context

The app reasons about percussion entirely in General MIDI numbers. `drum_kit.dart`
derives the lane layout from the numbers a score contains, `laneIndexOf` resolves
a stroke to a lane, the scorer matches at the piece's grain, and the engine echo
(`api/audio_core.rs::echo_event`) sounds `event.pitch` on the drum channel. That
is one vocabulary end to end, which is what keeps sound, flash, gate and score in
agreement — and it works precisely as long as the instrument speaks it.

An e-kit module often does not. Every pad is reassignable, and rim/bell/edge
zones have no General MIDI numbers to be assigned to in the first place. Measured
through rustysynth against the shipped kit (`FluidR3Drums-bank128.sf2`), **every
number from 27 to 87 produces sound**, toms among the loudest in the file. So the
beta report — a tom that makes no sound — is a number arriving outside that
window, and the consequence is worse than silence: `laneIndexOf` returns nothing,
so there is no pad flash, no Wait-Mode gate release and no scoring credit either.
Nothing in the app displays the incoming number, so the player cannot even
discover that this is what is happening.

One constraint shapes everything below. The engine holds **one** Flutter sink:
`midi_event_stream` does `*SINK.lock() = Some(sink)` and the input callback reads
that global, so each Dart call to `events()` *replaces* the previous subscriber
rather than joining it. Several places already call it (`player_notifier`,
`midi_status_notifier`, and four lesson views), which means only the most recent
one is actually fed. A monitor is by definition a second reader, so this has to
be resolved first.

## Goals / Non-Goals

**Goals:**
- Make what the instrument sends **visible**, with no interpretation layered on
  top: the raw number first, the app's reading of it second.
- Let a player teach the app what their pads mean, by playing them.
- Keep one interpretation of a stroke. Sound, flash, gate and score must remain
  incapable of disagreeing, which means one translation point rather than four.
- Change nothing for an uncalibrated device or a keyboard.

**Non-Goals:**
- Velocity, dynamics and ghost notes. A stroke is still sounded at the
  schedule's own loudness — the interim `add-drum-input-mapping` task 2.4 pinned
  deliberately. Calibrating *what* a pad is does not touch *how hard* it was hit.
- Multi-zone pads that should resolve to different pieces (a rim shot vs a head
  strike). One number, one piece, this time round.
- Any server storage or cross-device sync of the mapping. It describes a piece of
  hardware sitting in one room.
- Auto-detection of a kit by name or manufacturer. A lookup table of modules is a
  maintenance burden that ages badly, and the player striking their own pads is
  both more reliable and shorter.

## Decisions

### D1 — Fan the MIDI stream out in Dart, not in the engine

One process-wide broadcast stream, held by a keepAlive provider, wrapping a
single `midi_event_stream()` subscription. Every current consumer reads from it;
the monitor becomes one more listener.

*Why not fan out in Rust:* the engine would grow a sink registry, lifetime
management for each, and a bridge API change — to solve a problem that only
exists on the Dart side. The engine's single-sink model is fine; what is wrong is
that Dart treats it as multi-subscriber.

*This also settles an existing hazard.* Today the last `events()` caller silently
starves every earlier one, and two consumers already co-exist: the lesson
player's AppBar chip (`LessonMidiChip` → `midiStatusProvider`) and whichever
exercise view is on screen. The body subscribes after the AppBar, so what is
starved today is the chip's event-driven refresh — mild enough that nobody has
reported it, and entirely dependent on build order. A monitor is a *new* second
reader, and in the game player the consumer it would starve is the player itself.
So this has to be settled before a monitor exists, not after.

### D2 — Translate at the single point where a live event enters, and mirror it in the engine

The mapping is a plain `Map<int, int>` applied where a live MIDI event is first
seen. Everything downstream keeps its General MIDI vocabulary, unchanged and
untested-against — that is the point: no call site learns that mapping exists.

The engine is the exception, and unavoidably so: it sounds a live stroke from
inside the MIDI callback (`add-drum-input-mapping` §8, the fix for input
latency), before the event ever crosses the bridge. So the engine gets the table
pushed to it — the same shape as `set_midi_echo`: the app keeps the policy, the
engine applies it. A `HashMap` behind the same lock the echo mode already uses,
read in the callback.

*Why not translate on the Dart side only:* the echo would sound the raw number
while the rest of the app scored the translated one. Two answers to "what did I
just play" is the exact failure mode this change exists to close.

*Why not translate inside `drum_kit.dart`:* it would be a second place that has
to know about devices, and the pure kit model would stop being a pure function of
the score.

### D3 — Key the mapping by port name

The MIDI port name is what the app already uses to identify a device
(`selectPort`, the port dropdown, the persisted `midiPort` preference). Reusing
it costs nothing and keeps two kits from overwriting each other.

*Trade-off:* two identical modules on one machine share a name and therefore a
mapping. That is the right answer more often than not (same model, same factory
map), and the alternative — USB serial numbers — is not exposed by `midir` and
differs per platform.

### D4 — The calibration pass records the *next* stroke, never a pending one

A step arms, then waits. Any stroke stamped before the step began is discarded.
This is the same lesson as `add-drum-input-mapping` §8's Wait-Mode fix, where a
stroke left over from a previous lap opened the next run's first onset: a
timestamp only means something relative to the moment that is asking.

Conflicts are surfaced, not resolved silently. If the number offered for the tom
is already the snare's, the pass says so — because on a real kit that means the
player hit the wrong pad, and quietly reassigning it would produce a mapping that
is wrong in two places at once.

### D5 — The monitor shows raw and resolved side by side, and stays honest about silence

Three facts per event, in decreasing rawness: the number as received, the number
after translation (only when they differ), and the piece it resolves to in the
loaded score's kit — or an explicit "matches nothing". It also flags a number the
loaded SoundFont has no sample for, since that is the specific shape of the beta
report and the one thing the player cannot deduce from anything else on screen.

Bounded history (a fixed ring), so a session left open overnight cannot grow.

### D6 — Ship the monitor first

The monitor stands alone: it is a diagnostic, it needs no mapping, and it answers
the tester's actual question ("which MIDI code does the app use?") the day it
lands. The calibration pass then builds on the fan-out and the event model the
monitor establishes. Splitting the delivery this way also means the mapping is
designed against a real observed kit rather than a guessed one.

## Risks / Trade-offs

- **[The fan-out changes a path every input feature depends on]** → It is one
  provider and six call sites, all of which already treat `events()` as
  fire-and-forget. Each converted consumer keeps its existing test, and the
  fan-out gets its own tests for the property that matters: two listeners both
  receive every event, and one unsubscribing does not disturb the other.
- **[A pushed mapping and the app's own could drift]** → Same mitigation as the
  echo mode: the app owns the policy and pushes it on every input that can change
  it (device change, calibration completing, mapping edited, leaving the player),
  and pushing is idempotent so over-calling is safe. The pure decision is
  host-tested in `audio_core.rs`, as `echo_event` is.
- **[A wrong mapping is worse than none — it silently misattributes strokes]** →
  Which is why the monitor shows the translation, the mapping is reviewable as a
  table, and clearing it is one action. A player who calibrates badly can see
  that they did.
- **[Calibration on a kit that cross-triggers]** → A hard rim shot can trigger
  the head sensor too, so a step could record the wrong number. Recording the
  *next* stroke rather than the loudest, plus the conflict check, keeps this
  visible; the manual pass on the tester's kit is where the interaction gets
  tuned.
- **[Bridge API change]** → `flutter_rust_bridge_codegen generate` is required,
  as it was for `set_midi_echo`. No wire or gRPC surface is involved.

## Open Questions

- Should an unmapped number be **audible** or **silent**? Today it sounds
  whatever the SoundFont has at that index, which is honest ("something reached
  the app") but can be a cowbell where a tom was meant. Leaning audible, on the
  grounds that silence is what the tester reported as the bug — to be confirmed
  on the kit.
- Where the monitor lives: inside the settings modal's MIDI section, or its own
  screen reachable from it. The modal pauses playback while open, which is
  wrong for a surface whose whole purpose is watching live input — likely its
  own screen.
- Whether the calibration pass should offer the pieces of the **loaded score's**
  kit or a fixed standard kit. The score's kit is shorter and more relevant; a
  fixed kit is calibrated once and serves every score. Leaning fixed standard
  kit, offered from the settings rather than from a score.
