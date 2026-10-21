# add-lingua-italian-tokenisation — elision pieces, articulated prepositions, truncation: Italian's pre-pass (I7)

## Why

Change 6 of the [Italian programme](../../../docs/lingua/italian-programme.md), stage 1, decision I7
(2026-10-21). Change 5 (`add-lingua-italian-baseline`) made Italian a studied language served by the
baseline analysis, and its golden shows what that costs on raw Italian text: « l'uomo » is one
token with the lemma « l'uomo », « c'è » hides *essere* in the commonest verb phrase of the
language, « nella » hides *in* and *la*, « un po' » is read as `po` — the rule of no language trims a
trailing apostrophe — and so is « perche' » as `perche`. Italian writes its articles,
pronouns and prepositions joined to the next word far more often than French does: an elision in
nearly every sentence, an articulated preposition in most. A reader cannot be glossed, counted or
asked about *lo*, *di*, *in* or *poco* while they are glued to other words.

The owner settled I7 on 2026-10-21, reading the articulated prepositions as M21 reads French: the
`di` family stays whole — `del`, `dello`, `della`, `dei`, `degli`, `delle` are also the partitive
article, as `du` and `des` are — and the others are split into their two pieces (`al`, `dal`,
`nel`, `sul`, `col` and their forms), as `au` and `aux` are; an elided piece gets its own span,
as French's; a truncation (`po'`, `fa'`) reads as its word. This change writes that pre-pass, and
only that: the enclitic rule, NFC and the cascade are change 7's; the apostrophe that stands for
an accent (`perche'`, `E'`) is change 10's; the expression keys that read the pieces are change
12's.

## What Changes

- **Italian's tokenisation pre-pass** in `crates/lingua-core/src/analysis/tokenize.rs`, Italian's
  arm alone (design D2–D6):
  - **Elision**: an elided word from a closed list — `l'`, `un'`, `d'`, `c'`, `v'`, `m'`, `t'`,
    `s'`, `n'`, `ch'`, `gl'`, `dov'`, `com'`, `quest'`, `quell'`, `anch'`, `senz'`, `tutt'`,
    `bell'`, `grand'`, `sant'`, `mezz'`, `nient'`, `quant'`, with the straight or the typographic
    apostrophe and followed by a letter — is split from the word it is joined to. Each piece is a
    token with **its own source span**, the elided piece's span holding its apostrophe, and the
    elided piece is read as the word it stands for (`lo`, `una`, `di`, `ci`, `questo`…), its
    written capital kept.
  - **The `di` family stays whole** (I7): `del`, `dello`, `della`, `dei`, `degli`, `delle`, and
    `dell'` read as `dello`, its span holding the apostrophe.
  - **The other articulated prepositions split** (I7): `al`, `allo`, `alla`, `ai`, `agli`, `alle`
    → `a` + the article; `dal…` → `da`; `nel…` → `in`; `sul…` → `su`; `col`, `coi` → `con` + `il`,
    `i`; the two tokens sharing the source span, as Spanish's `al` and French's `au` do; the
    elided ones (`all'`, `dall'`, `nell'`, `sull'`) → the preposition + `lo`, sharing the span,
    apostrophe included.
  - **A trailing apostrophe belongs to its word**: Italian's words seam keeps it, where the
    edge trim of every language drops it today, so that the truncations can be read and the
    apostrophe that stands for an accent is kept for change 10.
  - **Truncation**: a word ending in an apostrophe with no letter after it, from a closed list —
    `po'`, `mo'`, `fa'`, `va'`, `da'`, `sta'`, `di'`, `be'` — is read as the word it stands for
    (`poco`, `modo`, `fai`, `vai`, `dai`, `stai`, `dici`, `bene`), one token, its span holding the
    apostrophe. Any other word ending in an apostrophe (`perche'`, `e'`) stays one token, read as
    written, until change 10.
  - Hyphenated runs keep the shared compound rule; an elision on a run's first piece is split off
    (`l'anglo-americano` → `lo` + `anglo-americano`). Italian has no hyphenated inversion.
  - The pieces are lemmatised as any word is: by the baseline (the pack's forms, else the lowercased
    form) until change 7's cascade.
- **Italian's analyser version becomes `0.2.0`.** The fixture pack's manifest follows.
- **The Italian golden is re-blessed**; the fixture pack gains the words the pre-pass writes that it
  lacked (`lo`, `la`, `il`, `i`, `gli`, `le`, `una`, `di`, `ci`, `questo`, `poco`, `fai`…).
- **A click between two pieces opens the piece that starts there** already (matrix change 40): the
  extension changes nothing; no Italian reaches it before change 20.
- **English, Spanish, French and the six goldens do not move.**

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`:
  - ADDED — *Italian tokenisation pre-pass* (elision, the `di` family whole, the other
    articulated prepositions split, truncation, `0.2.0`).
  - MODIFIED — *Italian is a studied language served by the baseline analysis* and *An Italian
    invariance baseline runs beside the English, Spanish and French ones*, both added by
    `add-lingua-italian-baseline` (change 5), after which this change archives; its MODIFIED blocks
    carry change 5's wording with only what moves at `0.2.0` rewritten, every scenario name kept.
- No extension requirement: *A click opens the piece of a split word under the pointer* (matrix
  change 40) is language-neutral and already covers Italian's pieces.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core/src/analysis/tokenize.rs` (Italian's arm: the elision table, the
  articulated-preposition table, the truncation table, the casing of a read word),
  `analysis/mod.rs` (`ITALIAN_ANALYZER_VERSION = "0.2.0"`), `crates/lingua-core/tests/italian_fixtures.rs`
  (at least 80 cases over a stand-in lexicon).
- `crates/lingua-wasm/tests/{italian_baseline.rs, support/italian.rs}` and `baseline/it-fr.golden`
  (re-blessed once); `tests/languages.rs` and `languages_wasm.rs` where they assert Italian's
  version or pieces; `scripts/lingua-data/testdata/it-fr/` (the manifest at `0.2.0`, the pieces'
  words in `forms.tsv`, `freq.tsv` and `gloss.tsv`).
- `apps/lingua-extension/test/packs.spec.ts` (`coreAnalyzerVersion("it", modRs)` reads `0.2.0`).
- Untouched: every committed table and pin, the six goldens, the English, Spanish and French arms,
  the extension's bundle, the agent, the server, the site.
