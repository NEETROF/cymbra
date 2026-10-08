# Tasks

## 1. The messages (apps/lingua-extension)

- [x] 1.1 `_locales/{fr,en,es}/messages.json`: `extensionDescription` and the three commands' descriptions, the French byte for byte, the English and Spanish drafts (D1; M9, M10).

## 2. The build and the checks

- [x] 2.1 `tool/manifests.mjs` (pure) imported by `build.mjs`, and `tool/packs.mjs`'s `shippedNatives` held equal to `pairs.ts`'s: the localised manifest and the shipped natives' folders only when a non-French native ships; `default_locale` per D2; a French-only build byte for byte today's (D2). Test: the manifest step with `en-fr`, `es-fr` (unchanged) and with es-en added (localised, `en` default) for each target.
- [x] 2.2 `tool/check_version.mjs`: every committed description within 112 characters, naming the language; `_locales/fr` equal to the manifest's literal French (D3). Tests: an over-long Spanish description fails; a French message that differs from the manifest fails.
- [x] 2.3 `tool/check_variants.mjs`: references, `default_locale`, folders and shipped natives agree (D3).
- [x] 2.4 `apps/lingua-apple`: both extension targets' "Copy Lingua extension" phase removes `${DEST}/_locales` before its rsync.
- [ ] 2.5 [manual] D4's check on the three browsers with a local `packs.json` holding es-en, and the owner's validation of a localised Safari archive, before change 34 merges. Should change 35 (en-es) merge first, its package is the first localised one and this validation applies to it instead; once Apple has accepted one localised archive, the other change adds a folder of the same shape and needs no second validation. The first localised local build fails `yarn check:variants` on its hard-coded `SHIPPED_PAIRS` gate before the locale checks report — change 34 (or 35) edits that constant with `packs.json`, so the locale checks are read from the build run with that edit. The scenario *A browser in German* has this task as its only verification.

## 3. Gates and docs

- [x] 3.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; the three built manifests byte for byte today's.
- [x] 3.2 `STORE-LISTING.md`: the summary comes from `_locales` once a non-French native ships (D2); `README.md`'s release notes.
- [ ] 3.3 [manual] The owner reviews the English and Spanish descriptions (M9).
- [x] 3.4 `openspec validate localise-lingua-manifest --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-manifest` exits 0; change 27 is marked done in `docs/lingua/language-matrix-programme.md`.
