# generalise-lingua-analysis-by-language — one core, an analysis per studied language

## Why

The core analyses English only, by construction:
- `StudiedLanguage` has one variant, and the tokenisation pre-pass and the function-word tables
  destructure it with an irrefutable `let`.
- `lemmatize(form, lexicon)` takes no language, and consults the English irregulars **before** the
  pack's forms. 13 of the 110 English irregulars are also Spanish words, so a Spanish « has » (of
  *haber*) would come out as « have ».
- A single `ANALYZER_VERSION` is checked by strict equality against every pack, so a fix to a
  Spanish rule would invalidate the English pack.

Spanish is the next studied language (`docs/lingua/spanish-programme.md`, change 5). This is the
first core change of release R2, a silent English release. It lets the core hold a second
language without moving English: the language becomes a dispatch, the English arm moves verbatim,
analyser versions become per language, and a pack says which language it studies. Spanish
analysis itself (its pre-pass, enclitics, accents and function words) comes later, in
`add-lingua-spanish-analysis`.

## What Changes

- **`StudiedLanguage::Spanish`**, appended after English. The derived order keys serialised maps,
  so a state that holds only English serialises byte for byte as before. The language also gains
  ISO 639-1 tags (`en`, `es`) and the list of languages the core can analyse, and Spanish blocks
  are detected.
- **Dispatch per language** at each language-dependent seam of the analysis:
  - the tokenisation pre-pass (the `n't` split is English's);
  - the function-word tables;
  - the lemmatisation cascade, now `lemmatize(form, studied, lexicon)`.

  English's arm of each is the current code, moved verbatim. Spanish gets the **baseline**:
  segmentation, the pre-pass rules that belong to no language, and the pack's forms. It has no
  function words, and never meets an English table or rule.
- **An analyser version per language.** English keeps `1.1.0` (`ANALYZER_VERSION` remains
  English's). Spanish starts at `0.1.0` while it is the baseline. A page analysis reports the
  version of the language it was analysed as.
- **A pack names its language.** The core reads `meta.studied` into the pack (`Pack::studied()`).
  It refuses a pack whose language it has no analyser for, and checks the pack's
  `analyzer_version` against **that language's** analyser.
- **Callers pass the language.** The pack builder passes the pair's studied language to
  `lemmatize`; for en-fr that is English, so the pack is unchanged. The wasm engine keeps its
  English constant here; `generalise-lingua-wasm-engine` replaces it with `Pack::studied()`.
- **The extension's version reader is anchored.** `apps/lingua-extension/build.mjs` finds
  `ANALYZER_VERSION` with a pattern that also matches inside `SPANISH_ANALYZER_VERSION`; only the
  order of the two constants in the file would keep it right.
- **English does not move.** The English invariance baseline
  (`crates/lingua-wasm/tests/english_baseline.rs`) stays byte-identical, and the en-fr pack keeps
  its sha256.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *Analysis by studied language* and *An analyser version per studied
  language*. The first means each language has its own pre-pass, cascade and tables, a rule
  written for one language never runs on another, and a language with no rules yet gets the
  baseline. No existing requirement is rewritten. The open changes touching this capability
  (`add-lingua-phrase-gloss`, `add-lingua-expression-table`) hold different requirements.
- `lingua-data-packs`: ADDED — *A pack names the language it studies*: read from the metadata,
  refused when the core has no analyser for it, and versioned against that language's analyser.
  The only open change touching this capability (`add-lingua-expression-table`) holds a different
  requirement.

## Impact

- **Products.** Cymbra Lingua only:
  - `crates/lingua-core` (analysis and packs);
  - the two `lemmatize` call sites of `crates/lingua-pack`;
  - one pattern in `apps/lingua-extension/build.mjs`.

  The extension, the Apple app and the agent plugin keep analysing English, exactly as before.
  ID, Music, Live, the back office and the site are untouched.
- **Release.** Part of R2, a silent English release: nothing a reader sees changes, and the en-fr
  pack is byte-identical. No Spanish pack exists yet; tests build synthetic ones.
- **Compatibility.** Builds from this change on can read a state that names Spanish; released
  builds cannot (they report it as malformed). Nothing can produce such a state until a later
  change lets a reader choose Spanish. Backup v2 (written only when a language other than English
  is present) belongs to that work, not to this change.
