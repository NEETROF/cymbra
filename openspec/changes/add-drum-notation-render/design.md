## Context

Three sibling changes left percussion notation in a deliberate interim: the
parser reads it (`add-unpitched-notation`), the gate admits it and the console
refuses to draw it (`add-drums-access`), and the app plays it in the cascade
while withholding the Staff and Partition modes (`add-drum-kit-view`). This
change is the named lift for those interims.

Four facts shape the design, all verified against the code and the corpus.

**The geometry already works; the drawing does not.** The crate's layout
(`min_width`, system wrapping) is note-density arithmetic over time positions
and handles unpitched notes already — `add-drums-access` stated it: "a
percussion score's geometry is available (it parses and lays out)". What is
keyboard-shaped is the painters: `painter.ts` places heads with `yForPitch`
over `note.pitch` (an unpitched note has none — `paintNote` returns after the
`pitch == null` check, drawing nothing), and both clef helpers
(`smufl.ts:clefGlyph`, `smufl.dart:SmuflGlyphs.clef`) **default an unknown sign
to treble**, so the percussion sign would silently draw a G clef today.

**The written positions follow the treble mapping.** In every fixture and in
MuseScore/Finale exports, an unpitched note's `display-step`/`display-octave`
places on the five-line staff exactly where a treble staff would put the same
pitched spelling: snare C5 = third space, kick F4 = bottom space, closed hi-hat
G5 = above the top line, crash A5 = first ledger. The parser already defaults
an empty `<unpitched/>` to B4 — the middle line (`lib.rs:536`).

**The app already carries most of the plumbing.** `notationToTimedNotes`
already fills `TimedNote.diatonic` from the display step/octave for an
unpitched note and `clefSign` with `'percussion'`
(`notation_playback.dart:284-310`); `TimedNote.voice` exists since
`add-drum-kit-view` (task 1.6). What is missing app-side: the percussion clef
and x-head glyphs, the no-armature rule, per-voice rests (`TimedRest` has no
`voice` field yet), and the mode toggle re-offer
(`player_screen.dart:940`).

**The fixtures carry no `<notehead>` elements.** The head shape (x for
cymbals) must therefore be derived from what the file does carry — the resolved
General MIDI number — and that derivation is exactly the piece of knowledge
that would otherwise be hardcoded twice, once per painter.

## Goals / Non-Goals

**Goals:**

- Engrave a percussion score correctly in **both** painters — percussion clef,
  written-position placement, x-heads for cymbals, two voices on one staff —
  against the unchanged shared geometry.
- Put the one shared table (which GM numbers are cymbals) in a single place
  with a single authority, so the two independent painters cannot drift.
- Lift the named interims: the console's unpreviewable state, the app's
  withheld notation modes, and the hand-colour deferral.

**Non-Goals:**

- Sounding (`add-drum-audio-channel`), listening (`add-drum-input-mapping`),
  judging (`add-drum-scoring`). The console's Play guard, the app's Wait-Mode
  omission for percussion and the server audio-teaser render skip all stay —
  each is named to its owning change.
- The cascade and the pad strip — done and untouched
  (`add-drum-kit-view`). In particular the kick-bar and chick encodings are
  cascade concepts and do **not** leak into notation.
- Ornament-level drum notation: ghost notes (parenthesised heads), drags and
  flams beyond the existing grace-note path, buzz rolls. The parser does not
  model them; drawing them is future fidelity work, not a correctness gap.
- Playback/cursor behaviour: the playhead, dimming, auto-scroll and highlight
  rules of `score-notation` are placement-independent and apply to a
  percussion score without modification.

## Decisions

### The head classification lives in the shared crate, beside the resolved GM number

The crate derives an engraved head class per unpitched note (drum / cymbal /
cymbal-open) from one table keyed on the 0-based GM number, at the same moment
it resolves the number itself. The field rides the existing model: serde ships
it to the console in the wasm JSON, frb ships it to the app — both painters
read it verbatim and neither owns GM knowledge.

*Rationale:* the brief question is not "can a painter map 42→x" (trivial) but
"what stops the two maps drifting". A value computed once in the crate cannot
drift, costs one enum per note, and follows the model's own precedent —
`Unpitched.gm_number` and `NotationMeasure.min_width` are already parse-time
derivations carried on the model.

*Alternative rejected:* a table per painter (Dart + TS) with a drift-pinning
test. Three copies of kit knowledge (the app already has `drum_kit.dart`), and
the pinning test can only compare the two it can reach — the console table
would be pinned to nothing at runtime.

*Alternative rejected:* exposing a `head_class(gm)` **function** through the
wasm exports and the frb bridge. Same single authority, but each painter must
thread an extra call and cache per note; carrying the resolved value beside
`gm_number` gives the same guarantee with zero calls.

