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
Change 13 creates `src/i18n/{fr,en,es}/languages.ts`, en and es typed `typeof fr` (`import type
{ languages as fr } from "../fr/languages.ts"`), and its lint reads string literals from the
syntax tree. The engine's enum names (`French`, `English`,
`Spanish`) are data in `state/profile.ts`; `translate/host/model-manifest.ts`'s "not French" message is gone since change 8.

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

Change 13's `fr/languages.ts` holds the words of each studied language (`english`, `spanish`:
`LanguageWords`) and the messages as functions of those words — `levelTitle(of)`/
`levelTitleEstimated(of)`, `myLevelTitle(of)`/`myLevelTitleEstimated(of)`, `estimatedLevelsNote(the)`,
`borrowedTypicalNote(from, the)`, `levelNameEstimated(level)`, `chooseLevelPrompt(of)`,
`noTextDetected(masculine)`, `noTextInYourLanguages`, `noVoiceInstalled(feminine)`,
`levelQuestion(of)`; `languageName`, `windowsVoiceLanguage` and `previewSentence` are field
reads, `levelName` has no language. This change keeps that shape and adds `levelScale`;
`language-labels.ts` maps a studied language to its words in the interface language's module
before calling them. `en/languages.ts` and `es/languages.ts` are `typeof fr`: the same
`LanguageWords` fields, each filled with the form that language's grammar uses (English has no
gender; Spanish `of: "de inglés"`, `the: "el inglés"`, `masculine: "inglés"`, `feminine:
"inglesa"`) and the messages in that language's own words: `levelTitle(language, "es", true)`
is « Niveau d'espagnol estimé » and, in change 13's drafts, the English and Spanish sentences the
owner reviews (M9). `windowsVoice` is per interface language too:
the menu path the message quotes is the interface's (« Anglais (États-Unis) », "English (United
States)", « Inglés (Estados Unidos) ») — Windows lists its voices in its own display language,
which may differ; the message says what the reader looks for, as today.
The `levelScale` entry is « CEFR » / "CEFR" / « MCER » (M19; change 13's `stats.noLevels` spells
the acronym in its text, and now reads this entry) — "CEFR" equal in French and English, on
`SAME_AS_FRENCH.en` (change 13's list of texts one language shares with the French alone), not
the same-everywhere list, which « MCER » would fail; the `preview` sentences (in the studied
language, identical in the three modules) are on the same-everywhere list.

### D2 — The module takes the interface language

`language-labels.ts` keeps its twelve exported names and signatures with one added, leading
parameter: the interface language (`levelTitle(language, lang, estimated)`), and delegates to
the catalogue's module for it; a thirteenth name, `levelScale(language)`, reads D1's entry for
the one surface that spells the scale in a sentence of its own (D3). The surfaces pass the
language they read with their preferences (changes 14–17 give each surface its `copy`; the
language is beside it). The module is synchronous, as its callers are; nothing is cached. It
comes off the lint's baseline with this change: it holds no literal.

Alternative: a module-level setter filled by each entry. A surface that forgot it would read
French silently.

The same rule reaches the two views that name languages on their own and were mounted without a
language: `mountStudiedLanguages` (`reading/studied-languages-view.ts`) and `levelRow`
(`onboarding/level-row.ts`) take the interface language as a required parameter — no default —
and the onboarding page, still French until change 17, hands them `DEFAULT_INTERFACE_LANGUAGE`
explicitly, saying which change replaces it. The popup's main panel, rendered by an entry script
no test runs, moves to `popup/render.ts` (`renderStats(document, language, copy, stats,
onReader)`), which a test mounts on `popup.html` in the three languages; the entry keeps one line
of wiring. A literal `"fr"` at any caller is then a failing test, not a silent default:
`test/lint-settings-hosts.spec.ts` also reads every call of the two views, as it reads
`mountSettings`'s.

### D3 — The level scale

`estimatedLevelsNote` and the statistics' note (change 13's `stats.noLevels`, in change 16's
file) read `levelScale`; a Spanish interface says « MCER ». Change 13's `stats.noLevels` was a
string spelling « CEFR » twice; it is now a function of the scale, `noLevels(scale)`, in the three
`stats` modules, and `stats/view.ts` calls it with `levelScale(interfaceLanguage)` — the scale's
name lives in `languages.levelScale` alone.

### D4 — The lint names every language in every interface language

`lint-language-labels` reads string and template literals from the syntax tree (change 13's
mechanism) in `.ts` outside `src/i18n/*/languages.ts` — the other catalogue modules included: a
language's name belongs to the languages' module alone — and the HTML pages' text, and forbids — as whole words,
case-insensitively — « anglais », « espagnol », « français », "English", "Spanish", "French",
« inglés », « español », « francés » and their inflections, with change 13's baseline for the
files not yet moved. Named exceptions, data not copy: the engine's enum names in
`state/profile.ts` (`French`, `English`, `Spanish` as values), and a studied-language code.
The lint reaches a developer-facing string too: `i18n/language.ts`'s warning, when the key cannot
be read, said « showing French » — "French" is a name — and now says « showing the default (fr) »,
naming the default by its code, which is data. The lint's boundaries are letters alone
(`\p{L}`): a digit, an underscore or punctuation beside a name does not end the match, and a
literal that names a language as data is listed in the exceptions, text for text, rather than
loosened out of the pattern.

## Risks / Trade-offs

- **A French message moves** → `settings-view.spec.ts`, `stats.spec.ts`, `stats-view.spec.ts`,
  `review-page.spec.ts`, `reader-app.spec.ts`, `onboarding-level-row.spec.ts` and
  `studied-languages-view.spec.ts` assert them; a test on the module itself is added for every
  message in every interface language.
- **A caller that forgets the language** → the parameter is required; the compiler.
- **An English word that is also a name** ("English" in a comment) → the lint reads literals.

## Migration Plan

One release, silent. No stored state, no wire.
