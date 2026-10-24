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
- **Measured again on the committed tables** (the implementation, on change 16's `tables/it/`, which
  keeps 2,865 verbs where the prototype read the section's every one): `grammar.tsv` holds 1,348
  form–lemma pairs read `Mood=Ind` and `Mood=Sub` in the first person plural of the present, 247 in
  the second person plural (`mangiate`), 1,782 read the indicative's second person singular beside
  the subjunctive's first (`parli`), and 2,673 gerund pairs. Opened on the dictionary form
  `forms.tsv` gives each of its 129,610 forms, the card says the moods once on 1,535 of them (1,330
  first persons plural, 205 second) and names them apart on 1,047 (`abbai`, `parli`), always the
  same pattern: the indicative's second person singular, the subjunctive's three; 71,690 forms show a
  grammar line, in each of the three interfaces, none of them a tag.

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
snapshot (task 4.1). Settled by the owner on 2026-10-25, as implemented: Italian's moods are merged
where their persons are the same (`parliamo`, `siamo`: « 1re personne du pluriel du présent de
l’indicatif ou du subjonctif »), and `parli`'s are named apart, the subjunctive's singular grouped
(*Open Questions*).

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
every other card up as before, and of its scenario *Every other card* where that scenario names an
Italian card on a single word: `d'accordo` is one written word and is no longer looked up as
before, while a one-token lemma still is (the review found the two texts at odds; the override
covers both).

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
re-blessed once, the pull request listing the probes added (implemented: 24 card probes and 40
lemmas, *Implementation* below). `test/word-card-it-fr.spec.ts` renders
every grammar and phrase probe of `it-fr.golden` with the French renderer into
`test/baseline/word-card-it-fr.txt`, as `word-card-fr-en.spec.ts` does for fr-en; the harness
(`test/word-card-french.ts`, which hard-codes `"fr"`) takes the studied language's code; and the
pack-update workflow's re-bless step (`lingua-pack-update.yml`, the `yarn vitest run … -u` line)
names `test/word-card-it-fr.spec.ts`, so a re-reduction of it-fr re-blesses the snapshot with the
golden. What the snapshot
shows wrong in the data goes to change 17's list, or to the refinement after it. Found at
implementation: `fate`, which the committed `tables/it/grammar.tsv` reads as *fare*'s second person
plural of the present subjunctive (its line 46129) beside the indicative and the imperative — the
subjunctive is `facciate` —, so the card says « 2e personne du pluriel du présent de l’indicatif ou
du subjonctif et 2e personne du pluriel de l’impératif de fare » on a common word: the renderer is
right and the data wrong, for change 17's list; and the committed it-fr table holds no `anch'io`, so
the golden's « anch'io » phrase shows its words' rows (`anche`).

### D7 — What does not move

Every form of the six other committed packs is rendered before and after the change and compared
(about 688,728 forms over the six packs, fr-en's and fr-es's 124,096 each with 51's 440,534); the other snapshots (`word-card-fr-en.txt`, `-fr-es.txt`,
`-es-en.txt`, `-en-es.txt`) and `selection-rows-fr.txt` pass without `-u`; `test/word-grammar.spec.ts` unchanged.
Re-run on the implementation (task 1.3): the six packs built from the committed tables, every form
of their studied language's `forms.tsv` opened on the dictionary form it gives, its readings, other
dictionary forms, pieces and sense headings rendered through `main`'s renderers (`4eebb6dad`) and
through this change's — 690,228 forms (en-fr's and en-es's 75,315 each, es-fr's and es-en's 144,952,
fr-en's and fr-es's 124,847: French's `forms.tsv` has grown since change 51's count) — give byte
for byte the same cards in their pair's interface language (35,033 en-fr, 87,705 es-fr, 87,705
es-en, 35,033 en-es, 66,684 fr-en and 66,684 fr-es forms showing a grammar line), and in all three
interface languages (2,070,684 cards, none moved).

### D8 — Order, and what later changes take

Implemented after the implementations of changes 12 (the names), 13 (the readings) and 16 (the
baseline on the committed tables): before 16, every grammar probe answers no reading and the snapshot
would pin nothing. Later changes take: change 22 (`add-lingua-pack-it-en`) and change 27
(`add-lingua-pack-it-es`) — their baselines and the card snapshots in English and Spanish over their
goldens; until then the English and Spanish rows are held by unit tests.

### Implementation

What the implementation decided where the decisions above left it open, and where it departs from
them:

- **The card's probes** (D6) are D6's seventeen — `parlo`, `parli`, `parla`, `parliamo`, `mangiate`,
  `parlate`, `parlavo`, `parlò`, `parlerò`, `parlerei`, `parlassi`, `parlando`, the present
  participle `parlanti` (filed under *parlare*), `case`, `rosse`, `grandi`, `città` — and seven more:
  `creduta` (the spec's Spanish scenario), `parlerebbe` and `parlato` (examples for the owner's read,
  task 4.1), `parlante` asked as itself (D6's « peut aussi être » line naming the verb), and, as
  French's card probes did, a determiner's plural without a gender (`tali`), a comparative (`minore`
  → *piccolo*) and a numeral (`mila` → *mille*): 24 in all, after the reference's 32. The lemmas with two sense runs or
  more are French's rule: the 40 commonest lemmas of `tables/it/freq.tsv` whose it-fr gloss has two
  runs or more (`tables/it-fr/senses.tsv`), in frequency order, none asked as itself already
  (`support/italian.rs`, `CARD_PROBES`, `MORE_LEMMAS`; `italian_baseline.rs` holds the order). The
  golden gains 64 probes and moves none: the review lines its reader shows hold no Italian card
  named without a space.
- **The renderers' unit tests** live in one spec, `test/word-grammar-italian.spec.ts`, which names an
  Italian form in the three interfaces (the spec's scenarios, every tense in its order, the
  participles, the infinitive, the unnamed forms) and reads change 13's committed tag pool
  (`tables/it/tags.tsv`, 83 tags): each tag is named by the three renderers or by none — a
  numeral's, `DET|Number=Plur`, a singular without a gender —, and no line shows a tag.
  `test/word-grammar.spec.ts`, `word-grammar-en.spec.ts` and `-es.spec.ts` are unchanged;
  `grammar-description.spec.ts` gains *what an Italian card names* and runs *What each renderer
  names* for `it`; `i18n.spec.ts` checks each draft against the French over an Italian word.
- **The harness** (`test/word-card-french.ts`) takes the studied language's code as an argument
  every caller passes, so `word-card-fr-en.spec.ts` and `-fr-es.spec.ts` pass `fr`: their snapshots
  do not move.
- **Review's branch** (D5): `pack_gloss`'s no-space branch reads the key through
  `reading_expression_key` for French and Italian alike — the function `french_expression_key`
  wraps for French, so French's cards read as before. Its test is in `tests/languages.rs`, beside
  change 12's, over the it-fr fixture re-stamped as it-en: `d'accordo` and `anch'io` show it-en's
  glosses, `perché` its word's.
- **Known conventions, kept** (the review's notes; each a possible later refinement, none changed
  here, since each would move other studied languages' cards or needs data this change does not
  hold):
  - in French, the persons of one tense come plural first (`numbers: ["Plur", "Sing"]`, the French
    renderer's convention for every studied language): `sono` reads « 3e personne du pluriel du
    présent de l’indicatif et 1re personne du singulier du présent de l’indicatif d’essere »;
  - the French renderer never names two genders of one number once (change 51's D6 merges them in
    the English and Spanish renderers only): `grandi` reads « pluriel, féminin pluriel et masculin
    pluriel de grande », `parlanti`'s « peut aussi être » line « le pluriel et le masculin pluriel
    de parlante »;
  - a numeral's form is unnamed on every card: `mila`, `milioni` and `miliardi` (`NUM|Number=Plur`)
    give no « pluriel de mille » line;
  - a determiner's plural without a gender is unnamed, so `tali`'s only line is another dictionary
    form's, the rare « peut aussi être le masculin pluriel de talo ».

## Open Questions

For the owner, none blocking (task 4.1): whether Italian merges its moods at all (M21 was settled for
French; `CARD_NAMES.it.moods` is proposed true), and whether a partial merge names `parli`'s shared
second person once (8,392 forms; a new rule of the description, no French or Spanish line moving).

Settled by the owner on 2026-10-25, having read `test/baseline/word-card-it-fr.txt` (task 4.1), each
as implemented:

- **Italian merges its moods** where the indicative and the subjunctive of one tense hold the same
  persons in one number: `CARD_NAMES.it.moods` stays true, `parliamo` reads « 1re personne du
  pluriel du présent de l’indicatif ou du subjonctif et 1re personne du pluriel de l’impératif de
  parlare » and `siamo` « 1re personne du pluriel du présent de l’indicatif ou du subjonctif
  d’essere ».
- **No partial merge for `parli`**: its moods stay apart, the subjunctive's singular grouped —
  « 2e personne du singulier du présent de l’indicatif et 1re, 2e et 3e personnes du singulier du
  présent du subjonctif de parlare »; the description gains no rule.
- **The French terms are approved** as D2 lists them: « présent de l’indicatif », « imparfait de
  l’indicatif », « passé simple », « futur », « conditionnel », « présent du subjonctif »,
  « imparfait du subjonctif », « impératif », « gérondif », « participe présent », « participe
  passé », and the merged « présent de l’indicatif ou du subjonctif ».

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
