## Why

`add-drum-kit-view` gives a percussion score a cascade and a pad strip, but the
strip is deliberately display-only: a tap produces no note, no sound, no
feedback. Meanwhile a stroke from a connected e-kit **already reaches the
player today** — the MIDI path is instrument-agnostic — and is synthesized as a
*piano pitch* (`player_notifier.dart` calls the pitched voice path
unconditionally), so the one input a drummer can currently give a drum score
answers in the wrong instrument. This change closes both gaps: the pads become
a playable controller emitting General MIDI percussion numbers through the
player's existing note-on/note-off entry points, and external drum input is
received on its own terms — any channel, attack-only semantics, sounded as a
one-shot.

This is the third of the four remaining drum changes
(`add-drum-notation-render` → `add-drum-audio-channel` →
**`add-drum-input-mapping`** → `add-drum-scoring`). It **consumes** the
percussion one-shot ("sound this General MIDI number now" — `drum_on`, with
its paired `drum_off`) that `add-drum-audio-channel` exposes on the audio
seam and explicitly leaves for this change to wire, with no engine
modification needed — it does not design the synth path — and it deliberately
stops short of judgment: strokes exist, sound
and flash; whether they are *right* is `add-drum-scoring`'s matcher. It is
independent of `add-drum-notation-render` and can land before or after it. The
audience is unchanged: percussion scores are reachable only by the drum
audience of `music-drums-visibility` (`drums.enabled` + `beta:midi-drums`,
backend-enforced) — nothing here re-derives gating.

## What Changes

**The pad strip becomes playable**

- A pointer-down on a pad emits a note-on for the pad's General MIDI number
  through the same entry points as a key press; pointer-up emits the matching
  note-off. This lifts the display-only interim `add-drum-kit-view` pinned in
  `keyboard-display` ("a tap produces nothing until `add-drum-input-mapping`").
- The emitted number is deterministic: each named piece's members carry a
  canonical order pinned in the kit table, and a tap emits the first member the
  loaded score actually uses — so the stroke sounds with the score's own
  vocabulary and the future matcher is not handed avoidable same-piece
  mismatches (acoustic vs electric snare).
- The **kick pedal** is playable and emits **General MIDI 36** (Bass Drum 1 —
  the number virtually every export and e-kit default map uses for the kick),
  falling back to 35 when the score writes its kick only as 35.
- The **hi-hat pad offers no gesture distinguishing open from closed**: it
  emits its lane's first present member — the closed 42 whenever the score
  uses it, 46 only for an open-only score. Open/closed is pedal-controlled on
  the instrument, not a second aim point, so a split pad or a modifier gesture
  would misteach the kit and be unplayable at tempo. An external kit produces
  46 naturally.
- The whole strip is live (no dead gutters between pads), multi-touch works,
  and every pointer-down is a fresh stroke even on an already-touched pad,
  because alternating two fingers on one pad is how a roll is played.

**Honest stroke feedback**

- A struck pad (or the pedal) shows a brief, time-based **struck flash** — one
  state, claiming nothing about correctness. There is no matcher yet, so
  expected/correct/incorrect states would be a judgment the product cannot
  honestly make; they arrive with `add-drum-scoring`. The flash decays on its
  own rather than tracking the hold, because percussion releases arrive within
  milliseconds. Feedback lives on the controller only; the cascade does not
  react to strokes — the same division the keyboard makes with its waterfall.

**External MIDI drum input**

- The event stream's channel-agnosticism becomes a stated property instead of
  an accident: strokes are accepted whatever channel the kit transmits on, and
  no percussion channel filter is introduced. E-kits commonly send on channel
  10 but are configurable; the loaded score's instrument, not the wire, decides
  how a note number is interpreted.
- Note-offs are bookkeeping, never meaning: they clear held state and nothing
  else. Velocity is received but not consumed — strokes sound at the uniform
  default loudness, exactly like the pitched path today.
- The instrument-sounds-itself rule carries over unchanged: a stroke from the
  connected kit is not synthesized twice when the drum module already sounded
  it.

**Sounding, consumed not designed**

