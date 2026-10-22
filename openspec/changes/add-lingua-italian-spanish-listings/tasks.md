# Tasks

Implemented after changes 21 and 26, after change 27's committed tables, change 29's figures and the owner's answer under M15; before change 30 (design D8).

## 1. The site (apps/site)

- [ ] 1.1 `test/lingua-text.spec.ts`: a test feeding nine pairs with change 2's routes asserts the Spanish home card « Lee la web en inglés, en francés o en italiano… », the Spanish page's « Pensada para hispanohablantes que aprenden inglés, francés o italiano » and its row « Italiano → español » (D1).
- [ ] 1.2 `test/astro/spanish-pages.spec.ts`: en-es, fr-es and it-es, with change 2's routes, render « Lee la web en inglés, en francés o en italiano… », no audience, and the button to `/es/lingua`.
- [ ] 1.3 `apps/site/README.md`'s home-card examples name Italian. If the owner names Wikidata (D6), the French, English and Spanish `glossesOrigin` in `src/lib/lingua-text.ts` and their pinned tests follow.
- [ ] 1.4 `yarn test`, `yarn build`, `yarn check:routes`; the Spanish Lingua page with nine pairs read at 375 px in a browser, the page not scrolling sideways (ten pairs if change 45 has listed en-it).

## 2. The listings

- [ ] 2.1 `apps/lingua-extension/STORE-LISTING.md`, each count recorded as the file counts, then `yarn format:check`:
  - the Spanish description (D2);
  - the single purpose and the remote code (D4);
  - the notes that say what Spanish speakers study (D4);
  - the claims table's rows (D6);
  - `## Summary` quoting change 30's Spanish summary;
  - the dashboards' procedure for change 30's submission (D7);
  - the variants for it-es's translation withheld (D5);
  - the single purpose's and the remote code's note: pasted with change 30's release (D4, D8).
- [ ] 2.2 `apps/lingua-apple/STORE-LISTING.md`, each count recorded:
  - the es-ES and es-MX subtitle, promotional text, keywords (heading in bytes), description and « What's New » paragraph (D2);
  - the review notes (D4), labelled for change 30's submission, the block without it-es kept for a submission before it;
  - the locales note, the fields table and the es description's lead note;
  - the review notes' lead paragraphs and *What the app itself says* (D4);
  - the *Screenshots* section (D7).
- [ ] 2.3 Change 46's Italian line on what Spanish speakers study, if change 46 is on `main`: both forms kept, labelled, the one with it-es for change 30's release (D3, D8). The I2 reviewer agent's pass on it, its verdict in the pull request.
- [ ] 2.4 The figures of D5, taken from `main` when this is implemented: it-en's and en-es's served sizes, it-es's committed coverage beside en-es's, change 29's figures and the owner's answer at change 30. Each sentence that depends on one is written to match.

## 3. Gates

- [ ] 3.1 `openspec validate add-lingua-italian-spanish-listings --strict` passes.
- [ ] 3.2 `python3 scripts/openspec_archive_order.py add-lingua-italian-spanish-listings` names its prerequisites.
- [ ] 3.3 The MODIFIED blocks are re-copied from changes 45's, 46's and `extend-site-spanish-locale`'s requirements as they then stand.
- [ ] 3.4 Row 31 of `docs/lingua/italian-programme.md` names this change's pull requests.

## 4. Owner

- [ ] 4.1 [manual] The owner reads the drafts (M9) and answers the design's Open Questions 1 to 7.
- [ ] 4.2 [manual] With change 30's release: the App Store es-ES capture on an Italian page (D7), the listings pasted after the site deploy that publishes it-es's figures, and what each dashboard shows recorded (I12, M18).
