## Why

The Synthesia mode and the on-screen keyboard are built around a piano: a
contiguous range of pitches (`kPianoLowest 21 .. kPianoHighest 108`), preset sizes
of 25/49/61/88 keys, and an auto-fit that widens the window to cover the piece.
None of it survives contact with a drum kit, which is an **unordered set of about
ten pieces**, not an interval. There is nothing to auto-fit and no 61-key subset of
a snare drum.

This change replaces both surfaces for a percussion score: the waterfall gets one
lane per kit piece present in the piece, and the keyboard becomes a strip of pads.
It is the mode where a drummer actually reads the music, so it is where the design
has to be right rather than merely working.

The design was settled against a drummer's feedback during exploration; the
settled reference is `mockups/cascade.html` together with the delta specs, while
the other mockups record earlier iterations (banner-marked as superseded), and
the alternatives that were tried and rejected are recorded in `design.md` so the
debate is not re-run later.

## What Changes

**One lane per kit piece, ordered by a rule**

- Lanes are derived from the pieces **actually present** in the piece, not from a
  fixed kit. A three-element groove gets three wide lanes, not ten narrow ones with
  seven empty.
- The order is a sort rule, not a list: hi-hat (or ride when there is no hi-hat)
  first, then snare, then toms high-to-low, then ride/crash and accent cymbals,
  then any auxiliary percussion (cowbell, clap, tambourine…) in stable General
  MIDI order — a terminal bucket that makes the rule total, so nothing the score
  contains is ever dropped or left unplaced.
- The invariant the rule protects: **position 1 is whatever is struck continuously,
  position 2 is the snare when the score has one**. Together they carry the
  overwhelming majority of a groove's notes, so they must fall inside a single eye
  fixation, and added pieces must appear on the right rather than pushing between
  them; a snare-less score simply closes ranks leftward.

**The kick is a full-width bar, never a lane**

- A lane encodes *where to aim*. The foot does not aim — it is always on the same
  pedal — so a lane spends horizontal space on information that does not exist and
  forces the eye to compare two distant positions to answer the only question that
  matters: does the foot land with the hand, or between? A bar answers it by
  intersection.
- **The bar is drawn beneath the note layer**, thinner than a note and attenuated.
  Drawn on top it hides the hand note exactly on coincidences — the information the
  bar exists to carry.

**The keyboard becomes a pad strip**

- Same order as the lanes, so the eye never translates between two mappings, with
  the kick as a single wide pedal beneath the pads rather than one pad among them.
- The keyboard's range concepts — range modes, auto-fit, preset coverage, adaptive
  height by key count — do not apply and are not carried over.

**Open vs closed hi-hat is a note variant, not a foot event**

- What players call "hi-hat pedal work" is, in the file, a different General MIDI
  number on the **hand** stroke (open vs closed), not a foot note. It is rendered
  as a variant of the note inside the hi-hat lane.

**Inverted-kit setting**

- Reverses the lane order and the pad strip together. It SHALL NOT reverse the
  notation modes: drum notation is a fixed convention, and a player with a mirrored
  kit reads the same score.

**Deliberately not included**

- The **pedal hi-hat "chick"** (General MIDI 44, foot alone): rare outside jazz, and
  designing its encoding without a real score in hand would mean designing it badly.
  The candidate approach and its fallback are recorded in `design.md`. In the
  interim a GM 44 note takes an ordinary lane through the sort rule's terminal
  bucket — visible and aim-able beats invisible-but-scheduled.
- Percussion **notation** rendering (percussion clef, alternative noteheads, two
  voices on one staff) — `add-drum-notation-render`. Until it lands, the mode
  toggle offers only the cascade for a percussion score: Staff and Partition are
  omitted rather than rendering undefined output.
- Percussion **audio** and pad input — `add-drum-audio-channel`,
  `add-drum-input-mapping`; percussion **scoring** and Wait Mode —
  `add-drum-scoring`. The interim is stated in the deltas rather than left
  implicit: the pads ship display-only with no feedback (a tap produces nothing
  until input mapping lands), and Wait Mode is not offered for a percussion score
  — with no input path, its gate would wait forever — so playback runs in the
  timed modes only.

## Capabilities

### New Capabilities

- `music-drum-kit-view`: the percussion controller and waterfall — how lanes are
  derived and ordered from the pieces present, the kick's full-width bar and its
  place in the draw order, the pad strip and its alignment with the lanes, the
  open/closed hi-hat variant, and the inverted-kit setting.

### Modified Capabilities

- `keyboard-display`: for a percussion score the on-screen keyboard is replaced by
  the pad strip, and the range apparatus (range modes, auto-fit, preset coverage,
  key-count-driven height) does not apply; the "shared range" contract between
  keyboard and waterfall becomes a shared **lane order** between pads and cascade.
  The input, feedback, polyphony, assist-key and hideability requirements are
  scoped to keyboard scores, with the pad interim stated explicitly: the strip is
  display-only until `add-drum-input-mapping`.
- `hand-color-coding`: the blue/amber convention is keyed to **voice** (hands /
  feet) for a percussion score rather than to staff 1 / staff 2, since a drum score
  is written on a single staff — voice 1 (stems up) is the hands, voice 2 (stems
  down) the feet, with a single-voice fallback by General MIDI number. The
  convention is stated there once, normatively, and referenced everywhere else.
- `hand-selection`: for a percussion score the selector reads **hands / feet**
  instead of right/left, keyed to the same voice convention as the colours, and is
  offered despite the single staff; hiding the feet hides the kick bar.

## Impact

**Products**

| Product | Consumes | New |
|---|---|---|
| **Music** (`apps/music`) | the parser's instrument classification and General MIDI numbers | the lane derivation, the cascade painter, the pad strip, the inverted-kit preference |
| **Back-office / ID / Live / Site** | — | untouched; this change is app-only |

**Code**

- `apps/music/lib/painters/`: a percussion cascade painter beside
  `synthesia_painter.dart`, and a pad-strip painter replacing
  `piano_keyboard_painter.dart` / `piano_layout.dart` for percussion.
- `apps/music/lib/painters/keyboard_range.dart`: untouched but bypassed — a
  percussion score never consults it.
- `apps/music/lib/state/player_data.dart` + `player_preferences.dart`: the derived
  lane layout and the persisted inverted-kit flag.
- `apps/music/lib/l10n/`: kit-piece names (fr/en).

**Depends on** `add-unpitched-notation` (the General MIDI number per note) and
`add-drums-access` (the instrument classification and the audience gate). It does
not depend on the audio or notation-rendering changes and can land before either.

**Reference material.** `mockups/cascade.html` is the settled design, including
the draw-order defect and its fix — it and the delta specs are the reference. The
other three mockups record earlier iterations and are banner-marked as
superseded: `mockups/lane-order-alternatives.html` argues the core-adjacency case
against the rejected pitch ordering, but the order it presents as the winner is
the initially-adopted snare-first one, later revised to hi-hat-first (see
`design.md`); `mockups/hihat-pedal.html` separates the two hi-hat-pedal cases but
carries the same superseded snare-first lane labels; `mockups/play-modes.html`
situates the cascade among the three play modes but still draws the kick as a
lane and a pad rather than the settled full-width bar and pedal.
