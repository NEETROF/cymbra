# add-lingua-italian-listings — the listings, the site and the back office, for French speakers who study Italian

## Why

Change 21 of the [Italian programme](../../../docs/lingua/italian-programme.md), the last of stage 1.
Change 20 (`enable-lingua-italian`) lists it-fr for French speakers. What a reader is told about
Cymbra Lingua does not know it:

- **The store listings** (`apps/lingua-extension/STORE-LISTING.md` for the Chrome Web Store and
  addons.mozilla.org, `apps/lingua-apple/STORE-LISTING.md` for the App Store) tell French speakers
  they read English and Spanish. Every French text names those two; the English and Spanish texts
  say French speakers learn English and Spanish; the fields that are one per store have no Italian
  path and little room — the test instructions are at 998 of 1,000 characters, the App Store review
  notes at 3,984 of 4,000, the French keywords at 93 of 100 bytes, the French subtitle at 29 of 30.
- **The site's Lingua pages** are built from the shipped pairs and refuse a studied language their
  tables do not describe: measured, with an it-fr row the site's build fails on every Lingua page. So
  change 20 cannot list it-fr until the site knows Italian — the order the French listings kept
  (#868 before #880).
- **The back office's Lingua screen** names English, Spanish and French, and would show Italian as
  its code, « it ».

## What Changes

- **The site describes Italian** (design D1, D2): its name in the French, English and Spanish tables
  and what its card and levels are (`STUDIED.it`: tense and gender named, levels estimated); three
  tests that used Italian as their unnamed language take German; a seven-pair test. Inert until
  change 20 writes it-fr into the coverage file; then the French page, the French home card and the
  coverage table name Italian with no further edit.
- **The French texts name Italian** (D3): the Chrome Web Store and AMO description, the App Store's
  fr-FR subtitle (`Anglais, espagnol ou italien`), promotional text, keywords (`italien` in for
  `mots`), description (step 3 as change 20's activation page; Italian's levels said estimated) and a
  « What's New » paragraph modelled on 1.5.0's for Spanish. Within every limit (design *Measured*);
  the claims table gains the French texts' Italian claims.
- **The English and Spanish texts** say French speakers study Italian too, one sentence each (D4).
- **The fields one per store** (D5): the single purpose, the remote-code answer (the bases and
  layers of changes 3 and 4, it-fr's among them), the test instructions (993 / 1,000, two phrases cut)
  and the App Store review notes (3,989 / 4,000: #994's block for the next submission, which names the
  anonymous counts, with Italian's route, MarkIT and the Italian Wiktionary credited).
- **Figures taken when known** (D6): it-en's size from change 2's catalogue, the dictionary sentence
  from change 16's committed coverage, translation as change 19 and the owner leave it.
- **The back office names Italian** (D7): « Italien », "Italian", inert until a device reports it.
- **Screenshots and dashboards** (D8): one App Store fr-FR capture more on an Italian page; the
  Chrome Web Store's French set at its cap, unchanged unless the owner says otherwise; no new
  dashboard language, and a dashboard procedure for change 20's submission.
- **Merged before change 20** (D9); the texts pasted with change 20's release. The wording is the
  owner's (M9).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED *The store listings name each studied language* (held by
  `add-lingua-french-listings`): every word and scenario kept, *French speakers learning Italian*
  added.
- `site-lingua-page`: MODIFIED *The Lingua page says what each studied language's card and levels
  are* and *The coverage table reads at a phone's width* (both held by `add-lingua-french-listings`):
  *Six pairs on a phone* narrowed (« and no other pair »), every other word and scenario kept,
  *Italian studied* and *Seven pairs on a phone* added.

- `site-locales`: MODIFIED *The French home page describes Lingua for the readers it serves* (held by
  `add-site-lingua-showcase`): *Today's pairs* reads « and no other pair glossed in French », *Italian
  studied* added — the card then reads « Lisez le web en anglais, en espagnol ou en italien… ».

*The stores' summary names what its readers can study* holds as written: change 20 commits the French
summary. The back office's *The studied languages are named in the console's language* already asks
for a name.

## Impact

- **Products.** Lingua: the two listing files (text only; nothing builds from them). The site
  (`apps/site`): `src/lib/lingua-text.ts`, `README.md`, `test/lingua-text.spec.ts`. The back office:
  `src/i18n/locales/{fr,en}.json`, `test/lingua.spec.ts`. Nothing of the extension's code, packs,
  engine or interface; Cymbra ID, Music, Live and the backend untouched.
- **What a visitor sees before change 20**: nothing; from change 20, Italian on the French, English
  and Spanish Lingua pages, the French home card and the table, and in the listings once pasted.
- **Order.** Merged before change 20, whose pull request lists it-fr (D9). Builds on changes 2, 13, 14,
  16, 18 and 19 (the facts the texts state), on `add-lingua-french-listings` and
  `add-site-lingua-showcase`; archived after them and change 20 (`.openspec.yaml`). The review notes
  it builds on are #994's, outside the programme.
- **Effort, against 1.5–3 ideal days**: 1.45–2.45 (design, *Effort*).
- **The owner** reads the drafts (M9), answers the open questions, captures the App Store screenshot,
  and pastes the listings with change 20's release after the site deploy that publishes it-fr's
  figures (I12, M18).
