# migrate-lingua-pack-sources-to-raw-dumps — one dump per Wiktionary edition, and no per-language extract

## Why

Change 38 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the first of stage 3, and the mitigation its risk 6 names: kaikki has marked its per-language
files deprecated (still served on 2026-10-08) and will keep only its dump of each whole Wiktionary
edition. Today the pipeline reads kaikki two ways. Three pairs pin a **per-language extract** —
en-fr the French Wiktionary's English section (201,084,572 B), es-fr and es-en the English
Wiktionary's Spanish section (1,054,565,864 B and 1,054,565,723 B: the same address, two
regenerations, pinned twice under two releases) — and two pairs read **whole-edition dumps** for
what no extract gives (es-fr the French and Spanish Wiktionaries' translation tables, es-en the
Spanish one's), deriving four small files in one pass and keeping no dump
(add-lingua-spanish-gloss-tables D1). Change 22 (en-es) is the first pair whose sources are
dumps alone, and its design leaves "the move to raw dumps for every pair" to this change.

Stage 3 multiplies the extracts if nothing is done: fr-en needs the English Wiktionary's French
section (583,677,104 B served today) and fr-es the Spanish Wiktionary's, beside the French
Wiktionary's own translation tables — five extracts for six pairs, two of them the same gigabyte
fetched and published twice, each at an address kaikki has announced it will remove. The
raw-dump model reads three addresses in all, one per edition (measured on 2026-10-08: the English
edition 2,981,058,381 B gzipped, the French 736,590,407 B, the Spanish 103,226,106 B), and derives
every file every pair reads from them in one pass per edition.

What this change does **not** do is re-pin a committed pair. The dumps kaikki served on the days
en-fr, es-fr and es-en were pinned were never kept, and kaikki keeps no history: the pinned
extracts are the only copy of those regenerations. A re-reduction "from the dumps at the same
snapshot" is therefore impossible, and a reduction from today's dumps is a dictionary update —
upstream drift of weeks — which the programme keeps apart from a tooling change (*en-fr and es-fr
output does not move*). The pinned extracts stay readable from their releases, every committed byte
keeps reproducing, and each pair moves to the dumps at its next update, reviewed as any update is.

## What Changes

- **`pack_sources.py`: one catalogue per edition.** `EDITIONS` names the three dumps and every
  file derivable from each (a language's entries, or a translation table); `DUMPS[pair]` names the
  files a pair reads, by edition; `KAIKKI`, the extract registry, is retired. An update reads only
  dumps, and `derive` makes one pass per edition, computing the sha256 and size of the
  decompressed bytes as it goes: the dump is recorded in the pin (`dump: {sha256, size,
  compressed_size}` beside its address and regeneration date), never kept, never published.
- **What a pair reads is what is published**, as today: its derived files, zstd-compressed, under
  its own release `lingua-pack-sources-<pair>-<snapshot>`, pinned by the sha256 of their
  decompressed bytes (change 21's D2 unchanged). The largest asset is the English Wiktionary's
  Spanish section, about 52 MB compressed; a dump is never an asset — the English edition's alone is
  2.78 GiB gzipped, above the 2 GiB a release asset may weigh.
- **A dump is read once per run.** The edition's whole catalogue is derived at the first read, kept
  for the run in `work/editions/`, and later pairs of the run copy what they read. The monthly dry
  run becomes one job over every pair instead of a matrix, so each dump is fetched once a month
  (about 3.6 GiB) rather than once per pair (about 14 GiB with six pairs); each pair of the loop
  reduces into a dry root of its own, so the reports the matrix wrote are still written, and a
  failing pair does not stop the loop.
- **Legacy pins read as recorded.** `fetch-pinned` reads a source that names an `asset` (an
  extract) or `files` (derived files) alike, keeps a legacy `kaikki` record in the pin instead of
  pruning it with the registry, and names the extract's raw file after its asset; the reduce job
  keeps reproducing en-fr's, es-fr's and es-en's committed bytes from their pinned extracts; no
  pin, table, manifest, pack or baseline moves in this change.
- **The equivalence is measured, not assumed.** Before the first update under the new rule, each
  pair is reduced both ways from the same regeneration — the per-language extract and the
  entries derived from the dump — and the tables must be byte for byte the same
  (`pack_report.py --identical`); the measurement is recorded in `SOURCES.md`, and it is the
  proof. On 2026-10-08 the served dumps still carry the regeneration dates es-fr's and es-en's pins
  record, so the measurement is first tried at the pinned snapshots themselves — if the dumps are
  still served that day, and skipped otherwise; that try gates nothing.
- **The update workflow**: the release notes name each edition's dump, its regeneration date and
  sha256, instead of an extract; the dry run loops over every pair in one job; `test_reduce_loops.py`
  follows.
- **Stage 3 is prepared**: the catalogue already lists the English Wiktionary's French section,
  the English entries' French translations, the French Wiktionary's English translations and the
  Spanish Wiktionary's French section — fr-en and fr-es (changes 43, 48, 49) register what they
  read and derive nothing new.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `lingua-data-packs`: ADDED *Every kaikki source is derived from one dump per Wiktionary
  edition*, *A pin recorded against kaikki's per-language extract stays readable*, *A dump is read
  once per run*; MODIFIED *Sources derived from whole Wiktionary dumps are pinned* — an extract is
  no longer a source of an update, and the requirement keeps only what the ADDED one does not
  say: a dump served gzipped or plain, a fetched file of other bytes refused; its three scenario
  names kept, the third (*An extract served plain*, change 22's) now about a dump served plain,
  read by the gzip magic. That requirement is held by the open change `add-lingua-pack-en-es`, so
  this change archives after it (`.openspec.yaml`). *A reader pair's pin records the studied tables
  it read* (change 21) stands as written: each pair still pins its own fetch and an update still
  publishes a pair's own assets only.

## Impact

- **Products.** Cymbra Lingua only, and only its data pipeline: `scripts/lingua-data`
  (`pack_sources.py`, `build.sh`, `SOURCES.md`, the three pairs' READMEs, `test_pack_sources.py`,
  `test_reduce_loops.py`) and `.github/workflows/lingua-pack-update.yml`. Consumed, unchanged: the
  pins and per-pair releases (pin-lingua-pack-sources, add-lingua-pack-es-en D2), the asset cache,
  `derive`'s two kinds of file and `translations_of`, the reduce job of `lingua-extension-check`,
  `pack_report.py --identical`. New: the edition catalogue, the dump record in a pin, one pass per
  edition per run, the dry run as one job, the measurements. ID, Music, Live, the back office, the
  site, `apps/lingua-extension`, the crates and the backend are untouched.
- **No byte moves.** The committed tables, pins, manifests, packs and both invariance baselines
  are as they are; `packs.json` and the site's figures are unchanged. The reduce job's runtime is
  unchanged (3 min 38 s for three pairs on 2026-10-08): a pinned re-reduction fetches derived
  files and extracts from releases and never a dump.
- **An update costs more download, once per edition.** es-fr's reads 3.6 GiB of dumps instead of
  a 1.05 GB extract and 0.8 GB of dumps; en-fr's 0.7 GiB instead of 0.2 GB; kaikki's observed
  throughput (about 4 MB/s during es-en's first update) puts the English edition's dump at about
  12 minutes. Measured on the implementation pull request's dry runs; the budget is one update
  within 45 minutes.
- **Effort.** 3–4.5 ideal days against the programme's 2.5–5: the catalogue and the pin record with
  their tests (1.5–2), the workflow and its loop tests (0.5–1), the measurements (0.5–1, mostly
  machine time), the documents (0.5). Outside the estimate: a per-edition derive job, worth
  writing only if the monthly job measured here runs longer than the owner accepts (about one
  day), and the French studied side's own files (change 43).
- **Not here.** Re-pinning any pair (each pair's next update); shipping anything; the French
  analyser and tables (39–47); the fr-en and fr-es pairs themselves (48, 49).
