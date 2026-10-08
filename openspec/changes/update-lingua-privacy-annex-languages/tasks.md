# Tasks

## 1. The annexes (apps/site)

- [x] 1.1 `src/pages/confidentialite.md` Annex B per D1–D3; `updated` moved.
- [x] 1.2 `src/pages/en/privacy.md` and `src/pages/es/privacidad.md` likewise.

## 2. The App Store notes

- [x] 2.1 `apps/lingua-apple/README.md`: the rows name the studied language; categories unchanged (the account's language maps to no Apple data type); the deletion URLs list the English and Spanish pages beside `/suppression-compte/`, the Spanish one not yet linked from the extension (change 29's 3.2).

## 3. Gates, review and docs

- [x] 3.1 In `apps/site`: `yarn check`, `yarn build`, `yarn check:routes`.
- [ ] 3.2 [manual] The owner reviews the three annexes and deploys the site (M18) only after `add-lingua-native-language-server` is deployed and checked from outside (its 5.2), and before the store release that carries `add-lingua-native-language-sync-client` (its 6.1) — the annex says the native language reaches Cymbra with each card and day. Until then no site deploy is cut from a `main` holding this change (change 29's 4.3 deploys a ref before it); `updated` is moved to the deploy day if it slips.
- [x] 3.3 `openspec validate update-lingua-privacy-annex-languages --strict` passes, and `python3 scripts/openspec_archive_order.py update-lingua-privacy-annex-languages` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 31 is marked done in `docs/lingua/language-matrix-programme.md`.
