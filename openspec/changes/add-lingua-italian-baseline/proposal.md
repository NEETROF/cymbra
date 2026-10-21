# add-lingua-italian-baseline — Italian as a studied language: the variant, its golden, backup v4

## Why

Change 5 of the [Italian programme](../../../docs/lingua/italian-programme.md), the first of stage 1
(Italian studied: it-fr). The programme's architecture says what Italian as a studied language
gets: a variant appended after French, its own analyser version, backup schema v4 written only
when Italian is studied, and a golden over an Italian corpus before any Italian rule is written.
This change lays that groundwork, and none of the behaviour that follows: change 6
(`add-lingua-italian-tokenisation`, I7), 7 (`add-lingua-italian-analysis`), 8 (the detection
guard), 9 (the forms tables), 13 (grammar), 14 (levels), 16 (the it-fr pack) each add behaviour
behind what this change puts in place — exactly as the French baseline (matrix change 39) did for
French.

Today the core knows three studied languages; Italian is none, and no native language either. An
`it` tag is refused wherever a studied language is read; no pack studying Italian can load; a
backup that named Italian would be malformed to every build. Three things have to exist before
the first Italian rule is written:

- the variant, with an analyser version of its own, so that an Italian rule never moves English,
  Spanish or French, and the seams that must answer for a fourth language are answered once,
  verbatim for the three others;
- a golden that freezes what the engine makes of raw Italian text today, so that each later change
  of the stage shows its effect as the diff of a re-bless — the decisions of I7 (elision pieces,
  articulated prepositions, truncation, the apostrophe for an accent) and the enclitic rule of
  change 7 all show on raw text, which no treebank exercises;
- the backup version that keeps released builds honest when Italian data exists.

The owner settled I5 (it-fr first), I7, I8 and I11 on 2026-10-21. This change depends on none of
them: the Italian arm is the baseline analysis the specs already define for a language whose
rules are not written — `l'uomo`, `dell'acqua`, `del`, `nel` and `dirmelo` stay whole, `po'` loses
its trailing apostrophe as any word's edge apostrophe does today, nothing is a function word, there
is no level and no second lemma. The golden shows the questions
I7 settles; it answers none of them.

## What Changes

- **`StudiedLanguage::Italian`**, appended after French, tagged `it`, detected through whichlang's
  Italian class, listed in `ALL`, with `ITALIAN_ANALYZER_VERSION = "0.1.0"`: a `0.x` version, as
  the spec requires of a language served by the baseline.
- **Italian's arm at every seam is the baseline analysis**: the tokeniser's language-neutral rules
  and no pre-pass of its own (no elision, contraction or enclitic split, text read as it came),
  the pack's form→lemma lookup else the lowercased form (`lemmatize_baseline`), no function-word
  table, no names rule, no guard — whichlang reads Catalan and Galician as Italian today, and the
  golden records it until change 8. A rule written for English, Spanish or French never runs on
  Italian.
- **A pack may study Italian.** It loads against Italian's analyser version, as the general rule
  says. Italian is still no native language: a pack glossed in `it` is refused as before, and the
  native-language choice offers nothing new (change 32).
- **An Italian invariance baseline**, `crates/lingua-wasm/tests/italian_baseline.rs`, beside the
  six others, on the harness they share: a thirteen-page raw-text corpus (`baseline/pages-it.txt`:
  authored pages per phenomenon plus two public-domain excerpts, named in the design, set in real
  Italian punctuation, one block committed in NFD on purpose), its probes, and
  `baseline/it-fr.golden`. It runs over a hand-written fixture pack
  (`scripts/lingua-data/testdata/it-fr/`) until the Italian tables are committed, with the engine
  started on the real en-fr pack, as a French-native reader's is. It runs wherever the others
  run, and `lingua-pack-update` re-blesses it with them.
- **Backup schema version 4**, written only when the state names Italian as a studied language —
  in its profile or in any per-language record — and read first, like 1, 2 and 3. Every build
  released since 1.5.0 reads the version before the rest of the file, so it refuses a version 4
  backup as « unsupported version 4 », by name, never as malformed. A reader of English, Spanish
  or French keeps the version they have, byte for byte.
- **Three compile-forced lines outside the core.** `apps/lingua-extension/tool/packs.mjs` maps `it`
  to its constant (`test/packs.spec.ts` holds that map to `language.rs`, so a variant added there
  fails the extension's tests first); the agent plugin's `language_name` names Italian; the wasm
  engine's per-language expression lookup takes the baseline arm, the pack's expression for the
  lemma. None changes anything a reader sees: no pair studying Italian is listed, the extension's
  `StudiedLanguage` type and its labels stay `en | es | fr`, and no Italian pack exists to install.
- **English, Spanish and French do not move.** The six goldens are unchanged byte for byte,
  without re-blessing; every pin is unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`: ADDED *Italian is a studied language served by the baseline analysis*; ADDED
  *An Italian invariance baseline runs beside the English, Spanish and French ones*.
- `lingua-data-packs`: MODIFIED *A pack names the language it studies* — an Italian pack at
  Italian's version loads, the committed packs of the six pairs do not change; held by
  `add-lingua-french-baseline`, `add-lingua-french-analysis` and `ignore-lingua-soft-hyphens`,
  after which this change archives, its text the latest holder's with Italian added.
- `lingua-decks-review`: MODIFIED *The backup records the reader's language profile* (schema
  version 4 for Italian; held by `add-lingua-french-baseline` and
  `add-lingua-native-language-sync-client`) and *A restore reads the backup's schema version
  first* (versions 1 to 4; held by `add-lingua-french-baseline`); every scenario kept.

## Impact

Lingua only. Nothing is consumed from Cymbra ID, Music, Live, the back office or the site, and
nothing new is offered to them.

- `crates/lingua-core`: `analysis/language.rs` (the variant), `analysis/{tokenize,lemmatize,
  function_words}.rs` and `engine.rs` (Italian's arm at each seam, verbatim the baseline: the
  words seam, `nfc_for`, `split_contraction`, `lemmatize`, the function words, `document_names`,
  `expression_piece`, `expression_window`),
  `analysis/mod.rs` (`ITALIAN_ANALYZER_VERSION`), `knowledge/profile.rs` (an Italian native
  language still maps to no studied language: there is none), `packs/pack.rs` (an `it` pack loads at
  `0.1.0`), `decks/backup.rs` (schema version 4), `tests/backup_format.rs` and a
  `fixtures/backup-v4-italian.json`.
- `crates/lingua-wasm`: `src/lib.rs` (`pack_gloss`'s Italian arm); `tests/italian_baseline.rs`,
  `tests/support/italian.rs`, `tests/baseline/pages-it.txt` and `it-fr.golden`; `tests/languages.rs`
  (an Italian pack added to an engine glossed in French).
- `scripts/lingua-data/testdata/it-fr/`: the fixture pack's tables, with the one-tag `tags.tsv`
  change 4 gives a fixture without grammar.
- `apps/lingua-extension/tool/packs.mjs` (`ANALYZER_CONSTANTS`), `apps/lingua-agent/rust/src/vocab.rs`
  (`language_name`).
- `.github/workflows/lingua-extension-check.yml` and `lingua-pack-update.yml`: the Italian
  baseline runs and is re-blessed with the others.
- Untouched: every committed table and pin, the six goldens, the extension's bundle (no listed pair
  studies Italian), the server, the site.
