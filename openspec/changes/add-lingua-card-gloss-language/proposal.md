# add-lingua-card-gloss-language — a card says the language of its gloss, and review shows one the reader can read

## Why

Change 11 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the eighth of stage 1, and the client half of decision M4 that lives on the device. The server
half, `add-lingua-native-language-server` (change 10), stores a card's gloss language; this
change gives the card the label on the device, writes it at creation from the engine's native
language, carries it through the local export and apply of card operations, and uses it in
review. The wire — sending the label, withholding non-French cards until the server stores the
label — is change 12.

Today a card's gloss is a string with no language. Every gloss is French, because every shipped
pack is glossed in French. The day a reader has two devices with two native languages (change
20), or restores a backup written under another native, a review shows a gloss the reader may
not read. M4 settles what review does then: it shows the current pack's gloss for the lemma —
for a word, or for an expression of the table — and keeps the card's own text when the pack has
none, which is what a free translation is.

This is a silent release. For a reader of French with French glosses, nothing moves: the label
is `fr`, written out of the backup, so every backup written today is byte for byte the same,
and the English baseline (S0) is unchanged.

## What Changes

- **`Card.gloss_language`** in `lingua-core`: the language the gloss is written in, `fr` by
  default, left out of the backup when it is `fr`. A card created on an engine takes the engine's
  native language (change 4 gave an engine one). The backup schema stays at version 2; a build
  released before this change ignores the field.
- **The local card operations carry it**: `exportCardOps` emits `gloss_language`, `applyCardOps`
  reads it (empty means `fr`). The sync client does not send it yet (change 12).
- **Review shows a gloss the reader can read.** When the card's gloss language is not the
  engine's native language, `reviewCurrent`'s `gloss` is the current pack's gloss for the lemma
  (a word) or the expression (a lemma with spaces, looked up in the pack's expression table); when
  the pack has none, the card's own text. The view model's keys do not change, so the English
  baseline pins the same bytes.
- **The agent plugin** creates its cards with its pack's native language, as the extension does.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-decks-review`: ADDED *A card says the language of its gloss*, *Review shows a gloss the
  reader can read*. *Card schema with provenance* is held by the open `add-lingua-reader` and is
  not modified: the new requirement stands beside it.

No requirement is modified.

## Impact

- **Products.** Cymbra Lingua only: `crates/lingua-core` (`decks/card.rs`, `decks/backup.rs`
  tests), `crates/lingua-wasm` (`add_card`, `export_card_ops`, `apply_card_ops`,
  `review_current`, tests beside `cross_native.rs`), `apps/lingua-extension` (`analyzer/port.ts`
  `CardOp`, no surface change), `apps/lingua-agent/rust` (card creation). ID, Music, Live, the
  back office and the site are untouched.
- **No byte moves.** Both baselines pass without `LINGUA_BLESS`; a French reader's backup is
  byte for byte the same; the review page shows the same French glosses.
- **Order.** Independent of changes 8–10 on the device; its label reaches the server only
  through change 12, after change 10 is deployed and checked from outside.
- **Not here.** Sending and reading the label, withholding non-French cards, statistics with the
  native language (change 12); an edited gloss or a free translation the reader types (none
  exists today; a gloss the pack has not is kept as it is).
