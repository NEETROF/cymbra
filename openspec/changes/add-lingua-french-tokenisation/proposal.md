# add-lingua-french-tokenisation — French's tokenisation pre-pass (M21), analyser 0.2.0

## Why

Change 40 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the third of stage 3 (French studied: fr-en, fr-es). Change 39 (`add-lingua-french-baseline`)
made French a studied language served by the baseline analysis, at analyser `0.1.0`, and froze
what that does to raw French text in `crates/lingua-wasm/tests/baseline/fr-en.golden`. Its design
names this change as the one that bumps French to `0.2.0` for the tokenisation; the owner settled
how French is cut into words on 2026-10-09 (M21): `au`/`aux` split, `du`/`des` whole, one
highlight span per elision piece.

What the baseline makes of French today, measured on change 39's thirteen-page corpus
(`baseline/pages-fr.txt`, 917 tokens in its French blocks):

- **Elisions are whole.** The corpus holds 63 elided words — 6.9 % of its words (the study
  measured 6.5 % of literary French) — and the baseline keeps each glued to the next word:
  `l'homme`, `qu'il`, `n'est`, `jusqu'au`. Each is counted, glossed and carded as one unknown
  « word » the pack does not have: `l'homme` is not `homme`. Five of them open a sentence: three
  (`L'homme`, `Lorsqu'il`, `D'abord`) are set aside as proper nouns, being capitalised and outside
  the lexicon, and two (`C’est`, `Qu’est-ce`) are glued to a narrow no-break space (below).
- **`au` and `aux` are one word**, so `à` and `le` are never read in them.
- **Hyphenated inversions are compounds.** `dit-il`, `a-t-il`, `Qu’est-ce`, `allez-vous-en`,
  `coupez-les` are 12 runs of the corpus: eleven counted as one unknown compound judged by its
  weakest part, `Donne-m'en` set aside as a name; the euphonic `-t-` is read as a part.
- **French punctuation glues to words.** UAX #29 gives the narrow no-break space (U+202F), which
  French sets before `?`, `!`, `;` and `»` and after `«`, the word-break class ExtendNumLet: 12
  tokens of the corpus carry it (`pas `, ` Je`, ` C'est`), none of them a pack word, and ` Je` and
  ` C'est` keep the space in their lemma. The no-break space U+00A0, which the corpus sets before
  `:`, does not glue.

The cascade, the closed classes, the names rule and NFC come in change 41; the expression keys in
change 44; the word card in change 51. Each of them reads what this change cuts.

## What Changes

- **French's tokenisation pre-pass** in `crates/lingua-core/src/analysis/tokenize.rs`, French's
  arm alone:
  - **U+202F is a space**: a French word is cut at the narrow no-break space before anything
    else reads it.
  - **Elision**: an elided word from a closed list — `c'`, `ç'`, `d'`, `j'`, `l'`, `m'`, `n'`,
    `qu'`, `s'`, `t'`, `jusqu'`, `lorsqu'`, `puisqu'`, `quoiqu'`, with the straight or the
    typographic apostrophe — is split from the word it is joined to. Each piece is a token with
    **its own source span**, the elided piece's span holding its apostrophe, and the elided piece
    is read as the word it stands for (`le`, `de`, `que`…; `si` before `il`/`ils`; `moi`/`toi`
    after a hyphen), its written capital kept. `aujourd'hui`, `presqu'île` and `quelqu'un` stay
    whole: what precedes their apostrophe is not on the list.
  - **`au` → `à` + `le`, `aux` → `à` + `les`**, sharing the source span, as Spanish's `al`/`del`
    do; **`du` and `des` stay whole** (M21).
  - **Hyphenated inversions**: a run the pack does not list whole whose pieces after the first are
    pronouns in lowercase (`dit-il`, `allez-vous-en`, `donne-m'en`), the euphonic `t` before
    `il`, `elle`, `on`, `ils` or `elles` allowed, is read as separate words, each with its own
    span; the `t` is no word. Every other run keeps the compound rule, after an elision on its
    first piece is split off (`l'arc-en-ciel` → `le` + `arc-en-ciel`).
  - The pieces are lemmatised as any word is: by the baseline (the pack's forms, else the
    lowercased form) until change 41's cascade.
