## Context

Every live note in the player converges on two entry points —
`PlayerNotifier.noteOn`/`noteOff` (`player_notifier.dart:425`, `:458`) — from
three sources: the MIDI stream (`_onMidi`, `:357`), the on-screen keyboard
(`player_screen.dart:234`), and the computer-keyboard assist. Below the one
source-sensitive decision (the instrument-sounds-itself sounding rule,
`audio-output-routing`), everything is source-blind: held-state bookkeeping,
key feedback, the Wait Mode gate, the scorer feed. `audio-output` relies on
that convergence, and so will `add-drum-scoring`.

Percussion is already half-connected to it. `add-unpitched-notation` resolves
drum notes to General MIDI numbers (normalizing MusicXML's one-based
`<midi-unpitched>`: the General MIDI number is the element value minus one),
`add-drum-kit-view` derives lanes and a pad strip from them
(`state/drum_kit.dart`), and the Rust event decoder is channel-agnostic by
construction (status high nibble only, `midi_core.rs:229` — pinned for channel
6, not yet for an e-kit's channel 10). So a stroke from a connected kit
**already arrives** at `noteOn` today. What is wrong is everything after
arrival: it is synthesized as a piano pitch (`_audio.noteOn(pitch)` is
instrument-blind), the pads it should light are a bare `CustomPaint` with no
`Listener` (`player_screen.dart:432`, display-only by explicit design), and
the keyboard-shaped scorer would arm on a full percussion run
(`_maybeStartRun`, `:138`, checks `isSelectiveRun` but not `isPercussion`) and
judge GM key numbers as piano pitches.

This change is the input half of the remaining pair: it makes strokes exist,
sound and show. `add-drum-audio-channel` (its predecessor) owns *how* a stroke
sounds; `add-drum-scoring` (its successor) owns whether it was *right*.

## Goals / Non-Goals

**Goals:**

- Make the pad strip a playable controller: taps emit real note events through
  the existing entry points, deterministically numbered.
- Receive external drum input on percussion's own terms: any channel,
  attack-only meaning, one-shot sounding, module-sounds-itself respected.
- Give every stroke honest feedback — a struck flash that claims nothing about
  correctness.
- Keep judgment fully out: no scored run, no Wait Mode, nothing for
  `add-drum-scoring` to repudiate.

**Non-Goals:**

- The percussion synthesizer, the drum SoundFont channel, and the one-shot
  hook itself — `add-drum-audio-channel`. This change calls the hook; it does
  not build it.
- The matcher and everything downstream of it: correct/incorrect pad states,
  Wait Mode for percussion, stroke equivalence (does a 40 satisfy a 38?),
  scoring, rewards — `add-drum-scoring`.
- Percussion notation input aids (nothing here touches the notation modes,
  which `add-drum-notation-render` owns; the two changes are independent).
- Velocity dynamics. Strokes sound at the uniform default loudness, like the
  pitched path — see Decisions.
- A computer-keyboard mapping for pads (desktop letter keys as drum pads) —
  see Open Questions.

## Decisions

### Every stroke goes through the existing entry points

Pad taps, pedal taps and external strokes all call the same
`noteOn`/`noteOff` the keyboard uses, carrying the General MIDI number as the
pitch.

*Rationale:* the convergence is the architecture's one load-bearing input
property: `audio-output` keys its sounding rules on it, and `add-drum-scoring`
will arm the gate and the scorer against these entry points without
re-plumbing input. A stroke that bypassed them would have to be re-taught to
every later consumer.

*Alternative rejected:* a dedicated percussion stroke bus beside the note
path. It looks cleaner ("strokes are not notes") but forks the convergence:
the instrument-sounds-itself rule, the held-state hygiene and the future
scorer feed would each need a second implementation, and the two would drift.

### Pads emit the first canonical member the score uses

Each named piece's General MIDI members get a canonical order pinned in the
kit table — hi-hat: closed 42, then open 46; snare: acoustic 38, electric 40,
side stick 37; kick: Bass Drum 1 36, then Acoustic Bass Drum 35; ride: 51, 59,
53 — and a tap emits the **first member the loaded score actually uses**. A
generic (terminal-bucket) piece has exactly one number and emits it.

*Rationale:* determinism first — one pad, one number, no runtime cleverness.
Emitting *inside the score's own vocabulary* keeps the stroke audible with the
score's own sound and spares `add-drum-scoring`'s matcher avoidable
same-piece mismatches: a score written with the electric snare (40) gets a pad
that emits 40, not a 38 the file never contains. It also keeps the flash
lookup trivially correct, since the emitted number is always a member of the
lane that must light.

*Alternative rejected:* a fixed canonical number per piece regardless of the
score (always 38 for the snare). Simpler to state, but the emitted number can
then be absent from the lane's member set, the struck-lane lookup misses, and
every stroke on such a score is a guaranteed matcher mismatch later.

*Alternative rejected:* emitting whatever number the nearest upcoming onset
carries. Clairvoyant input — free play away from the playhead would have no
answer, and the pad would change identity mid-piece.

### The kick pedal emits 36

*Rationale:* General MIDI has two kicks — 35 "Acoustic Bass Drum" and 36
"Bass Drum 1" — and 36 is what notation exports and e-kit default maps almost
universally use for *the* kick; 35 is the map's second kick. The canonical
order (36, then 35) plus the present-member rule means a score that writes its
kick only as 35 gets a pedal that emits 35 — the stroke stays in the score's
vocabulary either way. Both numbers drive the bar and classify as feet
(`kKickGmNumbers`, `kFootGmNumbers`), so nothing downstream can tell the
difference today; the choice matters only to the future matcher, which is
exactly why it is pinned now.

### No on-screen gesture distinguishes an open from a closed hi-hat stroke

The strip presents one hi-hat pad; its tap emits the lane's first present
member (the closed 42 whenever the score uses it — or 46 when the score's
hi-hat lane holds *only* the open number).

*Rationale:* on the instrument, open versus closed is the foot on the hi-hat
pedal modulating the same aim point — not a second place to hit. The kit view
already encodes this (one lane, note variant); a second pad would contradict
it, shrink both touch targets, and teach a kit that does not exist.

*Alternative rejected:* a split pad (closed left, open right). Two aim points
for one piece — the exact confusion the one-lane rule exists to prevent.

*Alternative rejected:* a modifier gesture (long-press or a second finger held
for open). Undiscoverable, and unplayable at tempo — an eighth-note hi-hat
line leaves no time to modulate a gesture.

*Consequence, stated honestly in the spec:* on-screen play offers no gesture
that distinguishes open from closed — the pad emits its lane's first present
member, so a score that uses the closed 42 can never emit the open 46 from
the screen. An e-kit produces 46 naturally and it flows through the same
path. Whether a closed stroke *satisfies* an open onset is matcher
equivalence — `add-drum-scoring`'s question, noted there.

### The whole strip is live — no dead gutters

Hit testing maps any pointer in the pads band to the lane whose horizontal
span contains it, and any pointer in the pedal band to the kick. The visual
insets between pads (3 px in the painter) are styling, not hit boundaries.

*Rationale:* a drummer at tempo aims at a region, not a rounded rectangle. A
tap swallowed by a decorative gutter is a ghost stroke, and a ghost stroke
reads as broken input — the same reasoning that gives the keyboard's black
keys a generous priority region in `Pointer Pitch Hit-Testing`.

*Alternative rejected:* hit-testing the drawn rectangles. Honest to the
pixels, hostile to the hands.

### Each pointer-down is a fresh stroke, even on a touched pad

*Rationale:* a roll is played by alternating two fingers on one pad. A
keyboard-style exclusivity rule (a held pitch must be released before it can
retrigger) would swallow every second stroke of it. One-shots have no voice to
steal, so there is nothing to protect. The held-state set keeps its existing
semantics (a set of numbers, releases clear entries); overlapping same-number
holds need no extra bookkeeping because nothing sustained depends on it.

### Feedback is one honest state: a time-based struck flash

A struck pad (or the pedal) flashes briefly and decays on its own — from a
stroke of any source. No expected, no correct, no incorrect. The cascade does
not react to strokes.

*Rationale:* with no matcher there is no truthful second state — a
three-state imitation of the keyboard would either claim correctness the
product cannot judge or require building a proto-matcher that
`add-drum-scoring` owns. Time-based rather than hold-based because percussion
releases arrive within milliseconds; a hold-driven highlight would be an
invisible flicker. Controller-only because that is the division the keyboard
already makes: keys light, falling notes do not.

*Alternative rejected:* keying a "correct" flash on
`onsetPitchesAt(elapsedMs)`. It is exactly a matcher — exact-pitch,
equivalence-blind — smuggled in as feedback.

### External input: channel-agnostic, stated; velocity received, not consumed

The decoder's channel-blindness (status high nibble only) is promoted from an
implementation accident to a normative property of the `midi` stream, with an
e-kit-shaped test (channel 10). No percussion channel filter is added
anywhere.

*Rationale:* General MIDI reserves channel 10 for percussion, but e-kits are
configurable and off-by-one channel confusion (0-based vs 1-based) is
endemic; a filter would silently drop a misconfigured kit's strokes — silence,
the worst failure an input path has. And the app never needs the channel: the
instrument context is score-scoped (piano XOR drums, `add-instrument-context`),
so for a percussion score *every* incoming note number is a stroke. Without
the normative statement, a future reader "fixing" the stream to filter
channel 10 would pass every existing test.

Velocity: the stream carries it (per the `midi` spec) and this change leaves
it unconsumed — strokes sound at the synthesizer's uniform default, exactly as
the pitched path always has. The hook does accept a velocity
(`add-drum-audio-channel` defines `drum_on(key, velocity)` and plays its own
schedule at `DEFAULT_VELOCITY`); this change fills the parameter with that
same default rather than the stroke's value, so consuming real velocities
later is one argument, not a signature change.

*Rationale:* live piano input is flattened today, and the percussion
*schedule* is flattened by the audio change's own decision; making live drum
strokes the one dynamic voice in the mix would have the backing track flat
under an expressive instrument — dynamics is one deliberate decision for both
instruments and both directions (schedule and live), later. The audio
change's open questions name this change as the one that may force it; the
answer here is: not yet, and knowingly.

*Alternative rejected:* pass real velocities for MIDI-source strokes now
"since the parameter is already there". The plumbing is trivial; the mix
decision is not.

### Releases are bookkeeping, never meaning

A percussion note-off clears the held-state entry its note-on created —
nothing else. No audible effect (a one-shot has no sustain to end), no
feedback effect (the flash is time-based).

*Rationale:* e-kits send note-off (or a velocity-0 note-on) within
milliseconds of the attack; some hardware is sloppy about it. A path where
releases carry meaning would inherit that sloppiness. Processing them for
hygiene keeps the shared entry points symmetric and the held set from
accumulating stale entries. The seam pairs `drum_on` with a `drum_off(key)`
(`add-drum-audio-channel`); whether the release is forwarded to it is
plumbing, bounded by the binding scenario: an immediate note-off must leave
the one-shot sounding to its natural end — a cymbal keeps ringing after the
stick has left it. If the engine's `drum_off` would clip the voice,
near-immediate releases are not forwarded; the scenario, not the pairing, is
the contract.

### Input is never suppressed — filters act on visibility and judgment

Hand selection (hands / feet / both) filters what falls and what will one day
be judged; it never filters what the player may do. A foot stroke during
hands-only practice sounds and flashes the pedal; a hand stroke during
feet-only practice sounds and flashes its pad. A stroke whose number resolves
to no lane at all (a crash over a hi-hat/snare/kick groove, any stroke on a
piece the score does not use) sounds and simply has no pad to flash — free
play, not an error.

*Rationale:* this is how the keyboard already behaves — unselected-hand keys
still sound, keys outside the piece still sound — and it is what keeps
feedback honest: an instrument that goes silent because of a display filter
feels broken, not filtered. The `visibleNotes` filtering
(`player_data.dart:673`) already stops at presentation; this decision writes
that boundary down for percussion before a matcher arrives and makes
suppression tempting.

### The scorer never arms for a percussion score

`_maybeStartRun` gains an `isPercussion` guard, the same never-arms mechanism
a selective run uses (`add-measure-range-practice`, D2): no run, so every
scorer feed call is a no-op by construction, no session result exists, and
the summary, the persisted history and the backend ingest sites (already
failing closed per `music-drums-visibility`) have nothing to receive.

*Rationale:* the scorer is keyboard-shaped — exact-pitch matching against
numbers one lane deliberately collapses, sustain judgment against one-shots
with no sustain. **Stacking note:** `add-drum-kit-view` left this armable — a
timed percussion run today opens a scored run over the drum part — which was
latent while the pads were inert and no drum input existed in practice; the
moment this change makes strokes real, that run would start judging them.
The guard therefore lands here, as a `performance-scoring` delta, not as a
silent fix.

*Alternative rejected:* arm the scorer but discard the result. It still shows
a keyboard-shaped gauge and summary during and after the run — confident wrong
feedback, the exact failure mode `add-drums-access` documents.

### Wait Mode stays withheld — for a new reason, recorded

The sibling's requirement ("Wait Mode is not offered for a percussion score,
for now") keeps its rule and scenarios, but its rationale — *no input path
exists, the gate would block forever* — becomes false the moment this change
lands. The body is refreshed: the gate's exact-pitch test is dishonest for
percussion (one lane collapses several numbers; the pad emits one canonical
member; a correctly aimed stroke would be refused whenever the file's number
differs), and the equivalence table that fixes it is the matcher, owned by
`add-drum-scoring`.

*Rationale for modifying rather than leaving it:* an archived spec whose
justification is knowably false invites the next reader to "correct" the
requirement instead of the rationale. **Stacking note:** this is a MODIFIED
requirement on top of `add-drum-kit-view`'s still-unarchived ADDED
requirement in `music-drum-kit-view` — the base text is the sibling's, not a
main-spec version (none exists yet).

## Risks / Trade-offs

**The one-shot hook is consumed while it is still paper** → the contract is
now written (`add-drum-audio-channel` defines `drum_on(key, velocity)` /
`drum_off(key)` on the injectable audio seam, exercises them from scheduled
playback only, and states that wiring live strokes is this change's and needs
no engine modification), but both changes are unimplemented proposals, so the
verbs can still shift under implementation. Mitigation: the consumption
surface is one call site behind the seam, and the delta specs here constrain
only *that strokes sound as one-shots*, not the hook's signature.

**Tap-to-sound latency is more exposed on drums than on piano** → percussion
play is timing play; the same output latency that is tolerable under a piano
line feels wrong under a snare. Mitigation: the stroke path adds no buffering
of its own (one call from pointer event to the hook), the existing
output-offset machinery applies unchanged, and the manual pass includes a
feel check on device.

**Double-sounding with a drum module** → an e-kit whose module is audible
would be doubled by the synth. Covered by the existing
instrument-sounds-itself rule, consulted unchanged for percussion — but the
manual pass verifies it with a real kit, because the rule has only ever been
exercised by pianos.

**The flash duration is a guess until felt** → too short is invisible under a
fast groove, too long smears rolls into a solid glow. Mitigation: the value is
one constant, tuned in the feel pass like the kit view's attenuation, and the
spec pins only "brief and time-based", not the number.

**A judgment leak would be permanent-looking** → if any path still arms the
scorer or offers Wait Mode for percussion, testers see confident wrong
verdicts. Mitigation: both guards are asserted by test (no scored run on a
percussion full run; no Wait Mode offer), not just implemented.

## Migration Plan

None. No schema, no wire protocol, no persisted data. The new behavior is
reachable only for a percussion score, itself reachable only by the drum
audience (`music-drums-visibility`). A keyboard score's input path is
untouched.

Ordering: implementation cannot start before `add-drum-audio-channel` lands
the one-shot hook (until then a percussion-score stroke keeps today's
behavior — the pitched piano voice — which this change removes for
percussion). Rollback is a revert: the strip degrades to display-only again
and external strokes go back to sounding as piano pitches, both interims the
sibling specs already describe.

## Open Questions

- **When do live strokes start passing real velocities?** The hook's
  parameter exists (`drum_on(key, velocity)`) and this change deliberately
  fills it with the schedule's own `DEFAULT_VELOCITY`; both changes' open
  questions now point at the same future decision — schedule shading and live
  dynamics together, for both instruments. (The two questions this bullet
  replaces — whether the hook should take velocity, and who wires live
  strokes — were settled by `add-drum-audio-channel`'s proposal: it does, and
  this change does.)
- **Does the engine's `drum_off` clip a sounding one-shot?** Its semantics
  beyond "paired" are not pinned in the audio proposal. The binding scenario
  here (an immediate release leaves the sound to its natural end) decides the
  wiring either way — see the releases decision.
- **Desktop percussion input.** A mouse is one pointer, so desktop pad play is
  one stroke at a time; the computer keyboard emits nothing for percussion
  (the assist keys are gate-driven and withheld). Mapping letter keys to pads
  would fix it and is deliberately deferred until a desktop drummer asks —
  recorded here so it is not lost.
- **The flash constant.** Duration and decay curve are feel-pass values; the
  spec pins the semantics only.
- **Matcher equivalence.** Whether a closed stroke satisfies an open-hi-hat
  onset, and whether 35/36 are interchangeable at judgment time, are
  `add-drum-scoring` questions — named here because the emission rule was
  chosen to make them as small as possible.
