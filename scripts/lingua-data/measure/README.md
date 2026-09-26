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
