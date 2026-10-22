# Design — add-lingua-italian-card-wording

## Context

Measured on main at `78a9d8d01`, before changes 18, 32 and 33 are implemented.

- **The contract.**
  - `GrammarRenderer` (`src/i18n/index.ts`) has five methods: `grammarLines`, `grammarBlocks`,
    `readingName`, `senseHeading` and `join`. The English and Spanish renderers are typed
    `typeof fr`, and their tables stay private to each module.
  - `src/reading/grammar-description.ts` (576 lines) is the language-neutral part:
    - `FEATURES`;
    - `CARD_NAMES`: what the card names per studied language;
    - `finiteKey`: English keyed `Ind/<Tense>`, the others `Mood/Tense`, `Cnd/` and `Imp/` bare;
    - `mergedMoodsKey`: `Ind|Sub/T`;
    - `formKind`, `nameReadings`, `mergeGenders`, `tenseOrder`.
  - `src/reading/reading-copy.ts` picks the renderer per interface language. Its readers are the
    word card's grammar blocks and sense headings (`wordpopup.ts`).
- **The three renderers.**

  | Renderer | Lines | Value strings | Templates |
  |---|---|---|---|
  | fr | 192 | ≈ 61 | 13 |
  | en | 191 | ≈ 55 | 11 |
  | es | 221 | ≈ 58 | 10 |

  Each holds:
  - 14 parts of speech (13 distinct names; PUNCT, SYM and X unnamed);
  - genders (4), ordinals (3) and numbers (2); persons (3) in the French renderer only;
  - `TENSES` keyed by studied language, 21 names today (en 2, es 9, fr 10);
  - `GERUNDS` (en, es); `PARTICIPLES` in the Spanish renderer only.

  What differs per interface language is the grammar of the line:
  - **French**: elision (« d’ », « de l’ »), persons merged by tense name, numbers « Plur » before
    « Sing », « peut aussi être » repeating the article.
  - **English**: no articles, hyphenated persons, merges by key with `mergeGenders`.
  - **Spanish**: articles « el »/« la », « y » becoming « e », « de » never elided, merges by key with
    `mergeGenders`.
- **The forms to name** (`scripts/lingua-data/tables/*/grammar.tsv`):
  - **English studied**: the present and the past indicative, the past participle, the gerund. No
    progressive or perfect tag exists, since those are several words each.
  - **French studied**: 8 finite keys and 3 non-finite ones (the infinitive and the two participles),
    plus the merged `Ind|Sub/Pres` and `Ind|Sub/Imp`.
  - **Spanish studied**: 9 finite keys, `Sub/Fut` among them (889 forms), and 3 non-finite ones.
  - **Italian studied** (change 18): 8 finite keys plus the merged `Ind|Sub/Pres`, with the
    infinitive, the gerund and the two participles.

  That makes 30 tense names, three gerunds (en, es, it), the two participles, the infinitive and the
  degrees.
- **What change 18 does.** `StudiedLanguageCode` gains `it`. The French, English and Spanish
  renderers gain `TENSES.it`, `GERUNDS.it` and Spanish's `PARTICIPLES.it`. `CARD_NAMES.it` is set.
  The card harness (`test/word-card-french.ts`) is parameterised. The it-fr golden carries Italian's
  grammar probes.
- **What change 33 leaves here**:
  - the Italian row of `reading-copy.ts`, the English renderer until now (its D2);
  - the parity test's exemption of `grammar` for Italian;
  - its requirement's « for every surface but `grammar` ».
- **Precedents.**
  - Matrix change 18 (`generalise-lingua-card-wording`) drafted the English and Spanish renderers
    (its D4), at 2.5–4.5 ideal days.
  - Matrix changes 23 and 24 (`add-lingua-english-card-wording`, `add-lingua-spanish-card-wording`)
    pinned them on the real es-en and en-es packs. Each built a baseline, a `word-card-*.spec.ts`
    snapshot, and the corrections the real forms called for.
  - Matrix change 24 named English's forms for a Spanish reader by the Spanish Wiktionary's English
    form-of wording, plus M10's « forma en -ing ».
