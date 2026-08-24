## Why

A percussion score can now be parsed (`add-unpitched-notation`), admitted and
gated (`add-drums-access`), and played in the cascade (`add-drum-kit-view`) —
but it still cannot be **read as notation** anywhere. Every notation surface is
keyboard-shaped: the app's Staff and Partition modes are withheld for a
percussion score ("not offered, for now" — `music-drum-kit-view`), and the
moderation console shows an explicit "not previewable yet" state instead of a
drawing (`web-notation-render`). Both interims exist for the same reason: the
painters would confidently engrave a drum part with pitched treble/bass
assumptions — wrong clef, wrong noteheads, a nonsense armature — which is worse
than withholding.

This change delivers percussion drawing and lifts those interims. It is the
first of the four remaining drum changes (`add-drum-notation-render` →
`add-drum-audio-channel` → `add-drum-input-mapping` → `add-drum-scoring`) and
the one the moderation pipeline is waiting on: until a moderator can see a drum
proposal engraved, percussion proposals can only be left pending
(`add-drums-access` accepted that queue-risk on the promise that this change
lands before testers are enrolled in numbers).

The work lands **twice against one shared geometry**, because the product has
two independent notation painters: the app's Dart painters
(`apps/music/lib/painters/staff_painter.dart` and `partition_painter.dart` —
the Partition screen instantiates the latter twice, once to measure and once to
paint, and the scrolling Staff mode is a third surface reused by the in-card
rating preview and the upload preview) and the console's TypeScript painter
(`apps/back-office/src/lib/notation/painter.ts` over the musicxml wasm). The
layout geometry (measure `min_width`, system wrapping) already handles
percussion; what is missing is the drawing itself, and a single authority for
the one piece of knowledge both painters need: which General MIDI numbers are
cymbals.

## What Changes

**Shared crate (`crates/musicxml-core`) — one classification, two consumers**

- The crate derives, beside the resolved General MIDI number it already carries
  on each unpitched note, an **engraved head class** (drum → ordinary oval
  heads; cymbal → x-form heads; the open hi-hat's open mark) from one table.
  Both painters consume the value verbatim — through the serde wasm JSON and
  the frb bridge, which both carry the field automatically — so neither painter
  hardcodes GM ranges and the two cannot drift. (The app's
  `lib/state/drum_kit.dart` keeps its **gameplay** table — lanes answer "where
  do I aim", heads answer "what is engraved" — with a drift-pinning test
  asserting the two agree on the cymbal set.)

**Percussion engraving rules — stated once, applied by both painters**

- **Clef and staff**: the percussion clef glyph on a single five-line staff; no
  key signature is ever drawn on a percussion staff (an unpitched part has no
  tonality); the time signature draws as on any staff. Both painters currently
  default an unknown clef sign to treble, so without this change the percussion
  sign would silently draw a G clef.
- **Vertical placement**: from the written position (`display-step` /
  `display-octave`, already parsed; an empty `<unpitched/>` already defaults to
  the middle-line B4 at parse), mapped to staff positions exactly as a treble
  staff maps pitched ones — the MusicXML convention the corpus follows (snare
  C5 on the third space, kick F4 in the bottom space, hi-hat G5 above the top
  line). Never from the General MIDI number, and a note whose GM number is
  unresolved is **still engraved** at its written position — omission-when-
  unresolved is a playback rule, not an engraving rule.
- **Two voices on one staff**: an explicit `<stem>` wins; absent one, voice 1
  stems up and voice 2 stems down; beams never merge across voices; rests are
  vertically displaced by voice in a two-voice measure; a shared onset engraves
  both voices legibly.
- **What notation does NOT adopt**: the cascade's encodings. The kick (GM
  35/36) and the pedal hi-hat "chick" (GM 44) are ordinary written notes on the
  staff — no full-width bar, no lane concept. The cascade's bar-encoding
  question stays with `add-drum-kit-view`.

