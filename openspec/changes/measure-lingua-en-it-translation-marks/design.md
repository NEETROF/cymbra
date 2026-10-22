# Design — measure-lingua-en-it-translation-marks

## Context

Read on main at `c4f4a8bd4`, where nothing Italian is implemented yet.

- **The corpus**: `apps/lingua-extension/tool/marks/corpus.json` holds 100 English selections from UD
  English-PUD (every tenth sentence, a noun, verb, noun, adjective in turn: 59 nouns, 23 verbs, 18
  adjectives), steps 0–99. Change 19 adds 100 Italian ones and pins UD Italian-PUD.
- **The precedents for English selections**:

  | Pair | Change | Model's soak | Judge | Figures | Withheld |
  |---|---|---|---|---|---|
  | en-fr | the Spanish programme's change 27 | none (shipped since the engine) | one judge, French | 96 of 97 correct, 3 % withheld | k 32, 63, 86 |
  | en-es | matrix change 26 | en-es 2.1 soaked, isolated and in one instance, 0 traps | one judge, Spanish | 96 of 96 correct, 4 % withheld | k 63, 78, 86, 93 |

  en-fr's one wrong mark is k 57, « qui » for « constituent ». en-es's doubtful lines, counted correct,
  are k 18, 70 and 77. Every judgment on record is « one judge », in practice an agent; the owner may
  re-judge any line in the pull request (`tool/marks/README.md`).
- **The criteria** (`tool/marks/README.md`):
  - correct: the marks cover the rendering of the selected word, possibly with its article,
    preposition or auxiliary, possibly split; an expression's rendering counts;
  - wrong: another word, only a function word, or part of a compound;
  - withheld: no mark, a sentence that traps twice included;
  - the tier: `firstTier` in `tool/marks/tier.mjs`.

  Change 50 added two habits: a second run checked byte for byte, and the doubtful lines named, the
  test holding the tier with them counted wrong.
- **en-it's model**: `en-it/base-memory/2.1` (change 2), one model on a direct route, about 36.5 MB
  decompressed. Its vocabulary is shared with it-en 2.0. Change 2 soaks it over the English corpus,
  isolated. The 2.1 family is where en-es trapped, and the harness counts a sentence that traps twice
  as withheld.
- **The harness refuses Italian today**: `measure_marks.mjs` calls `stopWords(native)` unconditionally,
  and `stop-words.mjs` holds French, English and Spanish, throwing for any other language.

## Goals / Non-Goals

**Goals:**
- en-it's marks measured on the committed English selections and judged in Italian, honestly, with no
  Italian reader on the team; listed if on the first tier.

**Non-Goals:**
- Deciding what happens below the tier: the owner, under M15, at change 45.
- fr-it's and es-it's marks (changes 49, 53), which reuse the Italian function words.
- Moving a mark onto a word with a long elided article (D6).

## Decisions

### D1 — Italian's function words

`ITALIAN` joins `stop-words.mjs` and `stop-words.d.mts`, and the header comment and the README's
« (fr, en, es) » follow. The gloss experiment splits text on any non-letter (`measure.mjs`), so
« dell'acqua » becomes « dell » and « acqua ». The set therefore holds the elided pieces too: « dell »,
« all », « dall », « nell », « sull », « quest », « quell ».

`test/translate-measure.spec.ts`'s table check gains `it`, reading `tables/it-*/senses.tsv` and
`tables/it/forms.tsv`, where those pieces are written with their apostrophe (`dell'`). Its lookup tries
the word, then the word with an apostrophe, so that no piece reads as « unknown ». The set touches the
experiment only: words under three letters are filtered, and the tier reads the engine's column alone.

### D2 — en-it measured

`node tool/assemble_model_site.mjs <dir>` pulls the en-it files from Mozilla's registry and checks
every digest. `measure_marks.mjs --pair en-it --models <dir>` then runs the 100 English selections:
- the sentence and the fragment each asked as the extension asks them;
- a trap answered as change 26 answers it;
- run twice, and the two `results-en-it.jsonl` byte for byte the same before the first is committed.

