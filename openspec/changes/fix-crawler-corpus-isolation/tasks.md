## 0. Operational precondition

- [x] 0.1 Keep production crawls **paused** until section 1 ships: any run under the current
      code recreates `.checkouts` inside the served corpus. (Production was realigned by hand
      on 2026-08-16 — 4.4 GB of checkouts and the three run artefacts moved out of
      `SCORES_DIR`, corpus back to 1.8 GB — so the nightly 04:00 UTC mirror is harmless in
      the meantime.) — held; the constraint now transfers to 7.3: the box still runs the old
      image and compose, so **crawls stay paused until both are deployed**.

## 1. Work location separated from the corpus root

- [x] 1.1 Add a work-location setting to the crawler config (`CYMBRA_SCORE_WORK_DIR`, same
      `CYMBRA_SCORE_*` convention as the existing overrides), resolved independently of the
      store root, with a default that is never a child of it. — `Config::work_dir`, default
      `./work` (sibling of `./output`); `Config`'s derived `Default` was replaced by a
      hand-written one so the field cannot default to an empty path.
- [x] 1.2 Resolve the source-checkout directory from the work location instead of
      `root.join(".checkouts")` in `crates/score-crawler/src/main.rs`, and thread it through
      `build_adapters`. — now `work_dir.join("checkouts")`. The run summary also prints both
      roots resolved (`Corpus: … | Work: …`); the old bare `Output: ./output` is what hid a
      month-old mount in production.
- [x] 1.3 Refuse to start when the resolved work location is inside the corpus root: exit with
      a clear error before any crawl or write. — `Config::resolved_work_dir`, called before
      `build_adapters`. Containment is compared lexically (both sides normalised, `..`
      collapsed) because neither directory need exist yet.
- [x] 1.4 Unit tests: checkouts resolve under the work location; a work location nested in the
      corpus root is rejected before writing; the default resolves outside the corpus root.
      — 8 tests in `config.rs`, including the exact production shape
      (`/var/lib/cymbra/scores/.checkouts`), equality with the root, `..` escaping, and the
      S3 backend having no local root to nest in.

## 2. Run artefacts out of the corpus root

- [x] 2.1 Write `manifest.json`, `manifest.csv` and `rejected.log` to the work location in
      `crates/score-crawler/src/output.rs`, leaving the servable object layout
      (`<prefix>/<shard>/<uuid>.mxl`) unchanged under the corpus root. — `OutputWriter` now
      takes the work dir as its second argument.
- [x] 2.2 Unit test: after a write, the corpus root contains only servable objects and the
      artefacts are found at the work location. — `corpus_root_holds_only_servable_objects`
      asserts the corpus top level is exactly `["low_confidence", "safe"]`.

## 3. Write-time idempotence (stop the accumulation)

- [x] 3.1 Derive the object key from the entry's content hash (`sha256`) instead of the
      per-run UUIDv7 `id`, so re-crawling unchanged content resolves to the same key. Keep the
      `<prefix>/<shard>/<name>.mxl` shape and the shard derivation intact. **Leave `id` a
      UUIDv7**: it is the catalog PK and `ORDER BY id` keyset pagination reads its ordering.
- [x] 3.2 Update the `crawl.rs` comment that claims the UUID is "the catalog PK AND the
      object-store key", which stops being true.
- [x] 3.3 Unit tests: the same content written twice yields one object and a stable key;
      different content yields different keys; the key no longer contains the row id.
      — `rewriting_identical_content_does_not_add_an_object` writes the same outcome twice and
      asserts the object count is unchanged while the row ids differ.

## 4. Mirror allow-list

- [x] 4.1 Rewrite `backend/deploy/sync-scores.sh` to sync the servable prefixes explicitly
      (`safe/`, `low_confidence/`, `user-scores/`) instead of syncing `$SCORES_DIR` whole, so
      anything else at the corpus root is not mirrored. Keep S3 key == `object_key` and keep
      the absence of `--delete`. — one `aws s3 sync` per prefix onto the same prefix in the
      bucket; the corpus count also stops counting non-servable `.mxl`, and a non-servable
      entry at the corpus root is warned about. Exercised against a fake corpus containing
      `.checkouts/` and `manifest.json`: both warned, neither mirrored.
- [x] 4.2 Update the script header and `DEPLOY.md` to state the allow-list and why it is
      deliberately redundant with section 1.

## 5. Corpus↔catalog reconciliation

