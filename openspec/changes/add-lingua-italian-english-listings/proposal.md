# add-lingua-italian-english-listings — the English listings and the site name Italian for English speakers

## Why

Change 26 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. Change 25 lists
it-en for English speakers. What an English reader is told about Cymbra Lingua does not know it: every
English text of the listings names Spanish and French; the French and Spanish texts say English speakers
learn Spanish and French; the fields one per store are at their limits once change 21 has written
Italian for French speakers — the test instructions at 993 of 1,000, the review notes at 3,989 of 4,000,
the English keywords at 99 of 100 bytes, the English subtitle at 30 of 30. And three requirements would
read false once it-en is listed: change 21's *French speakers learning Italian* (« no English … text
offers Italian »), its *Seven pairs on a phone*, and the English home card's *English speakers learning
Spanish and French* — the card being built from the shipped pairs, it names Italian the day change 25's
coverage file lists it-en.

## What Changes

- **The English texts name Italian** (design D2): the Chrome Web Store and AMO description, the App
  Store's en-US and en-GB subtitle (`Spanish, French or Italian`), promotional text, keywords (`italian`
  in for `words` and `deck`), description (step 3 as change 25's activation page; Italian's levels said
  estimated) and a « What's New » paragraph. Within every limit (design *Measured*).
- **The French and Spanish texts** say English speakers study Italian too, one sentence each (D3).
- **The fields one per store** (D4): the test instructions (988 / 1,000), the review notes (3,994 /
  4,000), the single purpose, the remote code.
- **Figures taken when known** (D5): it-en's size, the dictionary sentence only if it-en sits clearly
  under es-en and fr-en, translation as change 24 and the owner leave it.
- **The claims table** (D6), **screenshots and dashboards** (D7): one App Store en-US capture more; the
  Chrome Web Store's English set unchanged unless the owner says otherwise.
- **The site** (D1): no code; a test of the English home card and page with eight pairs.
- **Implemented before change 25** (D8); the texts pasted with change 25's release. The wording is the
  owner's (M9).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED *The store listings name each studied language* (held by
  `add-lingua-italian-listings`): *French speakers learning Italian* holds while it-fr is the one Italian
  pair; *English speakers learning Italian* added.
- `site-lingua-page`: MODIFIED *The coverage table reads at a phone's width* (held by
  `add-lingua-italian-listings`): *Six pairs* and *Seven pairs on a phone* hold « and no other pair »;
  *Eight pairs on a phone* added.
- `site-locales`: MODIFIED *The English home page describes Lingua for the readers it serves* (held by
  `add-lingua-french-listings`): *English speakers learning Spanish and French* holds « and no other pair
  glossed in English »; *English speakers learning Italian* added.

## Impact

- **Products.** Lingua: the two listing files (text only). The site (`apps/site`):
  `test/lingua-text.spec.ts`, no code. Nothing of the extension, the back office (Italian already named
  by change 21) or the backend.
- **What a visitor sees before change 25**: nothing; from change 25, Italian on the English home card and
  Lingua page and in the table, and in the listings once pasted.
- **Order.** Implemented before change 25 (D8); after change 21, on whose drafts it builds. Archived after
  the changes of `.openspec.yaml`.
- **Effort, against 1.5–3 ideal days**: 1.35–2.1 (design, *Effort*).
- **The owner** reads the drafts (M9), answers the open questions, captures the App Store screenshot and
  pastes the listings with change 25's release after the site deploy (I12, M18).
