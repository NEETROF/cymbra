# Design — add-lingua-translation-matrix-models

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `apps/lingua-extension/model-manifest.json` | `base` `https://models.cymbra.app/`, `sourceBase` Mozilla's registry; models `en-fr/base-memory/2.0` and `es-en/base-memory/2.0`, each `from`, `to`, `licence`, `mirror`, `files.{model,lex,vocab}` with `path` (`<id>/<decompressed sha256>/<file>.gz`), `size` (gzip as served), `unpacked`, `sha256` (decompressed), `source.{path, sha256}` (Mozilla's gzip); routes `en-fr` and `es-fr` |
| `tool/model-catalogue.mjs`, `tool/assemble_model_site.mjs`, `tool/mirror_models.mjs`, `tool/check_model_host.mjs` | read the catalogue; assemble the host from Mozilla's registry, then the mirror, then the host, keeping a file only when both digests and the decompressed size hold; create a mirror release per model once; check the host from outside |
| `build.mjs`, `tool/check_variants.mjs` | bundle the catalogue without `sourceBase`/`mirror`/`source`; check the bundled copy equals it, routes included |
| `src/translate/host/model-manifest.ts` | parses and validates the catalogue (route key, known ids, chain, ends); `modelsFor`, `routeOf` |
| `src/analyzer/pairs.ts` | `SHIPPED_PAIRS` from `packs.json` (`en-fr`, `es-fr`); `readerPairs(languages, native)` keeps shipped pairs only |
| `src/translate/host/model-db.ts`, `model-download.ts` | files stored by decompressed sha256; a stored file is not downloaded again; `prune(keep)` keeps any sha256 a kept model names |
| `.github/workflows/lingua-model-deploy.yml` | manual only: mirror job, then the Pages deploy of `cymbra-models`, then `check_model_host` from outside; `lingua-extension-release` and `lingua-apple-release` refuse a submission or delivery while the host misses a catalogue file |
| Mozilla's registry `db/models.json` | one en-es entry: `base-memory`, run `models/en-es/retrain_hr_fix_names_CUAEXUHoQum_cFqh-ZAryw/exported/`, files `model.enes.intgemm.alphas.bin.gz` (22,698,792 B), `lex.50.50.enes.s2t.bin.gz` (2,265,250 B), `vocab.enes.spm.gz` (409,312 B); the model's `uncompressedHash` `3b1c3995…244c`, 31,561,787 B |
| Firefox Remote Settings, `translations-models` en→es | version 2.1: model 31,561,787 B `3b1c3995…244c`, lex 4,198,436 B `7d51237c…f25c`, vocab 816,054 B `5ae254fa…58ad` (decompressed) |
| `tool/soak_engine.mjs` (change 9) | `--pair`, `--limit`, `--isolate` (one child per selection, a timeout), exit 1 on a trap or a timeout; refuses `--pair en-es` until a route is pinned |
| Tests | `test/model-manifest.spec.ts` holds the catalogue exactly ("no route for es-en"); `test/model-residency.spec.ts` and `test/model-controller.spec.ts` use the placeholder `en-es/base-memory/2.0` |

## Goals / Non-Goals

**Goals:**
- en-es 2.1 pinned as every model is, verifiable byte for byte from Mozilla's registry.
- A route per new pair, inert until the pair ships.
- The en-es trap measured on the English corpus before change 35.

**Non-Goals:**
- Shipping es-en or en-es (34, 35); their marks (26); the listings (36, 37).
- A fix of the en-es trap inside the engine or the model: change 9 bounds its cost; this change
  measures where it happens.
- A tool that writes catalogue entries: two models were pinned by hand and checked by the
  assembly; a third is not the moment to automate it (fr-en, in stage 3, may be).

## Decisions

### D1 — The entry is read from the files, and checked against two publications

The three gzip files are downloaded once from Mozilla's registry (`sourceBase` + the entry's run
path). `source.sha256` is each gzip's sha256, `size` its length; `sha256` and `unpacked` are the
decompressed bytes'. The model's decompressed sha256 must equal the registry's `uncompressedHash`,
and each file's must equal Remote Settings' for 2.1; a mismatch stops the change (the registry
would then not be 2.1). The id is `en-es/base-memory/2.1`, the paths
`en-es/base-memory/2.1/<decompressed sha256>/{model.bin.gz,lex.bin.gz,vocab.spm.gz}`, named as the two pinned
models' are, the mirror
`https://github.com/NEETROF/cymbra/releases/download/lingua-model-en-es-base-memory-2.1/`, the
licence MPL-2.0. `assemble_model_site.mjs` run locally over the new catalogue proves the entry: it
keeps a file only when every digest holds.

### D2 — Routes for the new pairs, one model each

`"es-en": ["es-en/base-memory/2.0"]` and `"en-es": ["en-es/base-memory/2.1"]`, appended after the
existing routes; the models are listed in the order they were pinned (en-fr, es-en, en-es). Each
route satisfies the parser's rules (it starts from the studied language and ends in the native),
so the bundled catalogue loads. A route is needed only when its pair is a reader's pair, and a
reader's pairs are shipped pairs (`readerPairs`): until changes 34 and 35 list es-en and en-es in
`packs.json`, no reader needs either route, and the controller downloads, keeps and loads exactly
what it does today.

Why both here: es-en's route costs nothing (its model is pinned and served), and change 26
measures both pairs through the catalogue's routes; a route added by the pair's enable change
would leave the harness without one.

### D3 — A file the two models share

en-es 2.1's vocabulary decompresses to the same sha256 as es-en 2.0's (`5ae254fa…`; the gzip
files differ). The device stores files by decompressed sha256, so a reader who holds es-en and
then needs en-es — possible only once the native language can change (change 20) — does not
download that file again, and `prune` keeps it while either model is kept. The cost the setting
states before a download is the sum over the files of the models the reader's pairs need and may then count that file
(409,312 B) although it is not fetched; this change leaves it so, as the requirement *The
setting's cost comes from the catalogue* states it.

### D4 — The en-es soak, by hand

`tool/soak_engine.mjs --pair en-es --models <dir> --isolate` over the 100 English selections of
`tool/marks/corpus.json`, with the models assembled locally from the registry (D1), never in CI
(M25's recommendation). `TRANSLATION.md` records the run: the engine pin, the date, the machine,
the selections that trapped by id, those that timed out, the time per selection and the memory
reported. The figures decide nothing here; change 35 reads them. A run that finds no trap says so:
the study's trapping inputs were not committed, and the corpus may not hold one.

### D5 — Documents and tests

`test/model-manifest.spec.ts`: three models, four routes, en-es's download 25,373,354 B and its
decompressed total, es-en's route alone 26,241,052 B, "nothing else is listed". The placeholder
id in `model-residency.spec.ts` and `model-controller.spec.ts` becomes `en-es/base-memory/2.1`.
`REVIEWERS.md` names the catalogue's three models and says which pair each route serves, and that
the package downloads none of them until the reader asks; `TRANSLATION.md` lists the routes.

## Risks / Trade-offs

- **Submissions refused until the host serves en-es** → the owner dispatches `lingua-model-deploy`
  right after the merge (task 4.1); the check from outside then passes.
- **Remote Settings' 2.1 is not the registry's file** → D1 refuses the entry; the change waits.
- **The soak changes nothing in the code** → it is a measurement for change 35, recorded where
  change 9 records its own.

## Migration Plan

No release: a catalogue entry, two inert routes and a measurement. The owner deploys the model host
after the merge; nothing a reader has stored moves.
