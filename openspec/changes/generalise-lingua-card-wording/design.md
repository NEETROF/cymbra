# Design — generalise-lingua-card-wording

## Context

See proposal.md (Why). Where the card's grammar wording lives today:

| Where | What |
|---|---|
| `crates/lingua-core/src/packs/grammar.rs` `Tag` | `{pos, features}` with a closed vocabulary: 17 UPOS; Case, Definite, Degree, Gender, Mood (Cnd/Imp/Ind/Sub), Number, Person, PronType, Reflex, Tense (Fut/Imp/Past/Pqp/Pres), VerbForm (Fin/Ger/Inf/Part) |
| `crates/lingua-core/src/engine.rs` `WordGrammar` | `{gloss, senses, readings, others, pieces}`, readings in the pack's order; `wordGrammar` in `lingua-wasm`; mirrored in `src/analyzer/types.ts` |
| `src/reading/grammar-labels.ts` | the tables (parts of speech, genders, persons, ordinals, numbers, `ENGLISH_TENSES`, `SPANISH_TENSES` keyed Mood/Tense), the French grammar (`elides`, `ofName`, `agreement`, `of`, `withArticles`, `joinFrench`), `readingName`, `namesOf` (persons merged), `tenseRank` (the French school's order for Spanish; the pack's order for English), `grammarLines`, `senseHeading` |
| `src/reading/wordpopup.ts` `renderGrammar` | calls `grammarLines(…, "en" | "es")`, builds `.grammar-line` nodes with `<em>` for a studied-language word; the host `#cymbra-lingua-host` sets no `lang` |
| `test/word-grammar.spec.ts` | 108 assertions on French wording, the one place the order of tenses is pinned (`hable`: subjunctive before imperative) |
| `crates/lingua-wasm/tests/baseline/*.golden` | `### word-grammar` probes pin the engine's JSON, no wording |
| Specs | *The word card says what the form is* (":…in words of the interface language… a name that depends on the studied language, such as the name of a tense, SHALL be provided for English"), *A Spanish card names its forms as French schools do*, held by no open change |

## Goals / Non-Goals

**Goals:**
- One description of a form, with no language in it.
- The French wording byte for byte, through its renderer.
- English and Spanish renderers that read as grammar in those languages, drafted for the owner's
  review.
- The card carries `lang`.

**Non-Goals:**
- The card's other copy (change 14) and the languages' names (change 19).
- Showing English or Spanish to anyone (the interface language's choice is change 20; the
  renderers wait).
- Lemma alternatives for homographs (M8, optional change).

## Decisions

### D1 — A `FormDescription` with no words

`describeForm(grammar, headword, surface, written)` in `src/reading/grammar-description.ts`
gives:
- `own: Reading[]` — the readings of the headword that are not the dictionary form, each
  `{pos, degree?, gender?, number?, persons: string[], mood?, tense?, verbForm?}`, persons
  merged within a tense and number as `namesOf` merges them today;
- `others: {lemma, readings: Reading[]}[]`, `pieces: string[]`, `sameAsHeadword: boolean`;
- `senses: {pos?, gender?}[]` for the headings.
It is `grammar-labels.ts`'s logic with every French word removed: the tables, the articles and
the joining go to the renderers. Its tests are the structural half of today's assertions.

### D2 — A renderer per interface language, in the catalogue

`src/i18n/<language>/grammar.ts` exports `renderForm(description, studied): GrammarLine[]` and
`senseHeading(sense)`, with its own tables: parts of speech, genders, numbers, persons (« 1re
personne » / "1st person" / « 1.ª persona »), degrees, the joining, articles and elision where
the language has them (French `le/la/l'/les`, `de/d'`; Spanish `el/la/los/las`, `de`; English
none), and the forms of the fixed sentences (« peut aussi être … de », "can also be … of", « también
puede ser … de »). `GrammarLine` keeps its shape (`{word}` segments for studied-language words).

The French renderer is `grammar-labels.ts`'s wording moved, byte for byte; `test/word-grammar.
spec.ts` runs on it unchanged. The English and Spanish renderers are drafted per `src/i18n/
README.md` (change 13) and the programme's M10: US English; Spanish neutral, tú; English
Wiktionary's form-of wording as the model for English ("third-person singular simple present
of leave"), RAE/ASALE terms for Spanish (« 3.ª persona del singular del presente de indicativo de
hablar »).

### D3 — Tense names keyed by pair; the order of tenses per renderer

Each renderer holds `TENSES[studied][moodTense]` — the name a reader of this native language
uses for the studied language's tense: en-fr « prétérit », es-fr « passé simple », es-en
"preterite", en-es « pasado simple », fr-en "past historic (passé simple)", fr-es « pretérito
perfecto simple », the programme's examples, and the full tables for en and es studied; fr
studied is filled by stage 3 (its tags exist already; the entries are drafted here so the table
is whole). Each renderer orders tenses as its grammar does: the French renderer keeps
`tenseRank`'s order (indicative, conditional, subjunctive, imperative for Spanish; the pack's
order for English); the English and Spanish renderers state theirs, and a test per renderer
pins it on `hable` and `went`.

### D4 — The renderer follows the interface language; the card says its language

`wordpopup.ts` picks the renderer by the interface language (change 13's key, read with the
card's other preferences) — `fr` for every reader today — and sets `lang` on the card host to
it, and `lang` of the studied language on each `{word}` segment and on the headword. The HUD and
the drawer get the interface language's `lang` on their hosts too (one attribute each).

Alternative: keep the French renderer as the only one until change 20. The renderers are
written here so the owner reviews the wording once, with the catalogue's drafts (M9), and so
the goldens exist before any reader sees them.

## Risks / Trade-offs

- **A French byte moves** → the 108 assertions run on the French renderer through the new
  description; the diff of `test/word-grammar.spec.ts` must be empty.
- **English wording that reads as a translation of French** → the English Wiktionary's form-of
  wording as the model; the owner's review; change 23 refines with es-en's dogfood.
- **A tag the description does not know** → the engine's vocabulary is closed (`FEATURES`); the
  description names every member, and a test enumerates them.

## Migration Plan

One release, silent. No stored state, no wire, no pack.
