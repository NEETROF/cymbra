# Design

## Context

See proposal.md — Why. Where things stand:

- `src/i18n/{fr,en,es}/grammar.ts`: each renderer holds `TENSES: Record<StudiedLanguageCode,
  TenseTable>` keyed `Mood/Tense` (`Ind/Pres`, `Ind|Sub/Pres`, `Cnd/`, `Imp/`…), `GERUNDS`, the
  participles' names, and the order a card names tenses in (`tenseOrder`); the language-neutral
  description (`src/reading/grammar-description.ts`: `finiteKey`, `formKind`, the M21 merge of change
  51's D3) turns a form's readings into keyed parts. `StudiedLanguageCode = "en" | "es" | "fr"`
  (`src/i18n/index.ts`). The French renderer's `fr` row exists for the override M2 reserves; no card
  has been named in French for a studied language yet.
- Change 51: French's rows, the `Ind|Sub` merge, the present participle, a language-neutral
  `selectWithinWord`, review's French no-space branch (`pack_gloss`, `crates/lingua-wasm/src/lib.rs`).
- Change 13: Italian's readings — `Tense=Past` (passato remoto), `Tense=Imp` (imperfetto),
  `Tense=Fut`, `Mood=Cnd` and `Mood=Imp` without a tense, `VerbForm=Ger`, `VerbForm=Part|Tense=Pres`,
  `VerbForm=Part|Tense=Past` with gender and number, unmerged.
- Change 12: Italian expressions keyed through Italian's reading; `pack_gloss`'s Italian arm for a
  lemma with a space. Change 16: the Italian baseline on the committed tables.
- **How it was measured**: change 13's prototype readings (`gram3.py`, scratch, the section's
  tables with the stress marks dropped), counting pairs that read `Mood=Ind` and `Mood=Sub` of one
  `Tense`, `Person`, `Number` — 10,580 (1pl present), 8,449 (2sg present), 1,550 (2pl present), and
  a few hundred table artefacts (a defective verb's `-` cells, given no form) —, and the gerund pairs
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

- The Italian interface's own card wording (Treccani's names, I6): change 33's.
- it-en's and it-es's snapshots (stage 2); the extension's studied languages (change 20).
- The polite imperative (`Lei`) named: change 13 stores none; a later change if the dogfood asks.

## Decisions

### D1 — Italian is a studied language the renderers name; the extension's type waits for change 20

`StudiedLanguageCode` gains `it`, so every `Record<StudiedLanguageCode, …>` table must hold an
Italian row — the type checker lists them. The extension's `StudiedLanguage` (what a page is read
as, what Réglages offers) stays until change 20, as change 51 left French's to change 52.

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
studied. The order is the grammars': indicative (present, imperfect, past, future), conditional,
subjunctive (present, imperfect), imperative; `Ind|Sub/Imp` is not listed — Italian's imperfect
subjunctive (`parlassi`) never spells the imperfect indicative (`parlavo`).

### D3 — The indicative and the subjunctive said once (M21, for Italian)

Change 51's merge reads the readings, not the language: where a form reads `Mood=Ind` and
`Mood=Sub` of one tense, persons and number, the description gives one `Ind|Sub` part. Italian
reaches it in the present: `parliamo` (first person plural, with the imperative), `parli` (second
person singular indicative and subjunctive; the first and third subjunctive apart), `diate`-like
second persons plural where a table spells them alike. The persons of one tense and number merge as
the card already merges them. No new rule: the `it` rows of D2 hold the `Ind|Sub/Pres` key.

### D4 — The gerund and the present participle

`GERUNDS.it`: « gérondif », "gerund", « gerundio »; the present participle's name exists since
change 51 (« participe présent », "present participle", « participio presente »), now reached by
Italian's `VerbForm=Part|Tense=Pres`. The past participle with its agreement is named as French's
and Spanish's are.

### D5 — An expression named without a space

`pack_gloss`'s no-space branch gains Italian beside French: a lemma without a space that Italian's
reading turns into two tokens or more, each a lemma (`reading_expression_key(lemma, Italian, …)`), is
read in the expression table at that key, the gloss of the headword it is named by first
(`expression_gloss_named`), and as a word where the table has nothing there. A one-token lemma
(`perché`, `po'` read `poco`) is a word. The extension's card already stores the gloss an expression
card showed whatever its name's spelling (change 51's D8). A test in `lingua-wasm` holds it.

### D6 — The Italian card, pinned on the real packs

`crates/lingua-wasm/tests/support/italian.rs` gains the card's probes over the committed tables
(change 16): one form per name of D2 — `parlo`, `parli`, `parla`, `parliamo`, `parlavo`, `parlò`,
`parlerò`, `parlerei`, `parlassi`, `parlando`, `parlante`, `parlata`, `case`, `città`, `rosse`,
`grandi` —, and lemmas with two sense runs or more, chosen from the committed tables; the golden is
re-blessed once, the pull request listing the probes added. `test/word-card-it-fr.spec.ts` renders
every grammar and phrase probe of `it-fr.golden` with the French renderer into
`test/baseline/word-card-it-fr.txt`, as `word-card-fr-en.spec.ts` does for fr-en. What the snapshot
shows wrong in the data goes to change 17's list.

### D7 — What does not move

Every form of the six other committed packs is rendered before and after the change and compared
(change 51 measured 440,534); the other snapshots (`word-card-fr-en.txt`, `-fr-es.txt`,
`-es-en.txt`, `-en-es.txt`) pass without `-u`; `test/word-grammar.spec.ts` unchanged.

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

3–4.5 ideal days, against the programme's 3.5–6: the type and the three renderers' Italian rows
0.75–1; the merge and the participles checked on Italian 0.5; review's branch and its test 0.5;
the probes, the golden and the snapshot 0.75–1.25; the six packs rendered before and after 0.5–1;
spec and programme 0.25.
