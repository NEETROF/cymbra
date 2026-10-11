# Measuring what new tables change for a reader

The update report (`pack_report.py`) says which rows of the tables change. This says what a reader
would see change: the extension's own engine reads the same corpus with a pack built from a base
ref's tables and with one built from the working tables, and compares every token — the lemma it
gets, whether that lemma is in the lexicon, whether a gloss is shown. Built for
`switch-lingua-inflections-to-esdb`; any change to the tables can be read the same way.

```bash
(cd apps/lingua-extension && yarn gen:wasm)      # the engine's wasm glue, once
scripts/lingua-data/measure/measure.sh origin/main en-fr
```

- `corpus.json` — the documents, by title only: 150 random and 10 recent Wikipedia articles, 20
  Wikinews articles (news) and 20 MDN guide pages at a fixed commit (technical prose). The
  articles are fetched as they are when the measurement runs, so figures move a little with them.
- `fetch_corpus.py` — their text, into `work/measure/corpus.json` (git-ignored, fetched once).
- `compare_packs.mjs` — the comparison, per sample and in total; the top lemma changes, gains and
  losses go to `work/measure/report.json`.

A token counts as *resolved* when its lemma is in the pack's lexicon, and *glossed* when the engine
shows a gloss for it. Proper nouns outside both lexicons are left out.

## Spanish forms on a treebank

`es-pud.sh` measures the es-fr pack, built from the committed tables, against UD Spanish-PUD with
lingua-core's real analyser (`lingua-pack-measure`, add-lingua-spanish-forms-tables D5). It fails
under the programme's gates:
- 98.5 % of words resolved;
- 93.5 % of content words taking the treebank's lemma;
- 97 % of auxiliaries taking it.

PUD is fetched at a pinned commit and checked by sha256. It is never committed, and the reduction
never reads it.

```bash
scripts/lingua-data/measure/es-pud.sh
```

## French forms on two treebanks

`fr-ud.sh` measures the fr-en pack, built from the committed tables, with the same harness
(add-lingua-french-forms-tables D9): on UD French-PUD, held to Spanish's gates above (its exit
status the script's), and on UD French-GSD's test section, reported beside it and not gated — the
reduction reads GSD's training and development sections, so a held-out section of the same treebank
is the weaker test. Both are fetched at a pinned commit and checked by sha256 into
`work/measure-fr/`. Neither is committed, and the reduction reads neither.

After the gates it reports French's readings (`fr_readings.py`, add-lingua-french-grammar-tables
D10) on both treebanks: per part of speech, over the words whose form the committed tables map to the
treebank's lemma, the share that carry a reading of their own and the share of those whose treebank
part of speech and features are among them — an auxiliary read as a verb, the conditional's and the
imperative's tense left aside, a participle without a tense read either way. The figures decide
nothing.

```bash
scripts/lingua-data/measure/fr-ud.sh
```