- **Card snapshots today.**

  | Snapshot | Lines | Grammar lines | Grammar probes | Phrase probes |
  |---|---|---|---|---|
  | en-es | 401 | 30 | 72 | 28 |
  | es-en | 467 | 36 | 68 | 33 |
  | fr-en | 887 | 57 | 97 | 81 |
  | fr-es | 741 | 57 | 97 | 81 |

  A form's readings are byte-identical across natives: `cross_native.rs` holds the studied sections
  of each language's packs equal, and the `studied_side` tests of the baselines compare the probes
  they share. A sense's heading is not a reading: it comes from the native side's sense groups
  (`WordGrammar.senses[].tag`), which `studied_side` strips before comparing.

## Goals / Non-Goals

**Goals:**
- An Italian reader's card names every form the other renderers name, in Italian grammar's words.
- The wording is pinned on real forms before any Italian reader sees it.

**Non-Goals:**
- The full cards of en-it, fr-it and es-it with their glosses: changes 36, 47 and 51, as change 18's
  D8 handed it-en's and it-es's to changes 22 and 27.
- Naming more than the other renderers name: no progressive, no perfect, no compound tense.
- Showing the Italian interface (change 45).

## Decisions

### D1 — Italian's own forms: Treccani's names, mood before tense

| Key | Name |
|---|---|
| `Ind/Pres` | indicativo presente |
| `Ind\|Sub/Pres` | indicativo o congiuntivo presente |
| `Ind/Imp` | indicativo imperfetto |
| `Ind/Past` | passato remoto |
| `Ind/Fut` | futuro semplice |
| `Cnd/` | condizionale presente |
| `Sub/Pres` | congiuntivo presente |
| `Sub/Imp` | congiuntivo imperfetto |
| `Imp/` | imperativo |
| gerund, participles, infinitive | gerundio, participio presente, participio passato, infinito |

These are the names of Treccani's *Enciclopedia dell'Italiano*:
- the « Futuro » article: « Il futuro (o futuro semplice) »;
- « Condizionale »: « condizionale presente »;
- « Indicativo »: « indicativo presente » and « indicativo imperfetto ».

Treccani is not consistent on word order. The « Indicativo » article also writes « presente
indicativo », the « Imperfetto » article writes « imperfetto indicativo », and the « Congiuntivo »
article abbreviates (« cong. pres. »). The Italian Wiktionary mixes the two orders too. The card puts
the mood first everywhere, so that « indicativo o congiuntivo presente » reads as one family with its
parts. « Passato remoto » and « futuro semplice » are indicative by name and take no mood.

The order of tenses is change 18's: indicative, conditional, subjunctive, imperative. Treccani's
« Modi del verbo » lists « indicativo, condizionale, congiuntivo, imperativo », the same order. Change
18's D2 says Treccani puts the congiuntivo first; this pull request corrects that sentence.

### D2 — French and Spanish forms: the Italian name of the same tense

| Key | French studied | Spanish studied |
|---|---|---|
| `Ind/Pres` | indicativo presente | indicativo presente |
| `Ind\|Sub/Pres` | indicativo o congiuntivo presente | — |
| `Ind/Imp` | indicativo imperfetto | indicativo imperfetto |
| `Ind\|Sub/Imp` | indicativo o congiuntivo imperfetto | — |
| `Ind/Past` | passato remoto | passato remoto |
| `Ind/Fut` | futuro semplice | futuro semplice |
| `Cnd/` | condizionale presente | condizionale presente |
| `Sub/Pres` | congiuntivo presente | congiuntivo presente |
| `Sub/Imp` | congiuntivo imperfetto | congiuntivo imperfetto |
| `Sub/Fut` | — | congiuntivo futuro (futuro de subjuntivo) |
| `Imp/` | imperativo | imperativo |

The French and Spanish renderers name each other's tenses by their own grammar, as the Spanish one
names French's in the RAE's terms. Italian does the same, and the terms are exact cognates:
- the Italian Wiktionary writes « terza persona singolare dell'indicativo passato remoto di parler »
  (`parla`) and « participio presente di parler ». The card drops « indicativo » there by D1's rule,
  as the Italian Wiktionary itself does for Italian's own passato remoto (`fu`);
- Studentville's table of Spanish maps the pretérito indefinido to the passato remoto, the imperfecto
  to the imperfetto, and the condicional simple to the condizionale presente. The Italian Wikipedia's
  « Verbi spagnoli » titles the last « Condizionale semplice (*condicional simple*) »; the card keeps
  the Italian tense's own name, as for every other tense, and D8 puts it to the reviewer agent.

