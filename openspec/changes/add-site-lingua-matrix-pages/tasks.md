# Tasks

## 1. The page (apps/site)

- [ ] 1.1 `src/lib/lingua-pairs.ts` (shipped pairs, coverage, route length) and `src/components/LinguaPage.astro`; the three pages one line each; the text tables per language, the French today's (D1). Test: `/lingua/` with today's pairs reads as the committed page.
- [ ] 1.2 Readers' pairs first, the audience sentence per language (D2). Tests with a pair list that holds es-en and en-es.
- [ ] 1.3 `/es/lingua/` and its nav link only when a Spanish-glossed pair ships (D3). Test both ways.
- [ ] 1.4 The coverage table per pair and the translation note per route (D4).

## 2. Gates, review and docs

- [ ] 2.1 In `apps/site`: `yarn check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:routes`; `python3 -m pytest scripts/lingua-data/test_gloss_coverage.py` unchanged.
- [ ] 2.2 [manual] The owner reviews the English and Spanish text tables (M9).
- [ ] 2.3 `openspec validate add-site-lingua-matrix-pages --strict` passes, and `python3 scripts/openspec_archive_order.py add-site-lingua-matrix-pages` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 30 is marked done in `docs/lingua/language-matrix-programme.md`.
