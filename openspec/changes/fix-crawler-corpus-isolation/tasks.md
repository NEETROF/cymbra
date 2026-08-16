## 0. Operational precondition

- [ ] 0.1 Keep production crawls **paused** until section 1 ships: any run under the current
      code recreates `.checkouts` inside the served corpus. (Production was realigned by hand
      on 2026-08-16 — 4.4 GB of checkouts and the three run artefacts moved out of
      `SCORES_DIR`, corpus back to 1.8 GB — so the nightly 04:00 UTC mirror is harmless in
      the meantime.)

## 1. Work location separated from the corpus root

- [ ] 1.1 Add a work-location setting to the crawler config (`CYMBRA_SCORE_WORK_DIR`, same
      `CYMBRA_SCORE_*` convention as the existing overrides), resolved independently of the
      store root, with a default that is never a child of it.
- [ ] 1.2 Resolve the source-checkout directory from the work location instead of
      `root.join(".checkouts")` in `crates/score-crawler/src/main.rs`, and thread it through
      `build_adapters`.
- [ ] 1.3 Refuse to start when the resolved work location is inside the corpus root: exit with
      a clear error before any crawl or write.
- [ ] 1.4 Unit tests: checkouts resolve under the work location; a work location nested in the
      corpus root is rejected before writing; the default resolves outside the corpus root.

## 2. Run artefacts out of the corpus root

- [ ] 2.1 Write `manifest.json`, `manifest.csv` and `rejected.log` to the work location in
      `crates/score-crawler/src/output.rs`, leaving the servable object layout
      (`<prefix>/<shard>/<uuid>.mxl`) unchanged under the corpus root.
- [ ] 2.2 Unit test: after a write, the corpus root contains only servable objects and the
      artefacts are found at the work location.

## 3. Write-time idempotence (stop the accumulation)

- [ ] 3.1 Derive the object key's identifier from the content fingerprint instead of a
      per-run UUIDv7, so re-crawling unchanged content resolves to the same key. Keep the
      shard-directory derivation and the `<prefix>/<shard>/<id>.mxl` shape intact.
- [ ] 3.2 Verify the ingest path still matches rows to objects correctly when two sources
      yield the same content (one object, two rows sharing `object_key`).
- [ ] 3.3 Unit tests: the same content written twice yields one object and a stable key;
      different content yields different keys; existing rows' keys are untouched (no
      re-keying, no migration).

## 4. Mirror allow-list

- [ ] 4.1 Rewrite `backend/deploy/sync-scores.sh` to sync the servable prefixes explicitly
      (`safe/`, `low_confidence/`, `user-scores/`) instead of syncing `$SCORES_DIR` whole, so
      anything else at the corpus root is not mirrored. Keep S3 key == `object_key` and keep
      the absence of `--delete`.
- [ ] 4.2 Update the script header and `DEPLOY.md` to state the allow-list and why it is
      deliberately redundant with section 1.

## 5. Corpus↔catalog reconciliation

- [ ] 5.1 Add a `reconcile-corpus` maintenance binary in `backend/server` alongside
      `backfill-titles`, reusing the server config and the `cymbra-storage` object port so it
      reconciles the same keyspace the server reads (local + S3).
- [ ] 5.2 Compute the unreferenced set as corpus objects minus the **set** of `object_key`
      values referenced by any `catalog_scores` row (never per row).
- [ ] 5.3 Dry run by default; `--apply` performs removals — matching `backfill-titles` /
      `backfill-mutopia-titles`.
- [ ] 5.4 Safety guards: abort without writing when the referenced set is empty or when the
      share of objects to remove exceeds a configurable threshold; report why.
- [ ] 5.5 Make removal reversible — move to a quarantine prefix (local and S3) — with a
      separate explicit purge step.
- [ ] 5.6 Unit tests: referenced objects are never removed, including one referenced by a
      second row; empty reference set aborts; over-threshold aborts; dry run writes nothing;
      quarantined objects are restorable before purge.

## 6. Deployment

- [ ] 6.1 Add the work-location mount to `backend/deploy/docker-compose.crawler.prod.yml`
      next to the `SCORES_DIR` mount, and document the required ownership.
- [ ] 6.2 Note in `DEPLOY.md` that `/opt/cymbra/backend` is not a git checkout, so compose and
      script changes must be copied to the box explicitly — the box ran a month-old crawler
      compose during the 2026-08-16 investigation precisely because this step was implicit.

## 7. Validation

- [ ] 7.1 `openspec validate fix-crawler-corpus-isolation --strict` passes.
- [ ] 7.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
      and `cargo llvm-cov --workspace --fail-under-lines 80` pass.
- [ ] 7.3 MANUAL / prod: deploy the compose + script, run a crawl, confirm the corpus root
      gains only servable objects while the work location fills outside it, and confirm a
      re-crawl of unchanged content leaves the object count unchanged.
- [ ] 7.4 MANUAL / prod: run `reconcile-corpus` dry, check the count against the ~145 430
      unreferenced objects measured on 2026-08-16 (290 637 files under `safe/` for 145 280
      rows; 141 under `low_confidence/` for 68 rows), then `--apply` to quarantine, then purge
      after a grace period.
- [ ] 7.5 MANUAL / prod: after the mirror change, confirm a non-servable entry left at the
      corpus root is not transferred by `sync-scores.sh`.
