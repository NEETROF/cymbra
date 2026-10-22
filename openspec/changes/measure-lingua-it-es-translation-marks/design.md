# Design — measure-lingua-it-es-translation-marks

## Context

Read on main at `eca06e584`, where nothing Italian is implemented yet. Change 19's it-fr verdicts do not
exist yet. The criteria below anchor on change 19's and change 24's drafted criteria, and on the
committed en-es and fr-es verdicts.

- **The selections**: change 19's 100 Italian ones, chosen by the corpus's rule from UD Italian-PUD
  (57 nouns, 24 verbs, 19 adjectives). On 34 steps they select a different word than the English
  selections do: 0, 3, 4, 5, 6, 7, 8, 9, 12, 15, 18, 19, 22, 26, 27, 29, 30, 31, 42, 46, 48, 58, 59, 61,
  62, 63, 66, 74, 75, 79, 80, 83, 94 and 99. en-es's and fr-es's verdicts are references, never copied:
  a reference holds only where the Italian word means what the English or French one does. Even outside
  the 34, some Italian words mean something else (2 « volta », 32 « punto di partenza », 54 « capodanno »,
  70 « sceneggiatore », 82 « agli sgoccioli », 98 « sospensione »); en-es's k 54 « Nuevo » alone, judged
  correct, does not carry over. UD Spanish-PUD is translated from the English original for most steps
  (for some of 75–99 the original is Spanish or Italian), so it shows Spanish idiom but not what the
  Italian word becomes.
- **The precedents for a pivot or a Spanish judgment:**

  | Pair | Route | Judged in | Result |
  |---|---|---|---|
  | es-fr | es-en → en-fr | French | 89 of 90 correct, 10 % withheld; k 79 « deux » wrong, carried from es-en's « two » |
  | fr-es | fr-en → en-es | Spanish | 90 of 91 correct, 9 % withheld; k 30 « la » wrong; k 25 doubtful |
  | en-es | direct | Spanish | 96 of 96 correct, 4 % withheld; k 18, 70, 77 doubtful |

  fr-es is the closest precedent: the same last model and the same judging language. Its k 15 was
  repaired by the fragment's reconciliation. Its pivot added
  about five withheld marks over fr-en's. Every judgment on record is « one judge », in practice the
  implementing agent; the owner was offered a re-judge and never used it.
- **The route**: change 2 sets `"it-es": [it-en 2.0, en-es 2.1]`. The harness reads the native from the
  last model and refuses a pair the catalogue does not route. The models come from
  `assemble_model_site.mjs`; no host redeploy is needed to measure.
- **The soaks**: change 19 soaks it-en and it-fr over the Italian selections. en-es was soaked by matrix
  change 25, and again as fr-es's second leg. No one soaks it-es's chain, it-en's English fed to en-es.

## Goals / Non-Goals

**Goals:**
- it-es's marks measured on the committed Italian selections through its two models and judged in
  Spanish; listed if on the first tier.

**Non-Goals:**
- Offering it-es's translation, or deciding what happens below the tier: the owner, under M15, at
  change 30.
- A second judge by default: the judged language is one the owner reads (D3).

## Decisions

### D1 — The chain soaked

