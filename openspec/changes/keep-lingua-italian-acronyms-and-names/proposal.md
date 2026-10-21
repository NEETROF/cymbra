# keep-lingua-italian-acronyms-and-names — an acronym in capitals stays the acronym; names leave the words to learn

## Why

Change 11 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Two rules
French and Spanish have and Italian does not yet:

- **Acronyms.** Every lookup of the engine lowercases first, so « NATO » reads as `nato`, *born*,
  *nascere*'s participle; « USA » as `usa`, *usare*'s third person; « ANSA », the press agency, as
  `ansa`, a handle. French met the same with « CA » read as *ça*, and matrix change 43c settled it:
  a word standing alone in capitals is looked up as written before the cascade, a hit being a row
  of a reviewed acronym table the pack holds as a form in capitals, its own lemma. Italian's news
  pages write their acronyms in capitals on every line (the corpus's `notizie` page: ONU, RAI, PIL,
  UE), and three of the commonest lowercase into verbs and nouns a learner meets.
- **Names.** A studied language's dictionary words — what the counts, the levels and the cards
  learn — are its reference pair's glossed lemmas less those the pair's edition files as proper
  nouns alone (matrix change 24c: one rule for the three languages, in the French Wiktionary's
  module, `reduce_edition_fr.py`). it-fr is glossed by the French Wiktionary's Italian section
  through that module, so the rule carries to Italian with no new code the day change 16 writes
  Italian's dictionary words; this change states it for Italian and holds it in the checks, so
  « Roma », « Garibaldi » and « Lombardia » are never words to learn, while « sale », « marco »
  and « roma » (the capital's name, and nothing else) are decided by the rule as written.

## What Changes

- **The rule, for Italian too** (design D1): before Italian's cascade, a token written wholly in
  capitals standing alone in capitals — French's reading, at the word's neighbours — is looked up
  in the pack's forms as written; a hit is the acronym, capitals kept, and the cascade does not run.
  Italian's pre-pass runs first, lowercasing for its comparisons: an articulated preposition or an
  elided form in capitals (« AI », « L' ») is read by it, never by the rule.
- **A reviewed table** (D2): `ACRONYMS` in `scripts/lingua-data/reduce_italian_acronyms.py`,
  imported by it-fr's reducer (Italian's reference) and by every pair glossing Italian — the rows
  whose lowercase reading would be another word's, each with the expansions or the sense the
  English Wiktionary's Italian section's own entry gives, its part of speech and its reason:
  `NATO`, `USA`, `ANSA`, measured on the corpora and settled by the owner (task 5.1); `ONU`, `RAI`,
  `UE`, `PIL`, `IVA`, `ISTAT`, whose lowercase is no Italian word, need no row and read as unknown
  words, as today. `tables/it/forms.tsv` gains each row as a form in capitals that is its own
  lemma; `lexical.tsv` lists it as a dictionary word; the builder accepts a form holding a capital
  in a pack studying French or Italian, wholly in capitals, two letters or more, its own lemma.
- **The gloss** (D3): a pair glosses a row from its own gloss of the acronym's entry when its
  edition has one (the French Wiktionary's Italian section glosses `NATO`, `USA`), else from its
  glosses of the expansions, each cut to its first sense, as 43c's D4 — Italian's addition, since
  its commonest acronyms are proper names whose expansions no pair glosses.
- **No rank, no level; a dictionary word** (D4): as `CA`; the card, the status and the key keep
  the capitals (43c's extension requirement, language-neutral, already covers it).
- **Names leave the words to learn** (D5): Italian's dictionary words are it-fr's glossed lemmas
  less those the French Wiktionary's Italian section files as proper nouns alone, by
  `dictionary_words` of `reduce_edition_fr.py`, which it-fr loads; the committed-tables checks hold
  `tables/it/lexical.tsv` to it, and the names rule of change 7 sets a document's names aside as a
  dictionary word never is.
- **Italian's analyser version bumps** (`1.2.0` → `1.3.0`); the Italian golden re-blessed (the
  version line; the `notizie` page where it writes a row's acronym).
- **English, Spanish, French and the six goldens do not move.**

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *An Italian word standing alone in capitals is read as the acronym
  its pack lists*.
- `lingua-data-packs`: ADDED — *Italian's acronyms are a reviewed table, glossed through their
  entry or their expansions*; ADDED — *Italian's dictionary words leave its names out*. The rule
  of change 24c (*A pack's dictionary words do not depend on its glosses*, held by it) is read,
  not modified; this change archives after it, after 43c (whose builder rule it widens), and after
  changes 7 and 9 (and, once proposed, 16 and 17: the row's gloss needs the pair's glosses).
- `lingua-browser-extension`: nothing — *A word's status and card keep its dictionary form's
  capitals* (43c) is language-neutral.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/analysis/pipeline.rs` (`alone_in_capitals` for Italian), `analysis/italian.rs`
  (the acronym step before the cascade, as `french::acronym`), `analysis/mod.rs` (`1.3.0`),
  `tests/italian_fixtures.rs`; `crates/lingua-pack/src/lib.rs` (a form in capitals accepted for
  Italian too), `tests/committed_tables.rs` (the rows present, `lexical.tsv` without the names).
- `scripts/lingua-data/reduce_italian_acronyms.py` (new), `reduce-it-fr.py` (the rows written, the
  gloss), `tables/it/{forms.tsv, lexical.tsv}` and `tables/it-fr/` re-reduced, `SOURCES.md`.
- `crates/lingua-wasm/tests/{italian_baseline.rs, support/italian.rs}` and `baseline/it-fr.golden`;
  `apps/lingua-extension/test/packs.spec.ts` (`1.3.0`).
- Untouched: the six pairs' tables, pins and goldens; the extension's bundle; the agent; the
  server; the site.
