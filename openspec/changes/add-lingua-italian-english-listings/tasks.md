# Tasks

Implemented after change 21 and before change 25 (design D8).

## 1. The site (apps/site)

- [ ] 1.1 `test/lingua-text.spec.ts`: a test feeding eight pairs with change 2's routes asserts the English home card « Read the web in Spanish, French or Italian… », the English page's audience and its row « Italian → English » (D1); `yarn test`, `yarn build`, `yarn check:routes`.

## 2. The listings

- [ ] 2.1 `apps/lingua-extension/STORE-LISTING.md`: the English description (D2), the Spanish sentence (D3), the test instructions, the single purpose and the remote code (D4), the notes that say what English speakers study, the claims table's rows (D6), `## Summary` quoting change 25's English summary, the dashboards' procedure for change 25's submission (D7); each count recorded as the file counts; `yarn format:check`.
- [ ] 2.2 `apps/lingua-apple/STORE-LISTING.md`: the en-US and en-GB subtitle, promotional text, keywords (heading in bytes), description and « What's New » paragraph (D2), the es and fr sentences (D3), the review notes (D4), the locales note and fields table, *What the app itself says* quoting change 25's English step 3, the *Screenshots* section (D7); each count recorded.
- [ ] 2.3 The figures of D5 taken from `main` when this is implemented: it-en's served size, its committed coverage beside es-en's and fr-en's, change 24's figures and the owner's answer at change 25; each sentence that depends on one written to match.

## 3. Gates

- [ ] 3.1 `openspec validate add-lingua-italian-english-listings --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-english-listings` names its prerequisites; the MODIFIED blocks re-copied from changes 21's and `add-lingua-french-listings`' requirements as they then stand.
- [ ] 3.2 `docs/lingua/italian-programme.md` row 26's status names this change's pull requests.

## 4. Owner

- [ ] 4.1 [manual] The owner reads the drafts (M9) and answers the design's Open Questions 1 to 5.
- [ ] 4.2 [manual] With change 25's release: the App Store en-US capture on an Italian page (D7), the listings pasted after the site deploy that publishes it-en's figures, and what each dashboard shows recorded (I12, M18).
