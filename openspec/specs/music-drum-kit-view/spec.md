# music-drum-kit-view Specification

## Purpose

The music-drum-kit-view capability governs how Cymbra presents a percussion
score to play against: lanes derived from the kit pieces the score actually
uses, the kick drawn as a full-width bar instead of a lane, a drawing of the
kit itself in place of the on-screen keyboard, a perspective stage alongside
the cascade, a grid taken from the score's own metre, and the inverted-kit
layout setting.
## Requirements
### Requirement: Lanes are derived from the pieces present

The cascade SHALL show one lane per kit piece that the loaded score actually uses,
derived from the General MIDI percussion numbers its notes resolve to — never a
fixed kit layout. A piece absent from the score SHALL NOT occupy a lane, so a
three-element groove is drawn as three wide lanes rather than as a full kit with
empty columns.

Several General MIDI numbers may denote one physical piece (the acoustic and
electric snares, for instance); numbers that map to the same piece SHALL share one
lane rather than splitting it.

#### Scenario: A sparse groove gets wide lanes

- **WHEN** a score uses only hi-hat, snare and kick
- **THEN** the cascade shows two lanes (hi-hat, snare) at full available width, and
  no lane for any absent piece

#### Scenario: A large kit gets its pieces

- **WHEN** a score uses hi-hat, snare, three toms, ride and crash
- **THEN** each of those pieces has its own lane

#### Scenario: Equivalent numbers share a lane

- **WHEN** a score uses two General MIDI numbers that denote the same physical piece
- **THEN** both are drawn in the same lane

### Requirement: Every resolved number is represented — nothing is dropped silently

The cascade SHALL represent every General MIDI percussion number the loaded
score's notes resolve to: as a note in a lane for a hand-struck piece, or as the
full-width bar for the kick. General MIDI 35 and 36 both denote the
kick — the acoustic and the electric flavour of the same pedal — and both SHALL
drive the bar. A number outside the named roles (cowbell, hand clap, tambourine,
claves, wood blocks…) SHALL take a lane of its own as a generic piece rather than
being dropped: a note the player cannot see but is still expected to play — or
one that silently vanishes from the groove — is a worse failure than an
imperfectly placed lane.

#### Scenario: A cowbell part gets a lane

- **WHEN** a score contains cowbell notes (General MIDI 56)
- **THEN** the cowbell has its own lane and its notes are drawn in it

#### Scenario: Both kick numbers drive the bar

- **WHEN** a score mixes General MIDI 35 and 36
- **THEN** both are drawn as the full-width kick bar, and neither occupies a lane

### Requirement: Lane order follows a fixed rule

The lane order SHALL be produced by sorting the pieces present, in this order:

1. the hi-hat — or, when the score has no hi-hat, the ride;
2. the snare;
3. the toms, highest to lowest;
4. the remaining cymbals (the ride when a hi-hat already took position 1, then the
   crashes and other accent cymbals);
5. everything else — the auxiliary percussion a drum-set part may carry (cowbell,
   hand clap, tambourine, claves, wood blocks…), after the cymbals, in stable
   ascending General MIDI order.

The order SHALL be a rule applied to the pieces present, not a fixed list, so that
adding or removing a piece never reorders the others. The terminal bucket makes
the rule **total**: every piece the score resolves falls into exactly one bucket,
so no piece is ever left without a position — the counterpart of the no-silent-drop
requirement above.

The rule protects one invariant: **position 1 is whatever the player strikes
continuously and position 2 is the snare, when the score contains one.** Those two
carry the overwhelming majority of a groove's notes, so they must sit adjacent —
inside a single eye fixation — and any further piece must be appended to their
right rather than inserted between them. A player moving from a sparse score to a
dense one therefore never has to relearn where to look. When a score has no snare
(tom exercises, cymbal studies exist), the remaining pieces close ranks leftward:
an empty bucket is skipped, never reserved.

#### Scenario: The core of the groove is leftmost and adjacent

- **WHEN** a percussion score containing a snare is loaded
- **THEN** the continuously-struck piece is in position 1 and the snare in position
  2, whatever else the score contains

#### Scenario: A snare-less score closes ranks

- **WHEN** a score uses only toms and a ride, with no snare and no hi-hat
- **THEN** the ride takes position 1 and the toms follow, highest to lowest, from
  position 2 — the empty snare bucket is skipped, not reserved

#### Scenario: An auxiliary piece sorts after the cymbals