`soak_engine.mjs --pair it-es` over the Italian selections, isolated and in one instance (about 75 and 9
seconds by fr-es's figures). The figures are recorded in `TRANSLATION.md` for change 30 to read.

### D2 — it-es measured

`node tool/assemble_model_site.mjs <dir>`, then `measure_marks.mjs --pair it-es --models <dir>` over the
100 Italian selections:
- the sentence and the fragment each asked as the extension asks them;
- a trap answered as change 26 answers it;
- run twice, the results byte for byte the same before the first is committed.

The ids match `results-it-en.jsonl`'s and `results-it-fr.jsonl`'s. The gloss experiment's columns read
`tables/it-es/gloss.tsv` with `tables/it/forms.tsv` when committed (change 27, or change 28's refined
table, whichever is on main at the run, its commit recorded). The judge judges them; they never gate.

### D3 — Judged in Spanish, by one judge

Change 38 needed two judges because nobody on the team reads Italian, the judged language there (I13).
Here the judged language is Spanish, which the owner reads. The risk is misreading the Italian source,
and that is checkable: by then the same 100 selections have committed verdicts in French (it-fr) and
English (it-en), which the owner reads too. So one judge, as in every Spanish-judged precedent, with
change 38's cheaper checks:
1. **The criteria first**: the Spanish examples below are committed before `results-it-es.jsonl`.
2. **Three references per line**:
   - it-fr's verdict on the same selection;
   - it-en's verdict on the same selection;
   - `results-en-es.jsonl`'s Spanish for the English PUD sentence, a reference on idiom only.

   A verdict departing from both it-fr's and it-en's gives its reason.
3. **Doubtful lines named**, and the test holding the tier with them counted wrong, as change 50's does.
4. **The owner** reads the wrong, doubtful and departing lines and may re-judge any (task 4.1). His
   reading does not gate: the tier rule decides.

**What the chain adds:**
- marks lost in the pivot (withheld at about it-en's plus five);
- a first-leg error carried through (es-fr k 79);
- an error added or repaired by the second leg (fr-es k 30, k 15);
- a word left in English or Italian.

The requirement judges a mark against the rendering in the pair's native language. So a mark on a word
the route leaves in English, or copies through untranslated from Italian, is wrong (« left in English »,
« left in Italian »), unless Spanish uses the word too (a loanword such as « Premium »). A subject pronoun
the route re-adds, marked alone, is wrong.

**The Spanish examples for Italian selections:**

| Case | Correct; wrong | Anchors |
|---|---|---|
| A compound past rendered by one Spanish preterite (5, 13, 17, 25, 31) | 5 « aumentaron », « crecieron »; 13 « publicaron », « se publicaron », « fueron publicadas »; 17 « comentó »; 25 « dijo », « Le dijo », « declaró »; 31 « elegí », « la elegí »; a compound Spanish with or without « ha »; wrong: « ha », « han », « fueron », « se », « Le » alone, an added « Él » or « Yo » alone | en-es k 13, 17, 25 correct; es-en k 25 « he » wrong; change 24's D2 |
| The passato remoto, the same rule | 37 « vi »; 65 « conoció »; 69 « lanzó », « publicó »; 81 « acusó »; tense never decides | fr-es k 65 « conoce » counted correct |
| A selection after an articulated preposition (6, 22, 82, 91) | 6 « curso », « el curso », « transcurso », « A lo largo (de) », wrong « En », « A », « lo » alone and « historia »; 22 « mejor », « A lo sumo », wrong « En », « el », « casos »; 82 « (en las) últimas », « últimos coletazos », « camino (de salir) », wrong « en », « las », « está »; 91 « (los) últimos », wrong « de los », « sus » | fr-es k 6, 91; en-es k 82 « camino » |
| A fused « della / del / dei / dell' » after the selection (0, 3, 11, 16, 40, 50, 51, 55, 63, 74, 76, 87, 96, 98) | the mark may carry the « de (la) » or « del » beside it, never only that, never the next noun: 0 « parte », « gran parte », wrong « transición », « de la », a withheld likely; 3 « compañía », « empresa », « sociedad », wrong « surcoreana »; 50 « caída », wrong « dinastía »; 96 « parte(s) » (before « dell' »), wrong « del »; 76 « (los) agentes », wrong « del » | fr-en and fr-es k 0 withheld |
| An elided piece | 86 « único », « el único », wrong « el » alone and « Gobierno », a withheld likely; 29, after the truncation « Fa' », « caer », « Deja caer », « Suelta », « Tira », wrong « Deja » alone and « micrófono » | es-en k 86; fr-es k 29 |
| A clitic (9, 31, 33, 97) | 9 « hubiera », « hubiese », « haya », « existieran », wrong « le gustaría », « que » alone; 33 « atribuye », « reconoce », « Se le atribuye », wrong « Se », « le », « Se le » alone and « mérito »; 97 « se conocieron », « se reunieron », « se encuentran », never « se » alone; a Spanish enclitic (« enseñarlo ») is one word | en-es and fr-es k 97 |
| An Italian « si » | 61: the selection is « riguarda », beside an impersonal « si considera »: « En cuanto a », « En lo que respecta (a) », « Con respecto a », correct; « (se) considera » wrong; Spanish's added « se » on a passive is carried with the verb, never alone | en-es and fr-es k 61 judged other selections |
| Gender and number never decide | 79 « enemigas », « enemigos », « rivales », wrong « dos »; 71 « estereotipada »; 93 « capitaneado », « comandado »; 27 « idénticos »; 67 « similar(es) »; 96 « partes »; 59 « buenos » or « bien », wrong « resultados »; 15 « principal », « mayor », wrong « su » alone | es-en « two » and es-fr « deux » wrong; fr-es k 93; fr-en k 15 |
| One Italian word, a Spanish compound or expression | 54 the whole « Año Nuevo » (en-es's « Nuevo » alone does not carry over); 78 « despertador », « alarma »; 83 « canción », wrong « de éxito »; 57 « conforman », « constituyen », wrong « que »; 77 « avanza », « anda »; 89 « ganar », « vencer », wrong « pudo » alone; expressions: 58 « luego », 42 « Debido a », 80 « A diferencia », 46 « En realidad », « De hecho » (« actualmente » doubtful), 60 « momento », 30 « vez », « la última vez » (wrong « la », « última » alone), 2 « Una vez » | fr-es k 30 |
| A loose rendering the mark follows | correct, but doubtful; the Italian words differ from the English ones, so expect 70 « guionista » (« sceneggiatore ») and 98 « suspensión » (« sospensione ») plainly correct | en-es k 18, 70, 77 as a class, not as verdicts |

### D4 — The list

Change 19's structural test lists a judged pair exactly when its committed judgment reaches the tier. So
when counting the doubtful lines wrong would take it-es below the tier, those lines are committed `wrong`
with their reason, as change 38 commits its disagreements; otherwise they stay `correct` with a doubtful
reason. The committed figures are then the strict ones wherever it matters.

- **On the first tier**: `MARKED_PAIRS` gains `it-es`, inert until change 30 lists the pair. The exact
  lists of `test/translate-marks.spec.ts` and `test/translate-relay.spec.ts` follow, with a relay case
  modelled on fr-es's but asserting no `SHIPPED_PAIRS`, since it-es is not shipped. This is the tier
  rule's, not the owner's.
- **Short of it but at least 75 % correct with at most 30 % withheld**: not listed, so translated without
  a mark.
- **Under 75 % or over 30 % withheld**: not listed. The figures go to the owner, who settles under M15 at
  change 30 whether it-es's translation is offered unmarked or not at all.

Changes 29 and 38 both edit the exact lists; whichever is implemented second rebases on the other.

### D5 — Tests and documents

Whichever the outcome, `test/translate-marks.spec.ts` gains *An Italian selection, through English, judged
in Spanish*:
- it-es's totals, and its native language Spanish;
- its tier against whether it is listed;
- its ids equal to it-en's and it-fr's;
- no trapped line or unreconciled fragment, or those that occur named;
- the doubtful lines named, and, if it-es is listed, the tier held with them counted wrong;
- the gloss column filled when the table holds glosses.

Every other pair's results and judgments stay byte for byte.

The documents follow:
- `tool/marks/README.md`: introduction, file table, run and soak commands, judging paragraph, results
  row, experiment row;
- `TRANSLATION.md`: the measured and listed pairs, and the soak section;
- the usage comments of `tool/measure_marks.mjs` and `tool/soak_engine.mjs`;
- the comment above `MARKED_PAIRS`;
- programme row 29.

## Risks / Trade-offs

- **One judge** → the criteria committed first, three references per line, the strict tier, the owner's
  reading. Should the strict figures fall within about six points of the tier (under 96 % correct),
  the implementing agent calls in a blind second judge reading Spanish, by change 38's protocol
  (disagreements committed `wrong`), and records the decision in the pull request (task 3.2). A second
  judge does not reduce sampling error; it checks the judgment where it decides the tier.
- **The pivot** → measured as shipped. The precedents suggest about 97–99 % correct and 8–12 % withheld,
  within about six points of sampling error.

## Migration Plan

Nothing deploys, and no reader sees a change before change 30. Rollback is a revert.

## Effort

0.95–1.35 ideal days, against the programme's 1–2:

| Part | Ideal days |
|---|---|
| The Spanish criteria, committed before the run | 0.2–0.25 |
| The chain's soak | 0.1 |
| The run and its rerun | 0.1 |
| The judgment | 0.25–0.5 |
| The gloss column | 0.05–0.15 |
| The list, tests and documents | 0.25 |
