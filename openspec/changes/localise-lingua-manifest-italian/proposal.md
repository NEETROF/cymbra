# localise-lingua-manifest-italian — the manifest's text in Italian, ready before a pair glossed in Italian ships

## Why

Change 39 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3: Italian
speakers. What the browser shows about the extension — its description in the extensions page, which
both stores list as the summary, and its keyboard commands' descriptions in the shortcuts page — comes
from `manifest.json` and follows the browser's language through `_locales`
(`localise-lingua-manifest`). The build already copies a folder per native language a shipped pair is
glossed in, and refuses to build when that folder is missing: the day en-it ships (change 45), a
package without `_locales/it` fails, and `yarn check:version` fails too, because `LANGUAGE_NAMES` holds
no Italian words for the languages a description names (`add-lingua-french-listings`). This change
commits both ahead, inert until then.

## What Changes

- **`_locales/it/messages.json`** in `apps/lingua-extension` (design D1): the description and the
  three commands' descriptions in Italian, addressing the reader as « tu » (I6), in the words the
  Italian interface gives the same commands (change 33). The description names English, the language
  en-it studies, and only it: « Leggi il web in inglese: parole sconosciute evidenziate, percentuale
  reale. Offline e senza tracciamento. » (105 characters of the 112 Apple accepts). Commands are in the
  imperative, as the browsers' own Italian pages word theirs, and each contains the words Réglages
  gives the same command. The brand stays « Cymbra Lingua ».
- **The version check learns Italian's words** (D2): `LANGUAGE_NAMES.it` in `tool/check_version.mjs`,
  as the interface's Italian catalogue names the four languages, held equal to it by
  `test/check-version.spec.ts`.
- **Tests, no build change** (D3): `tool/manifests.mjs` already serves any native. A unit test builds
  each target's manifest with en-it listed and asserts `_locales/it` carried, English still the
  default, and every reference resolving. The two tests that pin the committed folders to fr, en and es
  gain `it`. A new test holds each command's description to Réglages' words in every language.
- **Reviewed by the Italian reviewer agent** (D4, I2), not by the owner; change 44 rereads it.
- **No package moves** (D5): while no pair glossed in Italian is listed, nothing carries `_locales/it`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *The manifest's Italian is committed before a pair glossed in
  Italian ships*. *The manifest speaks the browser's language among the shipped natives*
  (`localise-lingua-manifest`) is not modified: it already reads every native language a shipped pair
  is glossed in.

## Impact

- **Products.** Cymbra Lingua only: `apps/lingua-extension/_locales/it/messages.json` (new),
  `tool/check_version.mjs` (`LANGUAGE_NAMES.it`), `test/check-version.spec.ts`,
  `test/manifests.spec.ts`. ID, Music, Live, the back office and the site are untouched; the Apple host
  is change 40.
- **No byte moves** in any package: the three packages are built exactly as before while `packs.json`
  lists no pair glossed in Italian.
- **Prerequisites**: changes 20 (`LANGUAGE_NAMES` gains Italian in French, English and Spanish words),
  32 (the native type accepts `it`) and 33 (the Italian catalogue the check is held to, and the
  reviewer agent) implemented.
- **Order**: before change 45, which ships en-it and fails without it. Changes 50 (fr-it) and 54 (es-it)
  rewrite the description to name French and Spanish; the check fails them until they do.
- **Effort, against 0.5–1 ideal days**: 0.5–1.