- **WHEN** a score uses hi-hat, snare, a crash and a cowbell
- **THEN** the cowbell's lane comes last, after the crash — the core pair keeps
  positions 1 and 2

#### Scenario: The ride takes position 1 when there is no hi-hat

- **WHEN** a score keeps time on the ride and uses no hi-hat
- **THEN** the ride occupies position 1

#### Scenario: The ride joins the cymbals when a hi-hat is present

- **WHEN** a score uses both hi-hat and ride
- **THEN** the hi-hat is in position 1 and the ride is ordered with the cymbals,
  after the toms

#### Scenario: Added pieces do not disturb the core

- **WHEN** two scores are compared, one using three pieces and one using seven
- **THEN** positions 1 and 2 hold the same roles in both, and the extra pieces appear
  to their right

### Requirement: The kick is a full-width bar, not a lane

The kick SHALL be drawn as a bar spanning the full width of the cascade at its
onset, and SHALL NOT occupy a lane.

A lane encodes *where to aim*. The foot does not aim — it is always on the same
pedal — so giving the kick a lane spends horizontal width on information that does
not exist, and forces the eye to compare two horizontally distant positions to
answer the question that actually matters in drumming: does the foot land **with**
a hand stroke or **between** hand strokes? A full-width bar answers it by
intersection, without moving the gaze.

Whether an event counts as hand-struck or foot-struck follows the hands/feet
classification stated normatively in `hand-color-coding` — the two-voice
convention, with the single-voice General MIDI fallback — so the bar, the colours
and the hand filter all split the score the same way.

#### Scenario: A kick coinciding with a hand stroke reads as simultaneous

- **WHEN** a kick and a hand stroke share an onset
- **THEN** the bar passes through that note, and both are legible

#### Scenario: A kick between hand strokes reads as offset

- **WHEN** a kick falls between two hand strokes
- **THEN** the bar sits clear of every note

#### Scenario: The kick never consumes lane width

- **WHEN** the lanes are derived
- **THEN** the kick has no lane, and the available width is divided among the
  hand-struck pieces only

### Requirement: Foot bars are drawn beneath the note layer

The foot bars SHALL be painted **before** — therefore beneath — the hand notes, and
SHALL be thinner than a note and attenuated relative to one.

Painted above, a full-width bar covers the hand note precisely on a coincidence,
hiding the very information the bar exists to convey. Attenuation alone does not
fix it: a bar faint enough to reveal a note behind it becomes invisible across the
rest of the width, where it has nothing behind it. The three properties — order,
thickness, attenuation — SHALL hold together.

This defect is only observable on coincidences, so a score in which the foot and
the hands never align does not reveal it; the tests SHALL therefore exercise a
coinciding onset explicitly.

#### Scenario: The hand note stays visible on a coincidence

- **WHEN** a kick and a hand stroke share an onset
- **THEN** the note is drawn over the bar and remains fully legible

#### Scenario: The bar stays visible away from the notes

- **WHEN** a kick falls where no lane has a note
- **THEN** the bar is clearly legible across the full width

### Requirement: The drawn kit replaces the on-screen keyboard

For a percussion score the on-screen controller SHALL be a **drawing of the kit
itself** — one shape per lane, in the same order as the lanes, with the bass drum
front and centre rather than one piece among them — and the player SHALL strike
that drawing.

The drawing, its layout and its hit areas SHALL be **one object**. An input
surface computed anywhere but where the drawing happens is a hit area that drifts
from what the eye sees, and the drift is invisible until a player aims at a drum
and misses it.

A row of labelled rectangles under the falling notes said the same thing twice —
a row of names under a row of names — and it put the thing you strike somewhere
other than the thing you read. One drawing, struck directly, removes both.

Sharing the order is what makes the two surfaces one mapping: a player who learns
where a piece is among the falling notes finds it in the same place on the kit,
and never translates between two layouts.

While the row is sparse the pieces SHALL straddle a central gap the bass drum's
width, so no piece is drawn over it — which is also where a real kit puts it.

#### Scenario: The drawn pieces and the lanes share one order

- **WHEN** the lanes are derived for a score
- **THEN** the drawn kit presents the same pieces in the same left-to-right order

#### Scenario: The bass drum is not one of the row

- **WHEN** the kit is drawn
- **THEN** the bass drum sits centred and full-width under the pieces, and the
  pieces straddle it rather than overlapping it

#### Scenario: What is struck is what is drawn

- **WHEN** the player taps a drawn piece
- **THEN** the surface struck is the one whose shape contains the tap, resolved
  from the very geometry that painted it