**App (`apps/music`)**

- Staff and Partition are **offered again** for a percussion score — the
  `music-drum-kit-view` interim ("not offered, for now") is lifted; the cascade
  remains the default presentation.
- The hands/feet colour convention (blue/amber keyed to voice, stated
  normatively in `hand-color-coding`) extends from the cascade to the engraved
  notation, and the hand filter for percussion hides **events** by that
  convention — the single percussion staff is never collapsed the way an
  unselected keyboard staff is.
- The Staff-mode surfaces that reuse `StaffPainter` (in-card rating preview,
  upload preview) inherit the percussion path for free.

**Back-office (`apps/back-office`)**

- The console painter draws percussion instead of returning the
  `percussion_unsupported` state; the "not previewable yet" carve-out in
  `web-notation-render` disappears and the browser-render requirement applies
  to percussion scores like any other. The genuinely-broken states
  (undecodable, unparseable, oversized) are untouched.
- The console's **Play guard is not touched**: auditioning a percussion score
  still refuses until `add-drum-audio-channel` delivers the percussion channel.
  This change draws; it does not sound.

**Delegated obligations, accounted for**

1. `web-notation-render` carve-out (`add-drums-access`): **lifted** — the
   MODIFIED requirement below builds on that change's delta text.
2. `moderation-console` preview interim (`add-drums-access`): **lifted** for
   the preview half only; the Play half stays, and the delta says so.
3. `music-drum-kit-view` "notation modes not offered, for now": **lifted** via
   RENAMED — the title stops being true.