The ids match `results-en-fr.jsonl`'s and `results-en-es.jsonl`'s. Change 2's isolated soak is a
prerequisite, and a one-instance soak is recorded beside it for parity with en-es, fr-en, fr-es and
it-en (about 5 seconds). The gloss experiment's columns read `tables/en-it/gloss.tsv` with
`tables/en/forms.tsv` (change 36) when committed, and stay empty otherwise. Italian inflection fits the
experiment's stem rule (« pubblicate » / « pubblicare »).

### D3 — Judged in Italian, by two agent judges

Nobody on the team reads Italian (I2). So two agent judges judge, and the judgment is made checkable:
1. **The criteria first.** The Italian examples below are committed to `tool/marks/README.md` before
   `results-en-it.jsonl`, so that git's order shows it, as change 26 did.
2. **Three references per line**: the engine's sentence; UD Italian-PUD's human Italian (pinned by
   change 19, fetched, never committed); and en-fr's and en-es's committed verdicts on the same step. A
   verdict departing from both precedents gives its reason.
3. **Two judges**:
   - Judge A is the implementing agent, as in every precedent;
   - Judge B is change 33's Italian reviewer agent, working blind: it gets the results and the criteria,
     not A's verdicts.

   A line they disagree on is committed as doubtful (`doubtful: A …/B …`). The README reports how many
   lines they agreed on.
4. **The tier must hold with the doubtful lines counted wrong**, as change 50's test holds it.
5. **Every judgment committed.** The owner, who reads French and Spanish, can audit most lines against
   the en-fr and en-es anchors.

The Italian examples, PUD's human Italian quoted, anchored on en-fr's and en-es's verdicts:

| Case | Examples (step: correct; wrong) | Anchors |
|---|---|---|
| Compound past and passive: the auxiliary may be marked with the participle, never alone | 17 « ha suggerito », 25 « Ha detto », 30 « è morto » / « morì », 37 « ho visto », 65 « ha incontrato », 69 « ha pubblicato », 81 « ha accusato »; passives 13 « sono state pubblicate », 33 « è accreditato » / « Gli viene riconosciuto », 61 « è considerato »; wrong: « ha », « è », « sono state » or « viene » alone | en-fr 17 « a suggéré », 30 « est mort », 65 « a rencontré » with the auxiliary; en-fr 13 « diffusées », en-es 13 « publicaron » without |
| A dropped subject, a clitic | 9 « Vorrebbe » or « (Gli) piacerebbe »; wrong: « Gli » alone, an added « Egli » or « Lui » alone; 97 « si sono incontrati » with or without « si », never « si » alone | en-fr « aimerait », en-es « gustaría »; es-en k 25 « he » judged wrong |
| A phrasal verb or compound rendered by one Italian word | 57 « costituiscono » or « comprendono », wrong « che » alone; 77 « brancola » or « procede a tentoni », a loose rendering doubtful; 78 « sveglia »; 54 « Capodanno » (« Nuovo » if « Anno Nuovo », never « Anno » alone); 3 « sudcoreana » or « (della) Corea », never « Sud » alone; 94 « avamposti »; 75 « in grado », « riusciti » or « potuto », never « furono » alone | en-fr's one wrong mark, k 57 « qui »; en-es « conforman »; en-fr 78 « réveil » |
| An article or articulated preposition, carried but never alone | 6 « della storia » (wrong « della » alone), 66 « all'Università », 0 « della transizione », 12 « di emoglobina », 26 « di risarcimento », 23 « dei mercati », 10 « I medici », 52 « in battaglia », 59 « L'Esercito », 5 « quest'area », 62 « dell'origine » | en-fr « de l'histoire », « l'Université », « d'hémoglobine »; en-es « de compensación », « los médicos » |
| A noun modifier, an -ing form | 22 « delle armi », « armi » or « del tiro », wrong « lobby », « dei cacciatori » doubtful; 83 « di successo », « tormentone » or « hit », wrong « canzone »; 1 « (di) insegnare » or « insegnando », wrong « Invece di » alone | en-fr « de tir », en-es « de tiro »; en-fr « d'enseigner », en-es « enseñar » |
| A possessive | 24 « fermata » or « La mia fermata », wrong « La mia » alone, a mark over « prossima » doubtful; 58 « (i suoi) nemici », wrong « suoi » alone; 91 « ultimi », wrong « (dei) suoi » alone | fr-en k 92; fr-en k 15 « their » |
| Gender and number never decide a verdict | 93 « capitanata », 71 « stereotipata », 39, 15, 47, 67, 91, and 96 even rendered singular | en-fr « commandé », « stéréotypée »; en-es « estereotipada » |
| Expressions and loanwords | 79 « (il) momento » or « ora », wrong « Per » alone; 99 « D'altra », « altra » or « Dall'altro », wrong « parte »; 32 « punto », « epicentro » or « Ground », wrong « zero »; 27 « Premium » and 82 count as renderings; 86 « l'unico » and 63 « ufficiale » correct, though both precedents withheld these steps, so a mark there is a new verdict with its reason | en-fr « D'autre »; en-fr's and en-es's k 32, 63, 86 withheld |
| Loose renderings | 18, 70, 77: correct, but doubtful | en-es's doubtful lines |

