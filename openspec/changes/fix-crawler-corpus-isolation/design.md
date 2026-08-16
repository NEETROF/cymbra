## Context

The crawler has one notion of "where things go": the store root. `main.rs` derives the
source-checkout directory from it (`root.join(".checkouts")`) and `OutputWriter` writes the
manifest export and rejection log at its top level. That was harmless while the root was a
staging directory merged into the corpus later. `fix-catalog-serving-and-hub-feedback`
removed the staging step — for a good reason, a catalog row must never be visible before its
bytes are servable — and in doing so made the store root *the served corpus*. Everything the
crawler keeps for its own purposes moved in with the objects it produces.

Two invariants were silently lost, and production showed both: the served corpus is no longer
"only servable objects" (4.4 GB of git clones), and the nightly S3 mirror, which is an
unattended billed job, faithfully copies whatever is under `SCORES_DIR`.

Separately, the write path has never been idempotent. Object keys embed a fresh UUIDv7 per
run, so re-crawling identical content writes a new object every time; only the *ingest* step
deduplicates, by content fingerprint, leaving the freshly written bytes unreferenced. After a
few months this is half the corpus (~145 430 objects) — locally and in the bucket.

## Goals / Non-Goals

**Goals:**

- The served corpus root contains servable objects and nothing else, by construction rather
  than by operator discipline.
- The off-box mirror cannot carry non-servable content, even if the corpus root is polluted
  again by some future path.
- Repeated crawls of unchanged content converge instead of growing the corpus.
- Unreferenced objects can be found and removed safely, locally and in S3.

**Non-Goals:**

- Re-keying existing catalog rows. `object_key` is stored per row; existing objects keep
  their current keys and stay servable untouched.
- Changing when a row becomes discoverable — the availability invariant from
  `fix-catalog-serving-and-hub-feedback` stands unchanged and this change must not weaken it.
- Fixing catalog *metadata* quality (the `?` titles seen in production are faithful to their
  source bytes and are a curation matter, not an ingestion one).
- Reworking the moderation/confidence prefixes.

## Decisions

### D1 — An explicit work root, not a derived one

Introduce a work location configured **independently** of the store root
(`CYMBRA_SCORE_WORK_DIR`, mirroring the existing `CYMBRA_SCORE_*` convention), holding the
source checkouts, the manifest export and the rejection log. The crawler deployment mounts it
as its own volume next to the `SCORES_DIR` mount.

The default must be safe on its own, because the production incident came precisely from a
default that silently followed the output root. The default is therefore a sibling path, never
a child of the store root, and the crawler refuses to start if the resolved work root is
inside the store root — a wrong mount then fails loudly at startup instead of quietly filling
the corpus.

*Alternative considered:* keep deriving the path but hide it (a dot-directory) and rely on the
mirror's exclusions. Rejected: it keeps the served corpus impure, and makes correctness depend
entirely on one `--exclude` in a shell script.

### D2 — Content-addressed object keys, catalog id left alone

