# score-crawler

A polite, resumable crawler that harvests **only freely-redistributable** music
scores from online libraries, converts them to compact compressed MusicXML
(`.mxl`), and writes them into a vetted corpus with a provenance manifest. Part
of the Cymbra monorepo; it reuses the shared `cymbra-musicxml-core` parser and
(once the backend `score` module lands) ingests into the same store the app
reads.

> ⚠️ **Legal notice — read this.** This tool *filters* on licence: it keeps a
> score only when the licence it detects is explicitly one of **CC0 / confirmed
> Public Domain / CC-BY (any version) / CC-BY-SA (any version)**, and it routes
> user-declared / unverified public-domain material to a separate
> `low_confidence/` tree. Automated licence detection is **best-effort and can be
> wrong** — a mislabelled source page, an ambiguous notice, or a bug can let
> through material that isn't actually free, or wrongly reject material that is.
> **The final responsibility for verifying that any redistributed score is
> lawfully reusable rests entirely with you.** Review the manifest and the
> `rejected.log` before publishing anything, and consult the original source for
> each work.

## How the licence gate works

For every candidate the licence is determined **before** any heavy download. The
raw signal (a Creative Commons deed URL, an SPDX id, a free-text label, or a
source status string) is normalised to a canonical code and evaluated:

- **Accepted → `safe/`**: CC0, Public Domain (authoritative), CC-BY-\*, CC-BY-SA-\*.
- **Low confidence → `low_confidence/`**: the same free licences but *self-declared*
  by an uploader without independent verification (e.g. musetrainer). Marked
  `unverified`; never mixed into the safe corpus.
- **Rejected → `rejected.log`**: non-commercial (NC), no-derivatives (ND), all
  rights reserved, unknown, ambiguous, or anything else. Never downloaded into
  the corpus.

## Install

### Toolchain

- Rust (stable, edition 2024 — see `rust-toolchain.toml` at the repo root).
- `git` on `PATH` (used to clone the git-based sources).

Build:

```bash
cargo build -p score-crawler --release
```

### Optional external converters

Some sources ship formats other than MusicXML. Conversion shells out to these
binaries; install the ones you need (missing binaries make the affected items
degrade gracefully — they are recorded, never crash the crawl):

- **MuseScore CLI** (`mscore`) — MuseScore `.mscx`/`.mscz` → `.mxl`
  (headless: `QT_QPA_PLATFORM=offscreen`, or run under `xvfb-run`).
- **Verovio** — MEI → MusicXML.
- **python-ly** (`ly`) — LilyPond `.ly` → MusicXML (imperfect; on failure the
  original is kept and the item is flagged `failed_kept_source`).

MIDI is never treated as a score source.

**Run converters in Docker instead of installing them.** Set `converters.backend:
docker` in `config.yaml` and point `musescore_image` / `verovio_image` /
`lilypond_image` at images that carry the tool on `PATH`. Each conversion then
runs as `docker run --rm -v <tmp>:/work <image> <tool> …` — nothing heavy is
installed on the host. (Requires Docker; on macOS the temp dir must be under a
Docker-Desktop-shared path, which `$TMPDIR` is by default.)

## Usage

```bash
# Test run: two sources, 50 items each
score-crawler --sources openscore,mutopia --limit 50

# Everything the crawler knows about
score-crawler --all

# Resume an interrupted crawl (skips already-completed items)
score-crawler --resume

# Verbose (DEBUG) logging
score-crawler --sources cpdl --verbose
```

Configuration is `config.yaml` (see the example in this crate): enabled sources,
per-host delay (default 2 s), concurrency cap (default 2), a **descriptive
contact** embedded in the User-Agent, per-source quotas, and the output store.
The store is a local folder in dev; set the `CYMBRA_SCORE_S3_*` environment
variables to target S3/MinIO in prod (that path lands with the backend `score`
module).

Please set a real `contact` before running against live sites.

### Fan-out: one container per source

`docker-compose.yml` runs one crawler container per source in parallel, each
writing to its own `output/<source>/` tree:

```bash
# unbounded — crawl everything each source offers:
docker compose -f crates/score-crawler/docker-compose.yml up --build
# cap each source to N items (applied per source):
LIMIT=5 docker compose -f crates/score-crawler/docker-compose.yml up --build
```

`LIMIT` is optional: set → `--limit N` per source; unset → no limit.

## Output

```
output/
  safe/            <source>/<author>/<title>-<sha8>.mxl   # high-confidence corpus
  low_confidence/  <source>/...                           # unverified PD, to review
  manifest.csv                                            # one row per retained file
  manifest.json                                           # same, structured
  rejected.log                                            # excluded items + reason
```

Each manifest row carries exactly the fields needed to re-publish a score
lawfully — title, composer, arranger, source, source URL, normalised licence
code, licence URL, confidence, sha256, origin format, conversion status — plus
search metadata (key, time signature, measure count, …) and a
provenance-tracked difficulty (`level` + `level_source`, where a source-declared
grade wins over the non-authoritative heuristic estimate).

## Politeness

The crawler honours `robots.txt`, sends a descriptive User-Agent with your
contact, waits between requests, backs off exponentially on `429`/errors, caps
concurrency low, and deduplicates across sources. Please keep the defaults
conservative.

## Deduplication

Two layers, so the same score is never stored twice — within a site or across
sites:

- **Exact content** — SHA-256 of the canonical decoded MusicXML (not the `.mxl`
  bytes, so framing never defeats it). Enforced in-run, in the resumable state
  cache, and in the catalog (`sha256 UNIQUE` + `ON CONFLICT DO NOTHING`).
- **Musical fingerprint** — a hash of the *notes themselves* (onset, staff,
  pitch, duration on a divisions-independent grid), so the **same piece
  re-encoded** by a different editor, or found on another site, is caught even
  though its bytes differ. It ignores lyrics/dynamics, so two scores differing
  only in words (hymn verses over one tune) collapse to one — usually what you
  want for a playable-notes corpus; disable it if not. Transpositions are kept
  distinct (different playable notes).

At ingest, an entry already present by either key is skipped.
