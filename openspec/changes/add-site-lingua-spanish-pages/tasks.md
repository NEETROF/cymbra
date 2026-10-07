# Tasks

## 1. The figures

- [x] 1.1 `scripts/lingua-data/gloss_coverage.py` (design D2, D3). It measures each pair `packs.json` ships, from its committed tables: the share of the 5,000, 10,000 and 20,000 commonest lemmas with a French gloss. `--write` writes `apps/site/src/data/lingua-coverage.json`, and `--check` compares that file with a fresh measure. Specs, in `test_gloss_coverage.py`:
  - the measure on a small fixture;
  - the committed JSON matches the committed tables.
- [x] 1.2 `apps/site/src/data/lingua-coverage.json`, written by the script.
- [x] 1.3 `lingua-pack-update` writes the JSON again after rebuilding a pair's tables.

## 2. The pages

- [x] 2.1 `apps/site/src/pages/lingua.astro` and `en/lingua.astro`, in the wording the owner settles (design D1, D2):
  - the meta description and the tagline;
  - the word, level and language cards;
  - the coverage section, reading the JSON, rounded in the page's locale;
  - the Safari app, now live: its store button links to the App Store record, and the hero says so.

## 3. Checks

- [x] 3.1 `site-check` passes: the site builds.
- [x] 3.2 The lingua-data unit tests pass.
- [x] 3.3 `openspec validate add-site-lingua-spanish-pages --strict` passes.

## 4. Release (owner)

- [x] 4.1 `site-deploy` is dispatched with the release that ships Spanish.
