# review-lingua-italian-wording — one Italian vocabulary, reviewed across every surface before it is shown

## Why

Change 44 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 3. The matrix
programme's equivalent (matrix change 33) was the owner reading the English and Spanish interface
himself. Nobody on the team reads Italian, so I2 settles that a dedicated agent reviews the Italian
wording instead.

Each change that wrote Italian had the agent review its own strings in its pull request:
- the interface (change 33) and the card (34);
- `_locales/it` (39) and the Safari app (40);
- the site (41) and the store listings (46);
- change 45's captions.

No one has read them together. The same concept can drift between surfaces: the review panel, a
setting's name, a button. A drift is invisible to a reviewer who reads one pull request at a time, and
to an owner who does not read the language.

## What Changes

- **An Italian glossary** (design D1): about thirty product terms, each with its French forms, accepted
  Italian forms and rejected variants. Its data lives in the extension's package
  (`test/italian-glossary.json`) so its checks always run, and `docs/lingua/italian-glossary.md` presents
  it. Its precedence: change 39's pinned words, then Music's `app_it.arb` for Cymbra-wide terms, then the
  interface.
- **One pass over every Italian surface** (D2), about 11,900 words, by change 33's reviewer agent:
  - the interface, the card, `_locales/it`, the Safari app, the site, change 45's captions and captures,
    and the listings;
  - translations read against their French, and texts written per audience against change 46's D1 and
    claims table;
  - checked against I6, gender-free address, typography, the field caps and the glossary;
  - findings applied in this pull request or answered with a source, a meaning finding going to the
    owner;
  - the report committed.
- **A test that keeps the vocabulary one** (D3), defined exactly: the catalogue's text surfaces (through
  the parity test's `texts()`), `_locales/it` and `copy.js` against their French by key; whole-word
  matching by letters; rejected variants; stale exceptions failing.
- **When** (D4):
  - merged after change 45's pull request, before the build the owner dogfoods and photographs, and
    before change 45's release;
  - a corrected captured string re-runs the Italian capture.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-interface-language`: ADDED *The Italian wording is one vocabulary across its surfaces*.

## Impact

- **Products.** Cymbra Lingua's Italian text:
  - new: `apps/lingua-extension/test/italian-glossary.{json,spec.ts}`, `test/support/` (`texts()`),
    `docs/lingua/italian-glossary.md`, `docs/lingua/italian-wording-review.md`;
  - changed: wherever the pass corrects a string, in `apps/lingua-extension/src/i18n/it/`,
    `_locales/it`, `apps/lingua-apple` (`copy.js`, `SignInCopy`), `apps/site` (the `/it/` pages, the
    islands' Italian, `lingua-text.ts`, the layouts), the listing files, and the captures if a captured
    string changes.

  Cymbra ID and Music are untouched.
- **The owner**: not asked to review Italian (I2). He reads the report's summary, decides any finding
  that changes meaning rather than wording, and deploys the site's corrections.
- **Order**: after changes 45 and 46 are merged, before the build the owner dogfoods and before
  change 45's release.
- **Effort, against 2–4 ideal days**: 2.5–3.75 (design, *Effort*).
