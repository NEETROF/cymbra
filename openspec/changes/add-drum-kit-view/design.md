## Context

Synthesia and the on-screen keyboard are one system built on a single assumption:
notes live on a contiguous, ordered axis of pitch. `keyboard_range.dart` encodes it
directly — `kPianoLowest 21 .. kPianoHighest 108`, presets of 25/49/61/88 keys
(the `keyboard-display` spec still lists 37- and 76-key presets the code never
grew — a divergence to reconcile on its own, not smuggled into this change), an
auto-fit that snaps to octave boundaries and widens the window so the
cascade never drops a note. The `keyboard-display` capability then guarantees the
keyboard and the waterfall share that range, which is what keeps falling columns
above their keys.

A drum kit breaks the assumption at the root: it is an **unordered set of roughly
ten pieces**. There is no interval, no octave boundary, no 61-key subset. Every
concept in that file is inapplicable rather than merely inconvenient.

The design below was worked out interactively against a drummer's feedback, and
several first attempts were wrong in instructive ways. `mockups/cascade.html`
holds the resulting reference (the other mockups record the earlier iterations
and are banner-marked as superseded); this document holds the reasoning,
including what was rejected, so the same ground is not re-covered.

## Goals / Non-Goals

**Goals:**

- Give a percussion score a cascade and a controller that read correctly, deriving
  both from the pieces the score actually uses.
- Encode the thing drumming is actually about — whether the foot lands with the
  hands or between them — so it is readable without effort.
- Keep the controller and the cascade a single mapping the player learns once.

**Non-Goals:**

- Percussion **notation** (percussion clef, alternative noteheads, two voices on one
  staff) — `add-drum-notation-render`. This change touches only the cascade and the
  controller.
- Percussion **audio** and pad **input** — `add-drum-audio-channel`,
  `add-drum-input-mapping`. This change draws; it does not sound or listen.
- The **pedal hi-hat "chick"** — see Decisions.
- Any change to the keyboard for a keyboard score. The range apparatus stays exactly
  as it is; a percussion score simply never consults it.

## Decisions

### The kick is a bar across the width, not a lane

*Rationale:* a lane encodes **where to aim**. The foot does not aim — it is always
on the same pedal — so a kick lane reserves horizontal space for information that
does not exist. Worse, it makes the eye compare two horizontally distant positions
to answer the question drumming is actually about: does the foot land **with** the
hand or **between**? A bar answers it by intersection, in peripheral vision, without
moving the gaze. Secondary benefits: roughly a fifth of the width returns to the
pieces that do need aiming, and fast double-kick stays legible as spaced bars
instead of smearing inside a narrow column.

*Alternative rejected:* a lane per piece including the kick — the first design, and
the one that looks most consistent. `mockups/lane-order-alternatives.html` shows the
cost: the eye sweeps 324 px continuously to serve a column that only lights up on
fills.

*Precedent:* this is the Guitar Hero / Rock Band encoding, confirmed by the tester
before it was adopted here.

### Foot bars paint beneath the notes — and thin, and attenuated

*Rationale:* a full-width bar painted **over** the note layer hides the hand note
exactly on a coincidence, destroying the property the bar exists for. The fix is
primarily paint order; thickness and attenuation are what make the layering read as
layering rather than as occlusion.

*Alternative rejected:* attenuation alone. A bar faint enough to reveal a note
behind it disappears across the rest of the width, where it has nothing behind it —
the bar is a note, not a background.

*Alternative rejected:* interrupting the bar in the lane where a note coincides.
Clever, and it destroys the continuous line whose whole value is being readable
peripherally.

*Why this is called out as a requirement rather than left to the painter:* the
defect appears **only on coincidences**. A test score where feet and hands never
align renders perfectly with the layers inverted. It was missed in the first mockup
for precisely that reason.

### Lane order is a sort rule protecting one invariant

Hi-hat (or ride when there is no hi-hat), then snare, then toms high-to-low, then
the remaining cymbals — then everything else (cowbell, clap, tambourine, claves…)
in stable ascending General MIDI order. The terminal bucket is what makes the rule
total: a drum-set part is not limited to the canonical pieces, and a rule that
cannot place a cowbell either drops it silently or leaves the implementer to
invent a position. Neither is acceptable, so the rule ends with a bucket that
catches everything, ordered deterministically.

*Rationale:* the invariant matters more than the list — **position 1 is whatever is
struck continuously, position 2 is the snare when the score has one** (a snare-less
score closes ranks leftward; empty buckets are skipped, never reserved). Those two
carry the overwhelming
majority of a groove's notes and must sit inside a single eye fixation; further
pieces must append to the right rather than insert between them, so a player moving
from a sparse score to a dense one never relearns where to look. Hi-hat leftmost
also matches its physical place on a right-handed kit, which is what the pad strip
mirrors.

*Alternative rejected:* ordering by pitch, low to high — the first proposal. It is a
**notation** convention applied to a **gameplay** surface: it separates the two
dense lanes with a tom that is silent most of the time.