### Requirement: Open and closed hi-hat are note variants

An open hi-hat SHALL be drawn as a visual variant of the note **within the hi-hat
lane**, distinct from a closed one, and SHALL NOT produce a bar or a lane of its
own.

Open versus closed is not a foot event in the score: it is a different General MIDI
number on the **hand** stroke. Treating it as a foot event would invent a note the
file does not contain.

#### Scenario: Open and closed are distinguishable

- **WHEN** a score alternates open and closed hi-hat strokes
- **THEN** both appear in the hi-hat lane, visually distinct from one another

#### Scenario: An open hi-hat creates no foot event

- **WHEN** an open hi-hat stroke is shown
- **THEN** no bar and no additional lane is drawn for it

### Requirement: The pedal "chick" takes an ordinary lane until its encoding lands

The cascade SHALL render a pedal hi-hat note (General MIDI 44 — the foot closing
the cymbals alone, no hand stroke) as an ordinary lane through the sort rule's
terminal bucket, like any other piece outside the named roles, until its
dedicated encoding is designed against a real score (`design.md` records the
candidate — a second, hollow full-width bar — and its fallback). This concedes
the "the foot does not aim" principle for one rare event, deliberately: visible
and aim-able beats invisible-but-scheduled, and the no-silent-drop requirement
holds for the chick as for everything else.

#### Scenario: A chick gets a lane, not a bar and not a drop

- **WHEN** a score contains a General MIDI 44 note
- **THEN** it is drawn in its own lane, ordered after the cymbals — no second
  full-width bar is drawn, and no note is dropped

### Requirement: The notation modes are not offered for a percussion score, for now

Until `add-drum-notation-render` lands, the render-mode toggle SHALL offer only
the cascade for a percussion score: the scrolling Staff and the engraved
Partition SHALL be omitted. The existing notation painters have no percussion
path — no percussion clef, no alternative noteheads, no two-voices-on-one-staff
layout — and offering a mode that renders undefined output is worse than
withholding it. The restriction SHALL apply to percussion scores only; a keyboard
score keeps all three modes.

#### Scenario: The mode toggle offers only the cascade

- **WHEN** a percussion score is loaded
- **THEN** the mode toggle presents the cascade, and neither Staff nor Partition
  can be selected

#### Scenario: Keyboard scores keep the three modes

- **WHEN** a keyboard score is loaded
- **THEN** Synthesia, Staff and Partition are all offered, exactly as before

### Requirement: Wait Mode is not offered for a percussion score, for now

Until `add-drum-scoring` lands, a percussion score SHALL be playable in the
timed modes only, and Wait Mode SHALL NOT be offered for it. This change ships
the kit display-only (see `keyboard-display`) and no percussion input path
exists yet, so a Wait Mode gate would block forever on input that cannot
arrive. The restriction SHALL apply to percussion scores only.

#### Scenario: Wait Mode absent for a percussion score

- **WHEN** a percussion score is loaded
- **THEN** Wait Mode is not offered, and playback runs in the timed modes only

#### Scenario: Keyboard scores keep Wait Mode

- **WHEN** a keyboard score is loaded
- **THEN** Wait Mode is offered exactly as before

### Requirement: Every drawn piece carries its name, on itself

Each drawn piece SHALL carry the localized name of its kit piece (fr/en) —
charley/hi-hat, caisse claire/snare, and so on — so a player who does not yet map
positions to pieces can read the kit like a legend. A generic piece from the sort
rule's terminal bucket SHALL carry its General MIDI name.

A name SHALL be hung on **its own piece**, not on a baseline shared by the row: a
kit draws cymbals high and drums low, and a row of names lined up under shapes of
different heights makes the reader match them by column. The name SHALL be
clipped to its piece's width rather than allowed to run into its neighbour's.

#### Scenario: Names sit on the pieces they name

- **WHEN** the kit is drawn for a score using a hi-hat and a snare
- **THEN** each piece carries its localized name directly under its own shape,
  and neither name overruns the other's column

### Requirement: A groove can be read as a stage as well as a cascade

A percussion score SHALL offer a second play surface, the **stage**: the same
notes seen in perspective, sliding down their rail toward a hit line with the
drawn kit under it. It SHALL be offered for percussion scores only, and SHALL be
the **first** mode presented for one — it is the reading a beginner is drawn to,
and the first segment is what a beginner tries.

