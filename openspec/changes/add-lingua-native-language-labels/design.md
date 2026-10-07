# Design — add-lingua-native-language-labels

## Context

See proposal.md (Why). `src/analyzer/language-labels.ts` (111 lines): `LanguageWords {name, of,
the, masculine, feminine, windowsVoice, preview}` per studied language (`en`, `es`), and twelve
synchronous functions — `languageName`, `levelTitle(lang, estimated)`, `myLevelTitle`,
`estimatedLevelsNote` (says « CEFR »), `borrowedTypicalNote(lang, from)`, `levelName(level,
estimated)` (no language), `chooseLevelPrompt`, `noTextDetected(languages)` (a list),
`noVoiceInstalled`, `windowsVoiceLanguage`, `previewSentence`, `levelQuestion` — called from the
reader, Réglages, the studied-languages view, review, statistics, the popup, onboarding and the
ladder; no test imports the module, its messages are asserted through `settings-view.spec.ts`
and `stats.spec.ts`. `test/lint-language-labels.spec.ts` forbids « anglais(e|es) » outside the
module, one regex per line, in `.ts` and `.html`. The other « CEFR » is `stats/view.ts:275`
(change 16's file). `settings-view.ts:89` wraps `windowsVoice` in a French Windows menu path.
Change 13 creates `src/i18n/{fr,en,es}/languages.ts` typed `typeof fr.languages`, and its lint
reads string literals from the syntax tree. The engine's enum names (`French`, `English`,
`Spanish`) are data in `state/profile.ts`; `translate/host/model-manifest.ts` has an error message
saying "not French" (change 8 removes it).

## Goals / Non-Goals

**Goals:**
- A language's name, with the grammar around it, exists per interface language, once.
- The French messages are byte for byte the same.
- No name of a language is written outside the catalogue, in any interface language.

**Non-Goals:**
- The choice of the native language and its own list of names (change 20).
- The preview sentences: they are in the studied language and do not change.
- Removing `language-labels.ts`: it stays the module every surface calls (D2).

## Decisions

### D1 — One `languages.ts` per interface language, one shape, typed after the French

Change 13's `fr/languages.ts` holds today's `WORDS` and the twelve messages. `en/languages.ts` and
`es/languages.ts` are `typeof fr.languages` (change 13 D1): the same `LanguageWords` fields,
each filled with the form that language's grammar uses — English `of: "of English"`, `the:
"English"`, `masculine`/`feminine: "English"` (no gender); Spanish `of: "de inglés"`, `the:
"el inglés"`, `masculine: "inglés"`, `feminine: "inglesa"` — and the twelve messages written in
that language's own words: `levelTitle("es", true)` is « Niveau d'espagnol estimé », "Estimated
Spanish level", « Nivel de español estimado ». `windowsVoice` is per interface language too:
Windows lists its voices in the display language, and the menu path the message quotes is the
interface's (« Anglais (États-Unis) », "English (United States)", « Inglés (Estados Unidos) »).
The `levelScale` entry is « CEFR » / "CEFR" / « MCER » (M19) — "CEFR" equal in French and
English, on change 13's same-in-every-language list.

### D2 — The module takes the interface language

`language-labels.ts` keeps its twelve exported names and signatures with one added, leading
parameter: the interface language (`levelTitle(language, lang, estimated)`), and delegates to
the catalogue's module for it; the surfaces pass the language they read with their preferences
(changes 14–17 give each surface its `copy`; the language is beside it). The module is
synchronous, as its callers are; nothing is cached. It stays off the lint's baseline from this
change on: it holds no literal.

Alternative: a module-level setter filled by each entry. A surface that forgot it would read
French silently.

### D3 — The level scale

`estimatedLevelsNote` and the statistics' « CEFR » (`stats/view.ts`, moved by change 16 into the
catalogue) read `levelScale`; a Spanish interface says « MCER ».

### D4 — The lint names every language in every interface language

`lint-language-labels` reads string and template literals from the syntax tree (change 13's
mechanism) in `.ts` outside `src/i18n/`, and the HTML pages' text, and forbids — as whole words,
case-insensitively — « anglais », « espagnol », « français », "English", "Spanish", "French",
« inglés », « español », « francés » and their inflections, with change 13's baseline for the
files not yet moved. Named exceptions, data not copy: the engine's enum names in
`state/profile.ts` (`French`, `English`, `Spanish` as values), and a studied-language code.

## Risks / Trade-offs

- **A French message moves** → `settings-view.spec.ts` and `stats.spec.ts` assert them; a test on
  the module itself is added for every message in every interface language.
- **A caller that forgets the language** → the parameter is required; the compiler.
- **An English word that is also a name** ("English" in a comment) → the lint reads literals.

## Migration Plan

One release, silent. No stored state, no wire.
