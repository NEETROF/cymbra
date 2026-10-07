# generalise-lingua-gloss-reducer — one reducer for the French, English and Spanish Wiktionaries

## Why

The language matrix glosses packs in English and in Spanish
([programme](../../../docs/lingua/language-matrix-programme.md), change 6). The shared reducer is
written for « studied → French » pairs. Its form-of wording (« pluriel de », « forme de »), its
notes and placeholders (« Définition manquante »), its dangling coordinators, its letter senses and
its casing all belong to the French Wiktionary. The English Wiktionary (« plural of »,
« inflection of », untagged « synonym of ») and the Spanish one (« Forma flexiva », « Forma del
plural de », sense-link subscripts) need their own rules.

Two other gaps:
- `derive()` reads only translation tables written for a whole entry. The English Wiktionary
  writes them under senses: 68,579 English entries list Spanish translations under a sense,
  against 5,080 with a table for the whole entry.
- The rule digest covers every shared `reduce_*.py`. Any later tuning of the English or Spanish
  rules would re-pin en-fr and es-fr and re-bless both baselines, against the programme's rule that
  their bytes move once.

This is the third change of stage 1. **It is the one change of the programme where the en-fr and
es-fr packs' bytes move**: the new rule digest changes their `pack_version`, their pins and three
golden lines. Their tables, NOTICE and every analysis do not move. A scratch prototype re-reduced
both pairs from their pinned sources with the generalised rules: all seven data tables and NOTICE
came out byte for byte as committed.

## What Changes

- **Editions.**
  - An `Edition` holds a Wiktionary edition's cleaning rules: form-of and alt-of wording, notes and
    placeholders, dangling coordinators, pointer tags and fields, casing.
  - The French edition is today's rules, unchanged. The English and Spanish editions come from a
    census of their real kaikki data.
  - Each edition lives in its own rule module. A pair's reducer loads the shared rules and the
    module of its native language's edition.
- **The digest covers what a reducer loads.**
  - A pair's rules are its reducer plus every `reduce_*` module it loads, so an English-edition edit
    never re-pins a French-native pair.
  - A test fails when a reducer loads a rule module its recorded rules do not name.
- **Sense-level translation tables.** `derive()` also reads `senses[].translations`. This is
  measured as a no-op on es-fr's French and Spanish dumps, which write their tables per entry.
- **D4, as M5 restates it, in the spec:** a gloss is in the reader's native language, written by a
  person — never the studied language, never a third language, never machine-translated, never
  pivoted.
- **The proof.**
  - A pull-request job re-reduces every committed pair from its pinned sources and fails on any byte
    that differs.
  - `pack_report.py --identical` names the pair and the file.
  - `lingua-pack-update` gains `pair: all` (one branch, baselines re-blessed once) and
    `expect: identical`.
- **The re-pin.** en-fr and es-fr are reduced again with the new rules:
  - their seven data tables and NOTICE are unchanged;
  - `manifest.json` changes `meta.pack_version` only;
  - `pin.json` changes `pack.sha256`, `reducer.sha256` and `reducer.files`;
  - three golden lines are re-blessed: `en-fr.golden` `### pack`, and `es-fr.golden` `### pack` and
    `### beside en-fr`, each only in its `pack_version` token. **The owner approves this re-bless in
    the pull request.**

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`:
  - MODIFIED — *A pair's reduction rules include the rules it shares*: the digest covers the modules
    a reducer loads, and all pairs can be reduced in one run.
  - ADDED:
    - *The committed tables are what the rules make of the pinned sources*;
    - *A gloss is in the reader's language, written by a person*;
    - *A gloss is cleaned by the rules of the Wiktionary edition that wrote it*;
    - *A translation table is read wherever the edition writes it*.

No requirement this change modifies is held by an open change. The es-fr requirement's « A gloss
SHALL never be English or machine-translated » stays as written; it is a consistent subset of the
new requirement.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data`: the reducers, the edition modules, `pack_sources.py`, `pack_report.py`,
    `build.sh`, the tests and the docs;
  - `scripts/lingua-data/tables/{en-fr,es-fr}`: `manifest.json` and `pin.json` only;
  - `crates/lingua-wasm/tests/baseline`: three lines;
  - `.github/workflows/lingua-extension-check.yml` (the re-reduction job) and
    `lingua-pack-update.yml` (`all`, `expect`);
  - `apps/lingua-extension/REVIEWERS.md`: the rule modules.

  No core, engine, extension, agent or server code changes. ID, Music, Live, the back office and the
  site are untouched.
- **Release.** Silent: the packs a release ships carry a new `pack_version` and the same tables, so
  readers see nothing new. Shipping them is the owner's, as always.
- **Not here.**
  - M20, long English parentheses: it becomes an English-edition rule settled with es-en's review
    (change 21). Under the scoped digest, settling it re-pins no French-native pair.
  - Known French-rule defects the byte-identity gate freezes: « Synonyme de », « Exemple
    d'utilisation manquant », footnote markers, letters in en-fr. They are a separate change with a
    measured table diff and its own re-bless.
