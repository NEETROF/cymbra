# add-lingua-pack-it-fr — Italian glossed in French, coverage committed

## Why

Change 16 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Changes 5
to 15 give Italian its analysis, its forms, readings and levels in `tables/it/`, and every rule its
pages need; it-fr still glosses nothing: `tables/it-fr/gloss.tsv` is empty, `tables/it/lexical.tsv`
holds the two acronym rows of change 11 alone, and the Italian baseline runs over a fixture of two
hundred forms. A French speaker cannot read Italian with Lingua.

The source is the French Wiktionary's Italian section — the richest any pair has read, the programme
measured (1,310,041 senses). Measured here on kaikki's extract of it (1,199,070 entries; the file the
pipeline derives is 769,483,152 B): 167,694 proper-noun entries, 958,832 entries that are only an
inflection of another (« Forme de nom commun », « Pluriel de giorno »), and **68,985 headwords
glossed** (71,288 entries: 41,668 nouns, 13,278 adjectives, 12,838 verbs, 3,504 adverbs…), **10,880 of
them expressions** (a headword with a space; 10,704 after the shared filters). The Italian
Wiktionary adds 14,410 French translations of 10,086 Italian entries, the French Wiktionary's tables
62,689 Italian words. Against the commonest Italian lemmas, re-measured with the real shared rules on
a ranking like change 9's: **87.5 % of the 5,000, 79.6 % of the 10,000 and 70.5 % of the 20,000** from
the definitions alone, **89.2 / 82.2 / 73.4** with both fallbacks (en-fr's committed coverage: 95.1 /
90.1 / 78.9) — and 80.3 / 72.3 / 63.3 (82.7 / 75.0 / 66.3) if change 9 gives every word that is also
another lemma's form to the other lemma (design D7). The floor is therefore fixed on change 9's
committed ranks, before the first measurement.

Three of the section's defects bear on common words:
- **A gloss read as a link.** kaikki files some French definitions as links to a French word: `del`
  « Du. » as a form of « Du », `c’è` « Il y a », `coloro` « Celles et ceux » — 34 senses with a
  capitalised target, more with a lowercase one (`degli`, `delle`, `dello`, `li` « …: les »).
- **Names and acronyms gloss common words.** A capitalised entry merges into the lowercase lemma: 102
  of the 20,000 commonest are glossed by a proper noun or an acronym alone although the section has a
  lowercase entry — `te` « Province de Térame », `so`, `li`, `soldi` « Nom de famille ».
- **The studied word as its own gloss**: 243 of the glossed 20,000 are glossed only by the word
  itself — `novembre`, `classe`, `festival`, `tunnel`, `tu`, `virus` —, read one by one: cognates,
  right as they are, unlike fr-es's `et` « Et » (matrix change 49's D7).

## What Changes

- **it-fr's native side, from the French Wiktionary** (design D1, D2): `reduce-it-fr.py`, which
  writes Italian's studied side since change 9, writes `tables/it-fr/`'s glosses, sense runs,
  expressions, `NOTICE` and `manifest.json` (`record-build` writes the pin, a person the README)
  through the native side every pair shares (`reduce_common.native_tables`) with the French
  Wiktionary's rules (`reduce_edition_fr.FR`), in es-fr's order: the section's definitions first;
  then the Italian Wiktionary's French translations, in the table's order; then the French
  Wiktionary's translation tables read backwards — at most three words per part of speech from a
  table; a typographic apostrophe read as `'`. Every gloss is French written by a person (M5).
- **Sources from the editions' dumps** (D3): `EDITIONS["fr"]` gains `kaikki-fr-Italien.jsonl` and
  `kaikki-fr-traductions-it.jsonl`; `EDITIONS["it"]`, the Italian Wiktionary (43,849,715 B gzipped,
  regenerated 2026-10-21), gains `kaikki-it-traductions-fr.jsonl`; `DUMPS["it-fr"]` names them beside
  change 9's English section. Every French-dump derivation then writes about 0.77 GB more.
- **A gloss read as a link is a gloss** (D4): a sense kaikki tags as a form of a word no Italian entry
  heads, by its exact spelling, whose text the French edition's form-of pattern (`FR.form_of`) does
  not read as an inflection, glosses its headword: `del` (« Du. »), `degli`, `delle`, `dello`,
  `coloro` « Celles et ceux », and the expression `c’è` « Il y a » — a lowercase target only where
  the headword has no other glossed sense, so `pelle`, `collo` and `colle` keep theirs; `della`
  (linked to the Italian `la`) takes the translation tables' gloss.
