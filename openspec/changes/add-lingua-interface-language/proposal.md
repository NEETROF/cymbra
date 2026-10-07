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
  and `en` and `es` typed `typeof fr`. Keys are named by meaning, never by the French text.
  `src/i18n/index.ts` gives a surface its copy for a language.
- **Messages with slots and plurals.** A sentence built from fragments today is one message
  taking its parts; a count takes `Intl.PluralRules` forms. In French the forms are the current
  strings, so « carte(s) » stays « carte(s) ».
- **The interface language, under its own key**: `cymbra-lingua-interface-language` in
  `chrome.storage.local`, `fr`, `en` or `es`; absent means `fr` (M22). The background writes it from
  the profile's native language (change 4) whenever that is read or set, so the choice is the
  native language (M2) and a later override needs no migration. Every surface can read it before
  its first paint, as it reads its other preferences.
- **Formats by language**: numbers and dates through the interface language's locale; French
  keeps `fr-FR` and its current strings (« 25,8 Mo », « il y a 3 min. »).
- **A lint**: no French literal in `src/` outside `src/i18n/`, with a baseline of files that still
  hold literals; the baseline fails when it names a file that holds none, so it cannot go stale.
  `lint-language-labels` admits the catalogue.
- **Conventions and glossary**: `src/i18n/README.md` — the register (tu in Spanish, neutral; the
  French unchanged), the terms (M10, M19), the typography each language keeps, how a key is named.
- **No surface switches.** Changes 14–17 move the surfaces; the HTML pages' static text moves with
  their surfaces.

## Capabilities

### New Capabilities

- `lingua-interface-language`: *The interface language is the reader's native language, under its
  own key*; *Copy lives in one typed catalogue per language*; *The French interface is byte for
  byte what it was*; *Copy quoted in a Lingua requirement is the French interface's* — the umbrella
  rule the programme names: a requirement of any `lingua-*` capability that quotes copy quotes the
  French interface's, and every interface language carries the same message in its own words. No
  such requirement is modified for it.

### Modified Capabilities

None.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (`src/i18n/**`, `src/state/storage.ts`
  for the key, `src/background.ts` for its writer, `test/lint-copy.spec.ts`,
  `test/lint-language-labels.spec.ts`, `test/i18n.spec.ts`). The Apple host app's and the agent
  plugin's copy are changes 28 and 55. ID, Music, Live, the back office and the site are untouched.
- **No byte moves.** No surface imports the catalogue yet; the French copy on screen is the same
  literals; the 41 test files asserting French copy pass unchanged; the bundles grow by nothing a
  surface imports.
- **Order.** `refine-lingua-review-session` (#696) adds French copy to review, Réglages and the
  popup; those files are on the lint's baseline until changes 15 and 16 take them off, so #696
  lands before or after this change without a rebase, and before 15 and 16.
- **Not here.** Moving the surfaces (14–17); the card's wording and tense names (18); the labels
  of the native languages (19); the choice in Réglages and at onboarding (20); `_locales` and the
  listings (27); the host app (28); the agent (55).