- **French's analyser version becomes `0.2.0`.** The fixture pack's manifest follows.
- **The French golden is re-blessed**: 26 of its 136 probes move, 5 are added, the 110 others are
  byte for byte. The fixture pack gains the words the pre-pass writes that it lacked and two
  expressions whose keys change 44 makes reachable.
- **The extension resolves a click between two pieces to the piece that starts there** — with
  separate spans, `l’` and `homme` meet at one offset, which the hit test today gives to `l’`.
  Shared spans (`don't`, `del`) open as before. No French reaches the extension before change 52.
- **English, Spanish and the four goldens do not move** (en-fr, es-fr, es-en, en-es).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-analysis`:
  - ADDED — *French tokenisation pre-pass* (U+202F, elision, `au`/`aux`, `du`/`des`, `0.2.0`) and
    *French hyphenated inversions are read as words*.
  - MODIFIED — *French is a studied language served by the baseline analysis* and *A French
    invariance baseline runs beside the English and Spanish ones*, both added by
    `add-lingua-french-baseline` (change 39), merged and not archived: this change archives after
    it (`archiveAfter`) and its MODIFIED blocks carry change 39's wording with only what moves at
    `0.2.0` rewritten, every requirement and scenario name kept.
- `lingua-browser-extension`: ADDED — *A click opens the piece of a split word under the pointer*.
  No open change holds a requirement on the click.

## Impact

- **Products.** Cymbra Lingua only, and in it:
  - `crates/lingua-core` — *new*: French's pre-pass, `FRENCH_ANALYZER_VERSION = "0.2.0"`;
    *consumed*: the tokeniser's language-neutral rules (segmentation, the hyphen run, the compound
    rule, the digit drop, the single-letter rule), the baseline lemmatisation, the `Token` model
    (unchanged: every token already has its own span).
  - `crates/lingua-wasm` — the French baseline re-blessed, five probes added to its scenario (three
    phrases, two word grammars), its assertions moved to `0.2.0`; a French page under wasm checked
    against the host's spans.
  - `crates/lingua-pack` — two tests read French's constant instead of a literal `0.1.0` (the
    builder's, and the fr-en fixture's in `tests/pipeline_testdata.rs`).
  - `scripts/lingua-data/testdata/fr-en/` — the manifest at `0.2.0`, ten forms and two expressions
    added.
  - `apps/lingua-extension` — the hit test of `reading/scan.ts`; `test/packs.spec.ts` reads `0.2.0`
    for French; `test/selection.spec.ts` pins the selection case handed to change 51. No surface,
    type, label or pack list changes.

  ID, Music, Live, the back office, the site, the backend, the Apple host app and the agent plugin
  are untouched (the agent compiles the core and holds no French pack).
- **Release.** Silent. No listed pair studies French; nothing a reader sees changes.
- **Compatibility.** No stored format, wire field, pack byte or pin moves. A French pack must carry
  `0.2.0`; none exists outside the fixture.
- **Not here.** « pas » as a function word (M21) — a line of French's closed classes, which
  change 41 writes with the cascade, the names rule and NFC; the moods merged on a five-reading form
  such as « parle » (M21) — a rule of the French word card's description, change 51, reading the
  readings of change 45; expression keys through the analyser (44); which piece a one-word
  selection opens — a double-click on « l’homme », or a drag over « homme » that the selection's
  word snap widens to « l’homme » (51, D7); the Catalan and Occitan guard (42); the forms tables
  (43).
- **Effort, against 4.5–7.5 ideal days.** The pre-pass, its spans and its casing: 1.5–2.5.
  The French fixtures (at least 80 cases, rule by rule): 1–1.5. The golden, the probes, the
  fixture pack and the tests that flip at `0.2.0`: 0.75–1.25. The hit test, its test and the
  selection pin: 0.25–0.5. Specs and programme: 0.5. Total 4–6.25.
