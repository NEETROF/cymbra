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

- **An Italian glossary** (design D1): `docs/lingua/italian-glossary.md`, the product's Italian terms,
  each beside its French source term and the surfaces that use it. Examples: « Ripasso », « Impostazioni »,
  « riquadro laterale », « percentuale reale », « QCER », « Traduzione estesa ». The agent builds it from
  the surfaces as they stand, choosing one term per concept.
- **One pass over every Italian surface** (D2), by change 33's reviewer agent, against the French source,
  the glossary and I6:
  - « tu », QCER, Treccani's tense names, Italian numbers;
  - address free of gender;
  - typography and elisions;
  - each store's and platform's length limits.

  Its findings are applied in this change's pull request, or answered with a source. The report is
  committed as `docs/lingua/italian-wording-review.md`.
- **A check that keeps the vocabulary one** (D3): a test reads the glossary's concepts and fails when an
  Italian message, whose French counterpart uses a glossary term, does not use the Italian term. It covers
  the extension's catalogue, `_locales/it` and the Safari app's `copy.js`, with a reviewed list of
  exceptions.
- **When** (D4): after change 46, before change 45's release.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-interface-language`: ADDED *The Italian wording is one vocabulary, reviewed across its surfaces*.

## Impact

- **Products.** Cymbra Lingua's Italian text:
  - new: `docs/lingua/italian-glossary.md`, `docs/lingua/italian-wording-review.md`,
    `apps/lingua-extension/test/italian-glossary.spec.ts`;
  - changed: wherever the pass corrects a string, in `apps/lingua-extension/src/i18n/it/`,
    `_locales/it`, `apps/lingua-apple` (`copy.js`, `SignInCopy`), `apps/site` (the `/it/` pages, the
    islands' Italian, `lingua-text.ts`) and the listing files.
- **The owner**: not asked to review Italian (I2). He reads the report's summary and decides any finding
  that changes meaning rather than wording.
- **Order**: after changes 33, 34, 39, 40, 41 and 46 are implemented, and change 45's captions written;
  before change 45's release.
- **Effort, against 2–4 ideal days**: 2–3.5 (design, *Effort*).
