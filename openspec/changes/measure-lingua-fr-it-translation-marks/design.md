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
- **The route**: change 2 sets `"fr-it": [fr-en/base-memory/2.0, en-it/base-memory/2.1]`, 52,744,550 B
  gzipped. en-it 2.1 was checked against Mozilla's Remote Settings on 2026-10-22 (model 31,561,787 B
  decompressed). Change 2's task 2.1 soaks `--pair fr-it --isolate`. Until changes 2 and 38 land,
  `measure_marks.mjs` and `soak_engine.mjs` refuse fr-it (no route) and `stopWords("it")` throws.
- **A trial outside the repository**, on 2026-10-22, by the research for this proposal:
  - the harness first reproduced the committed `results-fr-en.jsonl` on 100 of 100 lines; the models and
    the PUD files were checked against the catalogue, Remote Settings and their pins;
  - fr-en then en-it on the 100 French selections, two runs byte for byte the same: 0 traps, 0
    unreconciled fragments, 7 withheld (k 0, 2, 22, 48, 57, 79, 99; fr-en withholds 4, fr-es 9);
  - one agent's reading: 90 of 93 marks correct (96.8 %), 85 of 93 (91.4 %) with the 5 doubtful lines
    counted wrong. Both on the first tier.
  - The three wrong lines: k 7, a mark on a comma, « estremo » sitting inside « all'estremo », which
    `reconcile.ts` cannot strip (change 38's D6 limit); k 15 « - il », fr-en's « their » carried through
    (the fragment alone came back « Principal », in English); k 30 « l'ultima », fr-es's k 30 twin.
  - The doubtful ones: k 25 (a stray « che » with « vorrebbe »), 29 (« Lascia il »), 75 (« agenti » for
    officiers), 92 (the compound « Le scie di sangue », as fr-en's), 94 (« mestiere »: commerce → trade →
    craft, a sense the pivot shifted).

  The trial is indicative and is not committed. The criteria below were drafted after it, from the
  precedents' verdicts and Italian grammar; the trial showed which cases occur. The protocol keeps the
  order that matters: the criteria committed before the results, and Judge B blind (D3).

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
1. **The criteria first**: the Italian examples below are committed to `tool/marks/README.md` before
   `results-fr-it.jsonl`, so that git's order shows it.
2. **Three references per line**:
   - fr-en's verdict on the same selection;
   - fr-es's verdict on the same selection;
   - UD Italian-PUD's human Italian for the same id (fetched at its pin, never committed).

   `results-en-it.jsonl`'s Italian for the English PUD sentence, once change 38 commits it, is a
   reference on idiom only. A verdict departing from both fr-en's and fr-es's gives its reason.
3. **Two judges**:
   - Judge A is the implementing agent;
   - Judge B is change 33's Italian reviewer agent, working blind in a checkout at the results' commit,
     given the results and the criteria, not A's verdicts. B's full verdicts are pasted in the pull
     request's description.

   A line they disagree on is committed `wrong` with the reason `doubtful: A …/B …`. The README reports
   how many lines they agreed on.
4. **The tier must hold with every other doubtful line counted wrong too**: lines both judges call
   correct but loose are committed `correct` with a doubtful reason, and the test holds the tier with
   them subtracted.
5. **Every judgment committed.** The owner, who reads French and Spanish, can audit the lines against
   fr-en's and fr-es's anchors.

**What the chain adds:**
- marks lost in the pivot (withheld at about fr-en's plus five);
- a first-leg error carried through (fr-en k 15 « their »);
- an error added by the second leg (fr-es k 30, on the same « the last time »);
- a sense the pivot shifts (« commerce » → trade → « mestiere »);
- a word left in English or French.

The requirement judges a mark against the rendering in the pair's native language. So a mark on a word
the route leaves in English, or copies through from French, is wrong (« left in English », « left in
French »), unless Italian uses the word too (« Premium »). A mark on punctuation is wrong. A subject
pronoun the route re-adds, marked alone, is wrong.

**The Italian examples for French selections:**

| Case | Correct; wrong | Anchors |
|---|---|---|
| A past, compound or simple; a passive: the auxiliary may be marked with the participle, never alone | 5 « aumentati », « sono aumentati », « progrediti »; 13 « diffuse », « pubblicate », « sono state diffuse »; 17 « suggerì », « suggeriva », « ha suggerito »; 33 « noto », « riconosciuto »; 69 « ha pubblicato », « pubblica », « esce »; 73 « interpretato », « recitato »; 81 « definiva », « ha definito », « accusava »; 97 « incontrati », « si sono incontrati »; wrong « ha », « è », « sono state », « si » alone | fr-en 13 « released », 69 « released », 81 « called »; fr-es 13 « publicaron », 69 « lanzó », 81 « llamó », 97 « se reunieron » |
| A conditional, a dropped subject | 9 « Vorrebbe », « desidererebbe »; 21 « Penso », « Credo »; 25 « vorrebbe », a mark also carrying a stray « che » or the closing quote doubtful; wrong an added « egli » or « lui » alone | fr-en 9 « like »; fr-es 9 « desearía », 25 doubtful « el \| querría” » |
| An article or an articulated preposition, carried but never alone | 4 « gli esperti », 10 « I medici », 42 « il clima », 44 « Le reazioni », 59 « L'esercito », 66 « all'Università », 74 « Nell'agosto » or « agosto »; wrong « gli », « all' », « Nell' » alone | fr-es 42 « el clima »; fr-en 66 « University » |
| An elision `reconcile.ts` cannot strip | 7 « estremo » inside « all'estremo »: a mark on « all'estremo » correct, a mark on « all' » or on punctuation wrong; the limit is measured as shipped | fr-en 7 « far », fr-es 7 « extremo » |
| An expression rendered by an Italian one | 6 « Nel corso », « Attraverso », « Per tutto »; 26 « Di conseguenza », « Come risultato »; 29 « Lascia cadere », « Fa' cadere », a mark on « Lascia » with the next noun's article doubtful, « il » alone wrong; 79 « (Per) ora », « Per il momento »; 57 (« Autrement dit ») « In altre parole »; 22 (« au mieux ») « nel migliore dei casi », « al massimo »; 0 (« la plus grande partie de ») « la maggior parte »; a withheld mark likely on each | fr-en and fr-es withheld 0, 57, 79; fr-es 22 withheld; fr-en 26 « As \| result » |
| A first-leg error carried through | 15 « principale » correct; « il », « loro », « - il », a possessive or article wrong | fr-en 15 « their » wrong; fr-es 15 « principal » correct |
| A second-leg error | 30 « volta », « l'ultima volta » correct; « l'ultima » or « la » alone wrong | fr-es 30 « la » wrong |
| A sense the pivot shifts, or a loose rendering | 94 « commercio » correct; « mestiere » correct but doubtful; 75 « ufficiali » correct, « agenti » doubtful; 52 « azione », « combattimento » | fr-en 94 « trade », fr-es 94 « comercio »; fr-en 75 « officers » |
| A compound carrying the selection | 92 « tracce », « scie »; a mark spanning « Le scie di sangue » doubtful | fr-en 92 doubtful |
| Gender and number never decide | 19 « moderna », 27 « ordinari », 39 « limitata », 51 « esatto » or « esatta », 71 « stereotipato », 93 « comandata » | fr-en and fr-es |

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

Changes 29, 38 and this one each edit the exact lists; whichever is implemented later rebases on the
others.

### D5 — Tests and documents

Whichever the outcome, `test/translate-marks.spec.ts` gains *A French selection, through English, judged
in Italian*:
- fr-it's totals, and its native language Italian;
- its tier against whether it is listed;
- its ids equal to fr-en's and fr-es's;
- no trapped line or unreconciled fragment, or those that occur named;
- the doubtful lines named, and, if fr-it is listed, the tier held with the doubtful lines committed
  `correct` counted wrong (a line committed `wrong` with a doubtful reason is not subtracted twice);
- the gloss column filled when the table holds glosses.

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
- **« all' » and its kin** → measured as shipped: a mark there is judged by D3's row, and widening
  `reconcile.ts` is a separate change (Open Question 3).

## Migration Plan

Nothing deploys, and no reader sees a change before change 50. Rollback is a revert.

## Effort

1.15–1.75 ideal days, against the programme's 1–2:

| Part | Ideal days |
|---|---|
| The Italian criteria, committed before the run | 0.25–0.3 |
| The chain's one-instance soak | 0.1 |
| The run and its rerun | 0.1 |
| The two judges | 0.35–0.6 |
| The gloss column | 0.05–0.15 |
| The list, tests and documents | 0.3–0.5 |

## Open Questions

For the owner:
1. **Two judges** (D3), as change 38 (recommended): Italian is the judged language and nobody on the team
   reads it (I13). One judge, as change 29, would rest on the agent alone.
2. **Implemented before changes 47 and 48** (recommended): the gloss columns stay empty; nothing else
   depends on them.
3. **« all' » and its kin**: measured as shipped here (recommended), and `reconcile.ts` widened in a
   separate change that rechecks every French pair byte for byte.
