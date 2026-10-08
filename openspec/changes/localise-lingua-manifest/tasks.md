# Tasks

## 1. The messages (apps/lingua-extension)

- [ ] 1.1 `_locales/{fr,en,es}/messages.json`: `extensionDescription` and the three commands' descriptions, the French byte for byte, the English and Spanish drafts (D1; M9, M10).

## 2. The build and the checks

- [ ] 2.1 `tool/manifests.mjs` (pure) imported by `build.mjs`, and `tool/packs.mjs`'s `shippedNatives` held equal to `pairs.ts`'s: the localised manifest and the shipped natives' folders only when a non-French native ships; `default_locale` per D2; a French-only build byte for byte today's (D2). Test: the manifest step with `en-fr`, `es-fr` (unchanged) and with es-en added (localised, `en` default) for each target.
- [ ] 2.2 `tool/check_version.mjs`: every committed description within 112 characters, naming the language; `_locales/fr` equal to the manifest's literal French (D3). Tests: an over-long Spanish description fails; a French message that differs from the manifest fails.
- [ ] 2.3 `tool/check_variants.mjs`: references, `default_locale`, folders and shipped natives agree (D3).
- [ ] 2.4 `apps/lingua-apple`: both extension targets' "Copy Lingua extension" phase removes `${DEST}/_locales` before its rsync.
- [ ] 2.5 [manual] D4's check on the three browsers with a local `packs.json` holding es-en, and the owner's validation of a localised Safari archive, before change 34 merges.

## 3. Gates and docs

- [ ] 3.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`, `yarn check:version`; the three built manifests byte for byte today's.
- [ ] 3.2 `STORE-LISTING.md`: the summary comes from `_locales` once a non-French native ships (D2); `README.md`'s release notes.
- [ ] 3.3 [manual] The owner reviews the English and Spanish descriptions (M9).
- [ ] 3.4 `openspec validate localise-lingua-manifest --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-manifest` exits 0; change 27 is marked done in `docs/lingua/language-matrix-programme.md`.
