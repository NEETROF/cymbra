# add-lingua-card-sentence-translation — a card keeps the translation its word card showed

## Why

When the reader opens a word card with the extended translation on, the card shows the
whole sentence translated on the device, with the part that translates the word marked.
The reader then adds the word to the deck, and that translation is thrown away: the card
keeps the sentence and the word's gloss only. At review time the meaning of the sentence —
the context that makes a word's sense clear — is gone. Translating it again during review
would cost a model on that device and, on an e-ink reader, a second repaint after the
reveal. The translation already exists at capture: keeping it is free, and it then reaches
every device the reader reviews on, including those without a translation model.

## What Changes

- **A card keeps the sentence translation its word card showed** when the word was
  captured: the translated sentence and where the word landed in it. The same holds for an
  expression captured from a selection card. A card captured while the translation was off,
  or not there yet, keeps none.
- **The review shows it with the answer**, after the reveal, under the gloss, with the
  word's translation marked — in the answer space of `refine-lingua-review-session`.
- **It travels with the card**: in the backup (an optional field, so a backup without a
  translation is unchanged and no schema version moves), to the reader's other devices
  (two additive fields on `CardOp`), and in the server's store (one migration). An empty
  translation sent by a device that predates the field never erases one already kept.
- **The privacy annex (French and English) says it**: the synced deck now holds, for such
  cards, the translation of the sentence; the extended translation's own paragraph says that
  the translation of a captured word's sentence is kept with the card.
- Unchanged: translation still runs only on the device, from the model it downloads; no new
  network call. Cards captured without a translation stay without one; translating their
  sentence at review time, where a model is installed, is a later change.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-decks-review`: added — a card keeps the translation of its sentence.

This change archives after `refine-lingua-review-session` (`archiveAfter` in
`.openspec.yaml`): it shows the translation in that change's answer space.

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.** The core (`crates/lingua-core`), its WASM build
  (`crates/lingua-wasm`), the extension (`apps/lingua-extension`) on Chromium, Firefox and
  Safari, hence the Apple container app (`apps/lingua-apple`) that ships its bundle.
- **Cymbra Lingua backend — extended, not new.** `backend/lingua`: the existing
  `DeckService` and its `CardOp` gain two additive fields (`buf breaking` passes), the cards
  table two columns, and the upsert one rule. No new RPC.
- **Public site — content only.** `apps/site`: the Lingua annex of the privacy policy, in
  French and English. No new page, no new capability.
- Cymbra ID, Music, Live and the back office are untouched. Nothing new is consumed from
  `id-*` or `platform-*`. The agent plugin (`apps/lingua-agent`) keeps its own store; a
  card it may push without the field never erases a kept translation.

**Code.**
- `crates/lingua-core/src/decks/card.rs` (the field), `decks/review.rs` (`apply_card_lww`
  keeping a local translation when the pulled one is empty).
- `crates/lingua-wasm/src/lib.rs`: `addCard`, `reviewCurrent`, the card ops.
- `backend/lingua/proto/deck.proto`, `backend/lingua/src/deck.rs`, `pg_deck.rs`,
  `backend/lingua/migrations/0006_lingua_card_sentence_translation.sql`.
- `apps/lingua-extension`: the word and selection cards' gesture, `reading/session.ts`
  (capture), the port, `sync/sync.ts` (the op mapping), `review/view.ts` (display).
- `apps/site/src/pages/confidentialite.md`, `apps/site/src/pages/en/privacy.md`.

**Data.** No migration of existing cards: they keep no translation. The server columns
default to empty.

**CI.** `rust`, `lingua-extension-check`, `proto`, `backend-it` and `site-check` already
watch these paths.
