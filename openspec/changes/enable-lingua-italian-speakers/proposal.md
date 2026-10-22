# enable-lingua-italian-speakers — the Italian interface, and English for Italian speakers

## Why

Change 45 of the [Italian programme](../../../docs/lingua/italian-programme.md), the last of stage 3.
Stage 3 prepared everything for Italian speakers, but none of it is shown:
- the native language (32), the interface (33) and the card's wording (34);
- the edition's rules (35) and the en-it pack (36);
- `_locales/it` (39), the Safari app's Italian (40) and the site's `/it/` pages (41);
- the e-mails' links (43) and the store listings (46).

Each change left the switch to this one: listing en-it, and adding Italian to the interface languages.
Change 42 is folded into this one: the Lingua pages' Italian tests and the coverage table's ninth pair.

A prototype measured the switch: en-it listed and `it` among the interface languages. It fails 17
tests in 11 files and four build gates. Changes 32, 33, 36 and 39 cover the gates and some of the
tests; the rest is this change's, listed in the design.

## What Changes

- **The switch** (design D1): `packs.json` appends en-it, and `INTERFACE_LANGUAGES` gains `it`. The
  tests that read the list keep their earlier lists as explicit arguments, as change 25's did.
- **The links** (D2):
  - the privacy policy, the leaving page and the account deletion page move to the `/it/` pages that
    change 41 pinned;
  - `deleteAccountUrl` gains the Spanish arm change 29 never landed, beside Italian's;
  - the account locale follows an Italian interface (I10).
- **The tests changes 32, 33 and 40 wrote for « not yet »** flip (D3).
- **Models** (D4): en-it's route is change 2's, and its translation is as change 38 decides; withheld,
  the route is dropped.
- **Captures** (D5): the capture harness gains a one-language mode for an Italian reader studying English
  alone. Its run makes the site's and the stores' Italian captures, with an Italian « second » shot taken
  on a second English page.
- **The site** (D6):
  - the coverage write lists en-it, and `/it/lingua/` builds with its media and captions;
  - change 42's leftovers: the ninth pair's table scenario and the Italian Lingua tests;
  - `/it/lingua/` is pinned, since it is the App Store's Italian marketing URL.
- **The requirements** (D7): twelve MODIFIED, each copied whole from its latest holder, and one ADDED,
  *English is studied by readers of Italian*.
- **This change's Italian** (D10), reviewed by the Italian reviewer agent before merge (I2).
- **The Apple host** (D8): no code; the change touches `apps/lingua-apple` so that release-please cuts
  the host app's release with en-it.
- **The order of release** (D9): what must be merged, deployed and released first, and the owner's
  steps.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-browser-extension`: ADDED *English is studied by readers of Italian*.
- `lingua-data-packs`: MODIFIED *The shipped pairs are one list* (held by
  `enable-lingua-italian-for-english-speakers`).
- `lingua-translation`: MODIFIED *The setting's memory follows the models the reader's pairs need*
  (held by `enable-lingua-italian-for-english-speakers`).
- `lingua-interface-language`:
  - MODIFIED *The interface language is the reader's native language, under its own key* (held by no
    open change);
  - MODIFIED *The account's e-mails follow the interface language* (held by
    `send-lingua-browser-locale-on-account-emails`).
- `lingua-privacy`: MODIFIED *Cymbra account deletion is reachable from Lingua* (held by
  `add-site-spanish-locale`).
- `site-discoverability`: MODIFIED *Every indexable page names its canonical address* (held by
  `add-site-lingua-showcase`).
- `site-lingua-page`:
  - MODIFIED *The Lingua page says what each studied language's card and levels are* (held by
    `add-lingua-italian-listings`);
  - MODIFIED *The coverage table reads at a phone's width* (held by
    `add-lingua-italian-english-listings`).

- `lingua-browser-extension` also: MODIFIED *Store and site captures come from the real extension, by
  one command* (held by `add-site-lingua-showcase`).
- `site-lingua-page` also: MODIFIED *The Lingua page shows the extension at work* (held by
  `add-site-lingua-showcase`).
- `lingua-usage-counts`: MODIFIED *Each browser's consent rule decides when counting starts, and one
  switch turns it off* (held by `add-lingua-usage-counts`).
- `lingua-apple-app`: MODIFIED *The app signs in with an email and a password through the extension's
  own flow* (held by `add-lingua-app-email-signin`).

## Impact

- **Products.** Cymbra Lingua and the site:
  - `apps/lingua-extension`: `packs.json`, `src/i18n/language.ts`, the three links, `tool/check_variants.mjs`,
    the capture harness, the tests, the documents;
  - `apps/site`: `lingua-coverage.json`, `/it/lingua/`'s media, `LINGUA_TEXT.it`'s captions, the
    one-language shot set, the pins, the tests;
  - `scripts/lingua-data`: the coverage write, `test_gloss_coverage.py`;
  - `crates/lingua-pack/tests/committed_tables.rs`;
  - `apps/lingua-apple`: its README's languages line.

  No backend change.
- **The owner**: the go-ahead, the dogfood on the five targets (I2), the releases, the deploys and the
  submissions (I12).
- **Prerequisites**: changes 2, 3, 4, 20–26, 32–41 implemented before the pull request; change 43
  deployed and change 46 on main before the release (D9).
- **Effort, against 4.25–8 ideal days**: 5–9.25 (design, *Effort*).