*Alternative rejected:* Rock Band's own order, snare first. Adopted initially, then
revised: it inverts the physical relationship, since the hi-hat sits left of the
snare on a right-handed kit. Rock Band's order is an artefact of its plastic
controller's colour layout, not an optimum.

*Consequence, noted because it changed mid-design:* with the hi-hat at position 1,
the ride can no longer sit next to it — that would push the snare to position 3 and
break the adjacency. The ride therefore joins the cymbals, except when the score has
no hi-hat, in which case it inherits position 1. The rule is keyed to the piece's
**function** (keeping time), not to its name.

### Open/closed hi-hat is a note variant; the "chick" is deferred

Two different things share the name "hi-hat pedal", and only one is a foot event.
**Open versus closed** is a different General MIDI number on the **hand** stroke — no
foot note exists in the file — so it is drawn as a variant inside the hi-hat lane.
The **chick** (General MIDI 44, foot alone, no hand) is a genuine second foot event.

*Decision:* implement the variant, defer the chick.

*Rationale:* the variant covers the common case and introduces no new concept. The
chick is rare outside jazz, and designing its encoding without a real score to test
against would mean designing it badly — the corpus may well contain none for a long
time.

*Interim, pinned in the spec:* until the encoding is designed, a General MIDI 44
note takes an ordinary lane through the sort rule's terminal bucket. That concedes
"the foot does not aim" for one rare event, but visible and aim-able beats
invisible-but-scheduled, and the no-silent-drop requirement applies to the chick
like everything else.

*Candidate approach, recorded not adopted:* a second full-width bar, hollow
(outline only) and in a distinct hue. Hollow-plus-hue works on two perceptual
channels at once, where solid-versus-dashed is a discrimination the eye loses at
scrolling speed; and the two superimpose legibly when both feet strike together — a
filled bar with a bright edge — where two dashed bars would not.

*Fallback if that proves unreadable:* give the chick a narrow lane at the far left,
next to the hi-hat's physical position. It forfeits the "the foot does not aim"
argument and buys unambiguity. A legitimate trade, not a failure.

### The inverted-kit setting stops at the controller

*Rationale:* the cascade and the pad strip are spatial maps of the kit, so they
mirror. Notation is a fixed convention — the snare is on the third space regardless
of how the kit is set up — so it must not. Input must not either: a hi-hat reports
its General MIDI number wherever it physically stands.

*Naming, which is the part most likely to be got wrong:* the setting describes **the
kit's layout, not the player**. Many left-handed drummers play a standard kit, open-
handed or cross-armed; a label naming handedness would invite exactly the people it
does not apply to. It is worth a longer label to avoid a support burden that is
invisible until it arrives.

### One derived layout, two consumers

The lane order is computed once from the score and consumed by both the cascade and
the pad strip, rather than derived independently by each.

*Rationale:* this is the same contract `keyboard-display` already makes between the
keyboard and the waterfall, restated over a different quantity. Two independent
derivations would drift — and the inverted-kit setting doubles the chance, since it
is a second input both consumers would have to honour identically.

## Risks / Trade-offs

**The whole design is judged on feel, not on tests** → lane widths, bar attenuation
and the open-hi-hat variant are all "does it read at speed" questions that a unit
test cannot answer. Mitigation: `cascade.html` is a shared reference before code, and
the manual pass drives real scores at real tempo, on a phone as well as a tablet.
Two known unknowns are already flagged for that pass: whether ~60% attenuation
survives motion (a scrolling bar reads weaker than a static one), and whether the
amber holds its contrast on the Paper theme, where it sits on ivory rather than on a
dark ground.

**A wrong paint order ships silently** → covered by a requirement and an explicit
coinciding-onset test, because no amount of playing a non-coinciding score reveals
it.

**The tester is a sample of one** → the ordering rule, and especially the ride's
placement, rest on one drummer's judgement plus reasoning. Mitigation: the rule is a
single sort function, so revising it is cheap and touches one place. It should be
re-examined once the beta has more than one drummer in it.

**`keyboard_range.dart` stays, unused by percussion** → two controller models
coexist. Accepted: the piano path is untouched and stays as tested, and merging them
would mean generalising an interval into a set for no benefit to either.

## Migration Plan

None. No schema, no wire protocol, no stored data beyond one boolean preference that
defaults to the standard layout. A keyboard score renders exactly as before; the new
paths are reachable only for a percussion score, itself reachable only by the drum
audience.

Rollback is a revert.

## Open Questions

- **The order of the toms among themselves, and the ride's placement.** Settled by
  reasoning plus one drummer. A score that alternates hi-hat and ride between verse
  and chorus is the case that will show whether the ride belongs with the cymbals or
  next to the time-keeping lane.
- **How often does the chick actually appear?** Entirely dependent on the repertoire
  the beta testers load. Rock and pop: almost never. One jazz drummer joining the
  beta moves it to the top of the list.
- **Do several General MIDI numbers collapse into one lane, and which?** The spec
  requires equivalent numbers to share a lane; the equivalence table itself
  (acoustic versus electric snare, the several crashes) is a judgement call best made
  against real files rather than from the General MIDI list.
