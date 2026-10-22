# Design — measure-lingua-fr-it-translation-marks

## Context

Read on main at `6fa38031f`, where nothing Italian is implemented yet: the route (change 2), Italian's
stop words (change 38) and the Italian reviewer agent (change 33) are proposals.

- **The selections**: the 100 French ones already committed in `tool/marks/corpus.json`, chosen by the
  corpus's rule from UD French-PUD (`db260db1…`, matrix change 50's D4). fr-en's and fr-es's results and
  verdicts on them are committed:

  | Pair | Route | Judged in | Result |
  |---|---|---|---|
  | fr-en | direct | English | 95 of 96 correct, 4 % withheld; k 15 « their » wrong; k 39, 92 doubtful |
  | fr-es | fr-en → en-es | Spanish | 90 of 91 correct, 9 % withheld; k 30 « la » wrong; k 25 doubtful |
  | es-fr | es-en → en-fr | French | 89 of 90 correct, 10 % withheld; k 79 « deux » wrong, carried from es-en |

  All 100 ids exist in UD Italian-PUD, pinned by change 19, whose human Italian is a reference.
- **The route**: change 2 sets `"fr-it": [fr-en/base-memory/2.0, en-it/base-memory/2.1]`. en-it 2.1 was
  checked against Mozilla's Remote Settings on 2026-10-22 (model 31,561,787 B decompressed); the trial
  below downloaded 52,744,550 B for the route (fr-en's 26,234,715 B from the catalogue, en-it's
  26,509,835 B of registry gzips). Change 2's task 2.1 soaks `--pair fr-it --isolate`. Until changes 2
  and 38 land, `measure_marks.mjs` and `soak_engine.mjs` refuse fr-it (no route) and `stopWords("it")`
  throws.
- **A trial outside the repository**, on 2026-10-22, by the research for this proposal. Its harness first
  reproduced the committed `results-fr-en.jsonl` on 100 of 100 lines, and its models and PUD files were
  checked against the catalogue, Remote Settings and their pins. fr-en then en-it on the 100 French
  selections, two runs byte for byte the same: 0 traps, 0 unreconciled fragments, 7 withheld (fr-en
  withholds 4, fr-es 9). One agent's reading: 90 of 93 marks correct (96.8 %), 85 of 93 (91.4 %) with
  that agent's doubtful lines counted wrong. The first tier needs 84 of 93: the strict margin is one
  line.
- **What the trial means for the protocol.** The harness is deterministic, so the committed run will
  reproduce the trial's output byte for byte, and committing the criteria before the results proves
  nothing about who saw what. The criteria below were drafted after the trial; they hold case classes
  anchored on fr-en's and fr-es's committed verdicts and on UD Italian-PUD's human Italian, and no verdict
  on a string the trial produced. The trial's per-line verdicts are kept out of the repository. The
  safeguard is Judge B's independence (D3).

## Goals / Non-Goals

**Goals:**
- fr-it's marks measured on the committed French selections through its two models, judged in Italian
  by two judges, and listed if on the first tier.

**Non-Goals:**
- Offering fr-it's translation, or deciding what happens below the tier: the owner, under M15, at
  change 50.
- Widening `reconcile.ts` to strip « all' » and its kin: a separate change, with a byte-for-byte
  recheck of the French pairs (Open Question 3).
- New stop words: change 38's Italian list serves fr-it.

## Decisions

### D1 — The chain soaked

Change 2's task 2.1 soaks `--pair fr-it --isolate`. This change adds the one-instance soak over the
French selections (about 10 seconds by the trial), recorded in `TRANSLATION.md` for change 50 to read.

### D2 — fr-it measured

`node tool/assemble_model_site.mjs <dir>`, then `measure_marks.mjs --pair fr-it --models <dir>` over the
100 French selections:
- the sentence and the fragment each asked as the extension asks them;
- a trap answered as change 26 answers it;
- run twice, the results byte for byte the same before the first is committed.

The ids match `results-fr-en.jsonl`'s and `results-fr-es.jsonl`'s. The gloss experiment's columns read
`tables/fr-it/gloss.tsv` with `tables/fr/forms.tsv` when committed (change 47, or change 48's refined
table, whichever is on main at the run, its commit recorded); otherwise they stay empty. The judges judge
them; they never gate.

### D3 — Judged in Italian, by two agent judges

Nobody on the team reads Italian (I13), so the protocol is change 38's:
1. **The criteria first**: the Italian criteria below are committed to `tool/marks/README.md` before
   `results-fr-it.jsonl`, as the precedents do. Here git's order shows only that the criteria hold no
   verdict on the results' strings; the run reproduces a trial they were drafted after (*Context*).
