# add-lingua-review-repeat-hint — a card that comes back says so

## Why

A card the reader misses comes back in the same session, after three other cards, up to three times
(`refine-lingua-review-session` D3). Nothing on the card says it: dogfooding the Cymbra Lingua app on
an iPhone (2026-10-13), the product owner read the same word coming back as a bug, and expects
readers to report it as one. The card should say why it is there again.

## What Changes

- **A card asked again in a session says so**: above its sentence, in place of the remaining count,
  « À revoir · 2ᵉ passage » on its second showing and « À revoir · 3ᵉ passage » on its third — the
  wording chosen by the product owner on 2026-10-13. A card's first showing is unchanged.
- **Every review host**: the side panel, the in-page drawer and the Cymbra Lingua app, which all
  render the same review view.
- **Counted by the review page**, which sees every card it shows: no engine, backup or sync change.
- Copy in French, English and Spanish.
- Unchanged: when a missed card comes back, how many times, what a later answer does, the end summary.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-decks-review`: the requirement « A missed card comes back in the same session » gains that
  the card says, when it comes back, that it is being asked again and for which time.

## Impact

**Products.**
- **Cymbra Lingua — changed behaviour.** The extension's review view (`apps/lingua-extension`,
  `src/review/`), shown by the side panel, the drawer and the Apple app (`apps/lingua-apple`, through
  the `app` build). New copy in the catalogue (`review` module, fr/en/es).
- Cymbra ID, Music, Live, the back office and the public site are untouched. No backend, no `.proto`,
  no flag. The core (`crates/lingua-core`) and its WASM build are untouched.

**Code.** `src/review/session.ts` (the showing count of the current card), `src/review/view.ts` (the
note), `src/i18n/{fr,en,es}/review.ts`.
