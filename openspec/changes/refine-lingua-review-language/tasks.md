# Tasks

## 1. The engine

- [x] 1.1 `crates/lingua-core/src/decks/review.rs`: `Deck::count_for(languages)`, every language when the list is empty. `crates/lingua-wasm/src/lib.rs`: `deck_count(languages?)` reads the list as `due_count` does (design D4). Specs:
  - a deck of English and Spanish cards counts each language;
  - no list counts every card.
- [x] 1.2 The port (`deckCount(languages?)`), the messaging port, the engine wrapper and the test fake pass it.

## 2. The review page

- [x] 2.1 `src/review/review-page.ts`: no « Toutes ». The language resolves from the host's page language, else the last chosen (`cymbra-lingua-review-language`), else the first accepted (design D1, D2). A segment chosen is remembered (design D3). The summary counts that language's cards and due cards. Specs:
  - beside a Spanish page, the Spanish cards and counts;
  - away from a page, the last chosen;
  - choosing English switches and remembers it, and the choice holds beside the same page;
  - an opening, or another page or book, brings the page's language back;
  - a page in another language brings its own back; a session under way keeps its language;
  - English alone shows no segment, with the counts as before.

## 3. The hosts

- [x] 3.1 `reading/session.ts` gives the drawer the document's language, and `getStats` counts the deck and due cards in it (design D2, D4).
- [x] 3.2 `sidepanel/sidepanel.ts` asks the active tab's `getStats` for the page's language, null without an answer (design D2).

## 4. Checks

- [x] 4.1 Rust: `cargo test -p lingua-core -p lingua-wasm` (the English baseline unchanged), `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] 4.2 In `apps/lingua-extension`: `yarn gen:wasm`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [x] 4.3 `openspec validate refine-lingua-review-language --strict` passes.
- [x] 4.4 `docs/lingua/spanish-programme.md`: D8 superseded by the owner's decision of 2026-10-05; change 33 also reconciles the review requirement.

## 5. Dogfood (owner)

- [ ] 5.1 On a browser with both languages: the review beside a Spanish page opens in Spanish, away from a page in the last chosen, and the popup's count follows the page.

## 6. Dogfood findings (Chrome, 2026-10-06)

- [x] 6.1 The page's language comes back when the review opens again or the page or book beside it changes, instead of a choice made beside another page (design D3). Specs: `pageChanged()` brings the page's language back over a choice; the drawer follows its document's new language.
- [x] 6.2 Chrome's side panel follows the window's active tab while it is open: tab activated, page loaded, the session's language announced with its figures (design D3).
- [x] 6.3 Beside a book, the book's own language, from its metadata, wins over what a section reads as: a cover has no text, a Project Gutenberg front matter is English (design D2). Specs: a cover, then an English licence, in a book declared Spanish.
