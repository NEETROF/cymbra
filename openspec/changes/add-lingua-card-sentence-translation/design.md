# add-lingua-card-sentence-translation — design

## Context

See proposal.md for the why. What exists:

- **The word card's translation.** With the extended translation on, the word card and the
  selection card (`src/reading/selection-card.ts`) ask the translation engine, bounded by
  `TRANSLATION_WAIT_MS` (15 s), and show a `MarkedTranslation` (`src/translate/markup.ts`):
  the translated sentence as plain text, plus `marks`, the spans where the selection landed
  (usually one, several when the engine split it, none when it dropped the tag). The engine
  runs off every painting thread and is reached through `createTranslatorPort`, which only
  the reading session creates.
- **Capture.** A press on « + Deck » arrives as a `Gesture` (`src/reading/wordpopup.ts`)
  carrying the lemma, the form seen, the sentence and the gloss the card showed;
  `ReadingSession.onGesture` (`src/reading/session.ts`) calls `addCard`.
- **The card** (`crates/lingua-core/src/decks/card.rs`): lemma, form, provenance (sentence,
  source, time), gloss, a reserved media slot, the FSRS state, `updated_at`. Backups carry a
  schema version that belongs to the file (`decks/backup.rs`, version 2); unknown fields are
  ignored on read.
- **Sync.** `CardOp` (`backend/lingua/proto/deck.proto`, fields 1–11) carries a whole card;
  the server upserts it under last-write-wins (`pg_deck.rs`: `updated_at`, then `device_id`)
  and the client applies pulled cards the same way (`Deck::apply_card_lww`). One precedent
  for a field older clients send empty: the page address, which the client keeps locally
  when the pulled one is empty (add-lingua-privacy-controls).

## Goals / Non-Goals

**Goals:**
- Keep what the word card already computed; spend nothing more at capture or at review.
- Old devices, old backups and the agent plugin keep working, and never erase a translation.

**Non-Goals:**
- Translating a sentence at review time, or for cards captured without a translation,
  seeded from a level, or created by the agent plugin: a later change.
- Editing or deleting a kept translation.
- Showing translations anywhere but the review card.

## Decisions

### D1 — Capture takes what the card shows, at the press

The `Gesture` gains `translation: MarkedTranslation | null`, filled from the content the
card is showing. `onGesture` passes it to `addCard`. If the translation has not arrived, the
card is created without one: waiting would hold the capture for up to 15 s, and attaching it
later would need a second write racing the reader's next gesture. A later change fills
missing translations at review time.

*Alternative.* Translate again in the background at capture: a second engine call for a
result the card already had or is about to have.

### D2 — The card holds an optional translation; the backup's version does not move

`Card` gains `sentence_translation: Option<SentenceTranslation>`, with
`SentenceTranslation { text: String, marks: Vec<Span> }`. Spans count UTF-16 code units of
`text`, the unit the extension indexes strings in; the core stores them without reading
them. The field is `#[serde(default, skip_serializing_if = "Option::is_none")]`: a backup
without a translation is byte-for-byte today's, and the schema version stays at 2. A build
that predates the field reads a newer backup and drops the translations only — refusing
the whole file, as a version bump would make it, is worse for that reader.

Marks are checked where they are used: a span outside the text, or reversed, is ignored and
the sentence is shown unmarked.

### D3 — `CardOp` grows two fields, and an empty translation never wins

```proto
message TextSpan {
  uint32 start = 1;
  uint32 end = 2;
}
// in CardOp:
string sentence_translation = 12;
repeated TextSpan sentence_translation_marks = 13;
```

Both are additive: `buf breaking` (rule set `FILE`) passes, and clients built before them
ignore them. Last-write-wins replaces a whole card, so a newer op from such a client — or
from the agent plugin — would carry an empty translation and erase a kept one. The rule is
therefore the page address's, on both ends: the server keeps the stored translation and
marks when the incoming text is empty, and `apply_card_lww` keeps the local ones when the
pulled text is empty. The cost is that sync cannot clear a translation, which nothing does.

### D4 — The server stores two columns

`0006_lingua_card_sentence_translation.sql` adds `sentence_translation TEXT NOT NULL DEFAULT ''`
and `sentence_translation_marks JSONB NOT NULL DEFAULT '[]'` to `lingua.cards`, idempotent and
schema-qualified like 0001–0005. The upsert sets each with a `CASE` on the incoming text being
empty; pulls return them. `FakeDeckRepo` follows the same rule, so the module's tests cover it
without Postgres.

### D5 — The review shows it in the answer space

`reviewCurrent` adds `sentenceTranslation: { text, marks } | null` to its view model.
`renderReview` writes it under the gloss in the answer space of `refine-lingua-review-session`
(D6 there), in italic, the marked spans in bold, built from text nodes only — the text came
from a page through the engine. Because it is stored, it appears with the gloss in the same
repaint, which is the point on e-ink. The answer space is sized for the gloss and three lines
of translation; a longer translation scrolls inside it rather than moving the answers.

### D6 — The privacy annex follows the data

In `apps/site/src/pages/confidentialite.md` and `en/privacy.md`: the « Deck de révision » row
lists « la phrase où vous l'avez trouvé, la traduction de cette phrase si la traduction
étendue l'a faite, sa traduction et son état de révision », and the « Traduction étendue »
paragraph adds that the translation of a captured word's sentence is kept with the card and
synchronised with it. The `lingua-privacy` requirement stays true as written (the deck, without
addresses, is synced), so it needs no delta.

## Risks / Trade-offs

- [More text derived from a page leaves the device] → only for a signed-in reader who
  captures with the extended translation on; the sentence itself already travels with the
  card; disclosed in the annex.
- [Marks computed by the engine can be missing or split] → stored as given, used only when
  in range; an unmarked sentence is still useful.
- [An older build restoring a newer backup loses the translations] → accepted over a refused
  restore; the extension updates itself, so the window is short.
- [A translation captured in one language of glosses stays in it if the reader later changes
  that language] → rare, and the same holds for a card's gloss today.

## Migration Plan

Additive everywhere. Deploy order: the migration and the server first (the columns default to
empty), then the extension. Rolling back the extension leaves the server's columns unused.
Rolling back the server code leaves the columns and their values untouched: pulls stop
returning translations for that window, and every device keeps its local copy.
