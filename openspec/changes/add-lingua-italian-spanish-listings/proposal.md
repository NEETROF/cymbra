# add-lingua-italian-spanish-listings — the Spanish listings and the site name Italian for Spanish speakers

## Why

Change 31 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. Change 30 lists
it-es for Spanish speakers, through English (I4, I11). Nothing a Spanish reader is told about Cymbra
Lingua knows that yet:
- Every Spanish text of the listings names English and French.
- Change 46's Italian texts say that Spanish speakers study English and French. No French or English
  text says what Spanish speakers study.
- The App Store's Spanish keywords would go over 100 bytes with `italiano`.
- The review notes are at about 4,000 of 4,000 characters once change 46 has written Italian speakers in.

Five scenarios would read false, or only true by accident, once it-es is listed:
- *fr-es held below its floor* (« the Spanish texts name English alone ») and *Spanish speakers
  learning French* (« name English and French »);
- *French speakers learning Italian* and *English speakers learning Italian* (« no Spanish text offers
  Italian to its readers »);
- the Spanish home card's *Spanish speakers learning English* (« names English as the language
  read »). The card is built from the shipped pairs, so it names Italian the day change 30's coverage
  file lists it-es.

The table on the Spanish page gains a ninth or tenth row, which no scenario covers.

## What Changes

- **The Spanish texts name Italian** (design D2):
  - the Chrome Web Store and AMO description;
  - on the App Store's es-ES and es-MX: the subtitle (`Inglés, francés o italiano`), the promotional
    text, the keywords (one word gives way to `italiano`), the description and a « What's New »
    paragraph.
  - Italian's levels are said to be estimated, its translation through English is given with its size,
    and the dictionary sentence names Italian. Every text stays within its limit (design *Measured*).
- **Change 46's Italian texts**, the only others that say what Spanish speakers study, gain Italian (D3).
- **The fields one per store** (D4): the review notes (cuts named), the single purpose, the remote code;
  the test instructions unchanged.
- **Figures taken when known** (D5): it-es's two-model download, its coverage, and its translation as
  change 29 and the owner leave it (M15).
- **Where the definitions come from** (D6): the Spanish texts' « escritas por personas » claim is checked
  against the pivot (I11) and Wikidata. The claims table gains its Italian rows.
- **Screenshots and dashboards** (D7): one App Store es-ES capture more; the Chrome Web Store's Spanish
  set unchanged.
- **The site** (D1): no code; a test of the Spanish home card and page with it-es listed.
- **Implemented before change 30** (D8); the texts pasted with change 30's release. The wording is the
  owner's (M9).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: MODIFIED *The store listings name each studied language* (held by
  `add-lingua-italian-audience-listings`, change 46):
  - *fr-es held below its floor* holds « nor it-es »;
  - *Spanish speakers learning French* holds « and not it-es »;
  - in *French speakers learning Italian* and *English speakers learning Italian*, the clause that no
    Spanish text offers Italian holds « while it-es is not listed »;
  - *Spanish speakers learning Italian* is added;
  - change 46's *A listing in Italian* is kept.
- `site-lingua-page`: MODIFIED *The coverage table reads at a phone's width* (held by
  `enable-lingua-italian-speakers`, change 45, whose *Eight pairs* and *Nine pairs on a phone* already
  hold « and no other pair »). It adds *Nine pairs on a phone, it-es the ninth* and *Ten pairs on a
  phone*.
- `site-locales`: MODIFIED *The Spanish home page describes Lingua for the readers it serves* (held by
  `extend-site-spanish-locale`). *Spanish speakers learning English* holds « and no other pair glossed
  in Spanish », and *Spanish speakers learning Italian* is added.

## Impact

- **Products.**
  - Lingua: the two listing files (text only).
  - The site (`apps/site`): `test/lingua-text.spec.ts`, no code.
  - Untouched: the extension, the back office (Italian already named by change 21) and the backend.
- **What a visitor sees.** Nothing before change 30. From change 30 on, Italian appears on the Spanish
  home card, the Lingua pages and the table, and in the listings once they are pasted.
- **Order.**
  - Implemented after changes 26 and 27, after change 29's figures and the owner's answer under M15,
    and before change 30 (D8).
  - Archived after the changes of `.openspec.yaml`, change 46 among them (its *A listing in Italian* is
    carried here) and change 45 (whose table block this one extends).
- **Effort, against 1.5–3 ideal days**: 1.35–2.75 (design, *Effort*).
- **The owner** reads the drafts (M9), answers the open questions, captures the App Store screenshot,
  and pastes the listings with change 30's release, after the site deploy (I12, M18).
