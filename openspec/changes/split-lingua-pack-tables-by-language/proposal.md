# split-lingua-pack-tables-by-language — a studied language's tables, kept once

## Why

Decision M24 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md)
(change 7): the studied side of the tables is kept once per studied language, and the native side
per pair. Today `tables/<pair>/` holds both sides.

A second pair of the same studied language would need a copy of forms, frequencies, readings,
levels, the tag pin and the dictionary words. That is 4.3 MB for English and 13 MB for Spanish. The
copies could drift apart, since two English reductions made on different days lemmatise
differently. Change 5's check would catch a drift, but only after it happened. es-en (change 21)
is the first such pair.

This is the fourth change of stage 1, and a silent release. The tables move and their bytes do
not:
- no reducer is edited, so no rule digest and no `pack_version` move;
- en-fr and es-fr build to the sha256 their pins record;
- both baselines pass unchanged.

## What Changes

- **Two kinds of folder.**
  - `tables/en/` and `tables/es/` hold the studied tables: `forms.tsv`, `freq.tsv`, `grammar.tsv`,
    `level.tsv`, the tag pin `tags.tsv`, the dictionary words `lexical.tsv`, and a `studied.json`
    naming the reference pair.
  - `tables/<pair>/` keeps `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`,
    `pin.json` and `README.md`.
- **The split happens after reduction, outside the reducers.**
  - A reducer still writes every table into its work folder.
  - `pack_sources.py split` files them by side.
  - Only the pair that `studied.json` names writes the studied folder, and it derives `lexical.tsv`
    from its glossed lemmas.
- **One pin per pair.** The reference pair's pin is the provenance of its studied language's tables,
  so en-fr's and es-fr's pins do not change by one byte.
- **The builder reads two folders** (`--studied`), and refuses a table found on the wrong side.
  Test packs keep their single folder.
- **Every consumer follows.** That covers:
  - the build, reduce and update loops (pairs are `tables/*-*/`, references first);
  - the committed-tables check;
  - `pack_report`, `gloss_coverage`, `measure`;
  - the extension's `gen_pack.sh`;
  - the reviewers' source archive and its README;
  - the baselines' and tests' table paths.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED.
  - *A studied language's tables are kept once*;
  - *Only the reference pair's reduction writes its studied language's tables*;
  - *A change to a studied language's tables reaches every pair of that language*;
  - *A pair's committed tables are its own and its studied language's*, an umbrella over every
    requirement that speaks of a pair's committed tables, including *The shipped pairs are one
    list*, which the open `enable-lingua-spanish` holds.

No requirement is modified. Change 5's comparisons of « two packs of one studied language » stay
true by construction: there is one copy.

## Impact

- **Products.** Cymbra Lingua only:
  - `scripts/lingua-data`: the tables, `build.sh`, `pack_sources.py`, `pack_report.py`,
    `gloss_coverage.py`, `measure/`;
  - `crates/lingua-pack`: inputs from two folders, the committed-tables check;
  - `crates/lingua-wasm/tests`: table paths;
  - `apps/lingua-extension/tool`: `gen_pack.sh`, `make_source_archive.sh`;
  - `.github/workflows/lingua-extension-check.yml` and `lingua-pack-update.yml`.

  No core, engine, extension, agent or server code changes. ID, Music, Live, the back office and the
  site are untouched (the site's coverage file does not change).
- **No pack byte, pin, digest, golden or analysis moves.** About 400 KB of derived `lexical.tsv` is
  added.
- **Not here.** What a non-reference pair's pin, `pack_version` and credits must say about the
  studied tables it reads: change 21 (es-en), the first such pair.
