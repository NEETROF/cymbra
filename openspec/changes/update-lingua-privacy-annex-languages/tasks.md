# Tasks

## 1. The annexes (apps/site)

- [ ] 1.1 `src/pages/confidentialite.md` Annex B per D1–D3; `updated` moved.
- [ ] 1.2 `src/pages/en/privacy.md` and `src/pages/es/privacidad.md` likewise.

## 2. The App Store notes

- [ ] 2.1 `apps/lingua-apple/README.md`: the rows name the studied language; categories unchanged.

## 3. Gates, review and docs

- [ ] 3.1 In `apps/site`: `yarn check`, `yarn build`, `yarn check:routes`.
- [ ] 3.2 [manual] The owner reviews the three annexes and deploys the site (M18).
- [ ] 3.3 `openspec validate update-lingua-privacy-annex-languages --strict` passes, and `python3 scripts/openspec_archive_order.py update-lingua-privacy-annex-languages` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 31 is marked done in `docs/lingua/language-matrix-programme.md`.
