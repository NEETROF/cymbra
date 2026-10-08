# Tasks

## 1. The listings

- [ ] 1.1 `apps/lingua-extension/STORE-LISTING.md`: the Spanish description, single purpose, permissions, remote code, test instructions (≤ 1,000), the summary quoted from `_locales/es`; the listing-language line (D1, D2).
- [ ] 1.2 `apps/lingua-apple/STORE-LISTING.md`: es-ES and es-MX name, subtitle, promotional text, keywords, description, « What's New », each with its count; the review notes; no beta, no price (D2, D3).
- [ ] 1.3 The screenshot list per platform and locale (D4).

## 2. Checks

- [ ] 2.1 Every count within its store's limit and equal to the text; `openspec validate add-lingua-spanish-audience-listings --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-spanish-audience-listings` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived).

## 3. Owner

- [ ] 3.1 [manual] The owner settles M16's locales, reviews the drafts (M9), captures the screenshots and pastes the listings with change 35's release; change 37 is marked done in `docs/lingua/language-matrix-programme.md`.
