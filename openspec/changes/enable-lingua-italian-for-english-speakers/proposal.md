# enable-lingua-italian-for-english-speakers — it-en ships: English speakers learn Italian

## Why

Change 25 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. Change 20
made the extension study Italian and listed it-fr for French speakers; changes 22–24 built it-en —
its glosses, its refinement, its marks. What remains is to list it-en, as `enable-lingua-french`
listed fr-en for English speakers: the list, the English description and activation page, which the
version check and *Guided activation* require to name what English readers study, and the memory
sentence's English cases. The English description, said the direct way (« Read Spanish, French and
Italian on the web: … »), is 114 characters, over Apple's 112.

## What Changes

- **it-en ships** (design D1, D2): `packs.json` ends with it-en, after it-fr; `testdata/it-en/` for
  test builds; listed only at or above change 22's floor and after change 23.
- **English speakers are offered Italian** (D3) after Spanish and French; nothing new is typed — change
  20 widened the extension's type.
- **Translation** (D4): the it-en model alone, marked if change 24 put it on the tier.
- **The memory sentence's English cases** (D5): Italian alone "about 200 MB"; Spanish, French and
  Italian "about 340 MB", the download their sum (≈ 78.8 MB).
- **The English description and activation page** (D6): recommended "The web in Spanish, French or
  Italian: unknown words highlighted, an honest percentage. Offline and private." (108); the page's
  `lede` and `step3` name Italian. The owner reads them (M9).
- **The published coverage** (D7) and the site's English Lingua page, data-driven; the fixtures
  refreshed. The English home card is change 26's.
- **Requirements true of their time** (D12): change 20's list, memory and activation requirements
  MODIFIED.
- **A dogfood pass** (D10) in the English interface; released on its own, TestFlight first, with change
  26's listings (D11).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: MODIFIED *The shipped pairs are one list* (held by `enable-lingua-italian`):
  the list ends with it-en; three scenarios name it; *No Italian for English and Spanish speakers*
  holds while it-fr is the one Italian pair; *English speakers learning Italian* and *No Italian for
  Spanish speakers* added.
- `lingua-browser-extension`: ADDED *Italian is studied by readers of English*.
- `lingua-translation`: MODIFIED *The setting's memory follows the models the reader's pairs need*
  (held by `enable-lingua-italian`): two English scenarios added.
- `lingua-apple-app`: MODIFIED *Guided activation* (held by `enable-lingua-italian`): *Every language
  an English speaker can study* holds while it-en is not listed; *Italian for an English speaker*
  added.

## Impact

- **Products.** Lingua only: `apps/lingua-extension` (`packs.json`, `tool/check_variants.mjs`,
  `_locales/en/messages.json`, the tests of design D9, the documents), `scripts/lingua-data`
  (`testdata/it-en/`, `test_gloss_coverage.py`), `crates/lingua-pack/tests/committed_tables.rs`,
  `apps/site` (`src/data/lingua-coverage.json`, the refreshed fixtures), `apps/lingua-apple`
  (`copy.js`'s `en` `lede` and `step3`). The backend, ID, Music, Live and the back office are
  untouched; so are the packs, tables and goldens.
- **Prerequisites, the owner's**: changes 22, 23 and 24 implemented; change 20 implemented and
  released; the owner's readings (M9) and the marks' answer (Open Question 2); test accounts.
- **Release**: on its own, TestFlight first, with change 26's listings; nothing submitted by this pull
  request.
- **Order**: after changes 20 and 22–24; before change 26's release. Archived after the changes of
  `.openspec.yaml`.
- **Effort, against 4–7.5 ideal days**: 3.25–6 (design, *Effort*).
