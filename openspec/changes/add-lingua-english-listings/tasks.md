# Tasks

## 1. The listings

- [ ] 1.1 `apps/lingua-extension/STORE-LISTING.md`: the English description; the summary quoted from `_locales/en`; the single purpose, the permission justifications and the remote-code answer updated in place, the remote-code answer naming es-en as the direct model for an English-native reader; the one test-instructions field rewritten for a non-French browser presetting the English interface studying Spanish, labels quoted from `src/i18n/en/`, how to choose French in Réglages, within 1,000 characters; the listing-language line and the French texts that say the product is French-only (D1, D2, D3).
- [ ] 1.2 `apps/lingua-apple/STORE-LISTING.md`: en-US and en-GB name, subtitle, promotional text, keywords, description, « What's New », each with its count, the en-GB text the en-US text; each locale's Privacy, Support and Marketing URLs; the review notes rewritten within 4,000 characters ("Purpose & audience", "Regional differences", "Third-party material" with the es-en sources, the translation model and its size); the French description's last line; no beta, no price (D2, D4).
- [ ] 1.3 The screenshot list per platform and locale (D6).
- [ ] 1.4 `apps/lingua-extension/STORE-LISTING.md`: the dashboards' languages — the upload without publishing, `fr` re-entered, `en` filled, AMO's check — and a place for both results (D5).

## 2. Checks

- [ ] 2.1 Every count within its store's limit and equal to the text; `openspec validate add-lingua-english-listings --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-english-listings` exits 0.

## 3. Owner

- [ ] 3.1 [manual] The owner settles M16's locales, reviews the drafts (M9), captures the screenshots and pastes the listings with change 34's release, after the site deploy that publishes its figures; change 36 is marked done in `docs/lingua/language-matrix-programme.md`.
- [ ] 3.2 [manual] The first package carrying `_locales` is uploaded to the Chrome Web Store without publishing (a dashboard upload of the build artifact, or a release input that skips `:publish`), the French listing re-entered under `fr` and the English filled under `en`, then submitted; addons.mozilla.org's default locale and summary checked after submission; both results recorded in `STORE-LISTING.md` (D5).
