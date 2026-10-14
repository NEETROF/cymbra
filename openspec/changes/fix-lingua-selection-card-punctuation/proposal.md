# fix-lingua-selection-card-punctuation — a card is never keyed by the punctuation the selection dragged in

## Why

In the iPhone dogfood of the review app (2026-10-14), the deck card « four-poster bed » came up
with « Pas de traduction pour cette expression. », although the en-fr pack's expression table
holds `four-poster bed → Lit à baldaquin` (#726). The review's front underlined
« four-poster bed, », comma included: the card was captured with the selection's raw text, and
a selection made with a finger often takes the comma, the full stop or the quote next to the
last word. The card's lemma is then `four-poster bed,`, which is no key of the pack, and review
has no way back to the pack's answer:

- **At capture**, the selection is snapped to whole words and trimmed of whitespace, never of
  punctuation. The whole-selection card is headed and keyed by that text unless the pack
  recognises the selection as one of its expressions; this card was captured before #726 added
  the entry, so it kept the raw text — comma included — as its lemma, with no gloss.
- **At review**, a card whose gloss is in the engine's native language is shown with its own
  gloss, and a card holding none is never looked up in the pack; when a card is looked up — one
  glossed in another language — it is read at its lemma as stored, edge punctuation included.

Every card a reader has captured that way since the first release is in the same state, and the
reader cannot fix it: there is no way to rename a card yet.

## What Changes

- **A selection is read without its edge punctuation.** Before a selection becomes a card —
  its headword, its form met, what the pack is asked, its place in its sentence, what a
  translation marks — the punctuation, symbols and whitespace at its two ends are trimmed:
  `four-poster bed,` is read `four-poster bed`, `« Bonjour »` is read `Bonjour`. Letters,
  digits and combining marks are never trimmed, and nothing inside the selection changes:
  `l’homme`, `d'abord`, `dit-il`, `rock 'n' roll` and `well-known` stay as written. A selection
  of punctuation alone opens no card.
- **Review reads a card without its edge punctuation, and without rewriting it.** A card holding
  no gloss is looked up in the pack held for its language, as a card glossed in another language
  already is, and every lookup reads the card's lemma trimmed of edge punctuation — so the
  existing `four-poster bed,` card shows « Lit à baldaquin ». The front marks the trimmed form in
  the sentence (no underlined comma), heads a card without a sentence by it, and compares the
  dictionary form with it.
- **No card is rewritten, merged or migrated.** The card keeps its stored lemma, form and
  history; the backup, the card operations and synchronisation carry the same bytes. A reader
  who wants the card renamed will do it with `add-lingua-card-edit`.
- Unchanged: a card with a gloss in the engine's native language shows it, the
  baselines' review views keep their bytes except a glossless card the pack glosses (re-pinned), and a selection without edge punctuation is read as
  today.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-browser-extension`: added — a selection is read without its edge punctuation before it
  opens a card (a requirement of its own, so that the two in-flight changes rewriting *Selection
  capture on any pointer* — `add-lingua-french-word-card`, `enable-lingua-french` — are left
  alone); added — the review's front reads a card's word without its edge punctuation.
- `lingua-decks-review`: modified *Review shows a gloss the reader can read* — a card without a
  gloss is looked up in the pack too, and every lookup reads the lemma without its edge
  punctuation.

## Impact

**Products.**
- **Cymbra Lingua — fixed behaviour.** The browser extension (`apps/lingua-extension`) on
  Chromium, Firefox and Safari, hence the Apple container app (`apps/lingua-apple`), whose review
  runs the extension's bundle unchanged; the core (`crates/lingua-core`) and its WASM build
  (`crates/lingua-wasm`), which the review's gloss lookup lives in.
- The agent plugin (`apps/lingua-agent`) is untouched: its `add_words` normalises the dictionary
  forms it is handed, it never reads a page selection, and it lists cards with their stored gloss.
- Cymbra ID, Music, Live, the back office and the public site are untouched. Nothing is consumed
  from `id-*` or `platform-*`: no flag, no RPC, no `.proto`, no backend change, no pack rebuild,
  no privacy annex change.

**Code.**
- `apps/lingua-extension/src/reading/selection.ts` — the edge trim of a capture: its text, its
  span in its sentence and the range a word is hit-tested at.
- `apps/lingua-extension/src/review/view.ts` — the front reads the headword and the form met
  trimmed.
- `crates/lingua-core/src/decks/card.rs` — the trim, shared by every lookup of a card's lemma.
- `crates/lingua-wasm/src/lib.rs` — `reviewCurrent` looks a card without a gloss up;
  `readable_gloss` reads the trimmed lemma.

**Data.** None: nothing is written, no card is rewritten, the backup schema does not change.

**CI.** `lingua-extension-check` and `rust` already watch these paths.
