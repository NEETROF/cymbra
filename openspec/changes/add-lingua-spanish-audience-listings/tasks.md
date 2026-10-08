# Tasks

## 1. The listings

- [ ] 1.1 `apps/lingua-extension/STORE-LISTING.md`: the Spanish description; the summary quoted from `_locales/es`; the test instructions gaining the Spanish path within 1,000 characters, labels quoted from `src/i18n/es/`; the single purpose, the permission justifications and the remote-code answer updated in place, the remote-code answer naming en-es as the direct model for a Spanish-native reader; the listing-language line (D1, D2, D3).
- [ ] 1.2 `apps/lingua-apple/STORE-LISTING.md`: es-ES and es-MX name, subtitle, promotional text, keywords, description, « What's New », each with its count, the es-MX text the es-ES text; each locale's Privacy, Marketing and Support URLs; the review notes gaining the Spanish path within 4,000 characters; no beta, no price (D2, D3, D4).
- [ ] 1.3 The screenshot list per platform and locale (D6).
- [ ] 1.4 `apps/lingua-extension/STORE-LISTING.md`: the dashboards' languages for `es`, and a place for both results (D5).

## 2. Checks

- [ ] 2.1 Every count within its store's limit and equal to the text; `openspec validate add-lingua-spanish-audience-listings --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-spanish-audience-listings` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived).

## 3. Owner

- [ ] 3.1 [manual] The owner reviews the drafts (M9), chooses the Support URL, captures the screenshots and pastes the listings with change 35's release, after the site deploy that follows it; change 37 is marked done in `docs/lingua/language-matrix-programme.md`.
- [ ] 3.2 [manual] The first package carrying `_locales/es` is uploaded to the Chrome Web Store without publishing, the Spanish listing filled under `es`, then submitted; addons.mozilla.org's Spanish summary checked after submission; both results recorded in `STORE-LISTING.md` (D5).