- Every stroke — pad, pedal, e-kit — sounds immediately through
  `add-drum-audio-channel`'s one-shot hook, never through the pitched piano
  voice path. This change adds no synth capability; it wires live strokes into
  the one the audio change builds.

**Guards — strokes exist, they are not judged**

- The keyboard-shaped scorer **never arms** for a percussion score (the same
  never-arms mechanism a selective run uses), so strokes fed to the scoring
  entry points are no-ops by construction and no session result exists for the
  summary, the history, or the backend ingest sites `music-drums-visibility`
  already fails closed. Without this, the scorer that `add-drum-kit-view` left
  armable would start "judging" the strokes this change creates.
- Wait Mode stays withheld — but its recorded reason is refreshed: the old one
  ("no input path, the gate would block forever") becomes false the moment this
  change lands. The true remaining reason is that exact-pitch gating is
  dishonest for percussion (one lane collapses several numbers); the
  equivalence table is the matcher's, owned by `add-drum-scoring`.

## Capabilities

### New Capabilities

- `music-drum-input`: the percussion stroke path — convergence of every stroke
  source on the player's note entry points, channel-agnostic acceptance,
  one-shot sounding through the audio seam, release and velocity semantics,
  input never suppressed by hand selection, and free play on pieces outside
  the score's kit.

### Modified Capabilities

- `keyboard-display`: the pad strip's display-only interim is lifted — pads and
  the kick pedal become playable with a deterministic emission rule, hit
  testing covers the strip with no dead zones, multi-touch extends to pads
  (including same-pad rolls), the pads gain the one-state struck flash, and the
  assist-key exclusion for percussion is re-motivated (the gate, not the input
  path, is what is missing). Two requirements are renamed because their titles
  no longer match what they govern.
- `music-drum-kit-view`: the Wait-Mode-withheld requirement keeps its rule and
  scenarios but replaces its rationale, which this change invalidates.
- `performance-scoring`: scored-run activation explicitly never arms for a
  percussion score until `add-drum-scoring`.
- `midi`: the event stream's channel-agnostic decoding is stated normatively so
  it cannot be "fixed" into a channel-10 filter later.

## Impact

**Products**

| Product | Consumes | New |
|---|---|---|
| **Music** (`apps/music`) | the kit-view lanes and pad strip, the note entry points, the audio seam's percussion one-shot (`add-drum-audio-channel`), the drum audience (`music-drums-visibility`) | pad/pedal input wiring, the emission rule, the struck flash, the percussion no-arm scorer guard |
| **Engine** (`apps/music/rust`) | the existing channel-agnostic event decoding | one pinned test; no behavior change, no public API change |
| **Back-office / ID / Live / Site** | — | untouched |

**Code**

- `apps/music/lib/screens/player_screen.dart`: a `Listener` on the pad strip
  (today a bare `CustomPaint` with an explicit "no Listener" comment at
  `player_screen.dart:432`), pad/pedal hit testing beside the keyboard's
  (`_onKeyboardPointerDown`, `player_screen.dart:234`).
- `apps/music/lib/state/player_notifier.dart`: the percussion branch of
  `noteOn`/`noteOff` (`:425`, `:458`) — one-shot sounding instead of the
  pitched voice — and the percussion guard in `_maybeStartRun` (`:138`).
- `apps/music/lib/state/drum_kit.dart`: canonical member order per named piece,
  the emission helper, pedal/lane resolution for the flash.
- `apps/music/lib/painters/drum_pad_strip_painter.dart`: the struck-flash
  states (pads + pedal).
- `apps/music/lib/state/player_data.dart`: short-lived struck timestamps for
  the flash, derived percussion-input state.
- `apps/music/rust/src/api/midi_core.rs`: one added test pinning channel-10
  note-on decoding (the property exists; the e-kit-shaped case is not pinned).

**Depends on** `add-unpitched-notation` (General MIDI numbers on notes),
`add-drums-access` (audience), `add-drum-kit-view` (lanes, pad strip, voice
convention) — all implemented — and **`add-drum-audio-channel`** (the
percussion one-shot on the audio seam), which must land first.
`add-drum-scoring` builds on this change: it arms the judgment these strokes
already feed.
