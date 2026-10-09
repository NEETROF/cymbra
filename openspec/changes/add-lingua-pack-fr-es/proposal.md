# add-lingua-pack-fr-es — French glossed in Spanish: fr-es's native side, held to a floor fixed before it is measured

## Why

Change 49 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
stage 3 (French studied: fr-en, fr-es), decision M6. Change 43 (`add-lingua-french-forms-tables`)
commits French's studied side once, in `tables/fr/`, written by fr-en's reduction; change 48
(`add-lingua-pack-fr-en`) glosses it in English and makes fr-en's glossed lemmas French's
dictionary words (`tables/fr/lexical.tsv`). No pair glosses French in Spanish: a Spanish speaker
cannot read French with Lingua.

The programme measured fr-es as the thinnest pair of the matrix — « 83.4 / 70.8 / 56.5 %, 24 % of
them definitions | Spanish Wiktionary + French Wiktionary translations » — and its risk 5 names it:
thin Spanish glosses read as a lesser product. The owner settled M6 on 2026-10-09: **fr-es ships
with its coverage published (es-fr's precedent), and a floor, and what happens below it, are fixed
before the committed measurement.** This proposal fixes both, from a prototype run on the real data
(design, *Measured*), so that the update whose tables are committed is measured against a rule
written beforehand, and never the other way round.

The sources are the three files change 38's catalogue already registers « for change 49 »: the
Spanish Wiktionary's French section (8,683 entries — its definitions), the French Wiktionary's
Spanish translations (56,466 French entries listing Spanish words) and the Spanish Wiktionary's
French translations, read backwards (18,449 Spanish entries listing French words). Every gloss is
Spanish written by a person (M5): no pivot through English, no machine translation. The prototype
reproduces the programme's figure — 83.4 / 70.9 / 56.9 % with the three sources, 24.0 % of the
glossed lemmas from a definition — and shows why the shape is en-es's and es-fr's: the definitions
alone gloss 40.0 / 28.5 / 18.3 %, and no two of the three sources reach what all three do.

The prototype also read fr-es through the rules en-es was given before Spanish speakers read it
(`refine-lingua-en-es-glosses`, 24b, whose `reduce_edition_es.py` change 43 says fr-es loads): the
Spanish edition's notes and pre-pass apply as they are and move little here — the section is small
and clean —, en-es's pair rules apply once transposed to French entries (a name's note on a common
word, possessive adjectives read as determiners), and French data shows two defects of its own: the
third commonest French word, `et`, is defined « Et » by the Spanish Wiktionary, and eleven letters
are glossed by themselves.

## What Changes

