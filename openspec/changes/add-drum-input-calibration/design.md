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

### D13 — Only a piece the player says the kit lacks stops being awaited

*Asked for as "notes on a piece marked missing should no longer be awaited in
Wait Mode".* The intent is right and the literal rule is a trap, so the trigger
moved.

Read literally — "no mapping entry ⇒ not awaited" — the majority case breaks: a
standard GM kit that never needed calibrating has **no entries at all**, so
nothing would be awaited and the gate would open on every onset with no stroke
played. Wait Mode would look like it was working and be doing nothing. Narrowing
it to "once the device has at least one entry" only moves the trap: calibrating
the single non-standard pad would stop the gate waiting for the eight that were
fine.

What is unambiguous is the answer the player already gives during the pass —
**"this kit has none"**. So that answer is now *stored* (it was thrown away),
per device, beside the numbers:

- the gate does not wait for those pieces, and the scorer does not judge them:
  being marked down for a pad you do not own is a verdict about your hardware;
- they are still **drawn** — the score is the score, and the run is still
  submitted (the focus mechanism, which hides and un-submits, says something
  else: "not this lap", a statement about the passage rather than the kit);
- the two silences stay apart everywhere: "not calibrated yet" is a gap to fill,
  "this kit has none" is an answer, and the table shows them differently.

The stored form is a **string where every other value is a number**
(`{"kitPieceChina": "none"}`). A build that predates absences drops non-int
values entry by entry, so it reads the numbers and treats the absences as "not
calibrated" — its own behaviour, rather than refusing the table whole.

One trap it opens deliberately: a player can tap "this kit has none" through
every step and end up with a gate that waits for nothing. That is why the
settings name absences in their own line before playing, rather than reporting
"everything is calibrated" and leaving the silence to be discovered mid-groove.

### D12 — The table is the score's kit, not the stored rows; a pass may cover part of it

*Raised by the product owner from a screenshot: four learned pads listed above
"Recommencer", on a groove that needs nine — "the user does not understand what
they are supposed to do here."* Exactly right, and the reason is that the table
answered a question nobody arrives with. Someone opens this surface because
something did not respond; listing what already works says nothing about the
five pieces that do not.

So the rows are now **the score's calibration targets**, learned or not, each
missing one saying so where its mapping line would be — and the actions are
"calibrate the N missing" (the ordinary answer for someone coming back) and
"start over". The second is offered only when it differs from the first.

That forces a semantics the pass did not have. It used to store *what it
learned* as the device's whole table, which was harmless while it walked the
whole standard kit and became **destructive the moment D10 scoped it to a
score**: calibrating a three-piece groove would have erased the toms learned
from another one. A pass now carries the device's stored table into its state
(`CalibrationState.known`), so:

- what it never asked about survives it, by construction rather than by a merge
  step someone must remember;
- a number an untouched piece already holds still **collides**, which a merge
  after the fact could not have caught — a partial pass would otherwise have
  handed one number to two pieces in silence;
- `dropped` exists because `recorded` can add but not remove: "this kit has
  none" on a piece that *was* learned takes its entry away rather than leaving
  the contradiction standing, and so does reassigning its number to another
  piece.

### D11 — One MIDI-input door per score; the raw read-out sits below the pass

*Raised by the product owner looking at the settings after D9/D10: "the monitor
serves no purpose now that each piece can be calibrated one by one."* Nearly
right, and the fix is placement rather than deletion.

What the pass genuinely made redundant is the monitor as a **first** step: a
drummer no longer needs to read raw numbers to find out what their pads send —
the pass asks for each piece by name and records it. What the pass structurally
cannot show is the rest:

- **A double trigger.** A rim that also fires its head sends two events; the pass
  records the first and advances, so the collision passes through it invisibly.
  The monitor shows both lines — this is the trap `tasks.md` §9.3 warns about.
- **A mapping that is *wrong*.** The table shows what is stored; the monitor
  shows what actually happens, stroke by stroke (`your 26 → read as 46`).
- **Velocity, channel, note-off.** A pad sending velocity 0, a module on an
  unusual channel, a kit that sends no note-off: no piece list ever shows these.
