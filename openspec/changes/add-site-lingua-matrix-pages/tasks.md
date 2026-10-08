# Tasks

## 1. The page (apps/site)

- [x] 1.1 `src/lib/lingua-pairs.ts` (shipped pairs from `lingua-coverage.json`, coverage, route length from the catalogue) and `src/components/LinguaPage.astro`; the pages one line each; the text tables per language, the French today's; `linguaStores` and `Community.astro` take `es`; `site-check.yml`'s filter gains the catalogue (D1). Test: `/lingua/`'s and `/en/lingua/`'s `<main>` with today's pairs read as the committed pages'.
- [x] 1.2 Readers' pairs first, the audience sentence per language (D2). Tests with a pair list that holds es-en and en-es.
- [x] 1.3 `src/pages/[locale]/lingua.astro` and its nav link and `hreflang` only when a Spanish-glossed pair ships (D3). Test both ways: no `/es/lingua/` with today's pairs; with en-es listed, `/es/lingua/` built, Spanish-glossed pairs first, linked from the Spanish nav. (`test/astro/lingua-page.spec.ts` renders the pages through Astro's Container API on committed pair lists.)
- [x] 1.4 The coverage table per pair and the translation note per route (D4).

## 2. Gates, review and docs

- [x] 2.1 In `apps/site`: `yarn check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:routes`; `python3 -m unittest discover -s scripts/lingua-data -p "test_gloss_coverage.py"` unchanged.
- [ ] 2.2 [manual] The owner reviews the English and Spanish text tables (M9) and deploys the site with the releases that ship the pairs (M18). The reviewers' suggestions left to the owner: in Spanish, « su forma de diccionario » → « la forma del diccionario », « explicada con claridad » → « en palabras sencillas », the two colons of the click card, « insignia de la casa », « tu mazo y tus estados », « a ningún sitio », « se lee en el suyo », « francohablantes », « lengua del lector » / « cada par », the repeated « en la propia página », « Unirse al Discord », « En español, la tarjeta… »; in English, the serial comma, "revision session", "Commonest words", "Extended translation, optional, serves…".
- [x] 2.3 `openspec validate add-site-lingua-matrix-pages --strict` passes, and `python3 scripts/openspec_archive_order.py add-site-lingua-matrix-pages` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 30 is marked done in `docs/lingua/language-matrix-programme.md`.