- [x] 5.1 Add a `reconcile-corpus` maintenance binary in `backend/server` alongside
      `backfill-titles`, reusing the server config and the `cymbra-storage` object port so it
      reconciles the same keyspace the server reads (local + S3). — logic in
      `cymbra_music::reconcile` (testable), binary is wiring; shipped in the backend image.
      **The port had no `list`**: added `ObjectStorage::list(prefix)` returning the union of
      both backends, implemented for `LocalFirstStore` (via `object_store`'s stream) and
      `FakeStore`. `delete` already removed from origin *and* cache, so one call per key
      covers both sides.
- [x] 5.2 Compute the unreferenced set as corpus objects minus the **set** of `object_key`
      values referenced by any `catalog_scores` row (never per row). — `PgReconcileRepo` reads
      every row with no status or edit filter; an object referenced by any row at all
      survives.
- [x] 5.3 Dry run by default; `--apply` performs removals — matching `backfill-titles` /
      `backfill-mutopia-titles`. `--apply` and `--purge` are mutually exclusive.
- [x] 5.4 Safety guards: abort without writing when the referenced set is empty or when the
      share of objects to remove exceeds a configurable threshold; report why. — threshold
      default 0.75 (above the known ~50% backlog, still catching "the query returned almost
      nothing"), checked only when about to write.
- [x] 5.5 Make removal reversible — move to a quarantine prefix (local and S3) — with a
      separate explicit purge step. — `quarantine/<key>`, outside every servable prefix, so a
      quarantined object immediately stops being mirrored and stops looking servable.
- [x] 5.6 Unit tests: referenced objects are never removed, including one referenced by a
      second row; empty reference set aborts; over-threshold aborts; dry run writes nothing;
      quarantined objects are restorable before purge. — 7 tests in `reconcile.rs`, plus
      "objects outside the corpus prefixes are ignored" and multi-page reference collection.

## 6. Deployment

- [x] 6.1 Add the work-location mount to `backend/deploy/docker-compose.crawler.prod.yml`
      next to the `SCORES_DIR` mount, and document the required ownership. —
      `${CRAWL_WORK:-/opt/cymbra/score-crawler/work}/<source>:/work/crawler` on all five
      services, **per source** so concurrent containers never race on checkouts or overwrite
      each other's manifest export, with `CYMBRA_SCORE_WORK_DIR=/work/crawler` on the shared
      env anchor.
- [x] 6.2 Note in `DEPLOY.md` that `/opt/cymbra/backend` is not a git checkout, so compose and
      script changes must be copied to the box explicitly — the box ran a month-old crawler
      compose during the 2026-08-16 investigation precisely because this step was implicit.
      Also documented: always `pull` before a run, and what the misleading
      "migration N … missing in the resolved migrations" error actually means.

## 7. Validation

- [x] 7.1 `openspec validate fix-crawler-corpus-isolation --strict` passes.
- [x] 7.2 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
      and `cargo llvm-cov --workspace --fail-under-lines 80` pass. — coverage 86.95% lines;
      556 tests green across the touched crates (444 music, 102 crawler, 10 storage).
- [x] 7.3 MANUAL / prod: deploy the compose + script, run a crawl, confirm the corpus root
      gains only servable objects while the work location fills outside it, and confirm a
      re-crawl of unchanged content leaves the object count unchanged. — **done 2026-08-16**
      (`LIMIT=20 … up musetrainer`). Corpus root stayed exactly
      `safe/ low_confidence/ user-scores/`; checkouts, `manifest.json`, `manifest.csv` and
      `rejected.log` all landed under `CRAWL_WORK/musetrainer/`. The run summary now prints
      both resolved roots (`Corpus: /work/output | Work: /work/crawler`).
      Object count 290 778 → 290 798 → **290 798**: the first crawl after the change writes
      each retained item once more under its new content-derived key (a one-time migration
      cost, and one new orphan per item for `reconcile-corpus` to collect), and the second,
      identical crawl adds nothing — which is the idempotence this change exists for.
      Ownership gotcha found and fixed here: the work dir must stay **root**-owned, not
      `1000:1000` — the crawler runs as root and git 2.35.2+ refuses a repo owned by another
      user, so the first attempt died with `prepare failed; skipping` on every source.
- [ ] 7.4 MANUAL / prod — **quarantine done 2026-08-16, purge still pending.** Dry run
      reported 290 799 objects / 145 348 referenced / **145 451 unreferenced** (50.0%, under
      the 0.75 abort threshold), matching the ~145 430 measured before the fix plus the 21
      objects today's test crawls migrated. `--apply` quarantined all of them.
      A second dry run then showed **18 survivors**: the crawler writes into the corpus as
      root, and a `1000` process cannot unlink inside a root-owned shard directory —
      `LocalFirstStore::delete` swallows that local failure, so the run had reported success.
      `chown -R 1000:1000 $SCORES_DIR` plus a second `--apply` cleared them.
      Final state verified three ways: 145 348 objects / 145 348 referenced / **0
      unreferenced**; the catalog holds 145 348 rows with 145 348 **distinct** object keys,
      so the two sets coincide exactly (nothing extra, nothing missing); and 30 randomly
      sampled referenced keys all resolve on disk.
      Remaining: `--purge` after the grace period, once the app has been exercised normally.
      Original wording kept below for reference.
- [ ] 7.4b MANUAL / prod: run `reconcile-corpus` dry, check the count against the ~145 430
      unreferenced objects measured on 2026-08-16 (290 637 files under `safe/` for 145 280
      rows; 141 under `low_confidence/` for 68 rows), then `--apply` to quarantine, then purge
      after a grace period.
- [x] 7.5 MANUAL / prod: after the mirror change, confirm a non-servable entry left at the
      corpus root is not transferred by `sync-scores.sh`. — **done 2026-08-16**. A
      `DO-NOT-MIRROR.txt` placed at the corpus root produced
      `WARNING: DO-NOT-MIRROR.txt is not a servable prefix — not mirrored`, the full mirror
      then ran to completion (`mirrored safe low_confidence user-scores -> s3://cymbra-scores`),
      and the bucket root afterwards listed **only** `safe/`, `low_confidence/` and
      `user-scores/` — the marker never left the box. Marker removed.
      Operational note: `/etc/cymbra/backup.env` is root-only, so a hand-run without `sudo`
      silently reports "SCORES_S3_BUCKET unset — local corpus only" and mirrors nothing; the
      cron runs as root and is unaffected.
