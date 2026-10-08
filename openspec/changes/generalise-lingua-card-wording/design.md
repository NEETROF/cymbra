# Design — generalise-lingua-card-wording

## Context

See proposal.md (Why). Where the card's grammar wording lives today:

| Where | What |
|---|---|
| `crates/lingua-core/src/packs/grammar.rs` `Tag` | `{pos, features}` with a closed vocabulary: 17 UPOS; Case, Definite, Degree, Gender, Mood (Cnd/Imp/Ind/Sub), Number, Person, PronType, Reflex, Tense (Fut/Imp/Past/Pqp/Pres), VerbForm (Fin/Ger/Inf/Part) |
| `crates/lingua-core/src/engine.rs` `WordGrammar` | `{gloss, senses, readings, others, pieces}`, readings in the pack's order; `wordGrammar` in `lingua-wasm`; mirrored in `src/analyzer/types.ts` |
| `src/reading/grammar-labels.ts` | the French tables (parts of speech — PUNCT, SYM and X unnamed; genders; persons; ordinals; numbers; `ENGLISH_TENSES` Past/Pres; `SPANISH_TENSES` keyed Mood/Tense), the French grammar (`elides`, `ofName`, `agreement`, `of`, `withArticles`, `joinFrench`, U+2019 apostrophes), `readingName(tag, studied)` (the gerund « forme en -ing » for English and « gérondif » for Spanish; the infinitive named for Spanish only; for English only the indicative's tenses), `namesOf` (persons merged on the French tense name), `tenseRank` (the Spanish table's order; the pack's order for English), `grammarLines`, `senseHeading`, `lineText`; the exported API `test/word-grammar.spec.ts` imports: `grammarLines`, `joinFrench`, `lineText`, `readingName`, `senseHeading` |
| `src/reading/wordpopup.ts` `renderGrammar` | calls `grammarLines(…, "en" \| "es")`, builds `.grammar-line` nodes with `<em>` for a studied-language word; the card is built in `ReadingSession`'s constructor and reads no preference of its own |
| `test/word-grammar.spec.ts` | 108 `expect` calls, most on French wording, some on layout and paging; the one place the order of tenses is pinned (`hable`: subjunctive before imperative); `null` asserted for `{PRON, Case: Dat}` and for an English `Mood=Sub` |
| `crates/lingua-wasm/tests/baseline/*.golden` | `### word-grammar` probes pin the engine's JSON, no wording |
| Specs | *The word card says what the form is* ("in words of the interface language… a name that depends on the studied language, such as the name of a tense, SHALL be provided for English"); *A Spanish card names its forms as French schools do* ("SHALL be named in French school terms… French articles and elision SHALL apply"), a rule about the French interface, held by no open change |
| Change 13 | `src/i18n/{fr,en,es}/grammar.ts` created as flat text entries (`posAdjective`, `englishPast`, `spanishPreterite`, `mayAlsoBe`…), en and es typed `typeof fr`; `grammar-labels.ts` does not read them yet and stays on the lint's baseline; the parity test calls each function with sample strings; the content script reads the interface language and hands it to `ReadingSession`, which hands it to the card (change 14) |

## Goals / Non-Goals

**Goals:**
- One description of a form, with no language in it, merged and deduplicated by tag.
- The French wording byte for byte, through `grammar-labels.ts`'s unchanged exported API.
- English and Spanish renderers that read as grammar in those languages, drafted for the owner's
  review; the rule that a Spanish card is named "as French schools do" rewritten as "as the
  interface language's grammar does".
- The studied-language words of a line carry their `lang`.

**Non-Goals:**
- The card's other copy (change 14: it also owns the `lang` of the injected hosts) and the
  languages' names (change 19).
- Naming tags the French card does not name today (PUNCT, SYM, X; Case, Definite, PronType,
  Reflex, Pqp): a renderer names what the French one names, no more.
- Showing English or Spanish to anyone before the interface speaks them (change 20).
- French as a studied language (stage 3 adds `fr` to `StudiedLanguage` and the tables).

## Decisions

### D1 — A `FormDescription` with no words, merged by tag

`describeForm(grammar, headword, surface, written)` in `src/reading/grammar-description.ts` gives:
- `own: Reading[]` — the readings of the headword that are not the dictionary form, each
  `{pos, case?, definite?, degree?, gender?, number?, persons: string[], mood?, tense?, verbForm?,
  pronType?, reflex?}` (every feature of the engine's vocabulary), persons merged within the same
  (mood, tense, number) tag and duplicates dropped by tag; `own` and each `others[i].readings`
  keep the pack's order of first occurrence, since the French renderer places a merged group
  where the pack's order gave its first reading;
- `others: {lemma, readings: Reading[]}[]`, `pieces: string[]`, `sameAsHeadword: boolean`;
- `senses: {pos?, gender?}[]` for the headings.
The French renderer re-merges by its own names, as today: the conditional and the imperative
are named without a tense, a missing mood reads as the indicative for English, and duplicates are
dropped by French name across parts of speech — so every input the French spec pins gives the
same bytes, which its unchanged run proves; the English and Spanish renderers merge by tag, as
their grammars need. The description carries every feature of the engine's vocabulary; a
renderer may leave one unnamed, as the French one does, and a test enumerates which each renderer
names.

### D2 — A renderer per interface language, behind the French one's API

Each renderer exposes `grammarLines`, `readingName`, `senseHeading` and its joining
function with today's signatures and `GrammarLine` shape, `Named.article` widened to `string` —
the French renderer keeps its four values, which `test/word-grammar.spec.ts` asserts by value;
`lineText`, the same in every language, is the description's, re-exported by `grammar-labels.ts`. `src/reading/grammar-labels.ts` stays
the French renderer, its exported API and wording unchanged, now reading the French tables from
`src/i18n/fr/grammar.ts` and the description from D1; `test/word-grammar.spec.ts` runs on it
without a change. `src/i18n/fr/grammar.ts` exports `const grammar: GrammarRenderer` — the interface declared in
`src/i18n/index.ts`: the renderer functions, and the tables behind them private to each module
(English has no articles; Spanish has) — and `src/i18n/en/grammar.ts`, `src/i18n/es/grammar.ts`
are typed `typeof fr` (change 13's form: `import type { grammar as fr } from "../fr/grammar.ts"`),
which here is `GrammarRenderer`: the compiler checks a renderer's functions, not its private
tables — *What each renderer names* checks those. This is the one module where change 13's rule
covers code, not texts; `src/i18n/README.md` says so. Change 13's parity test (`test/i18n.spec.ts`,
which today calls every entry with sample strings) treats `grammar` modules as renderers instead:
it calls `readingName` and `senseHeading` on the sample tags and `grammarLines` on the French
spec's inputs and asserts non-empty, non-French results.
`src/reading/grammar-labels.ts` keeps its exported API and its French strings (« peut aussi
être … de », « et », the elisions) — it stays on change 13's baseline until the French renderer's
strings move into `fr/grammar.ts`, which this change does: `grammar-labels.ts` then holds no
literal and comes off the baseline. `wordpopup.ts` picks the renderer by the interface language `ReadingSession`
hands it (change 14), `fr` for every reader today.

### D3 — What a renderer keys by studied language

Not only tense names: the gerund's name (« forme en -ing » / "the -ing form" / « forma en -ing »
for English, « gérondif » / "gerund" / « gerundio » for Spanish), whether the infinitive is named
(Spanish only today), which moods are named (English: the indicative only), and the order of
tenses. Each renderer holds the names per studied language in its own tables — `TENSES[studied]
[moodTense]`: en-fr « prétérit », es-fr « passé simple », es-en "preterite", en-es « pasado
simple » (the full tables for en and es studied; fr studied comes with stage 3's type), listed in
the order the card names them, which the description derives once (`tenseOrder`: the table's order
for Spanish, the pack's for English) and each renderer pins on `hable` and `went` — while what is
named at all, whether the infinitive is (`NAMES_INFINITIVE`) and which moods are (`finiteKey`), is
the description's decision, so a renderer only words it. The French renderer's values are today's.

### D4 — The wording of the drafts

English follows the English Wiktionary's form-of wording: "third-person singular preterite
indicative of venir", "past participle of walk" — except the gerund, which is M10's "-ing form"
("-ing form of go", D3), not Wiktionary's "present participle and gerund of go"; it is the one line
that can open with a hyphen, and the owner keeps or changes it under M9; Spanish follows RAE/ASALE
terms: « tercera persona del singular del pretérito perfecto simple de indicativo de venir »,
« participio de hablar », « gerundio de hablar ». Neither calques the French. The owner reviews
both (M9).

### D5 — The studied-language words carry `lang`

`renderGrammar` sets `lang` of the studied language on each `{word}` segment, and the card sets
it on its headword element;
the card host's `lang` (the interface language) is change 14's, with the HUD's and the drawer's.

## Risks / Trade-offs

- **A French byte moves** → the French spec runs on the French renderer through the new
  description; the diff of `test/word-grammar.spec.ts` must be empty.
- **English wording that reads as a translation of French** → D4's models; the owner's review;
  change 23 refines with es-en's dogfood.
- **A tag the description does not know** → the engine's vocabulary is closed (`FEATURES`); the
  description names every member, and a test enumerates what each renderer names.

## Migration Plan

One release, silent. No stored state, no wire, no pack.
