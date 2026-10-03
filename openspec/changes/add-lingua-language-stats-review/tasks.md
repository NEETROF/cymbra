## 1. The engine

- [ ] 1.1 `lingua-core`: `ReviewSession::start_for(deck, now, languages)`, and `start` as `start_for` with none. Tests:
  - a mixed queue in due order;
  - a filtered queue;
  - an empty list taking every language.
- [ ] 1.2 `lingua-wasm`:
  - `startReview(now, languages?)` and `dueCount(now, languages?)`;
  - `reviewCurrent` adds `language`.

  Native tests: a filtered count and session, and the card's language.

## 2. The review page

- [ ] 2.1 Port: optional languages on `startReview` and `dueCount`, and `ReviewCard.language`, in the engine mirror, the messaging port and the fake port.
- [ ] 2.2 `review-page.ts` and `review/session.ts`:
  - the filter when several languages are accepted;
  - the summary and the session following it;
  - the card's language tag;
  - each grade and mark-known recorded in the card's language.

  Specs: hidden with one language; « Espagnol » filters the count and the session; the tag; the recorded language.

## 3. Statistics per language

- [ ] 3.1 `dailystats.ts`: v3 per language, v2 read as English, recorders with a language, and both keys cleared. The session records reading and learning in the document's language. Specs:
  - migration;
  - recording per language;
  - the clear.
- [ ] 3.2 `sync.ts`: one statistic per day and language. Spec: two languages on one day give two statistics.
- [ ] 3.3 `stats/view.ts`: the language selector when several languages are accepted. Specs:
  - hidden with one;
  - choosing Spanish shows the Spanish ladder and figures.

## 4. Gates

- [ ] 4.1 Rust:
  - `cargo test -p lingua-core -p lingua-wasm`, the English baseline unchanged;
  - `cargo fmt --all --check`;
  - `cargo clippy --workspace --all-targets -- -D warnings`.

  In `apps/lingua-extension`:
  - `yarn typecheck`, `yarn lint`, `yarn test`, `yarn format:check`;
  - `yarn build`, `yarn check:variants`.
- [ ] 4.2 `openspec validate add-lingua-language-stats-review --strict` passes. In `docs/lingua/spanish-programme.md`, change 14 is marked done.