The projection SHALL be a true `1/z`: an object twice as far SHALL be half the
size. This is not a styling choice — it is what spaces the arrivals correctly.
A power curve bunches them near the hit line, so a run that is evenly played
reads as rushed, which is the giveaway of a fake perspective.

The stage SHALL derive everything it shows from the same objects the cascade
does — the same lanes, the same kick-as-a-bar rule, the same drawn kit, the same
open/closed variant — so the two surfaces cannot teach two different instruments
or two different moments.

#### Scenario: The stage is offered first, and only for percussion

- **WHEN** a percussion score is loaded
- **THEN** the stage is the first mode in the row; a keyboard score is offered
  no stage at all

#### Scenario: Distance halves the size

- **WHEN** two notes are shown, one twice as far from the hit line as the other
- **THEN** the nearer is drawn twice the size of the farther

#### Scenario: A note lands on the drum it names

- **WHEN** a note reaches the hit line
- **THEN** its rail ends on the drawn piece that note belongs to, wherever the
  layout put that piece

### Requirement: The grid is the score's own metre

Both play surfaces SHALL draw their reference lines from the score's **measure
table and beat length**, never from round millisecond intervals. A grid on
arbitrary spacing lands on no real tempo, so a correctly-placed note reads as
displaced against it — the surface then teaches an error the score does not
contain.

A bar line and a beat line SHALL be distinguishable by **length first** — the bar
line spanning the surface, the beat a shorter stroke inside it — and only then by
brightness, which is the hierarchy an engraved staff already uses. Each bar line
SHALL carry its **written** measure number, so an unrolled repeat numbers its bars
the way the paper does.

The metre SHALL be drawn **after** the full-width kick bars: a downbeat almost
always carries a kick, and a reference line hidden under a note is not a
reference.

#### Scenario: The grid falls on the beats

- **WHEN** a score at any tempo is played on either surface
- **THEN** the reference lines fall on that score's bars and beats, and a
  correctly-placed note sits on the line it belongs to

#### Scenario: A downbeat carrying a kick still shows its bar line

- **WHEN** a measure begins on a kick
- **THEN** the bar line is visible across the kick's bar, and both remain legible

#### Scenario: Bars are numbered as written

- **WHEN** a repeated passage is played a second time
- **THEN** its bar lines carry the same numbers they carried the first time

### Requirement: A passed note leaves the surface

A note whose instant has gone by SHALL leave the surface rather than come to rest
on the hit line. A note parked where it landed reads as still owed, and on a
perspective surface it also stops moving while everything behind it keeps
coming — the one object on screen that contradicts the motion of all the others.

#### Scenario: A passed note leaves

- **WHEN** the playhead has crossed a note's instant
- **THEN** the note keeps travelling out of the surface and stops being drawn,
  rather than resting on the hit line

### Requirement: The inverted-kit setting mirrors the controller only

The player SHALL be able to enable an **inverted kit** layout, which reverses the
lane order and the drawn kit **together** — they are one object, so the reversal
cannot reach one without the other. The setting's contract stops there, as
a property of the setting itself: it SHALL NOT alter notation — drum notation is
a fixed convention, and a player whose kit is mirrored reads the same score as
anyone else, on whatever notation surfaces exist now or later — and it SHALL NOT
perform any input remapping: the inversion applies to the derived presentation
layout only, never to note-interpretation data, so a hi-hat reports the same
General MIDI number wherever it physically stands.

The setting SHALL be persisted, SHALL default to the standard layout, and SHALL NOT
be inferred — nothing in a score or a device reveals how a player's kit is set up.

Its label SHALL describe the **kit's layout**, not the player: a left-handed drummer
may well play a standard kit, and a label naming handedness would invite them to
enable a setting that does not apply to them.

#### Scenario: The falling notes and the kit mirror together

- **WHEN** the inverted-kit setting is enabled
- **THEN** the lane order and the drawn kit are both reversed, and they still
  match each other

#### Scenario: The setting never alters notation

- **WHEN** the setting is enabled
- **THEN** it has no effect on how the score is engraved — the setting reaches
  the falling notes and the drawn kit, and nothing else

#### Scenario: The setting performs no input remapping

- **WHEN** the setting is enabled
- **THEN** only the derived presentation layout is reversed; no
  note-interpretation data changes, and an incoming General MIDI number keeps
  its meaning

#### Scenario: The setting persists and is never guessed

- **WHEN** a player enables the setting and returns later
- **THEN** it is still enabled, and it was never enabled on their behalf

