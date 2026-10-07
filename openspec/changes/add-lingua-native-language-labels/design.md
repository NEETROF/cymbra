# Design — add-lingua-native-language-labels

## Context

See proposal.md (Why). `src/analyzer/language-labels.ts` (111 lines): `LanguageWords {name, of,
the, masculine, feminine, windowsVoice, preview}` per studied language (`en`, `es`), and twelve
functions — `languageName`, `levelTitle(lang, estimated)`, `myLevelTitle`, `estimatedLevelsNote`
(says « CEFR »), `borrowedTypicalNote(lang, from)`, `levelName(level, estimated)`,
`chooseLevelPrompt`, `noTextDetected(languages)`, `noVoiceInstalled`, `windowsVoiceLanguage`,
`previewSentence`, `levelQuestion` — called from the reader, Réglages, the studied-languages
view, review, statistics, the popup, onboarding and the ladder. `test/lint-language-labels.spec.ts`
forbids « anglais(e|es) » outside the module, in `.ts` and `.html`, and asserts it scans more than
50 files. The spec *Languages are named in one place* (scenarios "A Spanish page without enough
text", "Checked by lint"). The back office's precedent: `languageLabel(code)` reads
`lingua.languages.<code>` from its locale files, the code when unnamed.

## Goals / Non-Goals

**Goals:**
- A language's name, with the grammar around it, exists per interface language, once.
- The French messages are byte for byte the same.
- No name of a language is written outside the catalogue, in any interface language.

**Non-Goals:**
- The choice of the native language and its own list of names (change 20).
- The voices' platform names and the preview sentences: they are the studied language's.

## Decisions

### D1 — One `languages.ts` per interface language, with the grammar that language needs

`src/i18n/fr/languages.ts` keeps today's `WORDS` shape; `en/languages.ts` has `{name, adjective}`
per studied language; `es/languages.ts` has `{name, de, masculine, feminine}` (« inglés », « de
inglés », « inglés », « inglesa »). Each exports the twelve messages as functions of the studied
language, written in its own grammar: `levelTitle("es", true)` is « Niveau d'espagnol estimé »,
"Estimated Spanish level", « Nivel de español estimado ». The French functions are the current
ones moved; their test file runs on them unchanged.

Alternative: one shape for the three languages. English would carry empty `of`/`the` fields and
French grammar would leak into English messages' structure.

### D2 — The seam stays until the surfaces move

`language-labels.ts` keeps its twelve exported names and signatures; each reads the interface
language (change 13's key, cached in the module as the surfaces already await the key before
painting) and delegates to the catalogue. Changes 14–17 replace the calls by the catalogue's
and remove the seam. The lint of change 13 keeps the module on its baseline until then.

### D3 — The level acronym is an entry

`estimatedLevelsNote` and the ladder's notes say « CEFR » today; the entry `levelScale` is
« CEFR » / "CEFR" / « MCER », used by every message that names the scale (M19).

### D4 — The lint names every language in every interface language

`lint-language-labels` forbids, outside `src/i18n/`, the words « anglais », « espagnol »,
« français », "English", "Spanish", "French", « inglés », « español », « francés » and their
inflections, case-insensitively, as whole words, in `.ts` and `.html` — with the baseline of
change 13 naming the files that still hold them (the HTML pages, the surfaces before 14–17), and
the same stale-baseline check. A word that is data (a language code `"en"`, a test's sentence)
is not a name; the lint reads source files, not tests.

## Risks / Trade-offs

- **A French message moves** → the existing assertions run on the French functions through the
  seam; the file's diff is empty.
- **An English word that is also a name** ("English" in a comment) → the lint reads string
  literals and HTML text, not comments.
- **A caller that bypasses the seam** → the lint.

## Migration Plan

One release, silent. No stored state, no wire.
