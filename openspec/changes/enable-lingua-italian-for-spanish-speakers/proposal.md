# enable-lingua-italian-for-spanish-speakers — it-es ships: Spanish speakers learn Italian

## Why

Change 30 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 2. Change 20 made
the extension study Italian; change 25 listed it-en for English speakers. Changes 27a–29 build it-es:
- pivot sources (27a);
- tables committed at or above a floor the owner fixes (27);
- a refinement (28);
- the marks of its two-model route, it-en then en-es (29).

What remains is to list it-es, as change 25 listed it-en. That covers:
- the list;
- the Spanish description and activation page, which the version check and *Guided activation* require
  to name what Spanish readers study;
- the memory sentence's Spanish cases, where Italian alone needs two models, not one as for English.

The Spanish description, said the direct way (« Lee inglés, francés e italiano en la web: … Sin conexión
y privado. »), is 119 characters, over Apple's 112.

## What Changes

- **it-es ships** (design D1, D2):
  - `packs.json` gains it-es after it-en, and after en-it if change 45 listed it first.
  - `testdata/it-es/` is the it-fr fixture glossed in Spanish, for test builds.
  - it-es is listed only at or above change 27's floor, and after change 28 (Open Question 3).
- **Spanish speakers are offered Italian** (D3), after English and French. Nothing new is typed: change
  20 widened the extension's type.
- **Translation** (D4): it-en then en-es, marked if change 29 put it-es on the tier. Short of the tier
  it is offered without a mark; the owner decides only under 75 % correct or over 30 % withheld (M15).
- **The memory sentence's Spanish cases** (D5), both « unos 340 MB »:
  - Italian alone needs it-en and en-es (≈ 51.6–51.7 MB);
  - English, French and Italian need en-es, fr-en and it-en (≈ 77.8–77.9 MB).
- **The Spanish description and activation page** (D6):
  - recommended description: « La web en inglés, francés o italiano: palabras nuevas resaltadas,
    porcentaje honesto. Sin conexión y privado. » (109);
  - the page's `lede` and `step3` name Italian;
  - the owner reads them (M9).
- **The published coverage** (D7): it feeds the site's Lingua pages and Spanish home card,
  data-driven, and the fixtures are refreshed. The requirements that listing it-es would make false (the
  Spanish listings', the home card's, and the table's at nine or ten rows) belong to change 31, which is
  implemented first.
- **Requirements true of their time** (D12): MODIFIED blocks for the list and the memory sentence
  (copied from change 45) and for *Guided activation* (copied from change 25); change 27's « …, and not
  shipped » requirement RENAMED, and change 22's it-en twin likewise in change 25's delta.
- **Against change 45** (D13): whichever of the two is implemented second re-copies the first's blocks
  and names it in `archiveAfter`; change 45's D1 says the same.
- **A dogfood pass** (D10) in the Spanish interface. it-es is released on its own, TestFlight first,
  with change 31's listings (D11).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: RENAMED change 27's *it-es is committed beside Italian's studied tables, held to a
  coverage floor, and not shipped* to « …, and shipped once Italian is enabled for Spanish speakers ».
  MODIFIED *The shipped pairs are one list* (held by `enable-lingua-italian-speakers`):
  - the list gains it-es, by its floor, en-it and it-es in the order `packs.json` first listed them;
  - *Building the listed packs*, *A list widened too early* and *French for English speakers alone*
    name it;
  - *No Italian for Spanish speakers* holds while no listed pair studies Italian glossed in Spanish;
  - *Spanish speakers learning Italian* is added.
- `lingua-browser-extension`: ADDED *Italian is studied by readers of Spanish*.
- `lingua-translation`: MODIFIED *The setting's memory follows the models the reader's pairs need*
  (held by `enable-lingua-italian-speakers`): two Spanish scenarios added.
- `lingua-apple-app`: MODIFIED *Guided activation* (held by `enable-lingua-italian-for-english-speakers`):
  *Spanish speakers without fr-es* holds while neither fr-es nor it-es is listed; *Italian for a
  Spanish speaker* added.

## Impact

- **Products.** Lingua only:
  - `apps/lingua-extension`: `packs.json`, `tool/check_variants.mjs`, `_locales/es/messages.json`, the
    tests of design D9 and the documents.
  - `scripts/lingua-data`: `testdata/it-es/`, `test_gloss_coverage.py`.
  - `crates/lingua-pack/tests/{committed_tables.rs, pipeline_testdata.rs}`.
  - `apps/site`: `src/data/lingua-coverage.json` and the refreshed fixtures.
  - `apps/lingua-apple`: `copy.js`'s `es` `lede` and `step3`.
  - Untouched: the backend, ID, Music, Live and the back office, and the packs, tables and goldens.
- **Prerequisites, the owner's:**
  - changes 27 (at or above its floor), 28 and 29 implemented;
  - change 20 implemented and released;
  - changes 25 and 26 implemented;
  - change 31 implemented: its listing texts, and its home-card and table scenarios;
  - the owner's readings (M9) and answers (Open Questions);
  - test accounts.
- **Release**: on its own, TestFlight first, with change 31's listings. This pull request submits
  nothing.
- **Order**: after changes 20, 25–29 and 31's implementation (D13). Archived after the changes of
  `.openspec.yaml`.
- **Effort, against 4–7.5 ideal days**: 3.25–6.5 (design, *Effort*).
