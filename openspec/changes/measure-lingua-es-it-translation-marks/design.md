# Design — measure-lingua-es-it-translation-marks

## Context

Read on main at `97c8aef7b`, where nothing Italian is implemented yet: the route (change 2), Italian's
stop words (change 38), the Italian reviewer agent (change 33) and change 49's protocol are proposals.

- **The selections**: the 100 Spanish ones already committed in `tool/marks/corpus.json` (56 nouns, 24
  verbs, 20 adjectives), chosen by the corpus's rule from UD Spanish-PUD at `pud.mjs`'s pin. es-en's and
  es-fr's results and verdicts on them are committed:

  | Pair | Route | Judged in | Result |
  |---|---|---|---|
  | es-en | direct | English | 94 of 96 correct, 4 % withheld; k 25 « he » wrong (a subject pronoun re-added), k 79 « two » wrong |
  | es-fr | es-en → en-fr | French | 89 of 90 correct, 10 % withheld; k 79 « deux » wrong, carried from es-en |

  PUD sentences are parallel across languages, so every id has UD Italian-PUD's human Italian, pinned by
  change 19.
- **The route**: change 2 sets `"es-it": [es-en/base-memory/2.0, en-it/base-memory/2.1]`; es-en is the
  model es-en and es-fr already use. Change 2's task 2.1 soaks `--pair es-it --isolate`. Until changes 2
  and 38 land, `measure_marks.mjs` and `soak_engine.mjs` refuse es-it (no route) and `stopWords("it")`
  throws.
- **No trial.** Change 49's review found that a trial run before the proposal, with a deterministic
  harness, leaves the criteria drafted after its output. This change runs none: the criteria below hold
  case classes from the committed verdicts and PUD's Italian, written before any es-it output exists.

## Goals / Non-Goals

**Goals:**
- es-it's marks measured on the committed Spanish selections through its two models, judged in Italian by
  two judges, and listed if on the first tier.

**Non-Goals:**
- Offering es-it's translation, or deciding what happens below the tier: the owner, under M15, at change
  54.
