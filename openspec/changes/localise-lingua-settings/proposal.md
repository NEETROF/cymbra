# localise-lingua-settings — Réglages reads its copy from the catalogue

## Why

Change 15 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
in stage 1, a silent release, the second of the four that move the interface's copy into the
catalogue of change 13. Réglages is the largest surface: ≈ 190 unique texts in
`reading/settings-view.ts` (63 literals, the eleven literal block titles the hosts' lint reads —
the twelfth, the level's, is `levelTitle(language)`),
`colour-settings-view.ts`, `book-display-view.ts`, `studied-languages-view.ts`,
`translation-setting.ts` (its `COPY`, `FAILURE` and `costText`), `account-setting.ts`
(`ACCOUNT_COPY`) and `sync/status.ts` (`lastSyncLabel`, `syncErrorCopy`). Three hosts mount it —
the popup, the side panel, the drawer — and `lint-settings-hosts` keeps the block titles string
literals in `settings-view.ts` so that no host duplicates them.

`refine-lingua-review-session` (#696) adds copy to `settings-view.ts` (« Rythme de révision »,
« Nouveaux mots par jour », « Fichier de sauvegarde », « Sources et confidentialité »): whichever
of the two merges second rebases on the other — this change extracts #696's strings if #696 is
on main by then; otherwise #696, rebased, puts its strings in the catalogue in three languages,
which the lint forces once `settings-view.ts` is off the baseline.

## What Changes

- **Réglages reads its copy from `src/i18n/<language>/{settings,studied-languages,colours,display,
  translation,account-setting,sync}.ts`** (`account-setting` is the Réglages block's; `account` is the account
  page's, change 17's), picked by the interface language the popup and the side panel read with
  their preferences and the drawer is handed by the reading session (change 14); the block titles
  are catalogue entries, and `lint-settings-hosts` imports them from the French catalogue module
  instead of reading `settings-view.ts`.
- **The messages built from fragments** (« Je connais les **3000** mots les plus courants », the
  shortcut lines, the colour explanation, `costText`, `stateText`) become slot messages the view
  renders; the sync's « il y a 3 min. » reads change 13's `syncedMinutesAgo`/`syncedHoursAgo` as
  written, and the French dates keep their strings.
- **The voices' labels** (`reading/speech.ts`, a French `Intl.DisplayNames` the lint cannot see)
  go through a `regionName(language, code)` helper and a `settings.voiceLabel` slot message.
- **The French byte for byte**: the `settings-view`, `colour-settings-view`, `book-display-view`,
  `translation-setting`, `account-setting`, `sync-messages` and `studied-languages-view` spec
  files pass unchanged — they mount without a language, and no language means French.
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
- **Order.** After change 13 and after change 14 (the session hands the drawer the interface
  language); #696 and this change rebase on whichever merges first (D5); independent of 16, 17.
- **Not here.** The native language's block (20); the languages' names in the titles and notices
  (19 — they stay French in a Spanish Réglages until then); the account messages the block shares
  with the account page (`account/flow.ts`'s, 17); the voices' platform names, which are the
  platform's.
