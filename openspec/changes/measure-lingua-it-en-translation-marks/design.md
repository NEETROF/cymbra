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
- The precedents of a direct route judged in English are es-en (matrix change 26: 94 of 96 shown
  marks correct, 4 % withheld) and fr-en (change 50: 95 of 96, 4 % withheld).

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
corpus steps and anchored on verdicts already committed, added before the run; they move no committed
verdict, as change 50's D6 said of French's:
- **a subject the model adds**: step 25 « Ha detto all'Andrew Marr Show »: a mark on « told », « said »
  or « He told » is correct, on « He » alone it is not — es-en's one other wrong mark, k 25, marked
  « he » for « said »; es-fr's k 25 « il a déclaré » was judged correct for carrying the subject the
  rendering adds. The passato remoto is one word in English as in Italian (es-en: k 65 « met », k 69
  « released », k 81 « accused »), so change 19's compound-past case does not arise; the reverse does —
  an Italian compound past's participle (steps 5, 13, 17, 25, 31) often renders as one English simple
  past: a mark on that word is correct, on « has » or « were » alone it is not;
- **a word after a fused preposition**: step 6's « Nel corso della storia » (also 22, 82, 91): a mark
  on « course » or « In the course » is correct, on « In » alone it is not; an expression's rendering
  (« Throughout ») counts;
- **a word after an elided piece**: step 86's « l'unico nel Governo »: a mark on « only », « only
  one » or « the only (one) » is correct — the article comes from « l' » (es-en k 86 « the only one »)
  —, on « the » alone it is not;
- **a clitic pronoun beside the selected verb** (steps 9, 31, 33, 97; none reflexive): 33 « Gli viene
  riconosciuto » « He is credited »: « credited » or « is credited » is correct, « He » alone is not
  (es-en k 33 marked « credited »); 31 « l'ho scelta » « I chose it »: « chose » is correct, « it »,
  « her » or « I » alone is not; 97 « si incontrano » « met »: « each other » alone is not; 9 « ci
  fossero » « there were »: « were » or « be » is correct, « there » alone is not;
- **one Italian word, an English compound**: step 54 « Il capodanno cinese » « Chinese New Year » and
  step 78 « sveglia » « alarm clock »: the whole compound is correct; « Year », « alarm » or « clock »
  alone is part of a compound, which the criteria count wrong — es-en's k 54 (« Year » judged correct)
  does not carry over, its Spanish selection being « Año » alone;
- **gender or number English does not mark, and a numeral beside the word**: step 79 « le due
  fazioni nemiche » « the two enemy factions »: a mark on « enemy » (or « hostile », « rival ») is
  correct, on « two » it is not, as es-en's k 79 was judged; step 71 « stereotipata » « stereotypical »
  or « stereotyped ».

`judged-it-en.tsv` committed; if change 22's table fills the gloss experiment's columns, the judge
judges them too, as es-en's were (75 / 78).

### D3 — The list

On the first tier (≥ 90 % of shown marks correct, ≤ 25 % withheld), `MARKED_PAIRS` gains `it-en`,
inert until change 25 lists the pair in `packs.json`, and the exact lists of
`test/translate-marks.spec.ts` and `test/translate-relay.spec.ts` follow. Short of it, it-en stays out
and is translated without a mark; under 75 % correct or over 30 % withheld, the figures go to the
owner under M15, who settles at change 25 whether it-en's translation is offered unmarked or not at
all. Either way `judged-it-en.tsv` is committed, which the structural test reads as change 19 left
it. This change decides nothing for the reader.

### D4 — Tests and documents

- **A test for the new scenario**, whichever the outcome: `test/translate-marks.spec.ts` « An Italian
  selection, judged in English » holds it-en's committed totals, its tier verdict against whether it is
  listed, and that `results("it-en")` has `results("it-fr")`'s ids; on the tier,
  `test/translate-relay.spec.ts` gains a case marking an Italian selection through it-en, as it has
  fr-en's.
- `tool/marks/README.md`: the introduction's list of measured pairs, the file list, it-en's figures,
  the Judging paragraph (Italian judged in English as in French), and the gloss experiment's row —
  « left empty: no `tables/it-en/gloss.tsv` yet (change 22) » if the table is missing at the run.
- `TRANSLATION.md`: the sentence naming the measured and listed pairs, it-en's figures beside its soak,
  and the run's traps.
- The comment above `MARKED_PAIRS`: the changes that measured the list, and it-en, if listed, inert
  until change 25, `enable-lingua-italian-for-english-speakers`.

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