- Widening `reconcile.ts` (change 38's D6 limit): a separate change.
- New stop words: change 38's Italian list serves es-it.

## Decisions

### D1 — The chain soaked

Change 2's task 2.1 soaks `--pair es-it --isolate`. This change adds the one-instance soak over the
Spanish selections, recorded in `TRANSLATION.md` for change 54 to read.

### D2 — es-it measured

`node tool/assemble_model_site.mjs <dir>`, then `measure_marks.mjs --pair es-it --models <dir>` over the
100 Spanish selections:
- the sentence and the fragment each asked as the extension asks them;
- a trap answered as change 26 answers it;
- run twice, the results byte for byte the same before the first is committed.

The ids match `results-es-en.jsonl`'s and `results-es-fr.jsonl`'s. The gloss experiment's columns read
`tables/es-it/gloss.tsv` with `tables/es/forms.tsv` if committed at the run (change 51, or change 52's
refined table, its commit recorded); otherwise they stay empty and change 52 reruns the experiment over
its refined table, the engine columns byte for byte. The judges judge them; they never gate.

### D3 — Judged in Italian, by two agent judges

Change 49's protocol, as change 38's:
1. **The criteria first**: the Italian criteria below are committed to `tool/marks/README.md` before
   `results-es-it.jsonl`; no es-it output exists when they are written.
2. **Three references per line**: es-en's verdict on the same selection; es-fr's; UD Italian-PUD's human
   Italian for the same id (fetched at its pin, never committed). es-it's second leg is en-it, so
   `results-en-it.jsonl`'s Italian for the English PUD sentence is a reference on idiom too, never a
   verdict. A verdict departing from both es-en's and es-fr's gives its reason.
3. **Two judges**: Judge A, the implementing agent; Judge B, change 33's Italian reviewer agent, working
   blind in a checkout at the results' commit from which this change's directory is removed, given the
   results, the criteria and the 100 PUD Italian sentences Judge A pastes into its brief, never A's
   verdicts or the pull request's description. A disagreement is a line the two give different verdicts
   (`correct`, `wrong`); a line one calls correct and the other correct but loose is committed `correct`
   with a doubtful reason. A disagreement is committed `wrong` with `doubtful: A …/B …`; the README
   reports the agreement.
4. **The tier must hold with every other doubtful line counted wrong too** (`wrong` where they decide the
   tier, D4).
5. **Every judgment committed**, auditable by the owner against es-en's and es-fr's anchors.

**What the chain can add:** marks lost in the pivot (withheld at about es-en's plus three to six, 7–10
%); a first-leg error carried through (es-en k 79 « two », carried to es-fr's « deux »); a first-leg
misalignment the second leg cannot repair (es-en's engine marked « improved » at k 72 and « alber » at k
78, both moved by es-en's own fragment; es-fr withheld k 78); a subject pronoun the first leg re-adds
(es-en k 25 « he »), which Italian usually drops; a sense the pivot shifts through the English word; a
word left in English or Spanish.

A mark on a word the route leaves in English, or copies through from Spanish, is wrong, unless Italian
uses the word too. A mark on punctuation alone is wrong; a colon, closing quote or comma carried along
with the rendering is not, as the precedents judge it (es-fr k 25 « il a déclaré: », k 82 « derniers", »).
A subject pronoun marked alone is wrong.

**The Italian criteria for Spanish selections**, by case. Examples marked PUD are UD Italian-PUD's human
Italian; the others are anchors from the committed verdicts:

| Case | Rule; examples | Anchors |
|---|---|---|
| A past, compound or simple; a passive; an imperfect | The participle, with or without its auxiliary, or the simple past or imperfect; any Italian past or passive of the verb counts, and tense never decides (PUD 97 « (si) incontrano » is a present). PUD: 5 « (sono) cresciuti », 13 « (sono state) pubblicate », 25 « (Ha) detto », 33 « (Gli viene) riconosciuto », 53 « rappresentavano », 69 « rilasciò », 81 « accusò »; 17 PUD « (ha) commentato » renders « sugirió » loosely, and « suggerito » counts. The auxiliary or « si » alone is wrong | es-en 13 « published », 69 « released »; es-fr 13 « publiées », 69 « a sorti » |
| A conditional; a dropped subject | The verb (PUD 9 « Vorrebbe »); a subject carried with the verb counts, as es-fr's « il a déclaré: »; an added « egli », « lui » or « lei » alone is wrong | es-en 9 « would \| like »; es-en 25 « he » wrong; es-fr 25 « il a déclaré: » correct |
| An article or an articulated preposition | Carried with the word, never alone | es-fr 59 « L'armée », 66 « l'Université » |
| An elision `reconcile.ts` cannot strip | PUD 7 « all'estremo », PUD 66 « all'Università »: a mark on the whole word counts; « all' » alone is wrong | es-en 7 « Far »; es-fr 7 withheld |
| An expression | The mark on the Italian expression, or its word rendering the selection, counts: PUD 4 « Tuttavia » (« sin embargo »), 6 « Nel corso » (« a lo largo »), 22 « Nel migliore (dei casi) », 52 « Tuttavia » (« de todas maneras »), 80 « A differenza », 82 « agli sgoccioli ». Where both precedents withheld (29, 30, 89), a mark is a new verdict with its reason, against PUD's Italian (89 « fu in grado ») | es-en 4 « However », 80 « Unlike »; es-fr 4 « Cependant », 80 « Contrairement » |
| A Spanish phrase rendered by one word | The one word counts: PUD 54 « capodanno » (« Año Nuevo »), 48 « I temporali » (« tormentas eléctricas ») | es-en 54 « Year », 48 « Thunderstorms »; es-fr 54 « An », 48 « Les orages » |
| A compound carrying the selection | PUD 39 « franco-monegasco »: Italian writes the compound too, so a mark on the whole compound is correct but doubtful, as change 49's row | es-en and es-fr 39 withheld |
| A first-leg error carried through | 79: « nemiche » (PUD « fazioni nemiche ») counts; « due » is wrong, as « two » and « deux » are | es-en and es-fr 79 wrong |
| A first-leg misalignment | 72: « metodo » counts (PUD « un metodo migliorato »), a mark on « migliorato » is wrong; 78: « sveglia » counts (PUD « la sveglia »), a leftover that is not Italian is wrong | es-en 72 « method » and 78 « alarm clock », moved from « improved » and « alber » by the fragment; es-fr 78 withheld |
| A sense the pivot shifts | Judged against the Spanish sentence by the README's rules: the rendering of the selected word there is correct (PUD 94 « commercio »); a rendering through another sense of the English word, the mark following it, is correct but doubtful | es-en 94 « Trade » |
| A loose rendering | Correct, doubtful where a judge finds it loose (D3.3); PUD's word is plainly correct (75 « in grado », 86 « l'unico ») | es-en 75 « able »; es-fr 75 « pu » |
| Gender and number never decide | 3, 15, 27, 51, 71, 93, 99 | es-fr 3, 27, 51, 93, 99 |

### D4 — The list

Change 19's structural test lists a judged pair exactly when its committed judgment reaches the tier.
Disagreements are committed `wrong` (D3), and so are the doubtful lines wherever counting them wrong would
take es-it below the tier, with their reason.

- **On the first tier**: `MARKED_PAIRS` gains `es-it`, inert until change 54 lists the pair; the exact
  lists of `test/translate-marks.spec.ts` and `test/translate-relay.spec.ts` follow, with a relay case
  asserting no `SHIPPED_PAIRS`. The tier rule's, not the owner's.
- **Short of it but at least 75 % correct with at most 30 % withheld**: not listed, so translated without
  a mark.
- **Under 75 % or over 30 % withheld**: not listed; the owner settles under M15 at change 54.

Changes 19, 24, 29, 38, 49 and this one each edit the exact lists; whichever is implemented later rebases
on the others.

### D5 — Tests and documents

`test/translate-marks.spec.ts` gains *A Spanish selection, through English, judged in Italian*: es-it's
totals and its native language Italian; its tier against whether it is listed; its ids equal to es-en's
and es-fr's; no trapped line or unreconciled fragment, or those that occur named; the doubtful lines
named and, if es-it is listed, the tier held with the doubtful lines committed `correct` counted wrong (a
line committed `wrong` with a doubtful reason is not subtracted twice); the gloss column as the run found
the table. Every other pair's results and judgments stay byte for byte.

The documents follow: `tool/marks/README.md` (the file table, run and soak commands, the judging
paragraph with the agreement, the results and experiment rows), `TRANSLATION.md`, the usage comments of
`tool/measure_marks.mjs` and `tool/soak_engine.mjs`, the comment above `MARKED_PAIRS`, programme row 53.

## Risks / Trade-offs

- **Italian judged by agents alone** → change 49's protocol: criteria first and blind, three references
  per line, a blind second judge, disagreements committed wrong, the strict tier.
- **The pivot** → measured as shipped; es-fr's precedent suggests about 97–99 % correct and 7–10 %
  withheld.
- **A strict margin unknown before the run** → whatever it is, the tier rule decides; below it, es-it is
  translated without a mark if above 75 %.

## Migration Plan

Nothing deploys, and no reader sees a change before change 54. Rollback is a revert.

## Effort

1.1–1.65 ideal days, against the programme's 1–2:

| Part | Ideal days |
|---|---|
| The Italian criteria, committed before the run | 0.25–0.3 |
| The chain's one-instance soak | 0.1 |
| The run and its rerun | 0.1 |
| The two judges | 0.35–0.6 |
| The gloss column (empty if changes 51 and 52 are not committed) | 0–0.05 |
| The list, tests and documents | 0.3–0.5 |

## Open Questions

For the owner:
1. **Two judges** (D3), as changes 38 and 49 (recommended).
2. **Implemented before changes 51 and 52** if their tables are late (recommended): the gloss columns
   stay empty, and change 52 reruns the experiment.
