# add-lingua-native-language-labels — languages are named in the interface language

## Why

Change 19 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 1, a silent release, decision M19. Languages are named in one place today,
`language-labels.ts`: « Anglais », « d'anglais », « l'anglais », « anglais », « anglaise »,
« Anglais (États-Unis) », and twelve functions that build « Niveau d'anglais estimé », « Pas de
texte espagnol détecté sur cette page », « Mon niveau d'espagnol »… — French grammar around a
French name, keyed by the studied language. The lint keeps « anglais » out of every other file.

An English-native reader needs "English level", "No Spanish text found on this page"; a
Spanish-native reader « Nivel de inglés », « No se detectó texto en español en esta página ».
Each language has its own grammar around a name: French and Spanish have articles and
contractions (« d'anglais », « de inglés »), English has none; French and Spanish have gendered
adjectives (« anglaise », « inglesa »), English has not. The level acronym follows the interface
language too: CEFR in French stays « CEFR », English says "CEFR", Spanish says « MCER » (M19).

This change moves the module into the catalogue (change 13), one entry per interface language,
with the French byte for byte, and widens the lint to every language's name in every interface
language, so that a name is never written where it is shown.

## What Changes

- **`src/i18n/<language>/languages.ts`** (change 13's module, kept in its shape): the words of
  each studied language's name in that interface language (`english`, `spanish`: `LanguageWords`
  — name, of, the, masculine, feminine, the Windows voice name in the interface's language, the
  preview sentence in the studied language), each language filling the fields with the forms its
  grammar uses, and the messages as functions of those words, as change 13 wrote them. The
  French entries are today's `WORDS` and sentences, byte for byte; `language-labels.ts` maps a
  studied language to its words in the interface language's module before calling them.
- **The level acronym** is an entry: « CEFR » in French, "CEFR" in English, « MCER » in Spanish.
- **`language-labels.ts`** stays the module the surfaces call, and takes the interface language
  as its first parameter — the surfaces pass the one they read with their preferences — and
  delegates to the catalogue; it holds no literal.
- **The lint** forbids « anglais », « espagnol », « français », "English", "Spanish", "French",
  « inglés », « español », « francés » and their inflections in string literals outside
  `src/i18n/*/languages.ts`, read from the syntax tree as change 13's lint does, with its baseline for the files
  that still hold them, and the engine's enum names as a named exception.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *Languages are named in the interface language*; MODIFIED
  *Languages are named in one place* (held by no open change): the one place is one module per
  interface language, `src/i18n/<language>/languages.ts`, and the lint admits those alone; both
  scenarios kept.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension/src/analyzer/language-labels.ts`,
  `src/i18n/{fr,en,es}/languages.ts`, `test/lint-language-labels.spec.ts`, the tests that assert
  the messages. The back office names languages in its own catalogue already. ID, Music, Live and
  the site are untouched.
- **No byte moves.** Every French message is the same; the surfaces call the same functions with
  the language they read.
- **Order.** After change 13 (archived after it); after 14–17, which hand each surface the
  interface language this module now takes.
- **Not here.** The names of the native languages offered to the reader (« Français », « English »,
  « Español » in their own language, as a choice lists them) are change 20's, with the choice.
