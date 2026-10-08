# Tasks

## 1. The catalogue and the pin (scripts/lingua-data)

- [ ] 1.1 `pack_sources.py`: `EDITIONS` (the three dumps' addresses and their catalogues, D1), `DUMPS[pair]` as the files each pair reads by edition (en-fr, es-fr, es-en re-registered; change 22's en-es re-registered if it landed first), `KAIKKI` deleted; `fetch_live` reads dumps only, derives an edition's whole catalogue at its first read of a run into `work/editions/<edition>-<snapshot>/` and copies what a later pair reads (D4), hashes the decompressed stream in the same pass and records `dump: {sha256, size, compressed_size}` (D3), deletes the dump, writes no extract record; `fetch_pinned` reads a record with `asset` or with `files` alike, from the release it names, with no registry check (D5); `assets --release` lists both shapes; a new `dumps --pin` lists the dump records for the notes (D7). `test_pack_sources.py`: *One pass per edition per run*, *The dump's identity*, *A legacy extract record*, *A pair reading two editions*, *The catalogue derived whole*, *A dump served plain* (change 22's magic test kept).
- [ ] 1.2 `build.sh`: the editions' folder passed to `fetch-live` (`LINGUA_EDITIONS`, default `work/editions`), nothing else; `test_reduce_loops.py` doubles it.

## 2. The workflow and the measurements

- [ ] 2.1 `.github/workflows/lingua-pack-update.yml`: the publish step's notes from `pack_sources.py dumps --pin` (D7); the monthly dry run as one job over every pair in `pairs` order, `work/editions` kept across pairs and removed at the end, each dump deleted after its pass (D4, D8); the header comment. `test_reduce_loops.py`: *The release notes name the dumps* (dump-only, legacy and mixed pins), *The dry run checks every pair in one job*, *A dump is fetched once per run* (`curl` doubled and counted).
- [ ] 2.2 The equivalence (D6), on the pull request, recorded in `SOURCES.md`: first, while kaikki still serves the regenerations the pins record (2026-10-02 for the French and Spanish dumps, 2026-10-03 for the English one), es-fr's derived files re-derived to their pinned sha256 and es-en's committed tables reproduced from the English dump; then, on one day, es-fr and es-en both ways for `kaikki-Spanish.jsonl`, en-fr both ways for `kaikki-Anglais.jsonl`, en-es (or its derived file) for `kaikki-en-traductions-es.jsonl` — `pack_report.py --identical` on every folder; and what a derived entries file differs by from the extract of the same regeneration (212 MB against 259 MB for the French edition's Spanish section).
- [ ] 2.3 The figures (D4, D8), on the pull request, recorded in `SOURCES.md` and the workflow's comment: the English edition's decompressed size, the download times at kaikki's throughput, `derive`'s pass per edition, a dry run of es-fr and the monthly job as one job (en-fr, es-fr, es-en, and en-es if committed), the peak disk use. An update above 45 minutes tightens the prefilter before anything else (Risks).
- [ ] 2.4 The `reduce` job of `lingua-extension-check` green on the pull request with every committed byte reproduced and no pin changed (*The committed pairs reduce as before*).

## 3. Documents

- [ ] 3.1 `SOURCES.md` (the editions' dumps: the catalogue, the sizes, the measurements, the legacy records; the per-pair sections naming sections of editions, not extracts), `tables/en-fr/README.md`, `tables/es-fr/README.md`, `tables/es-en/README.md` (*Take in upstream changes*), the header comment of `lingua-pack-update.yml`; change 22's `SOURCES.md` note on "change 38 switches the address" honoured (D9).

## 4. Gates

- [ ] 4.1 `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`; actionlint on the workflow; the `reduce` and `check` jobs green on the pull request; `yarn gen:pack:real` in `apps/lingua-extension` unchanged (the pinned packs).
- [ ] 4.2 `openspec validate migrate-lingua-pack-sources-to-raw-dumps --strict` passes, and `python3 scripts/openspec_archive_order.py migrate-lingua-pack-sources-to-raw-dumps` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 38 is marked done in `docs/lingua/language-matrix-programme.md`.

## 5. Owner

- [ ] 5.1 [manual] The owner decides when the shipped pairs move to the dumps: an update of es-fr (which brings es-en along, reduced from its own pin) and of en-fr, each its own reviewed pull request with the baselines re-blessed — soon after the merge, or when the monthly report calls for it (D5; *The next update moves a pair*).
- [ ] 5.2 [manual] The owner accepts the monthly job's measured duration as one job (D4), or asks for the per-edition derive job as a change of its own.
