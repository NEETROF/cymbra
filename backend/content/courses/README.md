# First-party course corpus (add-notation-courses)

One JSON file per course. This directory is the **source of truth** for the
solfège curriculum; `backend/scripts/gen_seed_courses.py` compiles it into the
idempotent `backend/scripts/seed_courses.sql`, and the Flutter corpus test
(`apps/music/test/courses/content_corpus_test.dart`) validates every file with
the **real client parser** — a course that fails there would silently lose
blocks in production, so the test gate is the contract.

```
edit *.json  →  cd apps/music && flutter test test/courses/content_corpus_test.dart
             →  python3 backend/scripts/gen_seed_courses.py
             →  psql "$CYMBRA_MUSIC_DATABASE_URL" -f backend/scripts/seed_courses.sql
```

## File envelope

```jsonc
{
  "id": "sol-u1-04-do-re-mi",        // = filename; sol-u<unit>-<nn>-<slug>
  "status": "published",
  "instrument": "piano",
  "track": "solfege",
  "level": "beginner",                // u1–u3 beginner, u4–u6 intermediate, u7 advanced
  "unit": "u1",
  "unitTitle": {"en": "…", "fr": "…", "es": "…", "it": "…"},
  "sortOrder": 104,                   // unit*100 + lesson number
  "schemaVersion": 2,
  "title": {"en": "…", "fr": "…", "es": "…", "it": "…"},
  "content": { /* the manifest, below */ }
}
```

`content` = `{ "schemaVersion": 2, "id": <same>, "instrument", "track", "level",
"title" (same), "summary" {i18n}, "blocks": [...] }`.

## Blocks

Pitches are compact scientific spellings (`"C4"` = middle C/do central, `"F#4"`,
`"Bb3"`); rhythm figures are `whole|half|quarter|eighth|16th` (+`dots`,
`rest:true`). Every i18n map carries **all four** locales `en fr es it`.

Passive: `text {text}` · `diagram {id}` (closed set, see `kCourseDiagramIds`) ·
`question {prompt, options[], answerIndex, feedback}` ·
`staff {clef: treble|bass, keyFifths?, time?: {beats,beatType}, labels?: bool,
elements: [{p, fig, dots?} | {rest:true, fig}], caption}` ·
`score {musicXml, prompt}` (1–2 measures, MINIMAL fixture shape).

Interactive (gate the lesson's Next):
- `readPlay {notes[], mode: drill|melody|set, clef?, keyFifths?, labels: always|afterMiss|never, prompt}` — read staff → play key/MIDI.
- `nameNote {notes[], clef?, keyFifths?, choiceCount?, prompt}` — name chips (they sound).
- `placeNote {targets[] (naturals only), clef, prompt}` — tap the staff position.
- `rhythmTap {pattern[], beats, beatType, bpm, passRatio?, prompt}` — tap the rhythm against the metronome; pattern must fill exactly 1 or 2 bars.
- `earChoice {notes[], choices: [{id, label}], answerId, gapMs?, harmonic?, reveal?, prompt}` — listen then choose.
- `buildChord {notes[] (2–5), prompt}` — toggle keys to build the chord.
- `playKey {notes: [midi…], prompt}` — legacy "play these keys" (keyboard only, no staff).

## Authoring rules (enforced by the corpus test)

1. First block is a short `text` hook (≤ 2 sentences, second person, warm); never
   two consecutive `text` blocks; end on a `text` recap naming what was learned.
2. ≥ 4 interactive blocks per course, of which ≥ 2 are v2 types — a lesson is
   something you *do*, not something you read.
3. One new concept per course; every drill only uses material already taught
   (earlier in the course or an earlier `sortOrder`).
4. Melodies are payoffs: end units on a real tune built from learned notes only.
5. French uses `tu`; tone encouraging everywhere; wrong answers are never
   shamed ("Presque…", never "Faux !").
6. Note names in prose follow the locale (fr/es/it solfège do-ré-mi, en letters
   C-D-E); never hardcode one convention across locales.