### D4 — The list

- **On the first tier**, with the doubtful lines counted wrong: `MARKED_PAIRS` gains `en-it`, inert
  until change 45 lists the pair. The exact lists of `test/translate-marks.spec.ts` and
  `test/translate-relay.spec.ts` follow, with a relay case on fr-en's model.
- **Short of it but at least 75 % correct with at most 30 % withheld**: not listed, so translated
  without a mark. Change 19's structural test reads the list as the judged pairs on the tier, so it
  holds.
- **Under 75 % or over 30 % withheld**: the same, and the figures go to the owner, who settles under M15
  at change 45 whether en-it's translation is offered unmarked or not at all. This change decides
  nothing.

### D5 — Tests and documents

Whichever the outcome, `test/translate-marks.spec.ts` gains *An English selection, judged in Italian*:
- en-it's totals;
- its tier against whether it is listed;
- its ids against en-fr's results;
- its native language Italian;
- the doubtful lines, and the tier with them counted wrong.

Every other pair's results and judgments stay byte for byte, checked by a diff that leaves en-it's
files out and prints nothing.

The documents follow:
- `tool/marks/README.md`: introduction, file table, run command, judging paragraph, results row,
  experiment row;
- `TRANSLATION.md`: the measured and listed pairs, en-it's figures beside change 2's soak;
- `measure_marks.mjs`'s usage comment and the comment above `MARKED_PAIRS`;
- programme row 38.

### D6 — Not here

`reconcile.ts` strips only one- or two-letter elided pieces, so a mark cannot move onto the word of
« dell'X », « all'X » or « quest'X ». The marks are measured as shipped. This is listed for change 45's
dogfood.

## Risks / Trade-offs

- **Two agents judging a language nobody on the team reads** → criteria first, three references, the
  blind second judge, the tier held with disagreements counted wrong, every verdict committed.
- **A model that traps** → the harness's twice-trapped rule, the soaks recorded.

## Migration Plan

Nothing deploys and no reader sees a change before change 45. Rollback is a revert.

## Effort

1.35–1.85 ideal days, against the programme's 1–2:

| Part | Ideal days |
|---|---|
| The Italian function words and their test | 0.25 |
| The criteria, committed before the run | 0.25 |
| The run and its rerun | 0.1 |
| Judge A | 0.25–0.5 |
| Judge B, blind, and the reconciliation | 0.25–0.4 |
| The gloss experiment's columns | 0–0.1 |
| The list, tests and documents | 0.25 |
