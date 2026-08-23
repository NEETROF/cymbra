## Context

`crates/musicxml-core` is the single parser behind four consumers: the app's FFI
engine (`apps/music/rust`), the backend `music` module, the score crawler, and —
through `musicxml-wasm` — the back-office notation preview. It models a note as
`NoteEvent { pitch: Option<Pitch>, is_rest: bool, … }`, where `Pitch` is
`{ step: char, octave: i32, alter: i32 }`.

Percussion notation does not fit that model. A drum note is a `<note>` carrying
`<unpitched><display-step/><display-octave/></unpitched>` and an `<instrument id>`;
the sound it denotes lives in the `<part-list>`, in
`<score-part>/<score-instrument>` paired with `<midi-instrument><midi-unpitched>`.
None of that is parsed today, so `NoteEvent.pitch` is `None` for every drum note
and `playback.rs::schedule()` (with its Dart mirror `notationToTimedNotes`) skips
them all. `Clef.sign` is a `char`, which cannot hold `percussion`.

The product decision framing the work: a score is piano **or** drums, never both.
Instrument is therefore a single scalar for the whole score — no multi-part
modelling, no multi-timbral audio, no per-note channel routing.

**Why this change stops at the parser.** `validate()` is the shared admission gate
for the app upload preview, the backend upload RPC and the crawler ingest. Opening
it makes drum scores enterable and readable by everyone, which the product requires
to be restricted to staff and `midi-drums` beta members — enforced by the backend,
not merely hidden by the app. That enforcement needs a queryable instrument column,
which needs a migration, which needs consumers updated. All of that is
`add-drums-access`. Landing it here would make a single change span the parser, a
schema migration, the wire protocol, server-side authorisation and two front ends.

## Goals / Non-Goals

**Goals:**

- Represent percussion notation faithfully in the shared parser: unpitched note
  events, the part-list instrument table, the percussion clef.
- Derive the score's instrument classification from the parse alone.
- Emit unpitched notes in both playback schedules (crate and Dart mirror), keeping
  them behaviourally identical.
- Leave the system's observable behaviour **unchanged**, so this change is safe to
  land on its own.

**Non-Goals:**

- Opening the validation gate. Explicitly deferred to `add-drums-access`, together
  with the access controls that must accompany it.
- The `is_piano` → `instrument` column migration, the wire protocol, the Score Hub
  and back-office filters, the rating-deck sourcing predicate, the feature flag and
  its enforcement — all `add-drums-access`.
- Percussion audio, notation rendering, the kit view, MIDI pad input,
  instrument-aware scoring — one change each, after `add-drums-access`.

## Decisions

### Unpitched is a separate channel on `NoteEvent`, not a repurposed `Pitch`

`NoteEvent` gains a channel holding the written position (`display-step`,
`display-octave`) and the resolved General MIDI number, alongside the existing
`pitch: Option<Pitch>`. A note is then exactly one of pitched, unpitched, or rest.

