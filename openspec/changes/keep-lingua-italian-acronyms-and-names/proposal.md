# keep-lingua-italian-acronyms-and-names — an acronym in capitals stays the acronym; names leave the words to learn

## Why

Change 11 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1. Two rules
French and Spanish have and Italian does not yet:

- **Acronyms.** Every lookup of the engine lowercases first, so « NATO » reads as `nato`, *born*,
  *nascere*'s participle; « USA » as `usa`, *usare*'s third person. French met the same with « CA »
  read as *ça*, and matrix change 43c settled it: a word standing alone in capitals is looked up as
  written before the cascade, a hit being a row of a reviewed acronym table the pack holds as a form
  in capitals, its own lemma. Measured on the English Wiktionary's Italian section (the file change 9
  reduces): 211 headwords written wholly in capitals, 42 of them with a lowercase that is a word or
  a form of the section, and among those the ones whose lowercase is *another* word's form — the
  only case a row repairs — are `NATO` (*nascere*), `USA` (*usare*, *uso*), `TARI` (the waste tax;
  *tarare*), `AGI` (the press agency; *agio*'s plural), `FAI` (*fare*), and a dozen two-letter
  province codes and function words (`MI`, `SO`, `TI`, `LE`, `NE`, `SE`, `SU`). `ONU`, `RAI`, `UE`,
  `PIL`, `IVA`, `ISTAT`: their lowercase is no other word's form, so the tables read them right with
  no row. `ANSA`, `CONI`, `SPA`, `ARPA`: no entry in the section, so no row is possible (a row chooses
  among the dictionary's entries, never writes one).
- **Names.** A studied language's dictionary words — what the counts, the levels and the cards
  learn — are its reference pair's glossed lemmas less those the pair's edition files as proper
  nouns alone (matrix change 24c: one rule for every language, in the French Wiktionary's module,
  `reduce_edition_fr.py` `dictionary_words`). it-fr is glossed by the French Wiktionary's Italian
  section through that module (change 16), so the rule carries to Italian with no new code the day
  change 16 writes `tables/it/lexical.tsv`; this change states it for Italian, with the one carve-out
  the acronym rows need, so « Roma », « Garibaldi » and « Lombardia » are never words to learn.

## What Changes

- **The rule, for Italian too** (design D1): before Italian's cascade, a token written wholly in
  capitals, standing alone in capitals by French's reading of its neighbours, is looked up in the
  pack's forms as written; a hit is the acronym, capitals kept, and the cascade does not run. The
  gate is the one French has in `resolve_lemmas` and `alone_in_capitals`, opened to Italian;
  `french::acronym` is language-neutral and serves as is. Italian's pre-pass runs first: an
  articulated preposition or an elided piece written in capitals beside the acronym is a neighbour
  in capitals (« NEGLI USA », « DELL'ONU » read by the cascade, as « CA HT » does), in lowercase it
  is not (« negli USA » reads the acronym); the pieces of « AI » are single letters, never a row.
- **A reviewed table** (D2): `ACRONYMS` in `scripts/lingua-data/reduce_italian_acronyms.py`,
  imported by it-fr's reducer (Italian's reference) and by every pair glossing Italian — the rows
  whose lowercase is another word's form, each with the expansions and synonyms the English
  Wiktionary's Italian section's own entry gives, its part of speech and its reason: `NATO` and
  `USA`, with `TARI`, `AGI` and `FAI` measured for the owner (task 5.1); the two-letter codes left
  out, a shouted everyday word (« lo SO ! ») being likelier than the code. `tables/it/forms.tsv`
  gains each row as a form in capitals that is its own lemma; the builder accepts a form or a lemma
  holding a capital in a pack studying French or Italian, wholly in capitals, two letters or more,
  its own lemma — 43c's builder requirement MODIFIED, every scenario kept.
- **The gloss** (D3): a pair glosses a row from its own glosses of the expansions the entry points
  at, as 43c's D4, and — Italian's addition, since the section's `NATO` names no expansion and
  `USA`'s is the English name — from its own glosses of the entry's synonyms (`Stati Uniti`,
  `America` for `USA`) when it glosses no expansion; a row glossed by neither has no gloss (`NATO` in
  it-fr, 43c's *No expansion glossed*: the status, the key and the card still work).
- **No rank, no level; a dictionary word whatever its sense runs** (D4): as `CA`; the card, the
  status and the key keep the capitals (43c's extension requirement, language-neutral).
- **Names leave the words to learn** (D5): Italian's dictionary words are it-fr's glossed lemmas
  less those the French Wiktionary's Italian section files as proper nouns alone, by
  `dictionary_words`, plus the acronym rows, glossed or not; the committed-tables checks and the
  builder's `check_studied` hold `tables/it/lexical.tsv` to it, the rows exempted as `CA` is.
- **Handed to change 16** (D6): it-fr's glosses and `lexical.tsv` are change 16's; this change
  ships the rule, the table, the builder and the forms rows, and change 16 writes the rows into
  `lexical.tsv` and glosses them by D3 and D5. This change archives after 16 and 17.
- **Italian's analyser version bumps** (`1.2.0` → `1.3.0`); the Italian golden re-blessed on its
  version line and the two probes added (the `notizie` page writes `ONU`, `RAI`, `PIL`, `UE`, none a
  row).
- **English, Spanish, French and the six goldens do not move.**

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED — *An Italian word standing alone in capitals is read as the acronym
  its pack lists*.
- `lingua-data-packs`: ADDED — *Italian's acronyms are a reviewed table, glossed through their
  entry's expansions or synonyms*; MODIFIED — *French's acronyms are a reviewed table, glossed
  through their expansions* (43c, open: the builder accepts a capital in a pack studying French or
  Italian; every scenario kept); ADDED — *Italian's dictionary words leave its names out*. The rule
  of change 24c is read, not modified; this change archives after 24c, 43c, and the Italian changes
  7, 8, 9 and 10 (and, once proposed, 16 and 17: the rows' gloss and `lexical.tsv` are theirs).
- `lingua-browser-extension`: nothing — *A word's status and card keep its dictionary form's
  capitals* (43c) is language-neutral.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/analysis/pipeline.rs` (the acronym gates in `resolve_lemmas` and
  `alone_in_capitals` opened to Italian), `analysis/mod.rs` (`1.3.0`), `tests/italian_fixtures.rs`;
  `crates/lingua-pack/src/lib.rs` (a capital accepted for Italian too), `src/tables.rs`
  (`check_studied` exempts the rows), `tests/committed_tables.rs` (the rows present as forms).
- `scripts/lingua-data/reduce_italian_acronyms.py` (new), `reduce-it-fr.py` (the rows written into
  `forms.tsv`; the gloss and `lexical.tsv` at change 16), `tables/it/forms.tsv` and `tables/it-fr/`
  re-reduced, `SOURCES.md`.
- `crates/lingua-wasm/tests/{italian_baseline.rs, support/italian.rs}` and `baseline/it-fr.golden`;
  `apps/lingua-extension/test/packs.spec.ts` (Italian's version).
- Untouched: the six pairs' tables, pins and goldens; the extension's bundle; the agent; the
  server; the site.
