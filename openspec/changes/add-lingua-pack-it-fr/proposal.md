# add-lingua-pack-it-fr — Italian glossed in French, coverage committed

## Why

Change 16 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Changes 5
to 15 give Italian its analysis, its forms, readings and levels in `tables/it/`, and every rule its
pages need; it-fr still glosses nothing: `tables/it-fr/gloss.tsv` is empty, `tables/it/lexical.tsv`
holds the two acronym rows of change 11 alone, and the Italian baseline runs over a fixture of two
hundred forms. A French speaker cannot read Italian with Lingua.

The source is the French Wiktionary's Italian section — the richest any pair has read, the programme
measured (1,310,041 senses). Measured here on kaikki's extract of it (`kaikki-Italien.jsonl`,
1,098,335,839 B, 1,199,070 entries, 2026-10-22): 167,694 proper nouns, 958,832 entries that are only
an inflection of another (« Forme de nom commun », « Pluriel de giorno »), and **68,985 headwords
glossed** — 41,668 nouns, 13,278 adjectives, 12,838 verbs, 3,504 adverbs —, **10,903 of them
expressions** (a headword with a space). Against the commonest Italian lemmas — a prototype ranking,
wordfreq's Italian order with inflected forms folded into their lemma by the English Wiktionary's
links, as change 9 ranks them — the section alone glosses **87.7 % of the 5,000, 81.8 % of the
10,000 and 71.9 % of the 20,000** (en-fr's committed coverage: 95.1 / 90.1 / 78.9). What it leaves
out at the top is mostly what change 9 does not rank at all — single letters and elision stems (`l`,
`d`, `c`), elided compounds (`all'interno`, `quest'anno`), forms carrying clitics (`farlo`) — then
names (`roma`, `napoli`), English loans (`web`, `club`, `team`, `app`) and a few inflections the
prototype ranked as lemmas.

Two of the section's defects bear on common words:
- **A gloss read as a link.** kaikki files 43 senses as forms of a capitalised French word: `del`
  « Du. » as a form of « Du », `al` « Au. », `ad` « À, dans sa forme euphonique », `c’è` « Il y a »,
  `coloro` « Ceux », `altri` « Autre … ». Read as links, `del` — an A1 word, a lemma of change 9's
  tables — has no gloss.
- **The studied word as its own gloss**: 243 of the glossed 20,000 are glossed only by the word
  itself — `novembre`, `classe`, `festival`, `tunnel`, `tu`, `virus` —, read one by one: cognates,
  right as they are, unlike fr-es's `et` « Et » (matrix change 49's D7).

## What Changes

- **it-fr's native side, from the French Wiktionary** (design D1, D2): `reduce-it-fr.py`, which
  writes Italian's studied side since change 9, writes `tables/it-fr/` too — `gloss.tsv`,
  `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json`, `README.md` — through the native
  side every pair shares (`reduce_common.native_tables`) with the French Wiktionary's rules
  (`reduce_edition_fr.FR`), as es-fr does: the section's definitions first; then the French
  Wiktionary's translation tables, read backwards (a French entry listing an Italian word glosses
  that word with the French headword); then the Italian Wiktionary's French translations — at most
  three words per part of speech from a table. Every gloss is French written by a person (M5).
- **Sources from the editions' dumps** (D3): `EDITIONS["fr"]` gains `kaikki-fr-Italien.jsonl` and
  `kaikki-fr-traductions-it.jsonl`; `EDITIONS["it"]`, the Italian Wiktionary (43,849,715 B gzipped,
  regenerated 2026-10-21), gains `kaikki-it-traductions-fr.jsonl`; `DUMPS["it-fr"]` names them beside
  change 9's English section. The update publishes them under it-fr's release, as change 9's did.
- **A gloss read as a link is a gloss** (D4): a sense kaikki tags as a form of a capitalised word
  that no Italian entry heads, and whose text the French edition's form-of patterns do not read as
  an inflection (« Pluriel de… », « Première personne… »), glosses its headword: the lemmas `del` « Du », `della` « De la », `coloro` « Ceux, celles », and
  the expression `c’è` « Il y a », which change 12 keys `ci essere`; `al`, no word of change 9's
  tables, glosses nothing.
- **Italian's dictionary words** (D5): `tables/it/lexical.tsv` becomes it-fr's glossed lemmas less
  those the section glosses by a proper noun's senses alone (`dictionary_words`, change 24c's rule),
  plus change 11's acronym rows; the acronym rows glossed by change 11's D3 (`USA` through `stati
  uniti` and `america`; `NATO` unglossed); change 14's levels narrowed to the dictionary words.
- **Expressions** (D6): `mwe.tsv` holds the section's glossed headwords with a space — 10,903 before
  the shared filters — and the single words change 6's pre-pass splits that the section glosses
  (`d'accordo`, `all'improvviso`), keyed by change 12's rules at build.
- **M6's rule for it-fr, fixed here** (D7): `FLOORS["it-fr"] = (85.7, 79.8, 69.9)` — the prototype's
  figures less two points, the rule en-es's, fr-en's and fr-es's floors follow; task 0.1 re-runs the
  prototype on change 9's committed ranks before the dispatch, and the floor may only rise from it,
  never fall. Below it, the tables are not committed and no package will list it-fr.
- **Coverage committed and shown** (D8): coverage against the floor, the glossed lemmas by source,
  a sample of 100 glosses marked by source, the pack's size — in the tables' README, `SOURCES.md` and
  the programme's measured figures, stored in no pack.
- **The Italian baseline runs on the committed tables** (D9): `PackSource::Tables`, the golden
  re-blessed once in this change's pull request — change 5's scenario *The committed tables replace
  the fixture* —, the fixture kept for the tests that build it. No package lists it-fr (change 20).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *Italian is glossed in French from the French Wiktionary's Italian
  section and the translation tables* and *it-fr is committed at its studied tables' snapshot, held
  to a coverage floor, and the Italian baseline runs on it*. Change 11's acronym requirement and
  change 12's expression requirements are implemented here, not modified; both changes archive after
  this one (their `.openspec.yaml` gains it in this pull request, as their tasks ask). This change
  archives after changes 5, 9 and 13, after change 24c whose rule it reads, and after the fr-en and
  fr-es changes whose shape and floor rule it follows.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `scripts/lingua-data/reduce-it-fr.py` (the native side, D4–D6), `test_reduce_it_fr.py`;
  `pack_sources.py` (the editions' files, `DUMPS["it-fr"]`, `FLOORS["it-fr"]`), `test_pack_sources.py`;
  `SOURCES.md`.
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
