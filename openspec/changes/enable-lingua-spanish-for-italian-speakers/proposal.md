# enable-lingua-spanish-for-italian-speakers — es-it ships, with its listings: Italian speakers learn Spanish

## Why

Change 54 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 4, the last pair
of the matrix. Change 50 lists fr-it for Italian speakers, with its listings; changes 51–53 build es-it,
the thin pair the owner put on the full pivot (I4, I11): its glosses (51), their refinement (52) and the
marks of its two-model route, es-en then en-it (53).

What remains is to list es-it, as change 50 lists fr-it, and to make what an Italian reader is told name
Spanish:
- the list;
- the Italian description and activation page, which must name every language Italian readers study;
- the memory sentence's Italian cases, where Spanish needs two models;
- the Italian listings, home card, Lingua page captures and table.

The listings travel in the same change, as change 50's do.

## What Changes

- **es-it ships** (design D1, D2): `packs.json` gains es-it at the end of the list, en-it ahead of it;
  listed only at or above change 51's floor, after change 52.
- **Italian speakers are offered Spanish** (D3) after English, and after French where fr-it is listed
  first; nothing new is typed.
- **Translation** (D4): es-en then en-it, marked if change 53 put es-it on the tier; otherwise offered
  without a mark, the owner deciding only under 75 % correct or over 30 % withheld (M15).
- **The memory sentence's Italian cases** (D5): Spanish alone, or English and Spanish, needs es-en and
  en-it (memory about 340 MB, download ≈ 52.8 MB); French and Spanish, with or without English, need
  three models held two at a time (memory about 340 MB, download ≈ 79.0 MB).
- **The Italian description and activation page** (D6): with fr-it listed, « Il web in inglese, francese
  o spagnolo: parole sconosciute evidenziate, percentuale reale. Senza tracciamento. » (110), « Offline
  e » giving way as change 50 foresaw; the page's `lede` and `step3` name Spanish. The Italian reviewer
  agent reads them (I2).
- **The Italian listings name Spanish** (D7), and the shared English fields that say what Italian
  speakers study.
- **The site** (D8): the Italian home card and Lingua page, data-driven; the Italian captures keep French
  as the second studied language where fr-it is listed, and take Spanish where it is not.
- **Requirements true of their time** (D12): seven MODIFIED, one ADDED, change 51's « …, and not
  shipped » RENAMED.
- **A dogfood pass** (D10) in the Italian interface; released on its own, TestFlight first (D11).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: RENAMED change 51's *es-it is committed beside Spanish's studied tables, held to a
  coverage floor, and not shipped* to « …, and shipped once Spanish is enabled for Italian speakers ».
  MODIFIED *The shipped pairs are one list* (held by `enable-lingua-french-for-italian-speakers`): es-it
  in the list; *Italian speakers learning Spanish* added.
- `lingua-translation`: MODIFIED *The setting's memory follows the models the reader's pairs need* (held
  by change 50): two Italian scenarios added.
- `lingua-apple-app`: MODIFIED *Guided activation* (held by change 50): *Spanish for an Italian speaker*
  and *French and Spanish for an Italian speaker* added.
- `lingua-browser-extension`: ADDED *Spanish is studied by readers of Italian*. MODIFIED *The store
  listings name each studied language* (held by change 50): *Italian speakers learning French* holds
  while es-it is not listed; *Italian speakers learning Spanish* added.
- `site-lingua-page`: MODIFIED *The coverage table reads at a phone's width* (held by change 50): ten,
  eleven and twelve pairs on the Italian page. MODIFIED *The Lingua page shows the extension at work*
  (held by change 50): *The Italian page with Spanish* added.
- `site-locales`: MODIFIED *The Italian home page describes Lingua for the readers it serves* (held by
  change 50): two scenarios added.

## Impact

- **Products.** Lingua and the site, as change 50: `packs.json`, `tool/check_variants.mjs`,
  `_locales/it/messages.json`, the extension's tests and listing file; `copy.js`'s `it` `lede` and
  `step3`, and the App Store listing; `test_gloss_coverage.py`; `committed_tables.rs`;
  `src/data/lingua-coverage.json`, the fixtures and the site's tests. The backend, ID, Music, Live and
  the back office are untouched; so are the packs, tables and goldens.
- **Prerequisites, the owner's**: changes 30, 31 and 50 implemented (change 50 released, recommended), 51
  (at or above its floor), 52, 53 and the owner's answer under M15; the reviewer agent's reading; test accounts.
- **Release**: on its own, TestFlight first; the site deployed first; the listings pasted after it.
- **Order**: after changes 30, 31 and 50–53, changes 30, 31 and 50 being hard prerequisites (D13), so the
  archive order cannot cycle. Archived after the changes of `.openspec.yaml`.
- **Effort, against the row's 3–6 ideal days, raised to 4.5–9** (design, *Effort*).
