# Tasks

## 1. The glossary and the pass

- [ ] 1.1 [agent] `apps/lingua-extension/test/italian-glossary.json` and `docs/lingua/italian-glossary.md`: each concept's French forms, accepted Italian forms, rejected variants and reason; built by D1's precedence (change 39's words, then `app_it.arb`, then the interface, then a platform's word).
- [ ] 1.2 [agent] The reviewer agent's pass over every surface of the design's *Context* table (D2): translations against their French, texts written per audience against change 46's D1 and claims table; Cymbra ID's Italian e-mails read, not edited; each finding applied in this pull request or answered with a source; a finding that changes meaning put to the owner.
- [ ] 1.3 `docs/lingua/italian-wording-review.md`: the surfaces read, each finding, its line and resolution, the count of strings read (D2).

## 2. The check

- [ ] 2.1 `texts()` extracted from `test/i18n.spec.ts` into `test/support/`; `apps/lingua-extension/test/italian-glossary.spec.ts` (D3): the catalogue's text surfaces (not `grammar.ts`), `_locales/it`'s messages, `copy.js` stripped of markup; the normalisation, the letter-bounded matching, rules 1–3, the keys written per audience left out.

## 3. Captures, snapshots, gates

- [ ] 3.1 Where a correction touches a captured string: `yarn capture` in its Italian one-language mode, the refreshed site media and `store-captures/it/` committed (D4).
- [ ] 3.2 Where a correction touches the card or its grammar: change 34's grammar snapshot and change 36's en-it card snapshot re-blessed.
- [ ] 3.3 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, and `yarn build`, `yarn check:variants`, `yarn check:version` if `_locales/it` changed; in `apps/site`: `yarn check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:routes`, the pinned slugs unchanged; `swift test` in `apps/lingua-apple/LinguaSignIn` if `SignInCopy` changed; every count of the listing files rechecked against its cap if a listing changed.
- [ ] 3.4 `openspec validate review-lingua-italian-wording --strict`; the archive order; `docs/lingua/italian-programme.md` row 44's status.

## 4. Owner

- [ ] 4.1 [owner] The report's summary read, and any meaning finding decided, before the build the owner dogfoods and photographs (D4).
- [ ] 4.2 [owner] The site deployed with the corrections, before change 45's release (I12).
