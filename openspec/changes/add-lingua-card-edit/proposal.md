# add-lingua-card-edit — the reader corrects the card in front of them

## Why

A card is what capture made of it, and capture is not always right. In the dogfood of the
review in the Cymbra Lingua app on iPhone (2026-10-14), three cards could not be fixed: a free
selection kept as « colloquially referred to as » when the expression worth learning is
« referred to as »; a card captured with its comma, « four-poster bed, »; and a card with no
translation at all, which the reader could have written themselves. Today the only ways out
are to keep reviewing a wrong card or to hide it from review and capture it again from a page
that may be gone. The review is where the reader sees the card, so it is where they should be
able to correct it.

## What Changes

- **A « Modifier » link on the review card** (French UI copy), beside « Je connais » and
  « Ne plus me le montrer », in every review host: the extension drawer, Chrome's side panel,
  Firefox, Safari and the Cymbra Lingua app. Choosing it shows the answer, if it was hidden,
  and opens an inline form with two fields — the word or expression (the card's dictionary
  form) and its translation — with « Enregistrer » and « Annuler ».
- **The form is checked before it is saved**: trimmed, inner spaces collapsed, edge
  punctuation stripped by the same rule capture uses (`fix-lingua-selection-card-punctuation`),
  never empty, at most 120 characters for the form and 200 for the translation. The card
  stays in its studied language.
- **A new form is looked up again in the pack.** A translation the reader typed always wins;
  otherwise the pack's gloss for the new form is taken when the pack has one, else the card's
  previous text stays. A translation the reader wrote is marked as theirs, so the review never
  replaces it with the pack's — even where the pack's gloss would otherwise be preferred.
- **A form the deck already holds merges the two cards** into the existing one, under a
  precise rule: the longer review history, the earlier capture, the reader's own translation,
  and the sentence on screen.
- **A renamed card moves its word's status**: the old form's `learning` is cleared, the new
  form becomes `learning` unless the reader already decided otherwise about it.
- **A rename synchronises as a removal and an addition.** The deck now records removed cards
  (tombstones), exports them as deleted card operations, and applies deleted operations under
  last-write-wins; a later re-capture of the same form brings the card back. The server already
  stores and returns tombstones.
- **The reader-written mark travels**: in the backup (left out when unset, so every backup
  without an edit keeps its bytes), in the card operations the Cymbra Lingua app relays to the
  extension, and to the server as one additive field of `CardOp`, stored in one new column.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-decks-review`: added — editing the card under review, the checks on an edited form,
  the gloss an edit settles, merging into a card the deck already holds, the word's status
  following a rename, and the backup keeping edits; modified — *Review shows a gloss the reader
  can read* (a reader-written gloss is always shown).
- `lingua-sync`: added — a card removed by an edit stays removed on every device; modified —
  *Complete card synchronisation, media excluded* (the reader-written mark is part of the card).
- `lingua-apple-app`: modified — *A review made in the app reaches the extension* (edits
  included).

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.** The browser extension (`apps/lingua-extension`) on
  Chromium, Firefox and Safari, hence the Apple container app (`apps/lingua-apple`), which
  ships the extension's review page and relays its changes through the App Group unchanged in
  shape. The core (`crates/lingua-core`) and its WASM build (`crates/lingua-wasm`) gain the
  edit, the merge, the tombstones and the reader-written mark. The backend module
  (`backend/lingua`) gains one additive `CardOp` field and one column, with no logic.
- **Consumed, not redeclared.** The status withdrawal of `lingua-knowledge-model` (*Withdrawals
  synchronise as cleared statuses*) carries the old form's cleared status; the existing
  `DeckService` LWW and tombstone storage carry the removal. No flag, no new RPC, no new route.
- Cymbra ID, Music, Live, the back office and the public site are untouched. The agent plugin
  (`apps/lingua-agent`) keeps its own store and no review UI: untouched (its deck file keeps
  its bytes, the new fields being left out when empty).

**Code.**
- `crates/lingua-core/src/decks/card.rs` (the reader-written mark), `decks/review.rs` (deck
  tombstones, rename, merge, LWW with deletions, re-keying the session), `decks/backup.rs`.
- `crates/lingua-wasm/src/lib.rs` — a `reviewEdit` binding, `readable_gloss`,
  `exportCardOps`/`applyCardOps` with deleted operations and the mark.
- `apps/lingua-extension/src/review/{view.ts,review-page.ts,session.ts}`,
  `src/analyzer/{port.ts,engine.ts,messaging-port.ts}`, `src/sync/sync.ts` (the field both
  ways), `src/styles/review.css`, `src/i18n/{fr,en,es}/review.ts`.
- `backend/lingua/proto/deck.proto` (field 13), `backend/lingua/src/{deck.rs,deck_grpc.rs,pg_deck.rs}`,
  `backend/lingua/migrations/0008_lingua_card_gloss_by_reader.sql`.

**Related changes.** `fix-lingua-selection-card-punctuation` strips edge punctuation at
capture; this change uses the same rule for an edited form, whichever lands first.
`add-lingua-review-translation-fallback` saves an accepted machine translation as a card's
gloss; this change lets the reader edit it. Neither depends on the other.

**Data.** Two optional fields in the backup and on the wire, one column with a default. No
existing record changes; no backup schema version moves.

**CI.** `lingua-extension-check`, `rust` and `proto` already watch these paths; `buf breaking`
passes an added field.
