# Design — add-lingua-french-word-card

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `apps/lingua-extension/src/reading/grammar-description.ts` (change 18) | the description: `describeForm` merges readings by tag (persons merged, the dictionary form left out on its own card); what is named is decided here and keyed by studied language — `NAMES_INFINITIVE` (`en: false, es: true`), `finiteKey` (Spanish: the conditional and the imperative without a tense; any other language: the indicative only, `Ind/<Tense>`), `formKind` (a participle only when its tense is `Past`, the gerund always), `tenseOrder` (Spanish's table order, English's the pack's), `nameReadings` (the persons of one tense and number named once, by tense key or, for the French renderer, by tense name), `composeLines` |
| `src/i18n/{fr,en,es}/grammar.ts` | the renderers, `TENSES: Record<StudiedLanguageCode, TenseTable>` for `en` and `es`, `GERUNDS`, Spanish's `PARTICIPLES` (« participio pasado » for English since change 24, « participio » for Spanish); English names two genders of one number once (`mergeGenders`, change 23's « gran »), Spanish does not — no English reading carries a gender |
| `src/i18n/index.ts` | `StudiedLanguageCode = "en" \| "es"`, the renderers' key; `src/analyzer/types.ts` `StudiedLanguage = "en" \| "es"`, the extension's, left un-widened by change 47 (its D4: « change 51 widens the renderer's code, change 52 widens `StudiedLanguage` ») |
| `src/reading/wordpopup.ts` | `renderGrammar` hands the content's `language` (`StudiedLanguage`, English when absent) to the interface language's renderer, and sets it as `lang` on each studied word |
| Tests | `test/word-grammar.spec.ts`, the French renderer's 108 assertions, unchanged since change 18; `word-grammar-en.spec.ts`, `-es.spec.ts`; `grammar-description.spec.ts` (*What each renderer names*: every tag of the vocabulary — over 2,000 — for `en` and `es` studied, each renderer naming what the French one names); `word-card-es-en.spec.ts`, `-en-es.spec.ts` and their snapshots (changes 23, 24); `selection-rows-fr.txt` |
| The selection | `selection.ts` `captureFrom` widens a selection to whole words over `WORD_CHAR` (`[\p{L}\p{N}'’-]`) and keeps only the widened range; `classifySelection` calls it a « word » without whitespace; `session.ts` `onCapture` resolves a « word » at the widened range's start (`hitAt`, `findTokenAt` made half-open by change 40); `selection-card.ts` `openForSelection` opens a word card, or the whole-selection card (`openExpression`) for text holding whitespace; `expressionCard` makes an expression covering the whole selection the card's answer (`expression: false`, headed by the match's key); `rowsFor` leaves function words and settled words out |
| `cardGloss` | `selection-card.ts`: a card keyed by text holding a space stores the gloss it showed; any other asks the single-lemma port (`ports.gloss`) |
| Review | `crates/lingua-wasm/src/lib.rs` `readable_gloss` (M4): a card whose lemma holds a space (`Card::is_expression`) reads the expression table, any other the gloss table; change 44 adds French's arm: a French card whose lemma holds a space reads the table at the key its name reads as (`french_expression_key`) |
| The French golden | `crates/lingua-wasm/tests/french_baseline.rs`, `baseline/fr-en.golden`: 143 probes on `main` since change 41's implementation, 31 `word-grammar` and 25 `phrase-gloss` among them (change 44 adds 4), over the hand-written fixture until change 48 switches it to the committed tables (`"readings":[]` until then); the engine starts on es-en beside it |
| The workflows | `lingua-extension-check.yml` runs `english_baseline`, `spanish_baseline`, `es_en_baseline`, `en_es_baseline` and `french_baseline`; `lingua-pack-update.yml` re-blesses them and the two card snapshots (`yarn vitest run test/word-card-es-en.spec.ts test/word-card-en-es.spec.ts test/row-gloss-tables.spec.ts -u`) |

**What change 45 gives the card** (its design): 125,193 readings of 88,579 French forms, 79 tags; a
verb form's mood, tense, person and number, the passé simple as `Tense=Past`, the conditional and
the imperative with no tense (Spanish's encoding, its D7), the infinitive, the present participle
`VerbForm=Part|Tense=Pres`, the past participle with its agreement; a noun's gender, both numbers on
a noun spelled alike (`temps`); the agreement of adjectives, determiners, pronouns and numerals;
`ADJ|Degree=Cmp`/`Sup` on 9 readings; `other` marks toward another kept word (9,284). « parle »
carries five readings, unmerged, for this change (its D7 and D13).

**How it was measured.** A prototype, never committed, in a scratch copy of `origin/main` at
`cf392473` (French at analyser `0.2.0`), run again at `da94a82e`, after change 41's implementation
(#832, French at `1.0.0`):
- `tables/fr/` from change 43's implementation (`claude/add-lingua-french-forms-tables-impl` at
  `2ed01a18`: 124,050 forms, 60,000 lemmas; merged since as #829, whose table holds 124,096 forms —
  863 rows differ, most a participle mapped to its verb, `abaissée` → *abaisser*, none of them a probe
  of D9 — so the counts below are taken again on the committed tables, task 4.5) with change 45's
  prototype readings on those tables
  (125,191 readings, the same 79 tags and 2,142 five-reading forms, its D9 cross-check), change 46's
  prototype levels, change 48's prototype fr-en native side and change 49's prototype fr-es native
  side (both before their implementations, so the glosses are indicative); packs built by
  `lingua-pack-build`;
- a scratch binary asking `engine::word_grammar` for every form of a forms table, opened on the
  dictionary form the page analysis gives it — the 124,050 French forms, and the 440,534 forms of
  en-fr, es-fr, es-en and en-es from their committed tables;
- the extension's renderers with this design's rules, run under vitest on those answers, and on
  `main`'s renderers for the comparison;
- the French golden switched to the tables (change 48's switch), the card's probes added, and a
  scratch `fr_es_baseline.rs`; the snapshots rendered by copies of `word-card-es-en.spec.ts`.

Every card answer is the same at both commits: a form's dictionary form comes from the forms table,
which change 41's cascade does not change, and the four other pairs' 440,534 answers are byte for
byte. Change 41's closed classes already leave `le`, `que` and `il` out of the whole-selection
cards' rows (« l’homme » shows `homme` alone, « dit-il » `dire`); change 44 is not implemented, so
those cards still show `à le` « a la, in the style… » for « Au revoir » and `il y avoir` for « il y
a »: the snapshots are blessed after it (D9).

## Goals / Non-Goals

**Goals:**
- Every French reading change 45 writes named in English and in Spanish, or left unnamed by a
  decision this design states and measures.
- M21's « moods merged on five-reading forms ».
- Change 40's and change 44's hand-overs: a selection inside a word written as pieces; an
  expression named without a space.
- fr-en's and fr-es's cards bounded by goldens and snapshots, as es-en's and en-es's are.
- en-fr, es-fr, es-en and en-es cards, and the French interface, byte for byte.

**Non-Goals:**
- Shipping French: `StudiedLanguage`, the labels, `packs.json`, Réglages, the dogfood on devices
  (52).
- The pairs' glosses (48, 49 and their refinements): what the snapshots show wrong in them is listed,
  not fixed.
- Wording every pair shares (a bare plural beside gendered ones): listed for change 56. An invariable
  plural on Spanish and English cards, listed here at first, is settled by
  `refine-lingua-card-invariable-plurals` (51b), on `main` before this change's implementation.
- `Pack::readings` for a lemma that is another word's form (change 45's open question 4):
  `fix-lingua-lemma-lookup` (41b), implemented before 45, 46 and 48.
- A conjugation table, choosing a reading from context.

## Decisions

### D1 — French is a studied language the renderers name; the extension's type waits for 52

`StudiedLanguageCode` (`src/i18n/index.ts`) becomes `en | es | fr`. Measured with `tsc --noEmit`
on a copy of `main` (`e3d6eb4a`) with the type alone widened, it breaks the `Record<StudiedLanguageCode,
…>` tables lacking `fr` — the three renderers' `TENSES` and `GERUNDS`, Spanish's `PARTICIPLES` and the
description's `NAMES_INFINITIVE` — and one test fixture (`grammar-description.spec.ts`'s tense table,
twice): ten errors, nothing else. D4's new kind adds a case to each renderer's `name`.
`wordpopup.ts` hands the renderer a `StudiedLanguage`, which `en | es | fr` still holds.
`StudiedLanguage` stays `en | es`, as change 47 left it for change 52: no content a page builds can say `fr` before then, so the renderers' French is
reached by tests only, and change 52 widens one type that the card already accepts.

Each renderer gets a French table, the French one included: change 18 drafted every renderer for
every studied language (English's has `en`, Spanish's has `es`) — a native cannot study their own
language, and those tables serve *What each renderer names* and the interface override M2 reserves;
change 47 did the same for the `french` entry of `src/i18n/fr/languages.ts`.

What French names is decided in the description, keyed by studied language as `NAMES_INFINITIVE` is
— one table, `CARD_NAMES[studied]` (not `profile.ts`'s `NAMES`, the backup's language names that
change 52 widens with `StudiedLanguage`): whether the infinitive, the present participle and the gerund are
named, whether the moods merge (D3), whether a plural spelled like the dictionary form is said on
its own card (D5). English and Spanish keep today's values, so their cards cannot move by
construction; the measurement says they do not (D10).

*As implemented* (`refine-lingua-card-invariable-plurals` D3, on `main` first): `CARD_NAMES` holds
the infinitive, the present participle, the gerund and the moods' merge — `{ infinitive,
presentParticiple, gerund, moods }`, English `false, false, true, false`, Spanish `true, false,
true, false`, French `true, true, false, true` — and no plural: 51b's `isInvariablePlural`, the
description's for every studied language, leaves out a plural read beside its singular, so D5's first
row is met with no key and no `composeLines` arm.

### D2 — French's tenses, and their order

`finiteKey` keys French as Spanish: `Mood/Tense`, the conditional and the imperative with no tense,
as change 45 stores them. The tables, listed in the order the card names them — the grammars' order,
indicative, conditional, subjunctive, imperative; `tenseOrder` reads every studied language but
English from its table:

| Key | English | Spanish | French (M2's reserve) | Forms | Lemma in top 1,000 / 5,000 |
|---|---|---|---|---|---|
| `Ind/Pres` | present indicative | presente de indicativo | présent de l’indicatif | 4,114 | 269 / 1,475 |
| `Ind\|Sub/Pres` (D3) | present indicative or subjunctive | presente de indicativo o de subjuntivo | présent de l’indicatif ou du subjonctif | 7,520 | 358 / 1,982 |
| `Ind/Imp` | imperfect indicative | pretérito imperfecto de indicativo | imparfait de l’indicatif | 5,722 | 401 / 2,327 |
| `Ind\|Sub/Imp` (D3) | imperfect indicative or subjunctive | pretérito imperfecto de indicativo o de subjuntivo | imparfait de l’indicatif ou du subjonctif | 33 | 4 / 31 |
| `Ind/Past` | past historic (passé simple) | pretérito perfecto simple de indicativo | passé simple | 3,855 | 288 / 1,468 |
| `Ind/Fut` | simple future | futuro simple de indicativo | futur | 5,184 | 499 / 2,865 |
| `Cnd/` | conditional | condicional simple | conditionnel | 2,663 | 369 / 1,680 |
| `Sub/Pres` | present subjunctive | presente de subjuntivo | présent du subjonctif | 1,325 | 247 / 946 |
| `Sub/Imp` | imperfect subjunctive | pretérito imperfecto de subjuntivo | imparfait du subjonctif | 482 | 92 / 209 |
| `Imp/` | imperative | imperativo | impératif | 6,485 | 348 / 1,997 |

« Forms » counts the French forms whose card names that tense on a line, its own or an « also » line,
the persons merged; a form naming two tenses counts under both. The non-finite names: the past
participle on 11,020 forms (339 / 2,297), the present participle on 2,836 (81 / 506, D4), the
infinitive on 34 (1 / 11) — Spanish's choice, a French infinitive spelled otherwise than its
dictionary form being mostly a 1990 spelling (`abimer`, `accroitre` → `abîmer`, `accroître`).
Measured again on the implementation (task 4.5) over the committed tables — the 124,096 forms of
`tables/fr/forms.tsv`, each opened on the dictionary form the page analysis gives it, through this
change's description and renderers: the figures above; the prototype's, on change 43's branch
(124,050 forms), were 4,114 (269 / 1,472), 7,522 (360 / 1,977), 5,722 (401 / 2,323), 33, 3,855
(290 / 1,466), 5,184 (499 / 2,863), 2,663 (369 / 1,678), 1,325, 482, 6,486 (348 / 1,992), and
10,945 (347 / 2,268) past participles — the participles mapped to their verb by the committed table
(`abaissée` → *abaisser*) account for most of the moves.

**English** follows the English Wiktionary's French form-of wording, as change 18's D4 asks: in the
French section the prototype read, "past historic" (36,304 glosses), "imperfect indicative"
(18,834), "simple future" (29,891, beside "future" 8,838 and "future indicative" 50), "conditional"
(32,298), "imperfect subjunctive" (36,026), "present participle" (6,320) — but for the passé
simple, which M10 settles: "past historic (passé simple)". Its terms are quoted as the card writes
them — « forma en -ing » is the Spanish card's literal name for an English gerund — and the
parenthesis sits inside the quotation, where « RAE numbers (20 000; 96 %) » gives its examples outside
any; the architecture's "past historic" for fr-en is its short form. The French words are interface
text, unmarked, as « -ing » is on a Spanish card of English (change 24). The cost is 15 characters on
the lines of 3,855 forms; nothing measured argues against it. Persons read as change 18 wrote them:
"first- and third-person singular".

**Spanish** uses the RAE's names (M10), the Spanish card's own for Spanish (« futuro simple de
indicativo », « condicional simple »), where the Spanish Wiktionary's French section writes
« futuro de indicativo » and « condicional »; its other names are the Wiktionary's — « presente de
indicativo » (381), « presente de subjuntivo » (334), « pretérito perfecto simple de indicativo »,
« pretérito imperfecto de indicativo », « pretérito imperfecto de subjuntivo », « imperativo »
(161). No pronoun after the person (« (je) »), as change 24 decided for English.

**French** is the French card's Spanish table without the future subjunctive, French having none,
and with the two merged tenses. It is shown to no one.

*Rejected — one table per pair, outside the renderers.* The architecture keys tense names by pair;
a renderer is its native language, so `TENSES[studied]` in each renderer is the pair key, as
change 18 built it.

### D3 — The indicative and the subjunctive said once (M21)

**The rule.** Where a form's readings name the indicative and the subjunctive of one tense, with the
same persons and number, the card names them once, as one tense: `Ind|Sub/<Tense>`, whose name each
renderer's table gives (D2). Everything else is named as before:
- « parle »: present indicative first and third singular, present subjunctive first and third
  singular, imperative second singular → "first- and third-person singular present indicative or
  subjunctive and second-person singular imperative of parler"; « primera y tercera persona del
  singular del presente de indicativo o de subjuntivo y segunda persona del singular del imperativo
  de parler »;
- « parlent » → "third-person plural present indicative or subjunctive of parler";
- « finissions » (imperfect indicative and subjunctive, present subjunctive, first plural) →
  "first-person plural imperfect indicative or subjunctive and first-person plural present
  subjunctive of finir";
- « soyez » (subjunctive and imperative) and « vis » (present and past historic) are not merged:
  their tenses or moods differ otherwise.

**Where.** The description's merge by tag already reduces « parle »'s five tags to three readings
— the imperative, second person; the present indicative and the present subjunctive, each of persons
1 and 3 —, as it does every one of the 2,142 five-reading forms (measured). The moods merge in
`nameReadings`, once the persons are merged by tense and number:
two groups keyed `Ind/T` and `Sub/T`, of one number and the same persons, become one group keyed
`Ind|Sub/T` at the place of the first, when `CARD_NAMES[studied]` merges moods (French) and the renderer
names the merged key. The merged name is a tense name, so « only tense names are keyed by pair »
still holds. Not in the tags (`describeReadings`): the description merges by tag, and a tag holds one
mood (change 45's D7, point 1); not in the core, which answers readings unmerged (change 40's D1).

**Why "or".** The form is one mood or the other, by its sentence; "and" already joins the list and
the persons. The English Wiktionary writes the same merge itself for French, "present
indicative/subjunctive" (11,223 glosses); change 40's D1 wrote « present indicative or subjunctive ».

**Measured.** 6,147 forms name the merge on their own line — 157 of them forms of the 1,000
commonest lemmas, 1,290 of the 5,000 — and 1,408 more on an « also » line only (`couvent`, the noun,
« may also be the third-person plural present indicative or subjunctive of couver »). Of the 6,147,
2,142 are the five-reading forms M21 names, the others 2,729 third persons plural (`parlent`), 1,080
second persons singular (`parles`), 162 like `finissent`, 33 imperfects (`finissions`) and one other
(change 45's D7). No two readings of one tense and number share some persons but not all: the rule
never has to choose.

*Measured again on the implementation* (task 4.5, the committed tables): 6,145 forms name the merge
on their own line, 157 / 1,297 of them forms of the 1,000 / 5,000 commonest lemmas, and 1,408 more
on an « also » line only; of the 6,145, 2,141 are five-reading forms, 2,890 third persons plural
(`parlent`, `finissent`), 1,080 second persons singular (`parles`), 33 imperfects (`finissions`: 20
first, 13 second persons plural) and one first person plural of the present.

**Wider than M21's words, stated.** M21 settles « moods merged on five-reading forms »; the rule merges
them, and the same two moods on 4,005 more forms (`parlent`, `parles`, `finissions`). Change 45's D7
leaves that to this change (« whether change 51 merges the moods on those 4,005 too is its call »);
it is read as what M21 asks of a French card rather than a limit, and the owner confirms it (open
question 1).

*As implemented:* `mergeMoods` in `grammar-description.ts`, called by `nameReadings` once the persons
are grouped, with `mergedMoodsKey` (`Ind/T` → `Ind|Sub/T`); a merged group takes the earlier of the
two places, and the places are filled in the pack's order, so the order of tenses holds
(`grammar-description.spec.ts`, *what a French card names*).

*Rejected — merging the five-reading forms only.* « parle » would be said once and « parlent »
twice, one phenomenon read two ways on one page; the rule reads the readings, never their count.
*Rejected — Spanish too.* No Spanish or English form meets the rule (measured: none of es-fr's 144,952
forms nor of en-fr's 75,315), but « hable »'s line is change 18's scenario; `CARD_NAMES.es.moods` stays
false, so Spanish could only move by a change of its own.

### D4 — The present participle, and no gerund

`formKind` names `VerbForm=Part|Tense=Pres` as a kind of its own where `CARD_NAMES[studied]` says so
(French): "present participle of parler", « participio presente de parler », « participe présent
de parler ». 2,836 forms carry it (2,393 on their own line: `parlant`, `étant`, `ayant`, `prenant`).
English and Spanish tables hold no such reading (measured), and their flag stays false.

The Spanish card names a French past participle « participio pasado », as it names English's (change
24), beside « participio presente »: the RAE's bare « participio » is Spanish's own, and would read as
the generic term beside a present participle.

French has no gerund: « en parlant » is the participle after « en » (change 45's D3), so
`VerbForm=Ger` is never named for French.

« participio presente », not « gerundio »: the Spanish Wiktionary's French entries write both
(« Gerundio de charmer » 7 times, « participio presente » twice), and a Spanish gerundio is what the
French gérondif translates; the RAE's « participio de presente » (its dictionary's entry) is the other
choice (open question 2).

### D5 — What a French card leaves unnamed

| What | French readings | What the card does | Why |
|---|---|---|---|
| A plural spelled like the card's own dictionary form (`temps`, `pays`, `heureux`, `plusieurs`) | 1,072 forms, 71 / 209 of the 1,000 / 5,000 commonest — 1,008, 69 / 201, under 51b's boundary as implemented (below) | no line on that form's own card; its other lines kept (`fils` still says the plural of `fil`) | the line only says the word looks the same in the plural, and on French's commonest words it is a homograph noun's (M8): `un` (rank 7) « may also be the masculine plural of un » through the number, `pas` the noun « step », `plus`, `si`, `nous`, `non`, `comment`, `mieux`, `vers` |
| A numeral's form (`millions`, `cents`, `trente-et-une`) | 14 readings, 14 forms | unnamed, as a Spanish card leaves Spanish's 19 | `cents` would read « masculine plural and plural of cent » beside its noun reading; `trente-et-une` « feminine singular of trente », a lemma change 43 gives it |
| A determiner's or pronoun's plural without a gender (`les`, `ces`, `ses`, `nos`, `quelques`) | 15 readings | unnamed, as `formKind` leaves Spanish's (`sus`) | the same rule for every language; with change 41 most are function words, not painted |
| A comparative or a superlative (`meilleures`, `moindre`, `pire`, `génialissime`) | 9 readings | named as such, its agreement unsaid | `meilleures`' own line already says « feminine plural of meilleur », and « may also be the comparative of bon » follows |

The first is keyed by studied language (`CARD_NAMES[studied]`): it is applied to the readings of a card
opened on its dictionary form, where the readings stay what they are (`les temps` is a plural). Spanish
cards say it of 667 forms (13 of the 1,000 commonest: `tu`, `menos`, `nadie`, `crisis`, `lunes`) and
English cards of 194 (7: `head`, `young`, `police`): leaving it out there moves en-fr and es-fr cards,
a change of its own if the owner wants it (open question 3). Measured, 1,007 of the 1,072 French forms
then show no grammar line at all — the other 65 keep another word's line, `fils` the plural of `fil`,
`bois` *boire*'s —, 65,883 forms showing one.

*As implemented* (`refine-lingua-card-invariable-plurals`, 51b, on `main` first; its D3): the first
row is 51b's rule, not keyed by studied language — a plural is left out where the card also reads the
form as a singular of that part of speech, so `gens`, `environs`, `plusieurs` and `frais` keep their
line. Measured on the implementation over the committed tables: 1,008 French cards lose the line, 69 /
201 of the 1,000 / 5,000 commonest (`un`, `pas`, `plus`, `si`, `nous`, `non`, `temps`, `fois`); 964
of them then show no grammar line, and 44 keep another line — another word's (`fils` → *fil*, `bois`
→ *boire*, `plus` → *plaire*, `as` → *avoir*), or their own feminine singular (`pop`, `antipersonnel`)
—, more than 51b's 28 because this change names French's finite forms, which 51b's measurement left
unnamed. 66,002 French forms show a grammar line in English. Numerals: 14 readings of 14 forms;
genderless plural determiners and pronouns: 15 readings; comparatives and superlatives: 9 readings —
as the table says. Open question 3 is settled by 51b for English and Spanish cards.

### D6 — The Spanish card names two genders of one number once

*As implemented*, the English renderer's merge moves into the description as `mergeGenders(names,
agreed, named, order)`, which the English and Spanish renderers call with their own words; the
English lines are byte for byte (D10). French gives a noun of both genders a reading per gender (change 45's D4), and a word may be the
plural of a feminine and of a masculine noun spelled alike (`sommes`: *somme* « sum », *somme*
« nap »). The Spanish renderer would say « también puede ser el femenino plural y el masculino plural de
somme »; it gains the English renderer's merge (change 23's « gran »): « el masculino y femenino
plural de somme ». 951 French forms (951 again on the implementation). No English reading carries a gender, so en-es's lines cannot
move (measured, D10).

A bare plural beside the same number's gendered ones is the wording every Romance card already shows
and this change leaves: on 793 French forms (791 on the committed tables; an adjective of one form for both genders beside a noun),
388 of them « plural and masculine and feminine plural of abolitionniste », the others as « plural and
masculine plural of acide », as es-fr says « pluriel,
féminin pluriel et masculin pluriel d’abolicionista » on 1,027 Spanish forms today, and es-en the
same in English — wording for change 56, since fixing it moves es-fr and es-en.

### D7 — A selection inside a word written as pieces

Change 40's D7 hands this over: within an analysed block, `l’homme` and `dit-il` are read as pieces
with spans of their own, `le` [0, 4) and `homme` [4, 9), but `captureFrom` widens a selection over
`WORD_CHAR`, which holds `’` and `-`, and `onCapture` resolves the widened word at its start.

**The rule**, for a one-word selection — no whitespace — in an analysed block:
1. the pieces of the widened word are the page's tokens inside it, painted or not (the block's tokens,
   as the reclassify hit reads them), grouped by span;
2. one span, or pieces sharing one (`don't`, `del`, `au`): as today, the word card at the widened
   start;
3. the selection as the reader made it lies inside one piece: that piece's card — a drag over
   « homme » opens `homme`, over « il » in « dit-il » opens `il`;
4. it covers several pieces: the whole-selection card, read by the phrase gloss as any selection is —
   a double-click on « l’homme » where the browser selects it whole (Unicode's word breaks join
   letters around an apostrophe) shows the row `homme`, `le` being a function word after change 41;
   a double-click on « d’abord » shows the expression `d'abord` as its answer (change 44). A browser
   that stops a double-click at the apostrophe selects « homme » alone, and rule 3 opens `homme`.

A selection in a block the page analysis did not cover keeps *A selected word resolves to its
dictionary form* (`add-lingua-phrase-gloss`): the analyser's answer carries no spans, so pieces
sharing one (`don't`) could not be told from pieces with their own; such blocks are short or not in
the studied language.

**How.** `Capture` keeps the selection as the reader made it (`selected: Range`) beside the widened
range; the page's own selection is still never touched. `onCapture` routes as above before
`openForSelection`, which gains the whole-selection route for a word. The hit test of a click
(change 40's `findTokenAt`) is unchanged.

*As implemented:* the rule is `scan.ts`'s `selectWithinWord(block, tokens, word, selected)`, beside
`findTokenInBlock`: the block's tokens whose ranges lie within the widened word, grouped by span (the
first of a shared span standing for it, as a click opens it), and the spans the reader's own range
overlaps — one: that piece (`{ kind: "piece" }`); several: the whole-selection card
(`{ kind: "pieces" }`); one span only, or a selection touching none (a hyphen between two pieces):
as before (`{ kind: "word" }`). `session.ts` reads the block as the reclassify hit does
(`blockTokensAt`, shared with it), and `openForSelection(sel, hit, whole)` takes the whole-selection
route. Overlap rather than containment, so a selection that takes in a blank beside its piece still
opens it.

**What moves.** Measured on the goldens' corpora — every written word, as `WORD_CHAR` widens it, whose
tokens have two spans or more: French 72 of 992 — 71 on the implementation, change 42's guard having
since refused the `mixte` page's Occitan block and its « l'aviá » — (59 elisions such as `l'INSEE`, `n'est`, `C’est`,
`jusqu'au`; 9 hyphenated pronouns, inversions such as `dit-il`, `a-t-il`, `Viendront-ils` and
imperatives such as `coupez-les`; 4 both, `Qu’est-ce`,
`Va-t'en`, `Donne-m'en`, `l'arc-en-ciel`); English 0 of 867 (en-fr, en-es), Spanish 0 of 585
(es-fr, es-en). The rule is language-neutral, so English and Spanish could move only on a hyphenated
run holding a digit — the tokeniser's language-neutral digit rule gives its pieces their own spans
(`24-year-old`: a drag over « old » opens `old` instead of `year`, the same fix). None is in the
corpora. `test/selection.spec.ts`'s case pinned by change 40 moves: its assertions on the widened
range and text (`l’homme`) hold, it gains the reader's range (`homme`), and its comment — the word
resolved at its start, `le` — is rewritten; the routing itself moves in the session's tests.

*A selected word resolves to its dictionary form* (`add-lingua-phrase-gloss`, open) says a hyphenated
selection opens the word card « from its page token when there is one »; rule 4 opens the
whole-selection card over a word whose pieces have spans of their own (`dit-il`, `Va-t'en`). That
change is open, so nothing here MODIFIES it: *Selection capture on any pointer*'s new sentence takes its
place for such a word, and its wording is best narrowed when it archives (open question 5).

*Rejected — stopping the widening at an elision.* It needs change 40's table of elided forms in the
extension, to tell `l’homme` from `aujourd’hui`, and nothing for an inversion.
*Rejected — a word card on the first piece that is neither elided nor a pronoun* (change 40's open
question 2): page tokens carry no function-word flag, and « + Deck » on `homme` from a selection of
« l’homme » would key a card by a word the reader did not select alone.

### D8 — An expression named without a space

Change 44 names a French expression by its headword; `d'abord`, `c'est`, `allez-y` hold no space.
Two consequences:
- **« + Deck » on the whole-selection card answered by `d'abord`** asks `ports.gloss("d'abord")`,
  since `cardGloss` tells an expression by a space, and the single-lemma port knows no `d'abord`:
  the card would be stored without the gloss *An expression is the card's answer* says is stored.
  The content a whole-selection match builds says it is the expression table's answer, the gesture
  carries it, and `cardGloss` stores the gloss the card showed for it, whatever its name's spelling;
  a word card still asks the pack.
- **Review** (M4) reads a card's pack gloss at its lemma: `d'abord` is no word of the gloss table, and
  `Card::is_expression` reads a space. Change 44's implementation (#835) reads the expression table
  through `french_expression_key` only in `readable_gloss`'s `is_expression()` branch; this change adds
  the case to the other branch, for a French pack: a lemma holding no space for which
  `french_expression_key` answers a key — it reads as two tokens or more, each a lemma — reads the
  expression table at that key (`de abord`); a lemma it answers `None` for (`aujourd'hui`, one token) is
  looked up as a word, as before. Nothing of 44's is written again: the key is its function, the core
  does not move, and `Card::is_expression` keeps its meaning for every other caller. Change 44's
  *Review finds a French expression card by its name* says « every other card SHALL be looked up as
  before »; 44 is open, so this change ADDs its case beside it rather than MODIFYING it, as 44 did with
  the requirements it narrowed (open question 5).

No baseline holds such a card (the French golden's cards are `horizon`, `récolte`, `rassurer`), so no
golden moves: a test in `crates/lingua-wasm/tests/card_gloss_language.rs` holds both cases.

*As implemented:* `expressionCard` marks the content of a whole-selection match `expressionAnswer`,
the card's gesture carries it (only when true, so every other gesture is what it was), and
`cardGloss` stores the gloss shown for a key holding a space or for such a gesture. In
`readable_gloss`, a French card that is no expression reads the table at `french_expression_key`'s
key when it gives one and the table holds it, and is looked up as a word otherwise. The two scenarios
run over fr-en's committed pack, which holds `de abord` named `d'abord` (change 44).

### D9 — The French card, pinned on the real packs

**Probes.** The French scenario (`tests/support/french.rs`) gains:
- 21 grammar probes, one for each name the card says or leaves unsaid on purpose: `parle`, `parles`,
  `parlent`, `parlait`, `parlera`, `parlerait`, `parlant`, `parlé` (*parler*), `dirigée` (*diriger*),
  `finissions` (*finir*), `sommes`, `suis` (*être*), `grandes` (*grand*), `maisons` (*maison*),
  `cette` (*ce*), `les` (*le*), `meilleures` (*meilleur*), `moindre` (*petit*), `millions`
  (*million*), `temps`, `heureux`;
- 40 lemma probes, `word-grammar <lemma> <lemma>`, so that their glosses and sense runs are the
  engine's: the commonest lemmas of `tables/fr/freq.tsv` whose fr-en or fr-es gloss has two sense
  runs or more, those the reference asks as themselves left out — chosen once on the committed
  tables (on the prototype's: `de`, `le`, `en`, `des`, `un`, `que`, `pas`, `qui`, `ce`, `sur`…);
- a phrase probe, « l’homme », the whole-selection card of D7.

The 31 reference probes, among them `porte` (*porter*, five readings) and `vis` (*vivre*, *voir*),
stay. `été été` and `porte porte` are pinned as `fix-lingua-lemma-lookup` (41b, PR #834, before 45, 46
and 48) leaves them: a string the pack holds only as another word's form reads nothing, so neither
shows *être*'s or *porter*'s readings or gloss — the prototype's snapshot, made before it, pins both
(« may also be the past participle of été » with « to be »), which the implementation's must not.
`fr-en.golden` gains the 62 probes; measured on the prototype — `main`'s 143 probes, before
change 44 adds its 4, after change 48's switch — 143 → 205 probes, none of the 143 moving.
*A French invariance baseline runs beside the English and Spanish ones*, held by changes 39, 40 and
41, names three reasons for moving the golden, none of them a probe added: its pull request says
why, as change 44's did (open question 5).

*As implemented* (task 4.1): the 40 lemmas, chosen once on the committed tables of 2026-10-10 — the
commonest lemmas of `tables/fr/freq.tsv` with two sense runs or more in `tables/fr-en/senses.tsv` or
`tables/fr-es/senses.tsv`, none of them a lemma the reference or the card asks as itself (none ranks
among them) — are `de`, `le`, `en`, `des`, `un`, `que`, `pas`, `qui`, `ce`, `sur`, `par`, `on`, `plus`,
`avec`, `mais`, `ça`, `tu`, `son`, `si`, `tout`, `nous`, `comme`, `être`, `bien`, `même`, `aussi`,
`deux`, `leur`, `moi`, `quand`, `après`, `avoir`, `non`, `alors`, `peu`, `autre`, `rien`, `dire`,
`monde`, `fois` (the last at rank 68). `fr-en.golden` goes from 147 probes on `main` (the 143 above
and change 44's 4) to 209: the 62 added at the end of their kinds, the 147 in their order, and one of
them moved — `phrase-gloss jusqu'au soir`, whose `jusqu'à` match now covers the article of « au »
(`"end":2` → `"end":3`, D12), the owner's decision this change carries. The phrase probe « l’homme »
answers `le` and `homme`, no expression.

**fr-es's golden.** `crates/lingua-wasm/tests/fr_es_baseline.rs` declares `Scenario { pair: "fr-es",
beside: &["en-es"], test: "fr_es_baseline", ..FRENCH }` — the French scenario glossed in Spanish, the
engine started on en-es as a Spanish-native reader's is, as the French scenario starts on es-en — and
writes `baseline/fr-es.golden`. A test compares it with `fr-en.golden` through `studied_side`, as
es-en's is compared with es-fr's: measured, 155 probes compared, none differing (the others: the
pack, beside, gloss, notice, licences and about lines). On the implementation (change 49 committed
fr-es's tables above its floor: 83.2 / 70.8 / 56.8 %, so 4.2 is this change's): 209 probes each, 159
compared on the studied side, none differing.

**Snapshots.** `test/word-card-fr-en.spec.ts` reads `fr-en.golden`, `test/word-card-fr-es.spec.ts`
reads `fr-es.golden`; each renders, with the interface in English or Spanish, every `word-grammar`
probe — the grammar lines through the renderer with `fr` studied, the gloss's pages and headings, the
row, as change 23's spec does, a word the pre-pass split on its piece — and every `phrase-gloss`
probe as the whole-selection card shows it: headed by the expression's name with its gloss when one
covers the whole selection, else its rows. They pin the lines in `test/baseline/word-card-fr-en.txt`
and `word-card-fr-es.txt`, re-blessed with `yarn vitest run <files> -u` (the flag after the files).
Measured on the prototype: 92 grammar probes each, 48 with a grammar line (54 lines), the order of
tenses on every one of them; fr-en 47 of its 91 glossed probes paged, fr-es 9; 26 phrase probes.
On the implementation: 92 grammar probes each, 47 with a grammar line (53 lines) — `été été` no longer
says « may also be the past participle of été », 41b's fix —, 89 glossed (`été été`, `porte porte` and
`au au` read no gloss), fr-en 46 of them paged and fr-es 9; 30 phrase probes (change 44's 4 and the
reader's included), 11 answered by an expression in fr-en and 9 in fr-es; `word-card-fr-en.txt` 706
lines, `word-card-fr-es.txt` 573. The two specs share their rendering, `test/word-card-french.ts`.

**Where they run.** `fr_es_baseline` joins the invariance step of `lingua-extension-check.yml` and the
re-bless line of `lingua-pack-update.yml`; the two specs join that workflow's vitest re-bless line,
so a re-reduction of fr-en, of fr-es or of French's studied tables re-blesses both goldens and both
snapshots on its branch.

**Order, and what they run on.** The snapshots read the goldens, never the tables. Until change 48's
implementation, `fr-en.golden` runs over the fixture and every grammar probe answers
`"readings":[]`: a snapshot would pin nothing. So this change is implemented after changes 45
(readings), 48 (the French baseline on the committed tables) and 49 (fr-es's tables) — and after 44,
whose names the whole-selection cards show, as they show change 41's closed classes (on `main`), and
so after 41b, which comes before 45. If change 49's committed measurement falls below its floor (M6:
81.4 / 68.8 / 54.5 % of the 5,000 / 10,000 / 20,000 commonest lemmas, settled on 2026-10-09), its
*Below it* applies: no fr-es table is committed, no package lists fr-es, and French ships for English
speakers alone. Then `fr_es_baseline.rs` and `fr-es.golden` have nothing to run on, and
`word-card-fr-es.txt` pins the Spanish renderer's grammar line of each of `fr-en.golden`'s grammar
probes — the readings are the studied side's, the same through either pack (*A form's readings do not
depend on the pack's native language*) — with no pages, headings or rows, which would be fr-en's English
glosses. The change that first commits fr-es's tables writes `fr_es_baseline.rs` and `fr-es.golden`
(task 4.2, carried and named in this change's pull request) and re-blesses the snapshot from it.

### D10 — What does not move

- **Every form of the four other pairs.** On the prototype, the 75,315 forms of en-fr, the 144,952 of
  es-fr, the 144,952 of es-en and the 75,315 of en-es, rendered through `main`'s renderers and through
  this design's, in the pair's interface language with their sense headings, give byte for byte the
  same 440,534 lines. Re-run on the implementation (task 5.1: the four packs built from the committed
  tables, every form opened on the dictionary form the phrase gloss reads, `main`'s renderers at
  `c8ddbf56` against this change's): the 440,534 forms byte for byte in their pair's interface
  language (253,214 grammar lines; 35,033 en-fr, 87,705 es-fr, 87,705 es-en and 35,033 en-es forms
  showing one). Rendered in all three interface languages (1,321,602 cards), the French and the
  English interfaces never move; the Spanish renderer's own Spanish lines — the table M2 reserves,
  shown to no reader, since a Spanish-native reader cannot study Spanish — move on 1,462 Spanish forms,
  D6's merge (« femenino plural y masculino plural de accionista » → « masculino y femenino plural de
  accionista »), as the English renderer's Spanish lines have merged since change 23.
- `test/word-grammar.spec.ts` passes without a change, as do `word-grammar-en.spec.ts`,
  `-es.spec.ts`, `i18n.spec.ts`, `wordpopup.spec.ts`, `selection-card.spec.ts` and the three committed
  snapshots (`word-card-es-en.txt`, `word-card-en-es.txt`, `selection-rows-fr.txt`); the prototype's
  whole suite passes but for two specs that need the generated gRPC clients the scratch copy lacked.
- *What each renderer names*, run for `fr` studied as well: over every tag of the vocabulary, the
  English and Spanish renderers name exactly what the French one names, in other words.
- `en-fr.golden`, `es-fr.golden`, `es-en.golden` and `en-es.golden`: nothing in the core's analysis,
  the packs or the engine's answers changes; `readable_gloss` changes for a French card only, and
  the phrase gloss's matching (D12) for a French selection only.
- The French interface: its renderer's English and Spanish lines are the 440,534 above; a selection
  moves only over a word written as pieces with spans of their own, none in an English or Spanish
  corpus.
- `fr-en.golden` gains probes and moves none but « jusqu'au soir », D12's.

The bundles holding the card (content, reader) grow by the three French tables, the `CARD_NAMES` table,
the merges and the selection's routing — about 3 kB as built, measured in the pull request.

### D11 — What later changes take from here

| Change | Takes |
|---|---|
| 52 enable | `StudiedLanguage` widened to `fr`, which the card already accepts (D1); the content's `language` then says `fr`, and the studied words their `lang`; the dogfood of the selection gestures on devices (task 6.2) and of change 40's two adjacent highlights |
| 53 listings | nothing of the card's wording; the listings may quote a card line the snapshots pin |
| 56 `refine-lingua-matrix-wording` | the shared wording D6 leaves: a bare plural beside gendered ones (the invariable plural on Spanish and English cards is settled by 51b) |
| `refine-lingua-fr-en-glosses`, fr-es's refinement | the glosses' defects below, re-measured on the committed snapshots; each re-reduction re-blesses the goldens and the snapshots |
| the change that first commits fr-es's tables, if 49's measurement falls below the floor (M6) | `fr_es_baseline.rs`, `fr-es.golden` and the glosses of `word-card-fr-es.txt` (D9) |

### D12 — A French match covers the article of « au » and « aux »

The owner decided on 2026-10-09, answering `add-lingua-spanish-expression-keys`' open question 3 (its
D5: a Spanish match ending on the `a` of « al » or the `de` of « del » covers the article sharing its
span), that French's « au » and « aux » get the same coverage, in this change. French's pre-pass
reads « au » as `à` + `le` and « aux » as `à` + `les`, sharing one span (change 40), and these are the
only French tokens sharing a span (`du`, `des` stay whole): a match whose last token is followed by a
token sharing its span covers it too, so no French match ends inside a written word.

**How.** `lingua-core` `engine.rs` `match_expressions` extends the match over the following tokens
sharing its last one's span for French as it does for Spanish (`shares_span`, which `gloss_phrase`
computes for every selection); Spanish's settled chain (its D6) stays Spanish's; English's split words
(`don't`) stay as they are. The match's key, gloss and status are unchanged, the tokens are what they
are without the table, a page's analysis and French's analyser version do not move, and no pack byte
moves — the matching is the engine's at reading time.

**What moves.** `fr-en.golden`'s `phrase-gloss jusqu'au soir`: the match `jusqu'à` covers `jusque`,
`à` and `le` (`"end":2` → `"end":3`); its whole-selection card does not move (its rows were `jusqu'à`
and `soir` before, `le` being a function word), but a selection of « jusqu'au » alone — one written
word whose pieces are two spans, so D7's whole-selection card — is now answered by `jusqu'à` whole.
No other probe of the French golden ends a match on the `à` of « au » or « aux »; fr-es's table holds
no `jusqu'à`, so `fr-es.golden` records no match there. A new requirement, *A French match never ends inside a written word*
(`lingua-analysis`), states it beside change 44's *French expressions are found on French's reading
of a selection*, held by that open change, which this change does not modify. The core's test
`a_french_or_english_match_ending_before_a_split_word_s_second_half_is_not_extended`, written by
44b for French and English, becomes `a_french_match_ending_on_the_a_of_au_or_aux_covers_its_article`
(« face au mur », « face aux murs », « jusqu'au », « jusqu'au soir ») and
`an_english_match_ending_before_a_split_word_s_second_half_is_not_extended` (« I don't »).

## Known data defects

What the committed snapshots (`word-card-fr-en.txt`, `word-card-fr-es.txt`, over the goldens of the
committed tables) show wrong that is not this change's wording, each with the module its fix lives in
(task 4.5, written again on the implementation; the prototype's list, made before 41b, 48 and 49,
is replaced). None is fixed here: `refine-lingua-fr-en-glosses` (48b, PR #861) and fr-es's refinement
(its proposal in progress) re-reduce the glosses and re-bless both goldens and both snapshots.

**The studied side** — `scripts/lingua-data/tables/fr/` (change 43's forms and change 45's readings,
`reduce-fr`'s French tables) or `crates/lingua-core`:
- « Others » that are true and read as noise on the commonest words (M8), change 45's `other` marks in
  `tables/fr/grammar.tsv`: `plus` « may also be the masculine plural past participle and first- and
  second-person singular past historic (passé simple) of plaire » and « … past participle of
  pleuvoir »; `mais` « may also be the masculine plural of mai »; `tu` « may also be the past
  participle of taire »; `suis` *suivre*'s present and imperative beside *être*'s.
- `meilleures` « may also be the comparative of bien » beside « … of bon » (`tables/fr/grammar.tsv`,
  `bien` given *bon*'s comparatives).
- Outside the snapshots, measured on the committed tables: `trente-et-une`, `trente-et-un`,
  `vingt-et-une`, `soixante-et-onze` read as forms of `trente`, `vingt`, `soixante` — no line, the
  numeral unnamed, but headed by another number (`tables/fr/forms.tsv`, change 43's mapping); `abime`,
  `coeur`, `aout` « masculine singular of abîme / cœur / août », a 1990 or unaccented spelling named by
  its agreement (`tables/fr/grammar.tsv`), as Spanish's `dia` is on es-fr's card.
- Fixed before this change by `fix-lingua-lemma-lookup` (41b): `été été` and `porte porte` read
  neither *être*'s nor *porter*'s readings or gloss — no line and no gloss — and the snapshots pin
  that (D9).

**fr-en's glosses** — `scripts/lingua-data/reduce-fr-en.py` and `tables/fr-en/`, for
`refine-lingua-fr-en-glosses` (48b): rows opening on another part of speech than the commonest —
`pas` « step, pace, footstep » before the negation, `son` « sound » before the determiner, `leur`
« (to) them » before « their », `même` « even » before « same », `bien` « good, all right, great »
(adjective) before « well »; proper nouns among common words (`le` « a surname from Vietnamese »,
`on` « a village in Luxembourg, Belgium »); `des` « of the; some, the feminine partitive article »;
`du` « forms the partitive article », a definition for a row; « see usage notes » inside `en`'s
senses; `plus`'s row a usage note cut at 80 characters (« used to express the comparative and
superlative of a following adjective or… »); `meilleur` glossed only as the noun « best ».

**fr-es's glosses** — `scripts/lingua-data/reduce-fr-es.py` and `tables/fr-es/`, for fr-es's
refinement: `être` opening on the noun (« Ser ») before the verb, and a usage note inside a sense
(« (être + participio) Haber »); `qui` « Quién. (Pronombre nominativo.) »; `que`, `peu`, `autre`
and `leur` (« Suyo, suya ») glossed only as pronouns; `rien` « Pequeño cantidad de algo » and an
adverb « Muy; Mucha… »; `du` and `des` a run with no part of speech (« [—] Contracción de la
preposición de y el articulo le, del »), `du`'s row that definition; `en`'s row opening on « En
cuanto a… »; `pas` « Paso » before the negation; `plus` « Más. Comparativo irregular de beaucoup »;
no `d'abord`, so « D’abord » shows its words' rows; the expression rows `à la` « A la » on « à la
maison » and `y avoir` « Haber » on « Y a-t-il encore du café ».

**Shared wording**, for change 56: a bare plural beside gendered ones (D6) — « plural and masculine
and feminine plural of abolitionniste » on 388 French forms, 791 in all. The invariable plural on
Spanish and English cards is settled by 51b.

## Risks / Trade-offs

- **A French card says less than its readings** (D5) → each omission is measured and named, the
  readings stay, and the snapshots pin the lines a reader sees.
- **Long lines on syncretic forms** (« finis »: participle, present, past historic, imperative) → the
  card wraps; the actions sit below the lines (`add-lingua-word-grammar` D6).
- **English or Spanish selections move** (D7) → only over a hyphenated run holding a digit, none in
  the corpora, and to the piece the reader selected.
- **A double-click on « l’homme » opens the whole-selection card**, not `homme`'s → a click on
  `homme` opens its card (change 40); change 52's dogfood judges the gesture on devices.
- **The French golden moves for probes added** (D9) → none of its probes moves; the pull request says
  why.
- **fr-es below its floor** (M6) → D9's fallback; the Spanish renderer is pinned on the studied side.
- **Wording read by the owner** (M9) → the English and Spanish tables and every snapshot line are in
  the pull request.

## Migration Plan

Nothing to migrate: no stored format, wire field, table, pin or pack moves; no reader studies French.
Rollback is a revert.

## Effort

4–5.75 ideal days, against the programme's 3.5–6: the description and the three renderers (D1–D6) 1–1.5;
their specs 0.5–0.75; the selection's routing and its tests (D7) 0.75–1.25; the stored gloss and
review by name (D8) 0.5–0.75; the probes, the fr-es golden, the two snapshots and the workflow lines
(D9) 0.75–1; the defects listed, specs and programme 0.5.

## Open Questions

For the owner, none blocking:
1. **The moods' merge beyond M21's words** (D3) — **settled by the owner on 2026-10-09: merged everywhere, as designed.** M21 names five-reading forms; the rule also merges
   the indicative and the subjunctive on 4,005 more forms (`parlent`, `parles`, `finissions`), as change
   45's D7 leaves to this change. Merging the five-reading forms only would say « parle » once and
   « parlent » twice.
2. **« participio presente » or the RAE's « participio de presente »** (D4) — **settled on 2026-10-09: « participio presente ».**, and « participio pasado »
   beside it.
3. **An invariable plural on Spanish and English cards** (D5) — **settled on 2026-10-09: aligned in a change of its own, proposed separately.** French cards leave it out (1,072 forms);
   Spanish's 667 (`tu`, `menos`) and English's 194 keep theirs unless a change of their own moves es-fr
   and en-fr cards.
4. **A double-click on « l’homme »** (D7) — **settled on 2026-10-09: the whole-selection card, as designed.** the whole-selection card, as change 40 recommended; the
   other choice needs a function-word flag on page tokens.
5. **Held wording** (D7, D8, D9), in open changes this one cannot MODIFY: *A French invariance baseline
   runs beside the English and Spanish ones* lists three reasons for moving the golden — probes added by
   a change (44, this one) are a fourth, best written in when change 41 archives; *A selected word
   resolves to its dictionary form* opens a hyphenated selection's word card — not for a word whose
   pieces have spans of their own, best narrowed when `add-lingua-phrase-gloss` archives; *Review finds a
   French expression card by its name* looks every other card up as before — not a French name without
   a space, best narrowed when change 44 archives.
