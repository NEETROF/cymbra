# Tasks

Implemented after change 21, after changes 22, 23 and 24 and the owner's answer to change 25's Open Question 2; before change 25 (design D8).

**Where it stood when implemented (2026-10-25).** Changes 22, 23 and 24 merged. Change 25's Open
Questions answered: 1, its English summary is version A, « The web in Spanish, French or Italian:
unknown words highlighted, an honest percentage. Offline and private. » (108 characters), as the
French, and its activation page takes D6's drafts (the owner); 2, settled by change 24's figures —
it-en is on the marks' tier (94 of 97 correct, 3 % withheld), so its translation is offered, marked;
3, change 23 merged. Change 21 (`add-lingua-italian-listings`, #1057) is **not** on `main`: this
change builds on its drafts, so its pull request is based on `main` and keeps to what a rebase on
#1057 merges — a section of its own in each listing file — and lists what waits on #1057's merge
(1.1, and 2.1's and 2.2's notes edited in place).

## 1. The site (apps/site)

- [ ] 1.1 `test/lingua-text.spec.ts`: a test feeding eight pairs with change 2's routes asserts the English home card « Read the web in Spanish, French or Italian… », the English page's audience and its row « Italian → English » (D1); `test/astro/english-home.spec.ts`: es-en, fr-en and it-en with change 2's it-en route render « Read the web in Spanish, French or Italian… » and the button to `/en/lingua`, as `add-lingua-french-listings` pinned each home-card scenario; `apps/site/README.md`'s home-card examples name Italian; `yarn test`, `yarn build`, `yarn check:routes`; the English Lingua page with eight pairs read at 375 px in a browser, the page not scrolling sideways. — **Waits on #1057's merge**: the test reads change 21's `names.it`, `STUDIED.it`, `SEVEN_PAIRS` and it-fr's route, without which a pair studying Italian fails the build. Written and run on change 21's branch: `yarn test` (16 files, 248 tests), `yarn build` and `yarn check:routes` pass, and, with it-fr and it-en added to the coverage file on a scratch copy, the three Lingua pages read at 375 px measure 375 px with the table 335 px in its 335 px box (eight rows; « Italian → English » 118.6 px), at 320 px the table scrolling in its 280 px box; landed by rebasing this change once #1057 merges.

## 2. The listings

- [ ] 2.1 `apps/lingua-extension/STORE-LISTING.md`: the English description (D2), the Spanish sentence (D3), the test instructions, the single purpose and the remote code (D4), the notes that say what English speakers study, the claims table's rows (D6), `## Summary` quoting change 25's English summary, the dashboards' procedure for change 25's submission (D7); each count recorded as the file counts; `yarn format:check`. — **Done but for the notes edited in place**: the texts, fields, claims, `## Summary`'s quote and the dashboards' procedure are in a section of their own, _Italian for English speakers (change 25's release)_ — EN 3,076 / 16,000 (2,993 without the dictionary sentence on Italian), ES with fr-es 3,436, test instructions 988 / 1,000 (995 with line breaks counted twice), single purpose 611, the English summary 108 / 112 —; `yarn format:check` passes. The *Listing languages* paragraph, the English description's header, the *Description* introduction, the claims table's lead sentence and its « Who learns what » row, the test instructions' and the single purpose's notes and the permissions' introduction are change 21's lines, edited by #1057: they wait on its merge, then point to the section.
- [ ] 2.2 `apps/lingua-apple/STORE-LISTING.md`: the en-US and en-GB subtitle, promotional text, keywords (heading in bytes), description and « What's New » paragraph (D2), the es and fr sentences (D3), the review notes (D4), the locales note and fields table, *What the app itself says* quoting change 25's English step 3, the *Screenshots* section (D7); each count recorded. — **Done but for the notes edited in place**: a section of its own, _Italian for English speakers (change 25's release)_ — subtitle 26 / 30, promotional text 139 / 170, keywords 96 / 100 bytes, description 3,219 / 4,000 (3,136 without the dictionary sentence on Italian), « What's New » 437 (387 without the voice), es-ES/es-MX 3,447, fr-FR 3,310, review notes 3,971 / 4,000 (3,993 with line breaks counted twice; change 21's block measured 3,975, not the 3,989 the design counted on, and a third cut keeps its margin), the en-US capture, change 25's step 3 quoted. The locales note, the *Fields* table and *What the app itself says* are change 21's lines, edited by #1057: they wait on its merge.
- [x] 2.3 The figures of D5 taken from `main` when this is implemented: it-en's served size, its committed coverage beside es-en's and fr-en's, change 24's figures and the owner's answer at change 25; each sentence that depends on one written to match. — Taken on `main` at `53da44cf4`: it-en's served size 26,242,301 B, « 26.2 MB » (so « 26.2 MB each » holds); its committed coverage 91.3 / 84.2 / 75.8 % (`gloss_coverage.py --pair it-en`) under es-en's 93.0 / 86.5 / 76.5 and fr-en's 93.9 / 87.2 / 76.8 at each top, so the dictionary sentence on Italian is written; change 24's marks, 94 of 97 right, 3 % withheld, and change 25's Open Question 2 answered by them: translation offered, marked.

## 3. Gates

- [x] 3.1 `openspec validate add-lingua-italian-english-listings --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-english-listings` names its prerequisites; the MODIFIED blocks re-copied from changes 21's and `add-lingua-french-listings`' requirements as they then stand.
- [x] 3.2 `docs/lingua/italian-programme.md` row 26's status names this change's pull requests.

## 4. Owner

- [ ] 4.1 [manual] The owner reads the drafts (M9) and answers the design's Open Questions 1 to 5.
- [ ] 4.2 [manual] With change 25's release: the App Store en-US capture on an Italian page (D7), the listings pasted after the site deploy that publishes it-en's figures, and what each dashboard shows recorded (I12, M18).