*Alternative rejected:* storing the display position inside `Pitch`. It types
cleanly and touches less code, but it is a lie the compiler cannot catch — every
existing consumer of `Pitch` (`midi_of_pitch`, the ambitus scan, the staff
painter's `diatonic` derivation) would silently compute a sounding pitch from what
is really a staff placement. A drum note on display-step E, octave 5 would report
MIDI 76 and enter the ambitus as a high E. Separating the channels makes each of
those call sites a compile error the implementer must answer deliberately.

*Alternative rejected:* an enum `NoteBody { Pitched(Pitch), Unpitched(…), Rest }`.
Cleaner in isolation, but it rewrites every existing `note.pitch` / `note.is_rest`
access across four crates and the FFI surface, for a change whose value is
elsewhere. Noted in Open Questions.

### `Clef.sign` becomes a typed sign covering the MusicXML vocabulary

MusicXML defines seven clef signs: `G`, `F`, `C`, `percussion`, `TAB`, `jianpu` and
`none`. A `char` can hold the first three and nothing else. The field becomes an
enum with the variants the renderers act on — `G`, `F`, `C`, `Percussion` — and any
other or unrecognised sign leaves that staff at its default clef rather than
failing the parse. `TAB` and `jianpu` are guitar-tablature and Chinese numbered
notation: out of scope for a keyboard-and-drums product, and degrading them is
strictly better than today, where they are silently truncated to a character.

The bridged Dart `TimedNote.clefSign` stays a `String` derived from the enum, so
the staff and partition painters that compare against `'G'` / `'F'` are untouched.
The breaking surface is the crate API and the bridged `Clef`, not the painters.

*Alternative rejected:* `String`. It avoids the enum churn but pushes string
comparison into every painter and lets an arbitrary sign reach the render path.

### The instrument table is resolved at parse time, not by consumers

The parser reads the part-list declarations and resolves each unpitched note's
`<instrument id>` to its General MIDI number while building the document.

*Rationale:* there are four consumers and the resolution rule (fall back to
unknown, never infer from the written position) is easy to get subtly wrong. One
resolution site means one place to test it. The raw table stays on the document for
a renderer that needs the instrument's name.

### An unresolvable percussion note is omitted from the schedule, never fabricated

If a note's General MIDI number cannot be resolved, the schedule drops it rather
than substituting a default kit piece.

*Rationale:* this repo's established habit for a missing signal — the tempo facet
is left null rather than defaulted to the playback BPM, the ambitus is left unknown
for a rest-only score. A fabricated snare hit is worse than a missing one: it is
indistinguishable from real notation and would be scored against the player.

### Instrument classification is derived here, but `is_piano` is not touched

`ScoreSummary` gains an instrument classification. `is_piano` keeps its current
definition (`staves >= 2`) and its column.

*Rationale:* `is_piano` is a **proxy**, not a detection — `<staves>` is the number
of staves in the part, so `staves >= 2` means "written on a grand staff, therefore
probably a keyboard". It is wrong in both directions: a single-staff piano piece
reads as not-piano, and an organ (three staves) or any two-staff arrangement reads
as piano. Replacing it with the real classification is the right move and is
exactly what `add-drums-access` does — but it is a schema migration with a backfill
whose consumers span the proto, two front ends and the rating-deck predicate.
Deriving the honest value here, and switching the readers there, keeps each change
reviewable. Nothing regresses in between because no percussion score can enter the
corpus until the gate opens.

## Risks / Trade-offs

**A breaking crate API change ripples to four consumers** → `Clef.sign` and the new
`NoteEvent` channel break `musicxml-wasm`, `audio-wasm`, `score-crawler`,
`backend/music` and the app FFI at compile time. That is the intent: every break is
a call site that must decide what percussion means for it. Mitigation is ordering —
land the model change and fix all consumers in one commit, and run
`flutter_rust_bridge_codegen generate` before the app's analyze step.

**The Dart mirror silently drifts** → `notationToTimedNotes` duplicates
`playback.rs::schedule()` by design, and nothing enforces their equality. A drift
means the back-office preview times a drum score differently from the app, which
surfaces as an unreproducible "the preview is wrong" report. Mitigation: the spec
makes parity a requirement, and both sides get the same fixture with the same
expected output so a divergence fails a test rather than a bug report.

**A stale back-office wasm reads as a code bug** → any `musicxml-core` change
requires `yarn gen:wasm`; skipping it makes the notation preview render against the
old parser. This has already cost debugging time on this repo once. Mitigation:
an explicit task, and diagnosing any preview oddity by comparing the wasm
artifact's mtime against the crate's commits before reading code.

**Dead code between the two changes** → the percussion parsing path is unreachable
through any product surface until `add-drums-access` lands, so a defect in it would
not be caught by using the app. Accepted, and mitigated by the work being pure and
directly unit-tested; it is also why the two changes should land close together.

**Coverage on a change with no user-visible behaviour** → the 80% gate applies. The
work is pure and host-testable, so this is a matter of writing the fixtures.

## Migration Plan

None. No schema change, no wire-protocol change, no backfill, no flag. The change
is additive to the crate's model and neutral everywhere else; `is_piano` keeps its
column, type and meaning.

Rollback is a revert, with nothing persisted to unwind: the gate never opened, so
no percussion score can have entered the corpus.

## Open Questions

- **Should `NoteEvent` eventually become a `NoteBody` enum?** Deferred (see
  Decisions). Revisit once percussion rendering has exercised the model.
- **How is a percussion score's difficulty assessed?** The current facets — ambitus,
  smallest note value, chord presence — are keyboard-shaped, and ambitus is
  meaningless here. Needed by `add-drums-access` (which surfaces the level) and by
  `add-drum-scoring`; it does not block this change.
- **Which General MIDI numbers collapse into one kit piece?** Several numbers denote
  the same physical drum (the acoustic and electric snares, for instance). That
  mapping belongs to the kit view; the parser deliberately reports the raw number.