2. **Three references per line**:
   - fr-en's verdict on the same selection;
   - fr-es's verdict on the same selection;
   - UD Italian-PUD's human Italian for the same id (fetched at its pin, never committed).

   `results-en-it.jsonl`'s Italian for the English PUD sentence is a reference on idiom only. A verdict
   departing from both fr-en's and fr-es's gives its reason.
3. **Two judges**:
   - Judge A is the implementing agent;
   - Judge B is change 33's Italian reviewer agent, working blind: a checkout at the results' commit from
     which this change's directory is removed, the results, the criteria, and the 100 PUD Italian
     sentences Judge A fetched and pastes into B's brief (B's tools are read-only and cannot fetch
     them); never A's verdicts, the pull request's description, or the trial. B's full verdicts are
     pasted in the pull request's description.

   A disagreement is a line the two judges give different verdicts (`correct`, `wrong`); a line one calls
   correct and the other correct but loose is no disagreement, and is committed `correct` with a
   doubtful reason. A disagreement is committed `wrong` with the reason `doubtful: A …/B …`. The README
   reports how many lines they agreed on.
4. **The tier must hold with every other doubtful line counted wrong too**: lines judged correct but
   loose are committed `correct` with a doubtful reason (`wrong` where they decide the tier, D4), and the
   test holds the tier with them subtracted.
5. **Every judgment committed.** The owner, who reads French and Spanish, can audit the lines against
   fr-en's and fr-es's anchors.