*Alternative rejected:* parsing `<notehead>` and drawing what the file says.
The corpus fixtures carry none, so it decides nothing for the shipped scores;
a crawled file with a bare drum part would still need the GM-derived fallback,
which then has to exist anyway. Revisit if real files with meaningful notehead
overrides (ride-bell diamonds, ghost parentheses) enter the corpus — recorded
as an open question, not smuggled in here.

The app keeps `drum_kit.dart` untouched: lanes answer *where to aim*
(gameplay), heads answer *what is engraved* (notation) — different questions,
different tables, each single-sourced. A drift-pinning test asserts the two
agree on the cymbal set (every `hiHat`/`ride`/`crash`-role GM number
classifies as a cymbal head and vice versa), so the one overlap they do have
is pinned.

### Placement is the written position, mapped as a treble staff

`display-step`/`display-octave` map to lines and spaces exactly as a treble
(G/2) staff maps pitched spellings, regardless of the percussion clef's `line`
attribute. Nothing is ever placed from the GM number.

*Rationale:* this is the MusicXML convention every conformant exporter
follows, verified against all four fixtures; it also means both painters reuse
their existing diatonic machinery (`yForPitch` / `_yForPitch` /
`TimedNote.diatonic`) with a fixed reference instead of growing a second
placement path.

*Alternative rejected:* placing by GM number (a mapping table from sound to
staff position). It inverts the authority — the file states where the note is
written; fabricating the position from the sound would contradict any score
whose kit layout differs from the table's, and fails entirely for a note whose
GM number is unresolved.

*Consequence, stated as a requirement because playback does the opposite:* a
note whose GM number is **unresolved is still engraved** at its written
position. The schedule omits such a note (a fabricated sound is wrong); the
engraving keeps it (the written position is authoritative and present). A
moderator must see the score as written — silently dropping heads from the
preview would misrepresent the file being judged.

### No armature on a percussion staff — and the clef fallback is fixed, not relied on

A percussion staff never draws a key signature, even when the file declares
`fifths ≠ 0`, and never draws the cancelling naturals of a key change. The
time signature draws normally.

*Rationale:* an unpitched part has no tonality; sharps on a drum staff are
wrong music. The fixtures declare `fifths 0` so the bug would be invisible in
tests built only on them — a crawled file is exactly where it would surface.

*Alternative rejected:* drawing whatever the file declares, on the grounds of
parse-faithfulness. Faithfulness applies to notes; the armature is a
projection of tonality onto a staff, and a percussion staff has none to
project.

The related trap is the clef: both `clefGlyph` helpers default unknown signs
to treble, so a percussion clef silently draws as a G clef **without any code
being wrong-looking**. The percussion sign gets an explicit mapping
(`unpitchedPercussionClef1`, SMuFL U+E069), and the tests assert the glyph —
not merely "a clef" — is drawn.

### Two voices: the file's stems win, the voice rule fills the gaps

Stem direction comes from the explicit `<stem>` when present (all four
fixtures carry it); when absent, voice 1 stems up and voice 2 down. Beam
groups already key on `staff_voice` in both painters and never merge across
voices. Rests displace by voice in a two-voice measure — voice 1 above the
middle line, voice 2 below — and stay at the midline in a single-voice
measure. On a shared onset both voices engrave at the shared column with their
opposite stems; if two heads would land on the same position, one is offset so
neither is hidden.

*Rationale:* parse-faithfulness first (the exporter engraved the part it
meant), convention as fallback (the two-voice drum convention is universal:
hands up, feet down — the same convention `hand-color-coding` keys the colour
split on). Rest displacement exists because a midline rest in a two-voice
measure sits exactly where the other voice's material runs; both painters
currently draw every rest at the midline, which is correct for one voice and a
collision for two.

*Alternative rejected:* deriving stems from the voice always, overriding the
file. It would "correct" scores that deliberately flip a stem (a hand pattern
split across voices for readability) and diverge from the engraved Partition
the moderator compares against.

### The cascade remains the default; notation is opt-in

Re-offering Staff and Partition does not change the default presentation of a
percussion score: the player opens in the cascade, and the toggle offers the
same mode set a keyboard score gets on the same device (phones already remap
Partition to Staff for keyboard scores — percussion inherits that rule rather
than growing its own).

*Rationale:* the cascade is the designed reading surface for the game
(`add-drum-kit-view` settled it against a drummer); notation serves the
drummer who reads, the moderator, and the lesson content. Parity-with-keyboard
is the requirement shape because an absolute list ("all three modes") would
contradict the existing phone behaviour.

*Alternative rejected:* defaulting to Staff for percussion to "showcase" the
new rendering. It would change the loaded-score behaviour testers already
have, for no reader benefit.

### The obligations split stays clean: this change draws, nothing else

