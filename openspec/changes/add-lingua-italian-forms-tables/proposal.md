# add-lingua-italian-forms-tables — Italian's studied side, committed once: forms, frequencies, the pipeline (I11)

## Why

Change 9 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1, decision I11
(2026-10-21). Changes 5 to 8 give Italian its analysis over a hand-written fixture of two hundred
forms. A reader needs the language: every inflected form of every lemma an Italian page can
hold, mapped to one lemma (M8), with the frequency rank that drives the counts, the levels and
the cards. French's studied side (matrix change 43) was reduced once, by its reference pair, from
the English Wiktionary's French section and committed in `tables/fr/`, where every pair studying
French reads it (M24). Italian's is reduced the same way, by it-fr, its reference pair (I5), from
the English Wiktionary's Italian section — larger than its French one (719,680 senses; 589,335
distinct forms; 771 MB as kaikki's extract, against French's 584 MB), because Italian's verb
paradigms are listed form by form.

Two things differ from French and the owner settled them as I11. There is no licence-clean Italian
treebank of GSD's size: UD Italian-ISDT, VIT, ParTUT and PoSTWITA are non-commercial, and the two
clean ones are small — UD Italian-PUD's 1,000 sentences (CC BY-SA 3.0) and MarkIT's ≈ 1,300
(CC BY 4.0, students' essays of marked constructions). So the counts that choose a form's lemma
are thinner, and wordfreq's frequency decides more often. And Morph-it! (CC BY-SA 2.0 / LGPL,
504,906 entries over 34,968 lemmas, from a newspaper corpus) is a second source of forms: it is
read for what it adds that the Wiktionary lacks, and kept only if the measurement shows it adds
without moving the gates.

## What Changes

- **A reducer for Italian's reference pair** (`scripts/lingua-data/reduce-it-fr.py`), with
  Italian's studied rules in it, as fr-en's holds French's; no shared module is edited:
  - **forms** from the Italian section's tagged inflections and form-of links, a doubtful or
    alternative inflection left out, and a form of a form reaching the word it is a form of along
    one part of speech (`parlata` → `parlato` → *parlare*);
  - **I7, as change 6 writes it**: every word its pre-pass writes a form; the elided pieces forms
    too, by the reviewed closed table (`l'` → *lo*, `un'` → *una*, `quest'` → *questo*, …), for the
    treebanks that write them as words; no plain word that begins with one (`c'è`, `l'ho`, `dov'è`:
    the pre-pass splits them); the `di` family (`del`, `dello`, `della`, `dei`, `degli`, `delle`)
    lemmas of their own, `dell'` a form of `dello`; the other articulated prepositions neither
    forms nor ranks; the truncations (`po'`, `va'`) forms of the words they stand for; a verb form
    carrying clitics (`dirmelo`, `vattene`) left out — the enclitic rule reads them;
  - **a spelling variant** — a form the dictionary marks as a misspelling or an accent-less
    spelling (`perche`, `citta`) — left out here (change 10 reads them by its reviewed list);
  - **one lemma per form**: a reviewed override list (empty), then the counts of UD Italian-PUD
    and MarkIT's training section, then the form's own entry, then the lemma's frequency, then
    alphabetical order; a form that is a name and a commoner word keeps the commoner reading
    (`roma` stays the city, `marco` the name against the coin by frequency — measured);
  - **Morph-it!'s forms**, read after the Wiktionary's: a form Morph-it! lists under a lemma the
    Wiktionary ranks, which the Wiktionary lists under no lemma, joins the table; a form the
    Wiktionary already reads is never moved (D3);
  - **the 60,000 commonest lemmas** by wordfreq `it` (its large list), wordfreq's elision stems
    (`l`, `d`, `un`, `dell`, `nell`, `all`, `quest`, …) no words, a hyphenated word ranked when a
    treebank attests it, at the lower of wordfreq's estimate and the treebank's own frequency; no
    word whose every form reads as another; their forms attested in wordfreq.
- **Committed tables.** `tables/it/`: `forms.tsv`, `freq.tsv` (60,000 rows), `studied.json` naming
  it-fr, and the two files every studied folder holds, empty until the changes that fill them:
  `tags.tsv` (change 13) and `lexical.tsv` (change 16). `tables/it-fr/`: an empty `gloss.tsv` (the
  glosses are change 16), `NOTICE`, `manifest.json` (Italian's analyser version, read from
  lingua-core), `pin.json` and `README.md`.
- **Pinned sources.** `kaikki-Italian.jsonl`, derived from the English edition's dump and published
  under it-fr's release; UD Italian-PUD and MarkIT at a commit (`PINNED`), read for their counts
  only; Morph-it! 0.4.7 at its sha256; wordfreq 3.1.1.
- **A measurement** (`scripts/lingua-data/measure/it-ud.sh`): the real analyser over UD
  Italian-PUD, held to Spanish's gates (98.5 / 93.5 / 97), and over MarkIT's test section,
  reported; both fetched at a pinned commit, checked by sha256, never committed, never read by the
  reduction — the counts come from MarkIT's training section alone, so the test section is held
  out.
- **The pipeline knows it-fr**: `build.sh` keeps 60,000 lemmas for an Italian pair;
  `lingua-pack-update` offers it-fr and runs one Italian dispatch at a time; the monthly check and
  the reduce job take it in through `pairs`; the extension check builds its pack from the committed
  tables against its pin; the Italian baseline moves from the fixture to the committed tables
  (change 5's *The committed tables replace the fixture*), re-blessed once. `packs.json` is
  unchanged: no package carries it-fr.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *Italian's forms and frequencies* and *Italian forms are measured on
  held-out treebanks*. No requirement is modified, and none of `lingua-analysis`: this change reads
  change 38's catalogue, change 5's Italian and change 6's pre-pass as they define them, and
  archives after the three (`archiveAfter`).

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `scripts/lingua-data/reduce-it-fr.py` (new), `reduce_italian_treebank.py` (new: PUD's and
  MarkIT's counts and parts of speech), `pack_sources.py` (the Italian dump, the two treebanks and
  Morph-it! registered as pinned sources, the `it` group), `build.sh` (60,000 lemmas for `it-*`),
  `measure/it-ud.sh` (new), `SOURCES.md` (an *IT → FR: Italian studied* section), their tests.
- `scripts/lingua-data/tables/it/` and `tables/it-fr/` (new, committed by the reduction's pull
  request as the update workflow proposes them).
- `.github/workflows/lingua-pack-update.yml` (it-fr in `pair`, the `it` group),
  `lingua-extension-check.yml` (it-fr built and checked like the six).
- `crates/lingua-wasm/tests/{italian_baseline.rs, support/italian.rs}` (the pack from the tables,
  the golden re-blessed once), `crates/lingua-pack/tests/committed_tables.rs` (Italian's tables
  held like French's: the elided pieces' words present, every rank on its own lemma, the pin's
  studied record).
- Untouched: the six pairs' tables, pins and goldens; the extension's bundle; the agent; the
  server; the site.
