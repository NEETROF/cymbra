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
`linguaPairs()` (`src/lib/lingua-pairs.ts`): the shipped pairs, taken from `src/data/lingua-coverage.json`
(whose `glossed` keys are exactly `packs.json`'s pairs, `gloss_coverage.py` and its test hold it), each
with its studied and native language, its coverage and its route length from
`apps/lingua-extension/model-manifest.json` — read in Astro frontmatter only, never from a Vue island;
`site-check.yml`'s `site` filter gains `apps/lingua-extension/model-manifest.json`. `linguaStores` and
`Community.astro` take `es`.
The three pages are one line each. The text around the data — tagline, the card and levels
sections, notes — is per site language, in a table beside the component; the French table is
today's French page, byte for byte where no pair is added.

### D2 — Readers' pairs first

The page in language L shows first the pairs whose native language is L, under an audience
sentence ("for French speakers learning English and Spanish"), then the others, each named "<studied>
for <native> speakers". With only French-native pairs, both pages are today's: `/en/lingua/` presents them under today's
sentence that the product is made for French speakers, not as "others".

### D3 — A page for a language no pair is glossed in

`/es/lingua/` is `src/pages/[locale]/lingua.astro`, whose `getStaticPaths()` returns `{ params: {
locale } }` for each native language of a shipped pair other than `fr` and `en` (`es` once en-es
ships) and `[]` until then — a fixed-path page would always be built; the static `lingua.astro` and
`en/lingua.astro` outrank it. One predicate in `lingua-pairs.ts` drives the Spanish nav link and the
`alternates`/`hreflang` of `/lingua/` and `/en/lingua/` (change 29's rule that a page names every
translation). Until then `/es/lingua/` answers the Spanish not-found page. When en-es ships, the page
appears with the site deploy that goes with it.

### D4 — Coverage and translation per pair

While every listed pair has the same native language, the coverage table keeps today's headers
(« Anglais », « Espagnol »; "English", "Spanish" on `/en/lingua/`) and caption; "<studied> →
<native>" headers, and a caption naming "a gloss in the reader's language", appear once pairs of two
native languages are listed; the translation note says,
per pair, whether extended translation serves it and whether directly or through English, from the
route's length.

## Risks / Trade-offs

- **French bytes of `/lingua/`** → a test renders it with today's pairs and compares with the
  committed page's text.
- **A Spanish page appearing with a deploy before en-es ships** → D3 builds it from the
  `glossed` keys of `src/data/lingua-coverage.json` (a key `<studied>-es`), which a site test
  holds equal to `packs.json`'s `pairs`, and `site-check` runs when either file changes.

## Deviations in the implementation

- (a) D2: the other readers are named per group, not per pair — one sentence each, "Also for <speakers> learning <studied>: the interface and the translations are in <native>" — and the pairs as "<studied> → <native>" in the table and "<studied> to <native>" in the translation note.
- (b) D3: a page per native language the site speaks (`LANGS`), not per native language of a shipped pair; the same predicate also points the Spanish footer and not-found links, and `musicStores`/`Downloads.astro` take `es` beside `linguaStores` and `Community.astro` (D1).
- (c) D1: the text slots go through `set:html` (an `{expression}` escapes `'` and would move the French bytes), so `lingua-pairs.ts` fails the build on a key that is not two language codes, a route that is not a list or a figure that is not finite, and `fill` escapes `&`, `<`, `>`.
- (d) Risks: the French bytes are compared with fixtures of the previous build's `<main>` (`apps/site/test/fixtures/lingua/`), rendered on their own pairs through Astro's Container API and checked on the build while the shipped pairs are theirs, not with "the committed page's text".

## Migration Plan

Merged with today's pairs, `/lingua/` is unchanged and `/en/lingua/` reads as before; the pages
move with the pairs of changes 34 and 35, at the deploys that ship them.
