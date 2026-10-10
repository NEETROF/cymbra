# add-lingua-fr-en-translated-expressions — fr-en's expressions completed by the French Wiktionary's English translations

## Why

Change 48 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(`add-lingua-pack-fr-en`, implemented with #855) glosses French in English from the English
Wiktionary's French section alone. Its design D3 measured the French Wiktionary's English
translations and declined them for words; for expressions it left them to the owner (its open
question 1): 24,138 more expressions, 3,248 occurrences in UD's 425,111 words beside the section's
32,998, about 363 KB of pack, « some compositional and wrong (« le grand » "the Great", « les
autres » "other people") », and a source fr-en's pin does not hold — change 48 removed
`kaikki-fr-traductions-en.jsonl` from `pack_sources.py`'s catalogue.

The owner decided on 2026-10-10: add them to fr-en, in a change of its own. This is that change,
outside the programme's 57, after `refine-lingua-fr-en-glosses` (48b), whose rules for expressions
(`LEFT_OUT`, the senses that only point) it keeps.

This change prototyped the table on the real data — the French Wiktionary's dump of 2026-10-02 00:10
(the very dump fr-es's pin records), fr-en's reducer and committed tables of `main` (reproduced byte
for byte), the pack builder and the engine's own phrase gloss over 1,060,904 words of French text
(UD French-GSD's sentences, Gutenberg, Wikipedia, Tatoeba and the baseline's pages) — and measured
what the table brings and each filter on a reviewed sample (design, *Measured*):

- Taken as es-en takes its direct table, the table's expressions are met 11,440 times (the section's
  60,359), « à la » "a la, à la" alone 2,173 times; with the section's rules for expressions applied
  to it, 38 of 200 sampled matches still read wrong, 13 partly.
- The section's expression must win: on 126 keys both reach, the builder's order gives the table's
  spelling 36 (« crème brulée » "creme brulee…" over the section's « crème brûlée »).
- No structural filter separates right from wrong: leaving out the expressions « compositional of
  their words' own glosses » keeps 35 % of the right matches and removes 8 of the 38 wrong ones; a
  determiner and one word removes 10 of them for 8 % of the right; the parts of speech, 20 for 16 %;
  the nouns alone, 30 for 58 %; three words or more, 34 for 79 %.
- What separates them is reading them where they are met: of the 362 expressions met at least five
  times (307 with a space, 78 % of their matches, and 55 split words), 51 read wrong in most of
  their contexts. Left out by name,
  with the rules below, a fresh sample of 100 matches reads 85 right, 6 partly, 9 wrong — the
  section's own expressions, sampled the same way, read 73 right and 26 wrong.

## What Changes

- **The French Wiktionary's English translations complete fr-en's expressions** (design D1):
  `reduce-fr-en.py` reads `kaikki-fr-traductions-en.jsonl` — the English translations the French
  Wiktionary's French entries list, derived from that edition's dump — through the shared
  `read_translations` (a proper noun's entry left out), a typographic apostrophe read as `'`, as
  es-en and fr-es read their direct tables: up to three translations in the table's order, as one
  sense, for a headword with a space or a single word French's tokenisation splits (change 48's two
  shapes; « s'asseoir » "sit, sit down, sit up"). No word is glossed by the table: `gloss.tsv`,
  `senses.tsv`, French's dictionary words and coverage do not move.
- **The section's expression wins** (D2): on its headword (the shared fallback's rule), on its key —
  a table headword whose words `tables/fr/forms.tsv` reads as a section expression's is left out (133;
  measured, the builder then gives no key to a table row the section reaches) —, and on what its
  rules leave out: `LEFT_OUT`'s expressions and the section's pointer-only expressions stay out of
  the table too (« à la », « sur ce », « boite à gants »).
- **A translation that repeats the headword is no gloss** (D3): 1,048 headwords glossed by
  themselves alone (« à deux » "à deux", « in situ ») are left out — a gloss in the studied
  language (M5) —, and 211 others keep their other translations.
- **Filters measured and rejected** (D4): compositionality, determiner pairs, parts of speech, three
  words or more, nouns alone, a rare word — each figure in the design.
- **A reviewed list** (D5): `LEFT_OUT` gains the 51 table expressions read wrong in most of the
  contexts they are met in, each with its reason — words met as they are where the table names a
  set phrase or a field's sense (« les autres » "other people", « le grand » "the Great", « au vent »
  "windward", « d'argent » "argent"), an address met as plain words (« à vous » "over", « mon cœur »
  "honey, darling"), a name met as itself (« côte d'azur » "blue band", « louis xiv »), a number
  (« à neuf » "anew"), a verb's form met as a noun's headword (« en laisse » met by « en laissant »),
  a wrong translation (« voie navigable » "thoroughfare").
- **fr-en's pin gains the French edition, its English edition kept** (D6): an update can keep an
  edition's recorded bytes (`pack_sources.py fetch-live --keep-edition`, `build.sh`
  `LINGUA_KEEP_EDITIONS`, `lingua-pack-update`'s `keep` input): dispatched for fr-en keeping `en`, it
  reads the French dump alone, publishes `kaikki-fr-traductions-en.jsonl` (≈ 1.9 MB compressed)
  under `lingua-pack-sources-fr-en-<snapshot>` — the owner's publication —, keeps `kaikki-en`'s
  record byte for byte, and moves no table of `tables/fr/`. The catalogue registers the file again
  (`EDITIONS["fr"]`, `DUMPS["fr-en"]`); the notice and manifest credit the French Wiktionary.
- **Measured, the result** (D7): 25,907 expressions offered (`mwe.tsv` 17,635 → 43,542 rows), 17,131
  keyed by the builder; the pack 2,537,386 → 2,989,253 B, 57 % of the 5 MiB budget; 60,359 → 68,382
  expression matches on the corpora (56.9 → 64.5 per 1,000 words, 52.2 → 55.7 % of sentences),
  882 of the section's displaced by a longer or earlier table expression — of 50 sampled, 37 read
  better, 11 the same, 2 worse.
- **What moves** (D8): fr-en's `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json`, README;
  `fr-en.golden` (`pack`, `notice`, `licences`, « Il est prêt à le faire » gaining « prêt à », and
  five phrase probes added); `fr-es.golden` gains the five probes, nothing else; the extension's
  `word-card-fr-en.txt` and `word-card-fr-es.txt` the same. en-fr, es-fr, es-en and en-es, fr-es's
  tables, pin and pack, and `tables/fr/` do not move.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *fr-en's expressions are completed by the French Wiktionary's English
  translations*, *An update can keep an edition's recorded bytes* and *fr-en's pin records the French
  Wiktionary's dump its expressions read*. Nothing is modified. The requirements this change builds on
  are open changes': the catalogue (`migrate-lingua-pack-sources-to-raw-dumps`), French's tables and
  reference pair (`add-lingua-french-forms-tables`), the keys the builder gives French headwords
  (`add-lingua-french-expression-keys`), fr-en's glosses and its pin (`add-lingua-pack-fr-en`) and
  fr-en's rules for expressions (`refine-lingua-fr-en-glosses`): all five are in `archiveAfter`.
  Sentences of those changes, and of `enable-lingua-french`, that this change makes true of their
  time are listed for their archives (design D9).
- `lingua-analysis`, `lingua-browser-extension`: none. The matcher and the card read expressions as
  written; what they show moves with the data.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *changed*: `reduce-fr-en.py` (the table, the merge rules, `LEFT_OUT`)
    and `test_reduce_fr_en.py`, `pack_sources.py` (the catalogue file, `DUMPS["fr-en"]`,
    `fetch-live --keep-edition`) and `test_pack_sources.py`, `build.sh`, `tables/fr-en/` (`mwe.tsv`,
    `NOTICE`, `manifest.json`, `pin.json`, `README.md`), `SOURCES.md`; *consumed*: `reduce_common.py`,
    `reduce_edition_en.py`, unchanged.
  - `.github/workflows/lingua-pack-update.yml` — the `keep` input.
  - `crates/lingua-pack/tests/committed_tables.rs` — fr-en's notice and sources.
  - `crates/lingua-wasm/tests` — `support/french.rs` (five phrase probes), `baseline/fr-en.golden`
    and `baseline/fr-es.golden` re-blessed.
  - `apps/lingua-extension/test/baseline/word-card-fr-en.txt` and `word-card-fr-es.txt` re-blessed;
    no source, bundle or package changes.

  ID, Music, Live, the back office, the site, the backend, the engine and lingua-core, the Apple host
  app and the agent plugin are untouched.
- **Licences.** The French Wiktionary (CC BY-SA 4.0 + GFDL), already credited by en-fr, es-fr, es-en
  and fr-es, is credited by fr-en for the English glosses of expressions.
- **Publication.** One release, `lingua-pack-sources-fr-en-<snapshot>`, published by
  `lingua-pack-update` on the owner's dispatch (M18).
- **Release.** Silent: no package lists fr-en before change 52.
- **Order.** After 48 and 48b (on `main`); either side of 43b and 43c (their re-reductions and this
  one's touch different tables of fr-en, and each re-measures on the other). Whether change 52 waits
  for it is the owner's (Open Question 2).
- **Effort**: 2.5–4.25 ideal days (design, *Effort*).
