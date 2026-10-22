# Design

## Context

See proposal.md — Why. Where things stand:

- `src/i18n/{fr,en,es}/grammar.ts`: each renderer holds `TENSES: Record<StudiedLanguageCode,
  TenseTable>` keyed `Mood/Tense` (`Ind/Pres`, `Ind|Sub/Pres`, `Cnd/`, `Imp/`…), `GERUNDS`, the
  participles' names, and the order a card names tenses in (`tenseOrder`); the language-neutral
  description (`src/reading/grammar-description.ts`: `finiteKey`, `formKind`, the M21 merge of change
  51's D3) turns a form's readings into keyed parts. `StudiedLanguageCode = "en" | "es" | "fr"`
  (`src/i18n/index.ts`). The French renderer has named en-fr's and es-fr's cards from the start; its `fr` row exists
  for the override M2 reserves, so no studied language whose moods merge has been shown in French yet.
- Change 51: French's rows, the `Ind|Sub` merge, the present participle, a language-neutral
  `selectWithinWord`, review's French no-space branch (`pack_gloss`, `crates/lingua-wasm/src/lib.rs`).
- Change 13: Italian's readings — `Tense=Past` (passato remoto), `Tense=Imp` (imperfetto),
  `Tense=Fut`, `Mood=Cnd` and `Mood=Imp` without a tense, `VerbForm=Ger`, `VerbForm=Part|Tense=Pres`,
  `VerbForm=Part|Tense=Past` with gender and number, unmerged.
- Change 12: Italian expressions keyed through Italian's reading; `pack_gloss`'s Italian arm for a
  lemma with a space. Change 16: the Italian baseline on the committed tables.
- **How it was measured**: change 13's prototype readings (`gram3.py`, scratch, the section's
  tables with the stress marks dropped), counting pairs that read `Mood=Ind` and `Mood=Sub` of one
  `Tense`, `Person`, `Number` — 10,580 (1pl present), 8,449 (2sg present), 1,550 (2pl present),
  counting a defective verb's `-` cells, which give no form; without them, as D3 and the proposal
  count, 10,526, 8,392 and 1,497 (1,455 of them `-iare` verbs) —, and the gerund pairs
  (13,093). The English Wiktionary's own Italian form-of glosses name the tenses "present indicative",
  "imperfect indicative", "past historic", "future", "conditional", "present subjunctive", "imperfect
  subjunctive", "imperative", "gerund", "present participle", "past participle" (counted over its
  verb entries: 35,120 "future", 29,535 "past historic", 6,300 "gerund"…).

## Goals / Non-Goals

**Goals:**

- An Italian card names every reading change 13 stores, in French, English and Spanish, in each
  grammar's terms and order, the moods merged as M21 asks.
- `d'accordo` reviewed in another language shows its gloss.
- The French card's snapshot for it-fr pins the wording; nothing of the other cards moves.

**Non-Goals:**

- The Italian interface's own card wording (Treccani's names, I6): change 34's (`add-lingua-italian-card-wording`).
- it-en's and it-es's snapshots (stage 2); the extension's studied languages (change 20).
- The polite imperative (`Lei`) named: change 13 stores none; a later change if the dogfood asks.

## Decisions

### D1 — Italian is a studied language the renderers name; the extension's type waits for change 20

