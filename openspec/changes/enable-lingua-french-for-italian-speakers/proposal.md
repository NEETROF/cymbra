# enable-lingua-french-for-italian-speakers — fr-it ships, with its listings: Italian speakers learn French

## Why

Change 50 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 4. Change 45 made
Italian an interface language and listed en-it; change 46 wrote the Italian listings, which name English
alone. Changes 47–49 build fr-it: its glosses (47), their refinement (48) and the marks of its two-model
route, fr-en then en-it (49).

What remains is to list fr-it, and to make what an Italian reader is told know it:
- the list;
- the Italian description and the host app's Italian activation page, which the version check and
  *Guided activation* require to name what Italian readers study;
- the memory sentence's Italian cases, where French needs two models;
- the Italian listings, the Italian home card, the Italian Lingua page's captures and its table.

Unlike stages 2 and 3, the listings travel in the same change (the programme's row since the start): only
the Italian texts and the shared English fields move, and the requirements that listing fr-it would make
false are modified in the same pull request.

## What Changes

- **fr-it ships** (design D1, D2): `packs.json` gains fr-it at the end of the list, en-it staying ahead of
  it so that English stays an Italian reader's default; listed only at or above change 47's floor, after
  change 48 (Open Question 1). Change 30 is amended so that it-es, too, is appended at the end.
- **Italian speakers are offered French** (D3) after English; nothing new is typed.
- **Translation** (D4): fr-en then en-it, marked if change 49 put fr-it on the tier; otherwise offered
  without a mark, the owner deciding only under 75 % correct or over 30 % withheld (M15).
- **The memory sentence's Italian cases** (D5): French alone, or English and French, need fr-en and en-it,
  about 340 MB, the download their sum (≈ 52.7 MB).
- **The Italian description and activation page** (D6): recommended « Il web in inglese o francese:
  parole sconosciute evidenziate, percentuale reale. Offline e senza tracciamento. » (110); the page's
  `lede` and `step3` name French. The Italian reviewer agent reads them (I2).
- **The Italian listings name French** (D7): Chrome Web Store and AMO, App Store it-IT (subtitle,
  promotional text, keywords, description, « What's New »), and the shared English fields that say what
  Italian speakers study.
- **The site** (D8): the Italian home card and Lingua page, data-driven; the Italian set of captures taken
  again with French as the second studied language, its seed state recorded again and its alt texts
  rewritten.
- **Requirements true of their time** (D12): eight MODIFIED, one ADDED, change 47's « …, and not shipped »
  RENAMED.
- **A dogfood pass** (D10) in the Italian interface; released on its own, TestFlight first (D11).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: RENAMED change 47's *fr-it is committed beside French's studied tables, held to a
  coverage floor, and not shipped* to « …, and shipped once French is enabled for Italian speakers ».
  MODIFIED *The shipped pairs are one list* (held by `enable-lingua-italian-for-spanish-speakers`): fr-it
  in the list, en-it before it; *Italian speakers learning French* added.
- `lingua-translation`: MODIFIED *The setting's memory follows the models the reader's pairs need* (held
  by change 30): *An Italian speaker reading English* holds for English alone; two Italian scenarios
  added.
- `lingua-apple-app`: MODIFIED *Guided activation* (held by change 30): *French for an Italian speaker*
  added. MODIFIED change 40's *The host app holds Italian before a pair glossed in Italian ships*: *A
  device in Italian once en-it ships* holds while no other pair is glossed in Italian.
- `lingua-browser-extension`: ADDED *French is studied by readers of Italian*. MODIFIED *The store
  listings name each studied language* (held by `add-lingua-italian-spanish-listings`): *Italian speakers
  learning French* added; change 46's *A listing in Italian* kept.
- `site-lingua-page`: MODIFIED *The coverage table reads at a phone's width* (held by change 31): *Ten
  pairs on a phone, en-it and fr-it* and *Eleven pairs on a phone* added. MODIFIED *The Lingua page shows
  the extension at work* (held by `enable-lingua-italian-speakers`): *The Italian page* holds while no
  other pair is glossed in Italian; *The Italian page with French* added.
- `site-locales`: MODIFIED *The Italian home page describes Lingua for the readers it serves* (held by
  `add-site-italian-locale`): *Italian speakers learning English* narrowed; *Italian speakers learning
  French* added.

## Impact

- **Products.** Lingua and the site:
  - `apps/lingua-extension`: `packs.json`, `tool/check_variants.mjs`, `_locales/it/messages.json`, the
    tests of design D9, the listing file, the documents, and the Italian captures (`tool/capture`);
  - `apps/lingua-apple`: `copy.js`'s `it` `lede` and `step3`, and the listing file;
  - `scripts/lingua-data`: `test_gloss_coverage.py`; `crates/lingua-pack/tests/committed_tables.rs`;
  - `apps/site`: `src/data/lingua-coverage.json`, the refreshed fixtures, the Italian captures, and the
    site's tests.

  The backend, ID, Music, Live and the back office are untouched; so are the packs, tables and goldens.
- **Prerequisites, the owner's**: changes 45 (implemented and released), 46, 44, 47 (at or above its
  floor), 48, 49 and the owner's answer under M15; changes 2, 33, 34, 39, 40 and 41; the owner's and the
  reviewer agent's readings; test accounts.
- **Release**: on its own, TestFlight first; the site deployed first; the listings pasted after it. This
  pull request submits nothing.
- **Order**: after changes 45–49, and after changes 30 and 31 (recommended). Against changes 30, 31 and 54,
  whichever is implemented later re-copies the earlier's shared blocks (D13); changes 30 and 31 are
  amended to say so. Archived after the changes of `.openspec.yaml`.
- **Effort, against the row's 3–6 ideal days, raised to 4.75–9.5** (design, *Effort*).