So no bracket is needed, as M10 needed "past historic (passé simple)" in English. The exception is
Spanish's future subjunctive, which Italian lacks. The Italian Wikipedia names it « Futuro congiuntivo
(*futuro (simple) de subjuntivo*) » in « Verbi spagnoli » and « congiuntivo futuro » in its
« Congiuntivo » article; the card writes « congiuntivo futuro (futuro de subjuntivo) », mood first, with
the bracket as Wikipedia gives it.

The gerund is « gerundio » for Spanish. French's gerund is not named (`CARD_NAMES.fr.gerund` is false),
and its present participle is « participio presente ».

### D3 — English forms: the Italian Wiktionary's wording, and « forma in -ing »

| Key | Name |
|---|---|
| `Ind/Pres` | presente semplice |
| `Ind/Past` | passato semplice |
| gerund | forma in -ing |
| past participle | participio passato |

English has no Treccani name, so I6 does not settle it. Matrix change 24's precedent does: name English's
forms as the Italian-language source of en-it writes them. The Italian Wiktionary writes « passato
semplice di eat », « terza persona singolare presente semplice di go », « participio passato di eat »
and « participio presente di go ». Italian teaching of English keeps those names and calls the -ing
forms « le forme in -ing » (EF Italia), which gives « forma in -ing », as M10 gave « forma en -ing ».

The English labels themselves ("Simple present", "Past simple"), which some Italian schoolbooks keep,
are the alternative. They are not taken: no other renderer names a tense in the studied language.

Matrix change 24 found the Spanish Wiktionary calling an -ing form « participio presente »; the en-it glosses
may do the same. That belongs to change 37, not to the card.

### D4 — The line's grammar in Italian

- **Parts of speech**: sostantivo, nome proprio, aggettivo, verbo, verbo ausiliare, pronome,
  preposizione, avverbio, congiunzione, interiezione, determinante, numerale, particella. PUNCT, SYM
  and X are unnamed, as in every renderer. A sense heading puts the noun before the gender:
  « sostantivo femminile », as Treccani's vocabulary labels `casa` « s. f. ».
- **Genders and numbers**: maschile, femminile, neutro, comune; singolare, plurale. Numbers are
  ordered `["Sing", "Plur"]`, readings merge by key, and the genders of one number are named once
  (« maschile e femminile plurale »), as the English and Spanish renderers do (`mergeGenders`).
- **Persons**: « terza persona singolare », with no « del » (Treccani's *Persona*, the Italian
  Wiktionary). Several persons in one tense and number are named once: « prima e terza persona
  singolare », with the noun in the singular, as the Spanish card does.
- **Participles**: « participio passato femminile plurale », the singular unsaid, as Spanish says it.
- **Contractions**: the preposition contracts with the tense's article: « del passato remoto »,
  « dell'indicativo imperfetto », « dell'imperativo », « dell'infinito ». No name in D1–D3 takes « lo »,
  so the renderer has no « dello » branch; a name that needs one adds it with its test.
- **« può anche essere »** takes each name's article: « il passato semplice », « l'infinito »,
  « la forma in -ing », « la terza persona… ».
- **« di »** before a word of the page never elides (« di eat », « di andare »), as the Italian
  Wiktionary writes it.
