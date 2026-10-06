# The marks measurement

How often the translated sentence marks the right French words for the reader's selection, in
English and in Spanish (release-lingua-spanish-translation). Spanish goes through English, so its
mark crosses two alignments. Decision D2 of the Spanish programme fixed what the result decides
before anything was measured:

- **≥ 90 % correct and ≤ 25 % withheld:** marked;
- **75–90 %:** unmarked;
- **below that:** withdrawn.

| File                   | What it holds                                                                                                                 |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `select_corpus.mjs`    | The selection rule: every tenth PUD sentence, the same in both languages, one word each (noun, verb, noun, adjective in turn) |
| `pud.mjs`              | PUD English and Spanish, fetched at pinned commits and checked by sha256 (CC BY-SA, never committed)                          |
| `corpus.json`          | The 200 selections: sentence id, token, word, offsets                                                                         |
| `../measure_marks.mjs` | The harness: the pinned engine and the catalogue's models, each selection marked exactly as `relay.ts` marks it               |
| `results-<lang>.jsonl` | Per selection: the French sentence with its marks bracketed, the fragment's own translation, and the experiment's mark        |
| `judged-<lang>.tsv`    | Every mark judged correct, wrong or withheld, with a reason for each wrong one                                                |

## Running it again

```bash
node tool/assemble_model_site.mjs /tmp/models                                    # or read models.cymbra.app: omit --models
node --experimental-strip-types tool/measure_marks.mjs --models /tmp/models     # ~15 s, writes results-*.jsonl
node tool/marks/select_corpus.mjs                                               # only to rebuild corpus.json; deterministic
```

## Judging

The criteria were written before the run (design D3).

- **Correct:**
  - the marks cover the French rendering of the selected word in that sentence;
  - they may include the article, preposition or auxiliary it carries, and may be split;
  - an expression's rendering counts, as « sin embargo » → « Cependant ».
- **Wrong:** the marks cover another word, only a function word, or only part of a compound.
- **Withheld:** no mark.

The rates read D2 as the study did:

- correct is the share of the shown marks;
- withheld is the share of all 100 selections.

## Results (2026-10-05)

Engine pinned by `engine-pin.json`; models `en-fr` and `es-en` base-memory 2.0.

|                                      | Correct (of shown marks) | Withheld | D2                                   |
| ------------------------------------ | ------------------------ | -------- | ------------------------------------ |
| **Spanish, engine (pivot es→en→fr)** | **89 / 90 — 98.9 %**     | **10 %** | **First tier: marked**               |
| English, engine                      | 96 / 97 — 99.0 %         | 3 %      | (reference; the study found 83 / 87) |
| Spanish, gloss-located (experiment)  | 76 / 78 — 97.4 %         | 22 %     | —                                    |
| English, gloss-located (experiment)  | 76 / 79 — 96.2 %         | 21 %     | —                                    |

What these numbers say:

- **Spanish is on D2's first tier.** One wrong mark in 90 (« dos facciones enemigas » marked « deux »
  instead of « ennemies »).
- **It holds under a stricter reading:** if the five expression marks counted as wrong, Spanish would
  still score 84 / 90, 93 %.
- **Sampling error:** about ±6 % at 100 selections.

**The experiment** marks the first word of the sentence that matches the pack's French gloss of the
selected word.

- It is about as precise as the engine.
- It withholds about one selection in five: a translation often picks a synonym the gloss does not
  list.
- Where the engine withholds, it would have found 7 of Spanish's 10 missing marks, bringing Spanish
  to 3 % withheld.

A later change may try it as a fallback; nothing here ships it.
