# Design

## Context

See proposal.md — Why. Where things stand once changes 2 and 19 are implemented:

- `apps/lingua-extension/tool/marks/corpus.json` holds 100 Italian selections beside the English,
  Spanish and French ones (change 19's D1), pinned on UD Italian-PUD; `measure.mjs` runs the
  extension's marking through a pair's route; `stop-words.mjs` holds the native language's function
  words (en among them) for the gloss experiment; `tier.mjs` the first tier.
- The it-en model was soaked over the Italian selections, isolated and in one instance (change 19's
  D2), its figures in `TRANSLATION.md`.
- `test/translate-marks.spec.ts` reads the list as the judged pairs on the tier, every judged pair's
  results and judgments checked (change 19's D4).
- The precedent of a direct route judged in English is es-en (matrix change 26: 94 of 96 shown marks
  correct, 4 % withheld).

## Goals / Non-Goals

**Goals:**
- it-en's marks measured on the committed Italian selections through its one model and judged in
  English; listed if on the first tier.

**Non-Goals:**
- it-es (change 29), en-it, fr-it and es-it (stages 3 and 4).
- Offering it-en's translation (change 25), or deciding what happens below the tier (the owner under
  M15, at change 25).

## Decisions

### D1 — it-en measured through its one model

`measure.mjs` over the 100 Italian selections through the it-en model, the sentence and the fragment
each asked as the extension asks them, a trap answered as change 26 answers it;
`results-it-en.jsonl` committed. The gloss experiment's columns read `tables/it-en/gloss.tsv` if
change 22 has committed it by the run, and stay empty otherwise; the tier reads the engine's column
alone.

### D2 — Judged in English

One judge marks each shown mark correct or not against the English rendering of the selected Italian
word, by change 26's written criteria, with Italian examples in English taken from change 19's
corpus steps, added before the run:
- **a subject the model adds**: step 37's « A Pyongyang vidi alcune persone »: a mark on « saw » or
  « I saw » is correct, on « I » alone it is not; the passato remoto is one word in English as in
  Italian (« conobbe » « met »), so the compound past change 19 judged for French does not arise;
- **a word after a fused preposition**: step 6's « Nel corso della storia » (also 22, 82, 91): a mark
  on « course » or « In the course » is correct, on « In » alone it is not; an expression's rendering
  (« Throughout ») counts;
- **a word after an elided piece**: step 86's « l'unico nel Governo »: a mark on « only » or « only
  one » is correct, on « the » alone it is not;
- **gender or number English does not mark**: an English rendering that drops the Italian gender
  still counts as the word's rendering — step 79 « nemiche » « enemies » or « hostile », step 71
  « stereotipata » « stereotyped ».

`judged-it-en.tsv` committed.

### D3 — The list

On the first tier (≥ 90 % of shown marks correct, ≤ 25 % withheld), `MARKED_PAIRS` gains `it-en`,
inert until change 25 lists the pair in `packs.json`, and the exact lists of
`test/translate-marks.spec.ts` and `test/translate-relay.spec.ts` follow. Short of it, it-en stays out
and is translated without a mark; under 75 % correct or over 30 % withheld, the figures go to the
owner under M15, who settles at change 25 whether it-en's translation is offered unmarked or not at
all. Either way `judged-it-en.tsv` is committed, which the structural test reads as change 19 left
it. This change decides nothing for the reader.

### D4 — The documents

`tool/marks/README.md` (it-en's figures, the Judging paragraph: Italian judged in English as in
French), `TRANSLATION.md` (it-en's figures beside its soak) and the comment above `MARKED_PAIRS`
(it-en, if listed, inert until change 25, `enable-lingua-italian-for-english-speakers`).

## Risks / Trade-offs

- [One judge] → change 26's written criteria, Italian examples in English fixed before the run; every
  judgment committed and re-checkable.
- [The rule never selects a word inside a multiword token] → as change 19 says: it-en's figures say
  nothing of a selection on an enclitic verb or an articulated preposition; change 25's dogfood reads
  them.

## Migration Plan

Nothing deploys: the list is inert until change 25. Rollback is a revert.

## Effort

0.75–1.25 ideal days, against the programme's 1–2: the run and the judging 0.5–1; the documents and
the list 0.25. No corpus and no soak: change 19's.