Today one UUIDv7 serves as both the catalog primary key and the object-store key
(`crawl.rs`: *"A stable UUID v7 identifies the score everywhere: the catalog PK AND the
object-store key"*). Minting it per run is exactly why a re-crawl of unchanged content writes a
second object.

Derive the **object key** from the content hash the run already computes (`sha256` of the
canonical MusicXML — the same value ingest deduplicates on), and **leave the catalog id a
UUIDv7**. Re-crawling unchanged content then resolves to the same key and rewrites the same
object — idempotent by construction, with no lookup.

Decoupling the two is safe because the server resolves bytes exclusively through the row's
stored `object_key` (`music/src/module.rs`), never by rebuilding a path from the id.

*Alternative considered:* make the id itself content-derived, keeping id == object key.
Rejected: the id is the catalog primary key, so a content-derived id changes the PK
derivation for every new row and drops the UUIDv7 time-ordering that `ORDER BY id` keyset
pagination reads (`music/src/pg.rs`) — real blast radius for no gain, since only the *key*
needs to be stable per content.

*Alternative considered:* query the catalog before writing. Rejected: the writer is DB-free
and runs before the DB is even connected, and a pre-write query would make every write depend
on catalog availability while still racing concurrent source containers.

*Note on multiplicity:* identical content never produces two rows — dedup at ingest (SHA-256,
against both the in-run set and existing rows) collapses it to one. So a content-derived key
does not create rows sharing an object; it makes a given content resolve to exactly one row
and exactly one object, however many sources or runs encounter it.

### D3 — Reconciliation as a maintenance bin, dry-run by default

A `reconcile-corpus` binary alongside `backfill-titles` in `backend/server`: it reuses the
server's config and the `cymbra-storage` object port, so it reconciles exactly the keyspace
the server reads (local-first, S3 origin). It lists corpus objects, subtracts the set of
`object_key` values referenced by any catalog row, and reports the remainder. `--apply`
removes them; without it nothing is written, matching `backfill-titles` and
`backfill-mutopia-titles`.

Two safety properties matter more than throughput here, because a correct run deletes about
half the corpus and a wrong one deletes served bytes:

- **Refuse to act on an implausible reference set.** If the referenced-key query returns zero
  rows, or if the proportion of objects to delete exceeds a threshold, the tool aborts rather
  than proceeding — a DB outage or a mis-scoped query must not be able to empty the corpus.
- **Quarantine before purge.** `--apply` moves unreferenced objects under a quarantine prefix
  (local and S3) rather than hard-deleting them, with a separate explicit purge step. The S3
  mirror runs with no `--delete` and the bucket is the durable origin; an irreversible mass
  delete driven by a single query is not a risk worth taking for a one-off cleanup.

### D4 — The mirror allows prefixes rather than excluding them

`sync-scores.sh` syncs the servable prefixes explicitly (`safe/`, `low_confidence/`,
`user-scores/`) instead of syncing `$SCORES_DIR` with exclusions. An allow-list fails closed:
anything new appearing at the corpus root is simply not mirrored, whereas a deny-list only
blocks the cases someone thought of. The script's contract — S3 key equals `object_key` — is
unchanged, since these prefixes *are* the key namespace.

## Risks / Trade-offs

- **Content-addressed keys change the shape of new keys** → Existing rows are untouched and
  keep serving; only newly written objects use the new derivation. The shard directory
  continues to come from the key's identifier, so the on-disk layout and the S3 keyspace are
  structurally identical.
- **Reconciliation still reasons over the referenced-key set, not per row** → Even though a
  content maps to a single row today, set-based reasoning is what makes the tool safe if that
  ever stops holding; it costs nothing and the spec states it.
- **Reconciliation deletes ~145 430 objects on its first real run** → Dry run reviewed first,
  an abort threshold, and quarantine-then-purge instead of direct deletion.
- **The crawler now fails to start on a bad work-dir mount** → Deliberate. The failure mode it
  replaces is silent corpus pollution discovered only by measuring a production disk.
- **Crawls stay paused until this lands** → Any run under the current code recreates
  `.checkouts` inside the served corpus; the operational note is carried in the tasks.

## Migration Plan

1. Land the work-root separation and the compose work-dir mount; deploy the compose file to
   the box (it is copied by hand — `/opt/cymbra/backend` is not a git checkout, which is how
   the box ran a month-old compose during the incident).
2. Deploy `sync-scores.sh` with the prefix allow-list.
3. Resume crawling; confirm the corpus root gains only servable objects and the work dir fills
   outside it.
4. Run `reconcile-corpus` dry, review the count against the ~145 430 measured, then `--apply`
   to quarantine, then purge after a grace period.

Rollback: the work root is configuration, so reverting the compose mount restores the previous
behaviour; the reconciliation quarantine is reversible until purged.

## Open Questions

- Does the scores bucket have versioning enabled? If it does, the quarantine step could be
  simplified to a direct delete; if not, quarantine is required as designed.
- Should the manifest export stay a per-run artifact at the work root, or move to a dated
  location so successive runs do not overwrite each other's export?