- **Name notes and acronyms keep off common words** (D5): an acronym's sense, and a proper noun's
  sense that only says surname, first name or hamlet, gloss no lemma the section gives a lowercase
  entry of its own, as fr-es's pass keeps a name's note off `pierre`; any other proper noun's sense
  stays (`luna` « Lune, satellite de la Terre », `cristo` « Christ »).
- **Italian's dictionary words, the acronym rows, the levels** (D5): `tables/it/lexical.tsv` becomes
  the dictionary words of *Italian's dictionary words leave its names out* plus change 11's rows; the
  rows glossed by change 11's rule, a synonym's gloss read from its entry whatever its part of speech
  (`USA` joins `america`'s first sense and `stati uniti`'s « États-Unis », a proper-noun entry); change 14's levels narrowed to them.
- **Expressions** (D6): `mwe.tsv` holds the section's glossed headwords with a space, then those the
  translation tables gloss (7,411), then the single words change 6's pre-pass splits, from the
  section (`all'improvviso`, `c’è`) and from the tables (`d'accordo`, `senz'altro`),
  each keyed at build by change 12's rules; change 12's figures re-measured on it.
- **M6's rule for it-fr** (D7): task 0.1 re-runs the study on change 9's committed ranks with both
  fallbacks and fixes `FLOORS["it-fr"]` (in `gloss_coverage.py`) at those figures less two points, up
  or down, settled by the owner before the first committed measurement and written into the
  requirement; never lowered after it. Below it, the tables are not committed and no package will
  list it-fr.
- **Coverage committed and shown** (D8): coverage against the floor, the glossed lemmas by source,
  a sample of 100 glosses marked by source, the pack's size — in the tables' README, `SOURCES.md` and
  the programme's measured figures, stored in no pack.
- **The Italian baseline runs on the committed tables** (D9): `PackSource::Tables`, the golden
  re-blessed once in this change's pull request — change 5's scenario *The committed tables replace
  the fixture*, its trigger read as « until it-fr's glosses are committed » —, the fixture kept for
  the tests that build it. No package lists it-fr (change 20).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *Italian is glossed in French from the French Wiktionary's Italian
  section and the translation tables* and *it-fr is committed at its studied tables' snapshot, held
  to a coverage floor, and the Italian baseline runs on it*. Change 11's acronym requirement and
  change 12's expression requirements are implemented here, not modified; change 12 archives after
  this one (its `.openspec.yaml` gains it in this pull request, as its task asks), and change 11,
  which first did too, archives before it since 2026-10-25 (that edge closed a cycle with change 14:
  change 11's design, *The archive order*). This change
  archives after changes 5 and 9, after change 24c whose rule it reads, and after the fr-en and
  fr-es changes whose shape and floor rule it follows. Change 14 is in its `archiveAfter` (its requirement is named here); change 13 is not named.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `scripts/lingua-data/reduce-it-fr.py` (the native side, D4–D6), `test_reduce_it_fr.py`;
  `pack_sources.py` (the editions' files, `DUMPS["it-fr"]`), `test_pack_sources.py`;
  `gloss_coverage.py` (`FLOORS["it-fr"]`), `test_gloss_coverage.py`; `SOURCES.md`;
  `.github/workflows/lingua-extension-check.yml` (the floor step runs it-fr).
- `scripts/lingua-data/tables/it-fr/{gloss.tsv, senses.tsv, mwe.tsv, NOTICE, manifest.json,
  pin.json, README.md}`, `tables/it/{lexical.tsv, level.tsv}` (the dictionary words, the levels
  narrowed); `docs/lingua/italian-programme.md` (the measured coverage).
- `crates/lingua-wasm/tests/{italian_baseline.rs, support/italian.rs}` and `baseline/it-fr.golden`
  (re-blessed once on the committed tables); `crates/lingua-pack/tests/committed_tables.rs`.
- `openspec/changes/{keep-lingua-italian-acronyms-and-names, add-lingua-italian-expression-keys}/
  .openspec.yaml`: this change added to their `archiveAfter`.
- Untouched: the six pairs' tables, pins and goldens; lingua-core; the extension's bundle and
  `packs.json`; the agent; the server; the site.
- **Effort, against 3–5 ideal days**: 3.5–5 (design, *Effort*).