4. `hand-color-coding` deferral ("what Staff and Partition do for a percussion
   score"): **defined** — the voice-keyed hands/feet colours apply to the
   engraved notation.
5. The server-side preview **render job** skip (`add-drums-access` task 4.11):
   **explicitly re-deferred, not lifted.** That job renders the **audio
   teaser** (a WAV clip through the piano font —
   `backend/music/src/score_preview_module.rs`), not notation; nothing
   notation-shaped is baked server-side (the console renders client-side from
   the bytes, on demand), so there is no notation catch-up to run for
   already-accepted percussion scores. The skip is the server-side twin of the
   console's Play guard, and `music-drums-visibility` already names
   `add-drum-audio-channel` as the change that lifts it — lifting it here would
   bake piano-font clips of drum parts, exactly what the skip exists to
   prevent.

**Deliberately out of scope**

- Percussion **audio** (`add-drum-audio-channel`), pad **input**
  (`add-drum-input-mapping`), **scoring** and Wait Mode (`add-drum-scoring`).
  Wait Mode stays not-offered for a percussion score; that interim belongs to
  `add-drum-scoring` and is not touched here.
- The cascade and the pad strip — done, `add-drum-kit-view`.
- Parsing `<notehead>` overrides. The corpus fixtures carry none; the head
  class derives from the resolved sound identity instead (see `design.md` for
  the rejected alternative and the revisit condition).

## Capabilities

### New Capabilities

- `music-percussion-engraving`: how a percussion score is **drawn**, stated
  once for every notation surface — the percussion clef and the no-armature
  rule, vertical placement from the written position, the shared head
  classification (drums vs cymbals, the open hi-hat mark) and where its
  authority lives, and the two-voice layout rules (stems, rests, shared
  onsets). The app's painters and the console's painter both implement this
  capability; it is `music-` because it defines how the product's scores are
  notated — the console mirrors the app, as `web-notation-render` already
  requires.

### Modified Capabilities

- `web-notation-render`: the percussion carve-out is lifted (MODIFIED, building
  on `add-drums-access`'s delta) and the interim "declared unpreviewable"
  requirement is RENAMED into the percussion-drawing obligation — the console
  draws a percussion score like any other.
- `moderation-console`: the badge-and-refuse requirement is MODIFIED (building
  on `add-drums-access`'s delta) — the preview interim is gone, the Play guard
  remains until `add-drum-audio-channel`, the badge stays.
- `score-notation`: `Partition Rendering State` is MODIFIED (from the main
  spec) — the keyboard staff-collapse rule is scoped to keyboard scores and the
  percussion path is stated; `Derived Playback Timing` is MODIFIED (building on
  `add-unpitched-notation`'s delta) — a derived unpitched note also carries its
  written staff placement so the scrolling Staff mode places it as written,
  never by its General MIDI number.
- `hand-color-coding`: `Hand Colours In The Render Modes` is MODIFIED (building
  on `add-drum-kit-view`'s delta) — the hands/feet colours extend from "the
  cascade only" to the Staff and Partition modes.
- `music-drum-kit-view`: the interim "notation modes are not offered" is
  RENAMED and inverted — the modes are offered, the cascade stays the default.

## Impact

**Products**

| Product | Consumes | New |
|---|---|---|
| **Music** (`apps/music`) | the parsed unpitched channel, the voice plumbing and colour convention from `add-drum-kit-view` | percussion drawing in Staff and Partition, the mode re-offer, hands/feet colours on engraved notation |
| **Back-office** (`apps/back-office`) | the same crate geometry through the wasm | percussion drawing in the console painter; the unpreviewable state retired |
| **Platform / ID / Live / Site** | — | untouched |
| **Backend** | — | untouched — the audio-teaser render skip is deliberately left for `add-drum-audio-channel` |

**Code**

- `crates/musicxml-core/src/model.rs` + `lib.rs`: the derived head class on
  `Unpitched`, its table and tests. A crate **public API** change ⇒
  `flutter_rust_bridge_codegen generate` for the app, and the console wasm must
  be rebuilt (`yarn gen:wasm`) — a stale wasm silently renders against the old
  parser and reads as a code bug.
- `apps/music/lib/painters/smufl.dart` (percussion clef + x-head glyphs; both
  clef helpers currently default unknown signs to treble),
  `staff_painter.dart`, `partition_painter.dart`.
- `apps/music/lib/state/notation_playback.dart` (voice on `TimedRest`, the
  written-placement guarantee) and `player_data.dart`.
- `apps/music/lib/screens/player_screen.dart:940` (the mode-toggle restriction
  to lift; `:598` routes percussion to the cascade and stays the default).
- `apps/back-office/src/lib/notation/`: `painter.ts` (the
  `percussion_unsupported` path retired, the percussion paint path added),
  `smufl.ts`, `geometry.ts` (the new field's type); `components/ScorePreview.vue`,
  `views/ScoreDetailView.vue`, `views/ReviewView.vue` (the state's consumers).

**Depends on** `add-unpitched-notation` (the unpitched channel, written
positions, GM numbers, the percussion clef sign), `add-drums-access` (the
audience gate; the console interims this change lifts) and `add-drum-kit-view`
(the mode-toggle interim, the voice plumbing on `TimedNote`, the hands/feet
convention). No code here depends on the audio, input or scoring changes — but
this change is **not** spec-independent of `add-drum-audio-channel`: both
modify the same `moderation-console` requirement, and the audio change's delta
is written on top of this change's version. It must be **archived after** the
three siblings above, since its deltas build on theirs, and **before**
`add-drum-audio-channel`, matching the declared pipeline order
(`add-drum-notation-render` → `add-drum-audio-channel`).

**Reference fixtures.** The four in-repo drum scores
(`apps/music/assets/scores/beginner/premiers_pas_batterie.musicxml`,
`beginner/rock_basique.musicxml`, `intermediate/groove_ouvert.musicxml`,
`advanced/autour_des_futs.musicxml`) are the ground truth the painters are
built and tested against: percussion clef on line 2, snare C5 / kick F4 /
hi-hat G5 / crash A5 written positions, two voices with explicit stems (voice 1
up, voice 2 down), rests carried per voice, and **no** `<notehead>` elements —
which is why the head class derives from the sound identity.
