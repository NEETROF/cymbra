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

## Italian forms on two treebanks

`it-ud.sh` measures the it-fr pack, built from the committed tables, with the same harness and
Italian's own analysis (add-lingua-italian-analysis D6, add-lingua-italian-forms-tables D8): on UD
Italian-PUD, held to Spanish's gates above (its exit status the script's), and on MarkIT's test
section, reported beside it and not gated — the reduction reads MarkIT's training and development
sections. Both are fetched at a pinned commit and checked by sha256 into `work/measure-it/`. Neither
is committed, and the reduction reads neither.

Italian's tables are committed by add-lingua-italian-forms-tables, the first update of it-fr writing
them; until then the script stops, saying so. `LINGUA_IT_PACK=<pack.lingua>` measures a pack built
elsewhere instead, as add-lingua-italian-analysis measured its cascade on a prototype of those
tables, and add-lingua-italian-forms-tables its tables reduced locally before the update.

```bash
scripts/lingua-data/measure/it-ud.sh
```

## Italian's detection guard

`it-guard.sh` measures the guard that keeps Italian's neighbours out of Italian
(add-lingua-italian-detection-guard, task 1.1). `it-guard-corpus.json` pins its samples:
- each neighbour's Wikipedia (Catalan, Galician, Venetian, Sicilian, Neapolitan, Sardinian, Corsican,
  Friulian, Latin, Occitan, Romanian), the articles an English seed list links to, by revision;
- the Italian corpora: UD Italian-PUD, UD Italian-MarkIT, a sample of Tatoeba's Italian export,
  Italian Wikipedia articles, *I promessi sposi* and *Pinocchio* from Project Gutenberg, and the
  baseline's `pages-it.txt`.

It fetches each into a directory of its own under DATA (`work/measure-it-guard/` by default), checked
by sha256, and then works offline. Two sources move upstream:
- Tatoeba regenerates its export weekly. Its sample is the 20,000 sentence ids committed in
  `it-guard-tatoeba-ids.txt` (drawn once, seed 42, from the export of 2026-10-17), so a later export
  is read with a warning, by id; Tatoeba may have edited or deleted some of those sentences since.
- Project Gutenberg offers no versioned URL. `files/N/N-0.txt` has not changed since 2024, but a
  re-posting would change its sha256, and the fetch would then refuse it.

lingua-core's own gate reads every block (`italian_guard_measure`, an example of the crate); a port
of the guard's reading explains each refusal, and must agree with the gate on every block read as
Italian. It prints:
- the share of each neighbour read as Italian, with and without the guard;
- every Italian block refused, with its markers;
- the recall gate, which is the exit status: at most one Italian block in a thousand, and none of
  `pages-it.txt`.

`it_guard.py words` measures D3's candidate words one by one on the same samples: the rules the
design's Measurement writes down, and the words they keep. Nothing fetched is committed.

```bash
scripts/lingua-data/measure/it-guard.sh
```
