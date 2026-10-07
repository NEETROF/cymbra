# localise-lingua-settings — Réglages reads its copy from the catalogue

## Why

Change 15 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 1, a silent release, the second of the four that move the interface's copy into the
catalogue of change 13. Réglages is the largest surface: ≈ 190 unique texts in
`reading/settings-view.ts` (63 literals, the twelve block titles the hosts' lint reads),
`colour-settings-view.ts`, `book-display-view.ts`, `studied-languages-view.ts`,
`translation-setting.ts` (its `COPY`, `FAILURE` and `costText`), `account-setting.ts`
(`ACCOUNT_COPY`) and `sync/status.ts` (`lastSyncLabel`, `syncErrorCopy`). Three hosts mount it —
the popup, the side panel, the drawer — and `lint-settings-hosts` keeps the block titles string
literals in `settings-view.ts` so that no host duplicates them.

`refine-lingua-review-session` (#696) adds copy to `settings-view.ts` (« Rythme de révision »,
« Nouveaux mots par jour », « Fichier de sauvegarde », « Sources et confidentialité »): the
programme has it merge before this change, so its strings are extracted here too.

## What Changes

- **Réglages reads its copy from `src/i18n/<language>/{settings,colours,display,translation,
  account-setting,sync}.ts`**, picked by the interface language the hosts already read with
  their preferences; the block titles are catalogue entries, and `lint-settings-hosts` reads them
  from the French catalogue module instead of `settings-view.ts`.
- **The messages built from fragments** (« Je connais les **3000** mots les plus courants », the
  shortcut lines, the colour explanation, `costText`, `stateText`) become slot messages the view
  renders; the sync's « il y a 3 min. » and the French dates keep their strings.
- **The French byte for byte**: the `settings-view`, `colour-settings-view`, `translation-setting`,
  `account-setting`, `sync-messages` and `studied-languages-view` spec files pass unchanged.
- **The baseline** loses these files; each gains one test in English.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-interface-language`: ADDED *Réglages speaks the interface language*.

No requirement is modified; *One Réglages on every surface* (held by `add-lingua-read-aloud`)
and the requirements that quote Réglages' French copy are read under the umbrella rule.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension` (the files above, the catalogue
  modules, `test/lint-settings-hosts.spec.ts`, `test/lint-copy.spec.ts`'s baseline). ID, Music,
  Live, the back office and the site are untouched.
- **No byte moves.** The spec files are the check; the hosts' bundles grow by Réglages' English and
  Spanish copy (measured per entry in the pull request).
- **Order.** After change 13 and after #696 merges (its copy is extracted here); independent of
  14, 16, 17.
- **Not here.** The native language's block (20); the languages' names (19); the voices' platform
  names, which are the platform's.
