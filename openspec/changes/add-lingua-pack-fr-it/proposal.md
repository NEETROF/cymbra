# add-lingua-pack-fr-it — French glossed in Italian, for Italian speakers

## Why

Change 47 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 4: Italian
speakers learning French. fr-it is the second pair glossed in Italian, after en-it (change 36).

Its sources (I11), in fr-es's order:
1. the Italian Wiktionary's French section, with the Italian edition's rules (change 35);
2. the French Wiktionary's Italian translation tables;
3. the Italian Wiktionary's French translations read backwards.

Measured on the committed French ranks, with change 35's rules prototyped, the three sources gloss 81.3
/ 70.1 / 57.8 % of the 5,000 / 10,000 / 20,000 commonest French lemmas, against fr-es's committed 83.2 /
70.9 / 57.1 and fr-en's 93.9 / 87.2 / 76.8. On French's dictionary words fr-it and fr-es are level
(91.2 % against 92.2 % at the 5,000, ahead at the 20,000).

The section itself is thin: 75.5 % of its senses are French inflections, and its definitions alone
gloss 28.8 / 20.9 / 13.8. The tables carry the pair: 70.2 % of the glossed top 10,000, against fr-es's
59.8 % and en-it's 12.8 %.

## What Changes

- **A reader pair** (design D1): `reduce-fr-it.py` reads French's committed studied tables and writes
  `tables/fr-it/`. Its rule digest is its own file, `reduce_common.py`, `reduce_edition_it.py` and
  `reduce_french_acronyms.py`.
- **The sources in fr-es's order** (D2). No pivot and no machine translation.
- **fr-es's add-pack rules, copied** (D3), not shared, so that fr-es's pin does not move:
  - letters out;
  - a word listed once across a row's parts of speech, read backwards;
  - the apostrophe read as `'` (4 lemmas, 1,177 expressions);
  - the studied word is no definition (4 rows);
  - an adjective tagged possessive or demonstrative read as a determiner (`ce`);
  - the Italian edition's typography.

  fr-es's name-note rule is Spanish-worded and left to change 48.
- **A borrowed gloss gives way** (D4), a rule of fr-it's own: a lemma glossed only through a form or
  pointer sense of another word (`fond` « fondere », `pensée` « pensare ») takes the direct table's
  gloss, else the inverted table's (18 / 26 / 40 rows fixed, coverage unchanged), and keeps the borrowed
  one when no table glosses it. The main spec's *A word that is only a form of another takes that
  word's gloss* is MODIFIED to allow it for fr-it alone.
- **Expressions** (D5): 15,295, keyed as fr-es keys them.
- **The floor** (D6): fixed before the first committed measurement at the measured figures less two
  points, about 79.3 / 68.1 / 55.8, the owner settling it. fr-es's floor would fail fr-it at the 5,000.
- **The sources registered** (D7): the Italian Wiktionary's French section, and `DUMPS["fr-it"]`. No
  committed pin moves.
- **The checks** (D8):
  - a baseline and golden over the French corpus;
  - the cross-native and committed-tables tests;
  - the selection card's row cut;
  - both workflows;
  - the full card snapshot in Italian that change 34 hands over (`word-card-fr-it.spec.ts`).
- **Coverage, share and sample published** (D9), by source.
- **Change 36's requirement amended** (D10): the Italian edition's rules are read by en-it and fr-it.
- **Known defects listed for change 48** (D11): name notes, labels before a colon, function words no
  source glosses (`des`, `du`, `ça`, `dont`), heraldic senses first on `un`, and the rest.
- **Not shipped** (D12): no package lists fr-it before change 50.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`:
  - ADDED *French is glossed in Italian from the Italian Wiktionary's French section and the two
    Wiktionaries' translation tables*;
  - ADDED *fr-it is committed beside French's studied tables, held to a coverage floor, and not shipped*;
  - MODIFIED *English is glossed in Italian from the Italian Wiktionary's English section and the two
    Wiktionaries' translation tables* (held by `add-lingua-pack-en-it`): the Italian edition's rules are
    read by en-it and fr-it, *A rule of the Italian edition* moves both digests, and its *Nothing else
    moves* is scoped to a commit by en-it's own reducer, fr-it's golden line naming en-it excepted;
  - MODIFIED *A word that is only a form of another takes that word's gloss* (main spec; no open change
    holds it): a pair whose reduction lets a translation table's gloss of the form win (fr-it) glosses
    the form with the table's words, keeping the borrowed senses only when no table glosses it.

## Impact

- **Products.** Lingua's data and tests:
  - `scripts/lingua-data/`:
    - new: `reduce-fr-it.py`, `test_reduce_fr_it.py`, `tables/fr-it/`, `testdata/fr-it/`;
    - changed: `pack_sources.py` (`EDITIONS["it"]`, `DUMPS["fr-it"]`), `test_pack_sources.py`,
      `gloss_coverage.py` (`FLOORS["fr-it"]`), `test_gloss_coverage.py`, `test_reduce_editions.py`,
      `test_reduce_loops.py`, `SOURCES.md`, `tables/fr-en/README.md`.
  - `docs/lingua/italian-programme.md` (the *Measured figures*, and its baselines line, which says the
    Italian-glossed pairs run over an Italian corpus).
  - `.github/workflows/{lingua-pack-update.yml, lingua-extension-check.yml}`.
  - `crates/lingua-wasm/tests/{fr_it_baseline.rs, baseline/fr-it.golden, cross_native.rs}`.
  - `crates/lingua-pack/tests/committed_tables.rs`.
  - `apps/lingua-extension/test/{row-gloss-tables.spec.ts, word-card-fr-it.spec.ts,
    baseline/word-card-fr-it.txt}`.

  No extension code, site, app or backend change.
- **The owner**: the floor (task 0.1), the sources release, the dispatch, and the reduction's pull
  request.
- **Prerequisites**, implemented first:
  - changes 3 and 4 (base and layer);
  - change 16 (`EDITIONS["it"]` and the two translation files);
  - change 32 (a native `it`), change 34 (the Italian renderer) and change 35 (the Italian edition's
    rules);
  - change 36 (en-it, which the golden runs beside);
  - change 35's heraldry-only rule for expressions, as change 36's pull request amended it.
- **Order**: before changes 48, 49's gloss columns and 50.
- **Effort, against 3.5–6 ideal days**: 3.5–6.25 (design, *Effort*).