- **fr-es's native side, a reader pair of French** (design D1, D2): `reduce-fr-es.py` reads
  `tables/fr/` as committed and writes `tables/fr-es/` alone — `gloss.tsv`, `senses.tsv`, `mwe.tsv`,
  `NOTICE`, `manifest.json`, `pin.json`, `README.md`. Definitions first (the Spanish Wiktionary's
  French section, by the Spanish edition's rules), then the French Wiktionary's Spanish translations
  in the table's order, then the Spanish Wiktionary's French translations read backwards, commonest
  Spanish word first — words and expressions alike, at most three words per part of speech. A
  typographic apostrophe in a French headword is read as `'`, as French's forms are.
- **Sources from the editions' dumps** (D3): `DUMPS["fr-es"]` names the catalogue's three files —
  no new derivation; the first tables come from an update dispatched on the branch, which publishes
  `lingua-pack-sources-fr-es-<snapshot>`; a re-reduction fetches three files of about 15 MB.
- **24b's rules where they fit** (D4–D6): the Spanish edition's notes and its pre-pass
  `read_as_meanings` (senses the edition marks obsolete after the others, one typography), loaded as
  en-es loads them; fr-es's own pass over the French entries — a name's note does not gloss the
  common word spelled like it (`pierre` « Piedra », no longer « …; Nombre de pila … Pedro »),
  possessive and demonstrative adjectives and their forms are determiners (`mon`, `ce`, `mes`); the
  tables' letters gloss nothing (`h` « H », `x` « X » removed), a Spanish word read backwards is
  listed once. Measured, the translators' notes and disused words en-es removes do not occur in
  fr-es's glosses: not ported.
- **The studied word is no definition** (D7, M5): a definition that only repeats the French headword
  yields to the French Wiktionary's Spanish words when they differ — `et` « Et » becomes « Y, e »;
  nine rows.
- **M6, fixed here** (D8): `FLOORS["fr-es"] = (81.4, 68.8, 54.5)`, the study's figures less two
  points, the rule en-es's and fr-en's floors follow, settled by the owner on this proposal's pull
  request, before the dispatch. **Below it**: at the first committed measurement, the tables are not
  committed, no package lists fr-es, and French ships for English speakers alone until a later
  regeneration of the dumps measures at or above the same floor; after that, any pull request that
  re-reduces fr-es under it — fr-es's own update, a rule, or French's studied tables moving — fails.
  The floor is never lowered after a measurement. Above it, fr-es ships with change 52 and its
  coverage is published beside the other pairs', as es-fr's was.
- **Measured and shown** (D9): coverage against the floor, the glossed lemmas by source and the share
  of definitions (23.9 % of all glossed, 40.2 % of the glossed top 10,000), a sample of 100 glosses
  marked by source, the pack's size — in the tables' README and the pull request, stored in no pack.
- **Checked like every reader pair** (D10): the reduce job (after fr-en, against the floor), the
  committed-tables test, the row cut's check, and French's cross-native test: the French baseline
  answered through fr-en and through fr-es, alike once glosses and senses are removed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *French is glossed in Spanish from the Spanish Wiktionary's French
  section and the French Wiktionary's translation tables* and *fr-es is held to a coverage floor
  fixed before its first committed measurement*. Nothing modified. The requirements this change
  applies are open changes': a reader pair's shape and pin (`add-lingua-pack-es-en`), en-es's floor
  kept in one place (`add-lingua-pack-en-es`), the Spanish edition's notes and pre-pass
  (`refine-lingua-en-es-glosses`), the catalogue (`migrate-lingua-pack-sources-to-raw-dumps`),
  French as a studied language (`add-lingua-french-baseline`), French's tables
  (`add-lingua-french-forms-tables`) and French's dictionary words (`add-lingua-pack-fr-en`): all in
  `archiveAfter`. One sentence of 24b is true of its time and not after this change — « which only
  en-es reads: they re-pin en-es alone » — and is best amended when 24b is archived (Open Question 5).
- `lingua-analysis`: MODIFIED *An analysis does not depend on the native language* — French joins
  English and Spanish (its test and the scenario *French through another native language*); the
  requirement's text and scenarios are otherwise kept. Held by no open change.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data/` — *new*: `reduce-fr-es.py` and its tests, `tables/fr-es/`; *changed*:
    `pack_sources.py` (`DUMPS["fr-es"]`), `gloss_coverage.py` (`FLOORS["fr-es"]`),
    `test_gloss_coverage.py`, `test_reduce_editions.py` (an edit of the Spanish edition now moves
    en-es's and fr-es's digests), `SOURCES.md`; *consumed*: `reduce_common.py`,
    `reduce_edition_es.py`, `tables/fr/`, unchanged.
  - `crates/lingua-pack/tests/committed_tables.rs`, `crates/lingua-wasm/tests/cross_native.rs` —
    fr-es's scenarios.
  - `apps/lingua-extension/test/row-gloss-tables.spec.ts` — fr-es among the pairs whose rows are
    checked; no source, bundle or package changes.
  - `.github/workflows/lingua-pack-update.yml` (the pair option), `lingua-extension-check.yml`
    (fr-es reduced after fr-en, its coverage against its floor).

  ID, Music, Live, the back office, the site, the backend, lingua-core, the engine, the Apple host
  app and the agent plugin are untouched.
- **Licences.** The Spanish and French Wiktionaries through kaikki (CC BY-SA 4.0 + GFDL), already
  credited by es-fr and en-es; the studied side credited as fr-en's notice credits it. No new licence.
- **Release.** Silent: no package lists fr-es before change 52, and then only above its floor.
- **What does not move.** en-fr, es-fr, es-en, en-es and fr-en — tables, pins, packs, goldens and the
  extension's snapshots — byte for byte; `tables/fr/` (read, never written); `fr-en.golden`, which
  fr-es does not enter.
- **What it weighs.** 1,693,382 B for the prototype pack (change 43's tables and fr-en's dictionary
  words), 1,911,078 B with changes 45's and 46's prototype tables: under the 5 MiB budget, beside
  en-es's 1,688,931 B.
- **Order.** After 43 and 48 (required: French's tables and dictionary words), after 45 and 46
  (planned: the readings the inverted table is read by, the levels its pin records); before 50
  (marks), 51 (the Spanish card for French), 52 (ships fr-es, or not, by the floor) and 53.
- **Effort, against 2.5–5 ideal days**: 2.5–4.5 (design, *Effort*).
