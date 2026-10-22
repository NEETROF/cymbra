# Tasks

## 1. The texts

- [ ] 1.1 `apps/lingua-extension/STORE-LISTING.md`: the Italian description for the Chrome Web Store and AMO, with its coverage sentence (D1); the Constants and Extra fields tables; the summary quoted from `_locales/it` (D3); `it` in the listing-languages line and the Graphics sets table; the claims table's Italian rows, each with the file that makes it true at change 45; both variants of extended translation while change 38 has not decided.
- [ ] 1.2 `apps/lingua-apple/STORE-LISTING.md`: the App Store it-IT name, subtitle, promotional text, keywords (counted in bytes), description and What's New for change 45's release, each with its count; the Fields table's it-IT column (Privacy `/it/privacy/`, Marketing `/it/lingua/`, Support as the owner chooses); the Locales note (a sixth locale for every later release, reviewed by the agent); *What the app itself says* with change 40's Italian step 3 (D1).
- [ ] 1.3 The shared English fields (D2): single purpose, permission justifications with the Italian labels, remote code naming en-it's model; the test instructions (993 at most, every label kept) and the review notes (4,000 at most, every label kept) with D2's measured cuts, their counts where the files record them; the data-usage prose's Italian consent labels.

## 2. Screenshots and procedure

- [ ] 2.1 The Italian screenshot sets listed (D4): the extension's from change 45's capture run, the App Store it-IT set per slot with its shot list.
- [ ] 2.2 « The dashboards' languages: Italian (change 45's first submission) », the seven steps applied to `it`, and its results table (D5).

## 3. Review and owner

- [ ] 3.1 [agent] The Italian reviewer agent's pass over every Italian text and the claims (D6), its findings applied or answered in the pull request.
- [ ] 3.2 [owner] The English fields reviewed, and the Open Question answered (D2); the support link (`/it/assistenza/` or `/en/support/`) and the Spanish one still unrecorded; the App Store it-IT screenshots from change 45's build; pasting the listings with change 45's submission, after the site deploy that follows it and an upload without publishing (D4, D5, D7).
- [ ] 3.3 [owner] The precedents' own tasks 3.2 closed (changes 36 and 37: their results tables recorded), so that the Italian procedure builds on facts (D5).

## 4. Gates

- [ ] 4.1 Every count recounted against its cap; `openspec validate add-lingua-italian-audience-listings --strict`; `python3 scripts/openspec_archive_order.py add-lingua-italian-audience-listings` (exit 10 naming only the `archiveAfter` changes); `docs/lingua/italian-programme.md` row 46's status; committed as `docs(lingua)` (proposal, *Impact*).
