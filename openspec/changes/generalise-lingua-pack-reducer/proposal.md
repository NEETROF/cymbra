# generalise-lingua-pack-reducer — one reduction per pair, the French side shared

## Why

The data pipeline reduces raw sources into committed tables through a single script,
`scripts/lingua-data/reduce-en-fr.py` (1,267 lines). About half of it is English by nature —
the ESDB and AGID parsers, English inflection rules, the own-word heuristics, the CEFR-J
lists — and about half works on the **French** side: cleaning a Wiktionary gloss, grouping its
senses by part of speech, the multi-word expressions, choosing one lemma per form, dense ranks,
the words a level list adds. Every `<studied>->FR` pair needs that second half, identically.

Spanish is next (the Spanish programme, ES→FR at the English perimeter). Copying the file would
duplicate ~500 lines that must never drift; worse, the "rules changed" gate hashes one file, so
a fix to the shared half would escape the check of any pair that did not copy it. This change
splits the reducer before a second pair exists, and proves the split moved nothing: the en-fr
tables reduce byte for byte as before.

## What Changes

- **`scripts/lingua-data/reduce_common.py`**: the rules every `<studied>->FR` pair shares, moved
  out of `reduce-en-fr.py` unchanged except for four values that were English and are now passed
  in as a `Studied` (the language's word pattern, the wording by which a French form-of gloss
  names its target, the coordinating conjunctions, and wordfreq's language code).
- **`reduce-en-fr.py`** keeps what is English (ESDB, AGID, English inflection rules, own words,
  CEFR-J / Octanove, grammar slots, NOTICE, `main`) and binds the shared rules to English under
  their existing names, so its 94 unit tests run unchanged.
- **The reduction rules of a pair are a set of files**: `reduce-<pair>.py` plus every shared
  `reduce_*.py`. `pin.json` records one digest over that set and lists the files; the check lane
  fails when any of them changed since the tables were reduced. The pack version names the same
  digest.
- **Source registry per pair**: a pair with no ESDB and no pinned CSV lists (any pair but en-fr)
  no longer breaks `fetch-pinned` / `fetch-live`; a pair with no registry at all fails with a
  message naming what to add.
- **Workflows by pair**: `lingua-extension-check` builds and checks every folder of
  `tables/`; `lingua-pack-update` takes a `pair` input (en-fr by default and for the monthly run),
  and puts the pair in its release, branch, commit and concurrency names.
- **Attribution**: the NOTICE credits wordfreq to **Robyn Speer**, the name its licence requires
  (its NOTICE: crediting her by another name voids the permission), with the Google Books Ngram
  acknowledgement it asks for. The manifest's analyser version is read from lingua-core instead
  of being written a second time in the reducer.
- **The en-fr pack changes once**: its seven data tables are byte-identical after re-reduction
  from the pinned sources; the NOTICE (attribution), the pack version (new rule digest) and
  therefore the pack's sha256 change. `pin.json` records the new pack.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED — *A pair's reduction rules include the rules it shares* (the rule
  set, its digest and the gate), and *A source is credited as its licence requires* (the NOTICE
  names an author where the licence makes that a condition). No existing requirement is rewritten;
  the only open change touching this capability (`add-lingua-expression-table`) holds a different
  requirement.

## Impact

- **Products.** Cymbra Lingua only: its data pipeline (`scripts/lingua-data`), two workflows, and
  the en-fr pack's attribution text and version string, which readers see in the extension's
  credits. No code of the extension, the core, the backend, the Apple app or the agent plugin
  changes. ID, Music, Live, the back office and the site are untouched.
- **Release.** The next extension and Apple releases carry a pack whose sha256 differs only
  through the NOTICE and the version; every page analysis is unchanged (same forms, ranks,
  glosses, levels, expressions, readings).
- **Reviewers.** The AMO source archive already carries `scripts/lingua-data` whole; REVIEWERS.md
  names the shared module (and no longer AGID, replaced by ESDB on 2026-09-26).