**What the chain can add:**
- marks lost in the pivot (withheld at about fr-en's plus three to five);
- a first-leg error carried through (fr-en k 15 « their »);
- a first-leg misalignment the second leg cannot repair (fr-en's engine marked « last » at k 30,
  reconciled to « time » by fr-en's own fragment; fr-es's « la » was wrong there);
- a sense the pivot shifts through the English word;
- a word left in English or French.

The requirement judges a mark against the rendering in the pair's native language. So a mark on a word
the route leaves in English, or copies through from French, is wrong (« left in English », « left in
French »), unless Italian uses the word too (« Premium »). A mark on punctuation alone is wrong; a closing
quote or comma carried along with the rendering is not, as the precedents judge it (en-es k 35, es-fr and
fr-es k 82). A subject pronoun the route re-adds, marked alone, is wrong.

**The Italian criteria for French selections**, by case. Examples marked PUD are UD Italian-PUD's human
Italian; the others are anchors from the committed verdicts:

| Case | Rule; examples | Anchors |
|---|---|---|
| A past, compound or simple; a passive | The participle, with or without its auxiliary, or the simple past; any Italian past or passive of the verb counts; the auxiliary or « si » alone is wrong. PUD: 5 « (sono) cresciuti », 13 « (sono state) pubblicate », 17 « (ha) commentato », 33 « (Gli viene) riconosciuto », 69 « rilasciò », 73 « Recitò », 81 « accusò », 97 « (si) incontrano » | fr-en 13 « released », 69 « released », 81 « called »; fr-es 13 « publicaron », 69 « lanzó », 81 « llamó », 97 « se reunieron » |
| A conditional; a dropped subject | The verb, conditional or not (PUD 9 « Vorrebbe »; 25 PUD « vuole » for « voudrait »); an added « egli » or « lui » alone is wrong; a mark also carrying a word of the neighbouring phrase is doubtful | fr-en 9 « like »; fr-es 9 « desearía », 25 doubtful « el \| querría”, » |
| An article or an articulated preposition | Carried with the word, never alone: « il », « gli », « all' », « nell' » alone are wrong | fr-es 42 « el clima »; fr-en 66 « University » |
| An elision `reconcile.ts` cannot strip | PUD 7 « all'estremo »: a mark on « all'estremo » counts; « all' » alone is wrong. Change 38's D6 limit, measured as shipped | fr-en 7 « far », fr-es 7 « extremo » |
| An expression | The mark on the Italian expression, or on its word that renders the selection, counts (PUD 6 « Nel corso », 29 « Fa' cadere »); a mark on a word that renders nothing of the selection, or where the selection's word was left out, is judged by the README's existing rules. Where both precedents withheld (0, 57, 79), a mark is a new verdict with its reason | fr-en 26 « As \| result », fr-es 26 « Como resultado »; fr-es 22 withheld |
| A first-leg error carried through | 15: « principal » rendered (PUD « maggiore », or « principale ») counts; a possessive or an article in its place is wrong, as fr-en's « their » is | fr-en 15 wrong; fr-es 15 « principal » correct |
| A first-leg misalignment | 30: the mark must carry « volta » (PUD « l'ultima volta »); a mark without it is wrong | fr-es 30 « la » wrong |
| A sense the pivot shifts | Judged against the French sentence by the README's rules: the rendering of the selected word there is correct (PUD 94 « commercio »); a rendering through another sense of the English word, the mark following it, is correct but doubtful, as change 29's loose renderings are | fr-en 94 « trade », fr-es 94 « comercio » |
| A compound carrying the selection | A mark spanning a compound with the selection's rendering inside (PUD 92 « Le tracce di sangue ») is doubtful, as fr-en's | fr-en 92 doubtful |
| A loose rendering | Correct, doubtful only where both judges find it loose; PUD's word is plainly correct (75 « agenti », 52 « battaglia ») | fr-en 75 « officers », fr-es 75 « oficiales » |
| Gender and number never decide | 19, 27, 39, 51, 71, 93 | fr-en 19, 27, 71, 93 |

### D4 — The list

Change 19's structural test lists a judged pair exactly when its committed judgment reaches the tier.
The judges' disagreements are committed `wrong` (D3), and so are the doubtful lines wherever counting
them wrong would take fr-it below the tier, with their reason; elsewhere they stay `correct` with a
doubtful reason. The committed figures are then the strict ones wherever it matters.

- **On the first tier**: `MARKED_PAIRS` gains `fr-it`, inert until change 50 lists the pair. The exact
  lists of `test/translate-marks.spec.ts` and `test/translate-relay.spec.ts` follow, with a relay case
  modelled on fr-es's, asserting no `SHIPPED_PAIRS` since fr-it is not shipped. This is the tier rule's,
  not the owner's.
- **Short of it but at least 75 % correct with at most 30 % withheld**: not listed, so translated without
  a mark.
- **Under 75 % or over 30 % withheld**: not listed. The figures go to the owner, who settles under M15 at
  change 50 whether fr-it's translation is offered unmarked or not at all.

Changes 19, 24, 29, 38, this one and later 53 each edit the exact lists; whichever is implemented later
rebases on the others.

### D5 — Tests and documents

Whichever the outcome, `test/translate-marks.spec.ts` gains *A French selection, through English, judged
in Italian*:
- fr-it's totals, and its native language Italian;
- its tier against whether it is listed;
- its ids equal to fr-en's and fr-es's;
- no trapped line or unreconciled fragment, or those that occur named;
- the doubtful lines named, and, if fr-it is listed, the tier held with the doubtful lines committed
  `correct` counted wrong (a line committed `wrong` with a doubtful reason is not subtracted twice);
- the gloss column as the run found the table: filled if fr-it's table was committed at the run, empty
  otherwise. If it was not, change 48 reruns the experiment over its refined table, the engine columns
  byte for byte, as a re-run of change 50's experiment would.

Every other pair's results and judgments stay byte for byte.

The documents follow:
- `tool/marks/README.md`: the file table, run and soak commands, the judging paragraph with the two
  judges' agreement, the results row, the experiment row;
- `TRANSLATION.md`: the measured and listed pairs, and the soak section;
- the usage comments of `tool/measure_marks.mjs` and `tool/soak_engine.mjs`;
- the comment above `MARKED_PAIRS`;
- programme row 49.

## Risks / Trade-offs

- **Italian judged by agents alone** → change 38's protocol: the criteria first, three references per
  line, a blind second judge, disagreements committed wrong, the strict tier, and fr-en's and fr-es's
  anchors for the owner's audit.
- **The pivot** → measured as shipped; the trial suggests about 97 % correct and 7 % withheld.
- **A thin strict margin** → the trial is one line above the tier with its doubtful lines counted wrong.
  Two more disagreements or doubtful lines give 83 of 93 (89.2 %): fr-it is then not listed and is
  translated without a mark, being above 75 %. The outcome is the tier rule's.
- **« all' » and its kin** → measured as shipped: a mark there is judged by D3's row, and widening
  `reconcile.ts` is a separate change (Open Question 3).

## Migration Plan

Nothing deploys, and no reader sees a change before change 50. Rollback is a revert.

## Effort

1.1–1.65 ideal days, against the programme's 1–2:

| Part | Ideal days |
|---|---|
| The Italian criteria, committed before the run | 0.25–0.3 |
| The chain's one-instance soak | 0.1 |
| The run and its rerun | 0.1 |
| The two judges | 0.35–0.6 |
| The gloss column (empty if changes 47 and 48 are not committed) | 0–0.05 |
| The list, tests and documents | 0.3–0.5 |

## Open Questions

For the owner:
1. **Two judges** (D3), as change 38 (recommended): Italian is the judged language and nobody on the team
   reads it (I13). One judge, as change 29, would rest on the agent alone.
2. **Implemented before changes 47 and 48** (recommended): the gloss columns stay empty; nothing else
   depends on them.
3. **« all' » and its kin**: measured as shipped here (recommended), and `reconcile.ts` widened in a
   separate change that rechecks every French pair byte for byte.