The console's **preview** interim is lifted here; the console's **Play** guard
and the server's audio-teaser render skip are `add-drum-audio-channel`'s, and
the Wait-Mode omission is `add-drum-scoring`'s. The brief's "server-side
notation-preview render job" turned out, on inspection, not to exist: the only
server-side preview render is the **audio** teaser
(`score_preview_module.rs:103-110`, skip annotated "`add-drum-audio-channel`
lifts it"; `music-drums-visibility` says the same). Notation is rendered
client-side from the score bytes on demand — console and app alike — so there
is **no stored notation artifact to backfill** for already-accepted percussion
scores: the moment the painters land, every percussion score previews, with no
catch-up pass. (Contrast `add-repeat-unrolling`, which had to regenerate
stored **audio** previews because the timeline they had baked changed.)

*Rationale:* lifting the audio skip here would bake piano-font WAV clips of
drum parts — precisely the "confident wrong preview" the skip prevents — and
would couple this change to the synthesizer work it deliberately excludes.

## Risks / Trade-offs

**Two independent painters, one convention** → the rules land twice and can
diverge in the details (a head one painter x-es and the other doesn't, a rest
one displaces and the other centres). Mitigation: the only *table* is in the
crate; the *rules* are pinned by running the same four fixtures through both
sides — vitest asserts the console SVG (glyphs and y-positions), Flutter tests
assert the painters — plus a side-by-side manual pass on the same fixture.

**Delta stacking on unarchived siblings** → three of the deltas below modify
requirement text that only exists in `add-drums-access` /
`add-drum-kit-view` / `add-unpitched-notation` deltas, not yet in
`openspec/specs/`. The stacking is stated per-file (each delta names whose
version it builds on), and this change must be archived **after** those three,
or the RENAMEs/MODIFIEDs will not find their targets. The constraint also runs
**forward**: `add-drum-audio-channel` modifies the same `moderation-console`
requirement this change modifies, and its delta is rebased on this change's
version (its RENAMED FROM points at the title this change leaves in place) —
so this change must archive **before** `add-drum-audio-channel`, per the
declared pipeline order; archiving in the reverse order would leave the audio
delta's RENAME without its target and clobber this change's preview lift.

**Bravura glyph coverage** → the design assumes `unpitchedPercussionClef1`
(U+E069) and the x-head series (`noteheadXWhole`/`Half`/`Black`,
U+E0A7–U+E0A9) exist in both bundled fonts (the app asset and the console's
same-origin subset). Verify **first** — a missing glyph renders as tofu or
nothing, and the fix (re-subsetting) is cheap early and disruptive late.

**A stale console wasm masks the whole feature** → the crate model gains a
field, and the console renders through a locally-generated wasm; testing
against a stale build shows the old behaviour with no error (the known
`yarn gen:wasm` gotcha). Named in the tasks, twice: once to rebuild, once to
check the mtime when anything looks unchanged.

**The open-hi-hat mark at small sizes** → the "o" above the head must survive
the in-card preview's `noteScale` and the console's responsive downscale.
Judged in the manual pass; the fallback (drop the mark below a scale
threshold, keep the x head) loses information the cascade still shows, so it
is a last resort.

**Head-coincidence offsetting is rare and hand-checked** → two voices striking
the same written position at the same instant is unusual (it requires the
same line/space in both voices); the offset rule is tested with a synthetic
fixture rather than corpus material, and the corpus pass may never exercise
it. Accepted: the failure mode without the rule (one head hidden) is worse
than the cost of one synthetic test.

## Migration Plan

None in data or wire terms: no schema, no proto, no stored artifacts. The
crate model gains one derived, serialized field — additive for every consumer;
the app bridge is regenerated (`flutter_rust_bridge_codegen generate`) and the
console wasm rebuilt (`yarn gen:wasm`) inside the same change. A keyboard
score renders byte-identically in both painters; the new paths are reachable
only for percussion scores, themselves reachable only by the drum audience
(`music-drums-visibility` — this change inherits that gate and re-derives
nothing).

Rollback is a revert. Archive order: after `add-unpitched-notation`,
`add-drums-access` and `add-drum-kit-view`, and before
`add-drum-audio-channel` (delta stacking above).

## Open Questions

- **`<notehead>` overrides.** Real repertoire uses diamond heads for ride
  bells and parenthesised ghost notes. The parser drops the element today and
  this change derives heads from the GM number instead. If crawled percussion
  files arrive with meaningful overrides, teach the parser the element and let
  it win over the derived class — the derived class then becomes the fallback,
  which is why it lives in the crate and not in the painters.
- **Should the console colour hands/feet?** The console painter is
  deliberately monochrome (ink on the theme background) for keyboard scores
  too; the blue/amber convention is a player aid. A moderator judging a
  two-voice drum part might still benefit. Left out — the console's contract
  is content/layout fidelity, not player chrome — but cheap to add later since
  the voice is in the geometry.
- **Cymbal whole/half x-forms.** The spec requires the x form to follow the
  duration class (open x for half/whole, per SMuFL). Tied cymbal chains are
  the one place whole-note cymbals appear in the fixtures; if the open-x
  glyphs read poorly at the app's staff scale, a single-form x with the usual
  open/filled distinction dropped is the recorded fallback — a legibility
  judgement for the manual pass.
