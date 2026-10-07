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

- **`src/i18n/<language>/languages.ts`**: the words of each studied language's name in that
  interface language — the name, the forms its grammar needs (French: of/the/masculine/feminine;
  Spanish: name, de-form, masculine/feminine; English: name, adjective) — and the twelve messages
  as slot messages taking the studied language. The French entry is today's `WORDS` and
  functions, byte for byte. The voice names Windows lists (« Anglais (États-Unis) ») are the
  platform's and stay keyed by studied language; the preview sentences are in the studied
  language and do not change.
- **The level acronym** is an entry: « CEFR » in French, "CEFR" in English, « MCER » in Spanish.
- **`language-labels.ts`** becomes the seam the surfaces call, reading the interface language and
  returning the catalogue's messages — until change 14–17 move each surface to the catalogue
  directly, at which point it is removed.
- **The lint** forbids « anglais », « espagnol », « français », "English", "Spanish", "French",
  « inglés », « español », « francés » and their inflections outside `src/i18n/`, in every `.ts`
  and `.html` of `src/`, with the baseline mechanism of change 13 for the files that still hold
  them.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *Languages are named in the interface language*. *Languages
  are named in one place* (held by no open change) stands: the one place is now the catalogue.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension/src/analyzer/language-labels.ts`,
  `src/i18n/{fr,en,es}/languages.ts`, `test/lint-language-labels.spec.ts`, the tests that assert
  the messages. The back office names languages in its own catalogue already. ID, Music, Live and
  the site are untouched.
- **No byte moves.** Every French message is the same; the surfaces call the same functions.
- **Order.** After change 13.
- **Not here.** The names of the native languages offered to the reader (« Français », « English »,
  « Español » in their own language, as a choice lists them) are change 20's, with the choice.