- **A keyboard.** Since D8 the pass is percussion-only, so for a pianist whose
  instrument is silent the monitor is the *only* diagnostic in the app.

So: the settings offer exactly one MIDI-input entry per score — the pass on
percussion, the monitor on anything else — and on a percussion score the monitor
moves one level down, to the bottom of the calibration surface, labelled for what
it is there ("Watch strokes live — when a pad is learned wrong, or fires two at
once"). A player meets the repair first and the diagnostic exactly where the
repair stops explaining things.

### D10 — …reversed: the pass calibrates the loaded score's kit (supersedes D7)

*Decided by the product owner after running D9's pass on the kit.* D7 reasoned
from the mapping ("hardware, not music") and got the ergonomics wrong. With D9's
zones the standard list is twenty-three steps, and a drummer opening a
hi-hat-and-snare groove answered "this kit has none" nineteen times to teach the
app three pads. The pass now asks for `PlayerData.calibrationTargets` — the
pieces and zones **the loaded file writes**, in the standard kit's order, with
anything the standard order does not name appended rather than dropped (a score
writing a bongo must not leave the bongo the one uncalibratable piece).

What D7 was protecting is real and is now paid for differently: a piece absent
from *this* score cannot be learned from it. That cost is bounded — the score
that writes the piece is the one that asks for it, and the pass is one tap from
the settings of that score — and the state it creates is made **visible before
playing** rather than left to be discovered mid-groove: the settings name, under
the calibration action, exactly the pieces this score asks for that the connected
device has no entry for ("Not learned yet for this piece: open hi-hat, ride
bell"), and say so plainly when there are none. That line is the reason the trade
is acceptable; without it, "the pass is short now" would just mean "the gaps are
invisible now".

The targets are read from `notes`, never `visibleNotes` or the focus selection:
practising one hand, or muting the ride for a lap, states what is *asked of the
player*, and must not shrink what the instrument can be taught. The standard kit
remains the fallback when there is no percussion score to read — the surface has
to work before a score is loaded, and a keyboard score's numbers are pitches that
would name pieces nobody struck.

### D7 — The pass calibrates a fixed standard kit, not the loaded score's
*(superseded by D10 — kept for the reasoning it records.)*

*Settled during implementation (task 6.7).* A mapping describes a piece of
hardware, not a piece of music: calibrating from a groove that has no toms would
leave that kit unable to map its toms at all, and the player would have to find a
score that happens to use every piece they own. The pass therefore offers
`kCalibrationPieceOrder` — the named kit, round the way a drummer sits at it
(kick, snare, hi-hat, toms high to low, then the cymbals) — and is reached from
the settings rather than from a score. Every step is skippable, so a kit that
lacks a piece costs one tap.

### D8 — The mapping applies to percussion scores only

*Found during implementation (task 5.5), and not in the original design.* The
first version translated every live event, which is wrong: the table says "this
pad is the snare", a statement about **kit pieces**. Applied to a keyboard score
it would bend that score's *pitches* — a drummer who calibrated their kit to send
31 for the snare would find a note transposed on the piano, for a table that was
never about pitch.

"Is a kit connected" cannot answer this — a module with keys is one device — but
what the score asks for can. So the seam is identity on anything that is not a
percussion score, and the engine is pushed exactly what the app applies, so the
two cannot drift. A keyboard score with a calibrated kit connected is byte-
identical to today, which is now asserted rather than assumed.

*Extended after the first beta pass (task 10.1).* The **entry point** now follows
the seam: the calibration tile is offered on a percussion score only. It was
offered on every score, so a pianist was invited to calibrate a kit whose mapping
the seam would then refuse to apply — an invitation to do nothing. The monitor
keeps the opposite treatment for the opposite reason: it interprets nothing, it
reports what arrived, and "nothing is arriving at all" is an answer a pianist
needs as much as a drummer.

### D9 — The pass asks at the grain the hardware has, not the grain the eye has

*Found by the beta's first calibration pass (task 10.2).* `kCalibrationPieceOrder`
was the list of *lanes*, and a lane deliberately collapses the numbers that share
one aim point: the closed and open hi-hat are one pad, the snare and its rim one
drum, the ride and its bell one cymbal. That is right for the eye and wrong for a
module, which fires each zone on a number of its own — so the parts of a kit most
likely to send something nonstandard were exactly the parts the pass could not
ask about, and a rim or an open hi-hat stayed silent and inert *after* a
calibration that reported success. The auxiliary pads (cowbell, tambourine…) were
missing for the same reason: no lane, no question.

The pass therefore asks for zones as well as pieces, keyed by the General MIDI
number they translate to (`gm:37`, `gm:44`, `gm:46`, `gm:53`) — the identity form
the terminal bucket already uses, so `canonicalGmOfPiece` needed nothing new.
Downstream is untouched by construction: `drumPieceIdOf(37)` is still the snare,
so a learned rim flashes the snare pad, satisfies snare onsets and scores as the
snare. The one distinction that survives translation is the one the app already
draws — open versus closed hi-hat, which `sameStrokeArticulation` shades a verdict
with and never gates on.

Two consequences, both deliberate:

- **The list now runs past most kits** (18 kit steps, then 5 auxiliary pads). So
  the pass gains a third exit beside "skip this one" and "stop": *finish here*,
  which completes and stores what it has learned. Without it the only way to keep
  a five-piece kit's mapping would be to tap "this kit has none" a dozen times,
  and the other exit — abandoning — is defined to keep nothing (D4). It is offered
  only once something is recorded, where it is not merely "stop" renamed.
- **The auxiliary pads come last**, after a line saying so, because they are the
  part of the list most kits answer "none" to. A drummer whose kit ends at the
  china finishes there.

The prompts speak the player's language: the zones and auxiliary pads gain
localised labels, read through the same table the pad strip labels a lane with,
so what a player is asked to hit and what lights when they hit it cannot read
differently. The monitor keeps naming numbers by the General MIDI standard — it
reports what the *standard* calls a number, which is a different question.

### D5 — The monitor shows raw and resolved side by side, and stays honest about silence

Three facts per event, in decreasing rawness: the number as received, the number
after translation (only when they differ), and the piece it resolves to in the
loaded score's kit — or an explicit "matches nothing". It also flags a number the
loaded SoundFont has no sample for, since that is the specific shape of the beta
report and the one thing the player cannot deduce from anything else on screen.

Bounded history (a fixed ring), so a session left open overnight cannot grow.

### D9 — The monitor states what the *standard* covers, not what the font samples

*Settled during implementation (task 3.3).* D5 promised a "will not sound"
marker: a number the loaded SoundFont has no sample for. What shipped is a
narrower, true statement — a number outside the General MIDI percussion map
(35–81) is reported as unknown to the app.

The two are not the same, and for the shipped kit they differ in both
directions. Measured through rustysynth, `FluidR3Drums-bank128.sf2` sounds every
number from **27 to 87**, so 27–34 and 82–87 are reported as unknown while in
fact sounding — pessimistic, but never misleading in the dangerous direction. A
*narrower* font would open the other case: a number inside 35–81 that the font
does not sample, silent with nothing on screen saying so.

*Why not read the font's real range:* it lives in the engine, so it needs a
bridge API. *Why not hard-code 27–87:* the kit font is swappable — bundled,
catalogue, imported (`selected_kit.dart`) — so a constant would be a guess about
the one thing that varies, which is precisely what this task exists to avoid.

*Why the gap is a diagnostic refinement rather than a hole:* a calibration
translates whatever a pad sends into its piece's **canonical** number, and every
canonical number this app can produce sits inside 35–81 — pinned by a test, so
adding a piece outside the map cannot silently reintroduce a pad that calibrates
and still makes no sound. Any GM-compliant drum font samples that range by
definition, and the shipped kit contains it whole. So a silent pad cannot
survive calibration; it can only survive **not being calibrated** — which is
exactly what the two markers the monitor does have are there to reveal.

Worth revisiting if imported fonts with exotic ranges become common.

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
*Settled:* which pieces the pass offers (D7).
