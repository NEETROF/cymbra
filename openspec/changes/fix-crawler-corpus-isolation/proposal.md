## Why

The score-crawler derives its **source-checkout directory from its output root**
(`root.join(".checkouts")`). Since `fix-catalog-serving-and-hub-feedback` pointed that root
at `SCORES_DIR` — the directory the server serves — the git clones of all five sources now
land *inside the served corpus*. Measured on production while validating that change's live
task 6.4: **4.4 GB and 254 105 source `.mxl`** under `SCORES_DIR/.checkouts`, plus
`manifest.json`, `manifest.csv` and `rejected.log` at the corpus root.

That would have shipped off-box: `sync-scores.sh` mirrors `$SCORES_DIR` to the scores bucket
with **no exclusions**, `aws s3 sync` does not skip dot-directories, and a root cron runs it
nightly at 04:00 UTC. The corpus was cleaned by hand before the cron fired, but nothing
prevents the next crawl from putting it back — the cause is in the code, not the box.

Measuring the corpus to clean it surfaced a second, older problem: **half of it is
unreferenced**. 290 637 files under `safe/` for 145 280 catalog rows, 141 under
`low_confidence/` for 68 rows — roughly **145 430 `.mxl` that no row points to**, mirrored to
S3 as well. Deduplication happens at *ingest* (by content fingerprint), not at *write*, so
every re-crawl rewrites identical content under fresh UUIDs and the corpus grows without
bound.

## What Changes

- **Working directories leave the served corpus (the cause).** The crawler's source
  checkouts and per-run artifacts (manifest export, rejection log) MUST be written to a work
  location distinct from the served corpus root, configurable independently of it. Only
  servable objects may exist under the corpus root.
- **The off-box mirror carries only servable objects (the defense).** `sync-scores.sh` MUST
  mirror servable object keys exclusively, with explicit exclusions, so that a working
  directory reappearing under `SCORES_DIR` for any reason can never reach the bucket. This is
  deliberately redundant with the fix above: the mirror is billed, nightly, and unattended.
- **Writes stop accumulating duplicates.** Content already ingested MUST NOT be rewritten
  under a new object key on a re-crawl, so repeated runs converge instead of doubling the
  corpus.
- **Corpus↔catalog reconciliation tool.** A maintenance command reports and, on `--apply`,
  removes corpus objects (local and S3) that no catalog row references. It follows the
  existing maintenance-bin convention (`backfill-titles`, `backfill-mutopia-titles`): **dry
  run by default**, never touching a referenced object.

## Capabilities

### New Capabilities
<!-- None — all changes modify existing capabilities. -->

### Modified Capabilities
- `corpus-ingestion`: the crawler's work location is separate from the served corpus root
  (only servable objects live there); the off-box mirror carries servable objects only;
  already-ingested content is not rewritten under a new key; unreferenced objects are
  reconcilable.
- `corpus-manifest`: the derived manifest export and the rejection log are written to the
  crawler's work location, not to the served corpus root.

## Impact

**Products.** Cymbra **Music** only — it consumes the corpus this change protects; its
requirements are unchanged. Nothing in ID, Live or the back office is touched.

**Code.** `crates/score-crawler/src/main.rs` (output root vs work root), the crawler
config/CLI, `crates/score-crawler/src/output.rs` (write-time dedup, manifest/rejection-log
destination), `backend/deploy/docker-compose.crawler.prod.yml` (a work-dir mount alongside
the `SCORES_DIR` mount), `backend/deploy/sync-scores.sh` (explicit exclusions), and a new
maintenance binary reusing the `cymbra-storage` object port so it reconciles exactly the
keyspace the server reads.

**Operations.** Production was realigned by hand during the investigation: the 4.4 GB of
checkouts and the three run artifacts were moved out of `SCORES_DIR` (corpus back to 1.8 GB,
nightly mirror harmless again). The ~145 430 unreferenced objects — local **and** in the
bucket — remain and are what the reconciliation tool is for. Crawls should stay paused until
the work-directory fix lands, since any run recreates `.checkouts` under the served corpus.