`StudiedLanguageCode` gains `it`, so every `Record<StudiedLanguageCode, …>` table must hold an
Italian row — the type checker lists them: each renderer's `TENSES` (D2), Spanish's `PARTICIPLES` (D4), and
`CARD_NAMES` in `src/reading/grammar-description.ts`, which says what the card
names per studied language: `CARD_NAMES.it = { infinitive: true, presentParticiple: true, gerund:
true, moods: true }` — the infinitive named as for Spanish and French (`pentire`, *pentirsi*'s
infinitive under change 9's reflexive rule), the present participle as for French, the gerund as for
Spanish, the moods merged as for French (D3, an owner's question). The extension's `StudiedLanguage`
(what a page is read as, what Réglages offers) stays until change 20, as change 51 left French's to
change 52. `GERUNDS`, a partial table the checker does not hold, gains its Italian row too (D4).

### D2 — Italian's tenses, and their order

| Key | French | English | Spanish |
|---|---|---|---|
| `Ind/Pres` | présent de l’indicatif | present indicative | presente de indicativo |
| `Ind\|Sub/Pres` | présent de l’indicatif ou du subjonctif | present indicative or subjunctive | presente de indicativo o de subjuntivo |
| `Ind/Imp` | imparfait de l’indicatif | imperfect indicative | pretérito imperfecto de indicativo |
| `Ind/Past` | passé simple | past historic (passato remoto) | pretérito perfecto simple de indicativo |
| `Ind/Fut` | futur | future | futuro simple de indicativo |
| `Cnd/` | conditionnel | conditional | condicional simple |
| `Sub/Pres` | présent du subjonctif | present subjunctive | presente de subjuntivo |
| `Sub/Imp` | imparfait du subjonctif | imperfect subjunctive | pretérito imperfecto de subjuntivo |
| `Imp/` | impératif | imperative | imperativo |
| gerund | gérondif | gerund | gerundio |
| present participle | participe présent | present participle | participio presente |

French: the terms French grammars of Italian use (the passato remoto is « passé simple », the
gerundio « gérondif »). English: the English Wiktionary's Italian wording, the source the forms come
from, with "past historic (passato remoto)" as M10 settled "past historic (passé simple)" — the
English term alone reads as a French notion to many readers. Spanish: the RAE's, as for French
studied. The order is the English Wiktionary's Italian table's — indicative (present, imperfect, past,
future), conditional, subjunctive (present, imperfect), imperative —, every Romance card's, and
Treccani's « Modi del verbo » (indicativo, condizionale, congiuntivo, imperativo); `Ind|Sub/Imp` is not listed — Italian's imperfect
subjunctive (`parlassi`) never spells the imperfect indicative (`parlavo`).

### D3 — The indicative and the subjunctive said once (M21, read for Italian)

Change 51's merge is gated by `CARD_NAMES[studied].moods` (false for English and Spanish, true for
French), and merges a tense's indicative and subjunctive only where both carry the same persons in
one number (`samePersons`): 51 found no French form whose two moods share some persons but not all.
With `CARD_NAMES.it.moods` true, Italian merges where the persons are the same: `parliamo` (the first
person plural, with its imperative named apart: « 1re personne du pluriel du présent de l’indicatif ou
du subjonctif et 1re personne du pluriel de l’impératif de parlare »), the second persons plural of
`-iare` verbs (`mangiate`, `studiate`: 1,455 of the 1,497 measured 2pl merges). `parli` is not merged:
its indicative is the second person singular and its subjunctive the first, second and third — 8,392
such second persons singular of `-are` verbs (`accusi`, `lavi`, `studi`) — so the card names the two
moods apart, as 51's rule says: "second-person singular present indicative and first-, second- and
third-person singular present subjunctive of parlare". A partial merge (the shared person named
once, the subjunctive keeping the rest) would be a new rule, which this change does not add: it is
the owner's question, with the merge for Italian at all (M21 was settled for French), read on the
snapshot (task 4.1).

### D4 — The gerund and the present participle

`GERUNDS.it`: « gérondif », "gerund", « gerundio »; the present participle's name exists since
change 51 (« participe présent », "present participle", « participio presente »), now reached by
Italian's `VerbForm=Part|Tense=Pres` through `CARD_NAMES.it.presentParticiple`; its number (change 13
gives `parlanti` `Number=Plur`) goes unsaid, the card's participle kind naming none. Spanish's
`PARTICIPLES.it`: « participio pasado », beside « participio presente »; the past participle's
agreement is named as Spanish's renderer names French's (« participio pasado femenino de credere »,
"singular" unsaid on a singular participle).

### D5 — An expression named without a space

