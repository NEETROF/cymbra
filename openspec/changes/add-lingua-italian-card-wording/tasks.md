# Tasks

## 1. The renderer

- [ ] 1.1 `apps/lingua-extension/src/i18n/it/grammar.ts` (D1–D5): `typeof fr`; parts of speech, genders, numbers, persons; `TENSES` with the four studied rows (D1–D3), `GERUNDS` (en, es, it), the participles and the infinitive; contractions with the tense's article, « può anche essere » with each name's article, « di » never elided before a word of the page, « e » joining; numbers `["Sing", "Plur"]`, merges by key, `mergeGenders`; the apostrophe of the Italian catalogue.
- [ ] 1.2 `src/reading/reading-copy.ts`: Italian takes the Italian renderer, and `test/reading-copy.spec.ts` checks `readingCopy("it").grammar` (D6); `src/i18n/README.md` names it; the comments of `src/reading/grammar-description.ts` and `src/analyzer/types.ts` that list the renderers.

## 2. Tests

- [ ] 2.1 `test/word-grammar-it.spec.ts` (D7): every key of D1–D3 named, the future subjunctive and the gerunds included; persons merged; agreement; the contractions and « può anche essere »; the sense headings; what stays unnamed; the spec's scenarios (`parlò`, `parla`, `habló`, `went`, `running`, `hablaba`, `rápidas`).
- [ ] 2.2 The renderer loops of `test/grammar-description.spec.ts`, `test/i18n.spec.ts` and `test/wordpopup.spec.ts` gain `it`; the parity test's Italian exemption of `grammar` removed (D6).
- [ ] 2.3 A grammar-only export of the card harness `test/word-card-french.ts`; `test/word-card-grammar-it.spec.ts` and its committed snapshot: every grammar probe of the en-es, es-en, fr-en and it-fr goldens, rendered in Italian, grammar lines only — no gloss, no sense heading (D7); `.github/workflows/lingua-pack-update.yml` re-blesses it with the other card snapshots.

## 3. Review and gates

- [ ] 3.1 [agent] The Italian reviewer agent's pass over the names and the snapshot (D8): « condizionale presente » for Spanish, « determinante »; its findings applied or answered with a source in the pull request.
- [ ] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`; every existing card snapshot byte for byte.
- [ ] 3.3 `openspec validate add-lingua-italian-card-wording --strict`; the archive order; the two MODIFIED blocks re-copied as changes 33 and 18 then stand; `docs/lingua/italian-programme.md` row 34's status.