- **The joining word** is « e ». No name opens with « e », so « ed » never arises.
- **Typography**:
  - the apostrophe is U+0027, the one the Italian catalogue writes (« l'inglese », change 33) and
    `_locales/it` writes (change 39), one glyph across the Italian interface;
  - pieces are written « «x» = », without inner spaces (change 33's D1);
  - every word of the page keeps its `lang`.

Sample lines:
- « terza persona singolare del passato remoto di parlare »
- « prima e terza persona singolare dell'indicativo imperfetto di hablar »
- « terza persona singolare del presente semplice di have »
- « può anche essere il passato semplice e il participio passato di read »
- « femminile plurale di rápido »

### D5 — One module, typed after the French

`src/i18n/it/grammar.ts` is `typeof fr` like the other two renderers, its tables private, its rows
keyed by `StudiedLanguageCode` so that the type checker requires all four. Under M2 the interface
language is the native language, and no pair studies its own native, so no reader reaches the `it`
row today: it exists because the type requires it, for the override M2 reserves, as the French
renderer's `fr` row does (`src/i18n/fr/grammar.ts`). Its names are D1's, which D2 reuses for French
and Spanish. It is about 210–230 lines.
`grammar-description.spec.ts` requires every name to differ from the French renderer's; none of D1–D3
collides. `src/i18n/README.md`, which names the English and Spanish renderers, gains the Italian one.

### D6 — The Italian interface reads it; change 33's exemption ends

`reading-copy.ts`'s Italian row takes the Italian renderer instead of the English one, and
`test/reading-copy.spec.ts`'s per-language check gains `readingCopy("it").grammar` →
`itGrammar`. The parity test's « every surface but `grammar` » for Italian becomes every surface.
The renderer loops of `grammar-description.spec.ts`, `i18n.spec.ts` and `wordpopup.spec.ts`, which
list `["en", "es"]`, gain `it`.

Two requirements held by open changes are MODIFIED, each copied whole:
- change 33's *The interface's copy is kept in one typed catalogue per language*: the Italian modules
  are typed after the French for every surface, and *The drafts are whole* reads every Italian entry;
- change 18's *An Italian card names its forms in the interface language's grammar*, whose list of
  interface languages (« French, English or Spanish ») would otherwise stay closed in the main spec:
  it gains Italian, with Treccani's names, and its passato remoto scenario the Italian line.

### D7 — A grammar-only snapshot, on real forms

Before change 36, no pack is glossed in Italian, so a full card snapshot cannot exist (the fixture
pack's `rewritten(studied, native)` changes metadata, not glosses). A form's readings are the same
whatever the native, so the grammar can be pinned now.

`test/word-card-grammar-it.spec.ts` renders, through the Italian renderer, the grammar lines of every
grammar probe of four goldens, through a new grammar-only export of the card harness
(`test/word-card-french.ts`, whose `render()` always writes gloss pages and selection rows):

| Golden | Studied language | Grammar probes |
|---|---|---|
| en-es | English | 72 |
| es-en | Spanish | 68 |
| fr-en | French | 97 |
| it-fr | Italian | change 18's |

That is 237 probes and 123 grammar lines before it-fr's, into a committed snapshot with no glosses
and no sense headings: a heading is the native side's sense group, so pinning it here would pin the
English and Spanish Wiktionaries' groups, which no Italian reader sees, and move with every
refinement of those glosses. Headings are held by the unit spec and the existing loop that every
renderer heads the same parts of speech. `lingua-pack-update.yml` re-blesses it where it re-blesses the other card snapshots, and
`lingua-extension-check` runs it with `yarn test`.

The es-en probes hold no `Ind/Fut`, `Sub/Fut` or gerund, so `test/word-grammar-it.spec.ts` (about 500
lines, as `-en` and `-es`) covers them with made-up readings, as the other renderers' unit specs do.
Changes 36, 47 and 51 add the full card snapshots of en-it, fr-it and es-it over their own goldens.

### D8 — Reviewed by the Italian reviewer agent

The renderer's names and sample lines are reviewed in the implementation pull request by the agent
change 33 defines (I2). Its findings are applied there, or answered there with a source. These are
the open points it settles:
- « condizionale presente » against Wikipedia's « condizionale semplice » for Spanish (D2);
- « determinante » against « articolo » for `DET`: Italian school grammar has no « determinante ».

The apostrophe is settled in D4; if the reviewer changes it, change 33's catalogue follows.

The owner is not asked to review Italian. Change 44 rereads the card with the rest of the Italian
wording, and dogfood waits for change 45.

## Risks / Trade-offs

- **Names an Italian reader would not use** → the sources are cited per name; the reviewer agent
  reviews them now and again in change 44; change 55 takes dogfood's corrections.
- **Treccani's own word-order mix** → one order (D1), stated, so a reader sees one shape.
- **The snapshot pins no sense heading** → headings are the native side's, pinned by change 36's
  full en-it snapshot over Italian's own sense groups; until then, the unit spec holds them.
- **A later change to the readings moves the Italian snapshot** → by design: it is re-blessed with the
  goldens, so the move shows in the pull request.

## Migration Plan

Nothing deploys, and no reader sees the renderer until change 45. Rollback is a revert.

## Effort

3.5–4.5 ideal days, against the programme's 3–5.5:

| Part | Ideal days |
|---|---|
| The renderer | 1–1.5 |
| Its unit spec | 1–1.5 |
| The test loops | 0.25 |
| The grammar-only snapshot | 0.5 |
| The reviewer agent's pass | 0.5 |
| The spec and the programme | 0.25 |