`pack_gloss`'s no-space branch gains Italian beside French: a lemma without a space that Italian's
reading turns into two tokens or more, each a lemma (`reading_expression_key(lemma, Italian, …)`), is
read in the expression table at that key, the gloss of the headword it is named by first
(`expression_gloss_named`), and as a word where the table has nothing there. A one-token lemma
(`perché`, `po'` read `poco`) is a word. The extension's card already stores the gloss an expression
card showed whatever its name's spelling (change 51's D8). A test in `lingua-wasm` holds it. The rule
takes the place, for an Italian card whose lemma holds no space, of change 12's sentence that looks
every other card up as before.

### D6 — The Italian card, pinned on the real packs

`crates/lingua-wasm/tests/support/italian.rs` gains the card's probes over the committed tables
(change 16): one form per name of D2 — `parlo`, `parli`, `parla`, `parliamo`, `mangiate`, `parlavo`,
`parlò`, `parlerò`, `parlerei`, `parlassi`, `parlando`, `parlate` (a past participle with no entry of
its own), a present participle the committed tables file under its verb, `case`, `città`, `rosse`,
`grandi` —, a probe kept only where the committed tables file it under its verb (`parlata` and
`parlante` have entries of their own, noun and adjective, and read as those; where the committed
tables file no present participle under its verb, the present participle shows only as an « peut
aussi être » line naming the verb), and lemmas with two
sense runs or more, chosen from the committed tables; the golden is
re-blessed once, the pull request listing the probes added. `test/word-card-it-fr.spec.ts` renders
every grammar and phrase probe of `it-fr.golden` with the French renderer into
`test/baseline/word-card-it-fr.txt`, as `word-card-fr-en.spec.ts` does for fr-en; the harness
(`test/word-card-french.ts`, which hard-codes `"fr"`) takes the studied language's code; and the
pack-update workflow's re-bless step (`lingua-pack-update.yml`, the `yarn vitest run … -u` line)
names `test/word-card-it-fr.spec.ts`, so a re-reduction of it-fr re-blesses the snapshot with the
golden. What the snapshot
shows wrong in the data goes to change 17's list, or to the refinement after it.

### D7 — What does not move

Every form of the six other committed packs is rendered before and after the change and compared
(about 688,728 forms over the six packs, fr-en's and fr-es's 124,096 each with 51's 440,534); the other snapshots (`word-card-fr-en.txt`, `-fr-es.txt`,
`-es-en.txt`, `-en-es.txt`) and `selection-rows-fr.txt` pass without `-u`; `test/word-grammar.spec.ts` unchanged.

### D8 — Order, and what later changes take

Implemented after the implementations of changes 12 (the names), 13 (the readings) and 16 (the
baseline on the committed tables): before 16, every grammar probe answers no reading and the snapshot
would pin nothing. Later changes take: change 22 (`add-lingua-pack-it-en`) and change 27
(`add-lingua-pack-it-es`) — their baselines and the card snapshots in English and Spanish over their
goldens; until then the English and Spanish rows are held by unit tests.

## Open Questions

For the owner, none blocking (task 4.1): whether Italian merges its moods at all (M21 was settled for
French; `CARD_NAMES.it.moods` is proposed true), and whether a partial merge names `parli`'s shared
second person once (8,392 forms; a new rule of the description, no French or Spanish line moving).

## Risks / Trade-offs

- [French terms for Italian unfamiliar to a reader] → the terms French grammars of Italian use; the
  owner reads the French snapshot (task 4.1).
- [English "past historic (passato remoto)"] → M10's form for French; the bracket names the Italian
  tense the reader's textbook uses.
- [A table artefact read as a merge] → change 13 gives a `-` cell no form; the snapshot shows any
  other.

## Migration Plan

Nothing deploys: the extension ships the renderers with its next release, inert until change 20.
Rollback is a revert with the snapshot's and the golden's.

## Effort

3.5–5 ideal days, against the programme's 3.5–6: the type, `CARD_NAMES.it`, `PARTICIPLES.it` and
the three renderers' Italian rows 1–1.25; the harness and the workflow line 0.25; the merge and the participles checked on Italian 0.5; review's branch and its test 0.5;
the probes, the golden and the snapshot 0.5–1; the six packs rendered before and after 0.5–0.75;
spec and programme 0.25.
