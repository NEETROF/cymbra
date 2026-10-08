# add-lingua-interface-language — one catalogue for the interface's copy, in French, English and Spanish

## Why

Change 13 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the tenth of stage 1, and the first of the interface's five. The interface speaks the reader's
native language (M2: one choice, the native language, under its own key). Today it speaks French
only: ≈ 450 unique texts, ≈ 2,500 words, in 24 TypeScript files and 6 HTML pages, each written
where it is shown. Three small copy objects exist (`reader/copy.ts`, the translation setting's,
the account setting's); the rest is literals. The browsers' `chrome.i18n` follows the browser's
language and cannot follow a reader's choice, so the catalogue is the extension's own.

This change builds the catalogue and the plumbing, and changes nothing a reader sees: the French
copy is extracted byte for byte (M23) into `src/i18n/fr`, the English and Spanish drafts beside
it typed `typeof fr` so that a missing key does not compile, with plurals through
`Intl.PluralRules` and slot messages for the sentences built from fragments; the interface
language is read from its own key; a lint forbids a French literal outside the catalogue, with a
baseline that names the files still holding literals today and shrinks with changes 14–17, which
move each surface's copy to the catalogue. Until then every surface still shows its literals,
which is why nothing moves.

The English and Spanish drafts are written here (M9: a draft in each pull request, the owner
reviews the English and the Spanish), with the terms M10 and M19 settled: US English, Spanish
with tú and no vosotros, « forma en -ing », RAE tense names, CEFR in English and MCER in Spanish.

## What Changes

- **A typed catalogue**, `apps/lingua-extension/src/i18n/{fr,en,es}/<surface>.ts`, one module per
  surface and language (popup, HUD, word card, selection card, review, statistics, settings,
  colours, display, translation setting, account, onboarding, reader, sync, language labels,
  grammar labels), the French one the source — every literal of the inventory, byte for byte —
  and `en` and `es` typed `typeof fr` (string-typed, as the site's `const en: typeof fr`, never
  `as const`). Keys are named by meaning, never by the French text. A surface imports its own
  module for each language; `src/i18n/index.ts` holds the helpers, not a map of every surface.
- **Messages with slots and plurals.** A sentence built from fragments today is one message
  taking its parts; a count takes `Intl.PluralRules` forms. In French the forms are the current
  strings, so « carte(s) » stays « carte(s) ».
- **The interface language, under its own key**: `cymbra-lingua-interface-language` in
  `chrome.storage.local`, `fr`, `en` or `es`; absent means `fr` for a device that predates the key
  (M22; change 20 presets a new install). The store's owner writes it at its start when it is
  absent, and after each write of the backup, from the native language as the extension serves it
  (change 4's gate: French when no shipped pair is glossed in the profile's language), so the
  choice is the native language (M2) and a later override needs no migration. Every surface can
  read it before it renders its copy, as it reads its other preferences; the static French of the
  HTML pages paints before any script and moves with its surface (changes 14–17).
- **Formats by language**: in English and Spanish, numbers and dates through the locale; in
  French, exactly what the surfaces write today (« 25,8 Mo » with its plain space, « il y a
  3 min. », a raw count in « 3 carte(s) »).
- **A lint**: no French string literal in `src/` outside `src/i18n/` — a literal holding an
  accented letter, « », ’ or one of the inventory's unaccented French words, read from the
  TypeScript syntax tree so comments and regular expressions do not count — with a baseline of
  files that still hold literals; the baseline fails when it names a file that holds none, so it
  cannot go stale. ASCII French with none of those words is beyond the lint: changes 14–17 move
  every literal of their files by reading them. `lint-language-labels` admits the catalogue.
- **Conventions and glossary**: `src/i18n/README.md` — the register (tu in Spanish, neutral; the
  French unchanged), the terms (M10, M19), the typography each language keeps, how a key is named.
- **No surface switches.** Changes 14–17 move the surfaces; the HTML pages' static text moves with
  their surfaces.

## Capabilities

### New Capabilities

- `lingua-interface-language`: *The interface language is the reader's native language, under its
  own key*; *The interface's copy is kept in one typed catalogue per language*; *The French
  interface is byte for byte what it was*; *Copy quoted in a Lingua requirement is the French
  interface's* — the umbrella rule the programme names, in its two clauses: copy quoted in a
  Lingua requirement is the French interface's, and every interface language carries the same
  message in its own words. No such requirement is modified for it. The host app's and the agent
  plugin's copy follow in changes 28 and 55.

### Modified Capabilities

None.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (`src/i18n/**` — the key's module
  `language.ts` among them, `src/state/store.ts` for the key's writer, `test/lint-copy.spec.ts`,
  `test/lint-language-labels.spec.ts`, `test/i18n.spec.ts`). The Apple host app's and the agent
  plugin's copy are changes 28 and 55. ID, Music, Live, the back office and the site are untouched.
- **No byte moves.** No surface imports the catalogue yet; the French copy on screen is the same
  literals; the 42 test files asserting French copy pass unchanged; the bundles grow by the key's
  reader only.
- **Order.** `refine-lingua-review-session` (#696) adds French copy to review, Réglages and the
  popup, which the byte-for-byte extraction must hold: as the programme says, #696 merges before
  this change, or this change is rebased onto it and extracts its copy too. #696 is not on main;
  its copy is extracted by changes 15/16, or by #696's rebase onto the catalogue once those files
  are off the baseline (change 15's D5).
- **Not here.** Moving the surfaces (14–17); the card's wording and tense names (18); the labels
  of the native languages (19); the choice in Réglages and at onboarding (20); `_locales` and the
  listings (27); the host app (28); the agent (55).
