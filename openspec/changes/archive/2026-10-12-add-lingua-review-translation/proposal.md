# add-lingua-review-translation — the review translates what the pack cannot answer

## Why

A card the pack has no gloss for — an expression, or a word the dictionary does not know —
comes back in review with nothing to reveal but « Pas de traduction pour cette expression. ».
When it was captured, its word card showed the sentence translated on the device, and that
translation was discarded, as `lingua-translation` requires. Every such card already in a
reader's deck is unanswerable at review, and capturing them again is not an answer. Found in
the Chrome dogfood of `refine-lingua-review-session` (2026-10-06) on « such as those made ».

The engine and the reader's models are on the device that reviews: the review can ask them
for the same translation again, when the card comes up, and show it with the answer.

## What Changes

- **A card without a gloss is translated at review**, on the device: an expression, or a
  single word the pack does not gloss, has its sentence translated with the word's place
  marked — the word card's own rule — when this device has extended translation ready for
  the card's language. A card with a gloss is unchanged.
- **Asked when the card comes up, shown after the reveal**, in the answer space of
  `refine-lingua-review-session`, labelled as a machine translation as the word card labels
  it. The engine is warmed when a review starts, so the translation is there at the reveal;
  a late one never holds the reveal back and says it is coming.
- **An expression whose sentence does not hold it** — cards captured before the sentence was
  taken from the selection's position — is translated alone, under a label that says so. A
  single word is never translated alone.
- **Nothing is stored**: the translation is computed for display and discarded with the card,
  as `lingua-translation` requires; no backup field, no sync, no server change, nothing new
  leaves the device.
- Unchanged: no model, or extended translation off, gives today's review, with no line about
  a translation.

This change comes before `add-lingua-card-sentence-translation` (#694), which will keep on a
card the translation its word card showed (decided by the product owner on 2026-10-07): a
kept translation will then be shown instead of asking the engine.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-translation`: added — the review translates the sentence of a card the pack cannot
  answer.

This change archives after `refine-lingua-review-session` (`archiveAfter` in
`.openspec.yaml`): it shows the translation in that change's answer space.

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.** The browser extension (`apps/lingua-extension`) on
  Chromium, Firefox and Safari, hence the Apple container app (`apps/lingua-apple`), which
  ships its bundle unchanged. It consumes the existing translation engine and models of
  `add-lingua-translation-delivery`, through the same translator the word card uses.
- Cymbra ID, Music, Live, the back office and the public site are untouched. Nothing new is
  consumed from `id-*` or `platform-*`: no flag, no RPC, no `.proto`, no backend change, no
  privacy annex change. The core (`crates/lingua-core`), its WASM build and the agent plugin
  are untouched.

**Code.**
- `apps/lingua-extension/src/review/review-page.ts` — a translator from the host; the
  translation asked when a card without a gloss comes up, the engine warmed at the start.
- `apps/lingua-extension/src/review/view.ts` — the translation in the answer space.
- `apps/lingua-extension/src/reading/drawer.ts` (the reading session's translator) and
  `src/sidepanel/sidepanel.ts` (`createTranslatorPort`, the messaging port only).
- `apps/lingua-extension/src/styles/review.css`.

**Dependencies.** The implementation builds on `refine-lingua-review-session` (#696): it
starts once that change is merged, or from its branch.

**Data.** None: nothing is written.

**CI.** `lingua-extension-check` already watches these paths; `test/lint-translator-placement.spec.ts`
keeps the engine off every thread that paints.
