# Design — add-site-lingua-matrix-pages

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `apps/site/src/pages/lingua.astro`, `en/lingua.astro` | 111 lines each, parallel; `languages = {'en-fr': …, 'es-fr': …}`; `Intl.NumberFormat` `fr-FR`/`en-US`; the tagline and meta description name English and Spanish; the English hero note "Made for French speakers learning English or Spanish: the interface and the translations are in French"; the card section quotes « Je connais », « + Deck », « Ignorer »; the coverage section `#couverture` "have a French gloss", one column per pair; the translation note "through English"; store badges from `linguaStores(lang)` |
| `src/data/lingua-coverage.json` | `tops [5000, 10000, 20000]`, `glossed {en-fr: [95.1, 90.1, 78.9], es-fr: [87.6, 77.2, 63.7]}`; written by `scripts/lingua-data/gloss_coverage.py --write` for the pairs of `apps/lingua-extension/packs.json`; checked by `test_gloss_coverage.py`; rewritten by `lingua-pack-update` |
| `apps/lingua-extension/model-manifest.json` | routes per pair: one model (direct) or two (through English) |
| `openspec/specs/site-lingua-page/spec.md` | *The Lingua page names its languages and publishes their coverage*: French and English pages, "a French gloss", the scenario *Translation for English only* |
| Change 29 | `es` locale, `alternates` |

## Goals / Non-Goals

**Goals:**
- One page structure, fed by the shipped pairs, in three languages.
- A reader lands on the pairs glossed in their language.

**Non-Goals:**
- Publishing figures for a pair not shipped (`gloss_coverage.py` measures shipped pairs only).
- The store listings; model sizes on the page.

## Decisions

### D1 — One component, fed by data

`src/components/LinguaPage.astro` takes the site language and renders the page from
`linguaPairs()` (`src/lib/lingua-pairs.ts`): the shipped pairs (`packs.json`), each with its studied
and native language, its coverage (`lingua-coverage.json`) and its route length (the catalogue).
The three pages are one line each. The text around the data — tagline, the card and levels
sections, notes — is per site language, in a table beside the component; the French table is
today's French page, byte for byte where no pair is added.

### D2 — Readers' pairs first

The page in language L shows first the pairs whose native language is L, under an audience
sentence ("for French speakers learning English and Spanish"), then the others, each named "<studied>
for <native> speakers". With only French-native pairs, `/lingua/` is today's page; `/en/lingua/`
keeps today's sentence that the product is made for French speakers.

### D3 — A page for a language no pair is glossed in

`/es/lingua/` is generated only when a shipped pair is glossed in Spanish (`getStaticPaths` over the
natives of `packs.json`), and the Spanish nav links it only then; until then `/es/lingua/` answers the
Spanish 404. When en-es ships, the page appears with the site deploy that goes with it.

### D4 — Coverage and translation per pair

The coverage table's columns are the shipped pairs, headed "<studied> → <native>", its caption "the
share of the commonest words that have a gloss in the reader's language"; the translation note says,
per pair, whether extended translation serves it and whether directly or through English, from the
route's length.

## Risks / Trade-offs

- **French bytes of `/lingua/`** → a test renders it with today's pairs and compares with the
  committed page's text.
- **A Spanish page appearing with a deploy before en-es ships** → D3 builds it from `packs.json`.

## Migration Plan

Merged with today's pairs, `/lingua/` is unchanged and `/en/lingua/` reads as before; the pages
move with the pairs of changes 34 and 35, at the deploys that ship them.
