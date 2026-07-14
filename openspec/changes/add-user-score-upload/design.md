## Context

Cymbra is a Rust engine + Flutter app (`apps/music`) with a gRPC (tonic)
backend (`backend/`) fronting Postgres, an object store (S3-compatible, already
in backend config per `backend-service`), Redis, OIDC auth (`backend-auth`,
Google/Apple/local), a modular architecture with **per-module Postgres schemas
and least-privilege roles** (`ops-db-access`, `job-infrastructure`), and a
`user` port/module (`user-account`). The app parses MusicXML through a Rust FFI
seam (`musicxml_core`, exercised via the `score-notation` capability) and renders
an engraved horizontal partition with tempo-driven playback
(`player_notifier`, `midi_service`, `audio-output`). Scores today come only from
a bundled, read-only asset catalog (`score-library`).

We want a signed-in user to contribute their own piano MusicXML score. The file
must be validated on the client (fast feedback, no wasted upload) **and** on the
server (never trust the client), stored durably in the object store with a DB
record attributing it to the user with a date and a chosen difficulty, and be
removable by its owner. The contribution flows through a dedicated three-step
screen: **upload → verify (horizontal, tempo-locked preview) → confirm**.

## Goals / Non-Goals

**Goals:**

- A gated, three-step contribution screen for authenticated users.
- Accept both plain MusicXML (`.musicxml` / `.xml`) and zipped MusicXML
  (`.mxl`); decode the zip to the underlying XML before validation.
- Client-side validation via the existing `musicxml_core` FFI: decodable,
  parseable, contains piano notes; block submit and the CGU gate otherwise.
- A verification preview that reuses the existing engraved partition + playback
  but **restricted**: horizontal layout only, playable **only at the score's own
  tempo** (no tempo slider, no practice/wait modes, no hand isolation controls).
- Mandatory **rights attestation** before upload — the user declares the basis
  (**author** or **public domain / free licence**) and confirms it via a CGU
  checkbox; mandatory **difficulty** (Beginner / Intermediate / Advanced) before
  confirm.
- Server re-validates every upload with the **same** `musicxml_core` logic,
  stores the (canonical) bytes in the object store, and writes a DB record:
  owner, object key, difficulty, rights basis + confirmation, `created_at`.
- Owner-only deletion of the record **and** the stored object.
- The owner's contributed scores are listable and openable in the player,
  reusing the existing score-loading path (bytes instead of asset path).

**Non-Goals:**

- Public sharing, discovery, browsing others' uploads, or moderation — uploads
  are **private to their owner** in this change.
- Editing an uploaded score (delete + re-upload instead).
- Non-piano instruments, MIDI-file import, or PDF/image scores.
- Practice features on the verification preview (only plain playback at score
  tempo).
- Quotas/billing beyond a basic per-file size cap.

## Decisions

### 1. Shared `musicxml_core` validation on both client and server

**Decision:** Validation is one Rust function in the shared MusicXML core
(host-testable, per `CLAUDE.md` coverage rules), returning a structured result
(ok + parsed summary, or a typed rejection reason). Today the parser lives at
`apps/music/rust/src/api/musicxml_core.rs` (pure `quick_xml` SAX, no FFI/IO
deps). **Lift it into a workspace crate** (e.g. `crates/musicxml-core`, already
anticipated by the commented `"crates/*"` glob in the root `Cargo.toml`) so
both `apps/music/rust` (FFI → app-side validation, via the
`notationEngineProvider` seam) and the **already-existing** `backend/score`
module (created by the crawler) depend on the **same** code. Add `.mxl` decoding
there — read `META-INF/container.xml` → rootfile → underlying XML — so both sides
decode identically and validate the same canonical XML. Extend the core with the
"is piano / has playable notes" check.

**Status (done — tasks 1.1–1.4):** the crate exists as `crates/musicxml-core`
(`cymbra-musicxml-core`), with `.mxl` decoding (`mxl.rs`, 32 MiB decompressed
guard) and a `validate(bytes) -> Result<ScoreSummary, RejectReason>` entry point
(`validate.rs`, 16 MiB input cap). `ScoreSummary` already carries `title`,
`composer`, `staves`, `measure_count`, `note_count`; `key_fifths` and the time
signature come from the parsed `ScoreDocument.attributes`. These are exactly the
fields the server pre-fills at upload (see decision 2b).

**Why:** Guarantees the client preview and the server gate agree — a file that
previews cleanly won't be rejected server-side for a different reason.
Alternative (separate XML validators per side) risks client/server drift and
double maintenance.

### 2. Server re-validates; client validation is only UX

**Decision:** The server never trusts the client's "valid" claim. On receiving
the upload it re-runs `musicxml_core` validation on the received bytes before any
storage or DB write; on failure it returns a typed gRPC error and stores nothing.

**Why:** Standard trust boundary — the client is attacker-controllable. Also caps
resource use (size limit enforced server-side too).

### 2b. Server-derived, tamper-proof metadata (client cannot set descriptive fields)

**Decision:** All **descriptive/musical metadata** of an uploaded score — `title`,
`composer`, `key_fifths`, `time_sig`, `measure_count`, `is_piano` (and derived
`title_norm` / `work_key`) — is **extracted server-side from the parsed file** at
upload time, from the same `ScoreSummary` + parsed `ScoreDocument` the shared core
returns (decision 1). The client MUST NOT send or be able to alter these; the
`UploadScore` request carries **only** the user-owned inputs — the file `bytes`,
the rights attestation (`rights_basis` + `rights_ack`), and the chosen `level` —
plus the raw `filename` (used for display/logging, not trusted as metadata). The server re-parses the
received bytes and fills the record from that parse, so the stored metadata is
**guaranteed to match the actual file content** (non-alteration): a client cannot
spoof a title/composer, and the metadata cannot drift from the bytes. This mirrors
exactly how the crawler populates `catalog_scores` (metadata captured at ingest
from the parse, never from an external claim).

The **one** descriptive field the user owns is `level` (difficulty) — recorded
with `level_source = 'manual'` — because it is a subjective judgement, not derivable
from the file.

**Client preview parity (read-before-upload):** because the app parses the file
client-side with the **same** shared core for the verification preview, it already
holds the identical `ScoreSummary`. The Verification/Confirmation steps SHALL
**display these derived fields read-only** (title, composer, key, time signature,
measure count) so the user can review exactly what will be stored **before**
confirming — but cannot edit them. What the user sees == what the server stores,
since both sides run the same core.

**Why:** Trust boundary + provenance integrity. Metadata that indexes and displays
a user's library must reflect the file, not a user-editable free-text field that
could be wrong, misleading, or abused. Deriving it once, server-side, keeps
`user_scores` metadata as trustworthy as the crawler's `catalog_scores`.

**Decided — one shared derivation in `musicxml-core` (Option 1).** The metadata
derivation is lifted **into the shared core**: move `extract()` + `normalize_text()`
from the crawler's `metadata.rs` into `musicxml-core`, and extend `ScoreSummary`
with `key_fifths`, `time_sig`, `is_piano`, `title_norm`, `work_key`. The crawler
then consumes the same function instead of its local copy, so `user_scores` and
`catalog_scores` derive metadata identically — no drift possible — and the new
fields ride the FFI mirror to the app for the read-only preview (task 7.4). (Task
1.5.)

**Shared code ≠ shared trust — the server always re-derives.** Using the same
function on both sides does **not** mean the server ingests the client's computed
values. The client sends **only** the file `bytes` (+ `level` + the rights
attestation `rights_basis`/`rights_ack`); the server **re-parses those bytes and
re-runs the derivation itself**, and that server result is authoritative and
written to the DB. The shared core only guarantees the server's derivation
**equals what the user saw** in the preview — it never lets a client-supplied value
reach storage. Likewise the server independently **validates/normalises every
client input**: `level` and `rights_basis` must be in their fixed sets, `rights_ack`
must be affirmative, `filename` is display-only and untrusted. There is no path by which a client value is stored without the server
re-deriving or re-checking it.

### 3. Upload transport: unary gRPC with size cap (revisit streaming later)

**Decision:** Add a new gRPC service (`ScoreUploadService` or fold into an
existing music-scope service) with `UploadScore(bytes, filename, level,
rights_basis, rights_ack) → score record` (**no client-supplied metadata** — see decision
2b), `ListMyScores() → [record]`, and `DeleteScore(id)`. Send the file as a
bounded `bytes` field with a server-enforced max message size (piano MusicXML is
small — typically well under a few MB).

**Why:** Simplest correct path; matches the existing unary gRPC surface. If very
large `.mxl` files appear later, switch to client-streaming without changing the
DB/storage design. Alternative (pre-signed S3 PUT direct from client) was
rejected: it bypasses server-side validation-before-store and complicates the
"validated then stored" ordering the proposal requires.

### 4. Storage ordering — validate → put object → write DB row

**Decision:** On a valid upload: (1) generate an object key
(`user-scores/{user_id}/{uuid}.mxl`), (2) put the object, (3) insert the DB record
referencing that key inside the module's schema. Deletion reverses it: delete row,
then best-effort delete object (a reconciler / orphan-sweep job can clean any
object left if the process dies between steps).

**Stored format — `.mxl`, matching the crawler.** The crawler stores its corpus as
zipped `.mxl` under `<prefix>/<shard>/<uuid>.mxl` (commit #82: keyed by the score's
UUID v7, sharded by the id's last two hex chars). To keep a **single read/decode
path** for the player and the same object-store port for both producers, user
uploads are **re-zipped to canonical `.mxl`** (the shared core already decodes
`.mxl` on read, task 1.2) rather than stored as raw `.musicxml`. Both producers
therefore key by the immutable UUID and store `.mxl`; only the key **prefix**
differs (`user-scores/{user_id}/…` vs `safe/…` / `low_confidence/…`), and the
`score_asset_source`/player path needs no per-source branching or dual-extension
handling.

**Why:** The DB row is the source of truth for ownership; an object without a row
is invisible and reclaimable. Doing the object put before the row avoids a row
that points at a missing object. Fully-transactional cross-store writes aren't
available (S3 isn't in the PG transaction), so we accept eventual cleanup of
orphaned objects rather than orphaned rows. Consider enqueuing the object delete
as an idempotent job (`job-infrastructure`) so a failed delete is retried.

### 4b. Read path — local-first, S3 fallback ("rebond S3")

**Decision:** The storage port's `get(key)` serves **local disk first, then falls
back to S3 on a miss** (and populates the local copy so the next read is local).
So the read chain is: user's app ← server (local hit) — or, on a local miss,
server ← S3 (lazy pull) ← app. S3 holds the **complete** set of bytes and is the
durable origin; the local folder is a **warm cache/mirror**, safe to wipe or
rebuild.

**Why:** Two producers feed the same read path with different "home" storage:
- the **bulk crawler** (`add-score-crawler`) writes its corpus to the server's
  **local disk** and mirrors it to S3 (`backend/deploy/sync-scores.sh`, nightly
  cron) — so those scores are already local (fast, no S3 call on the hot path);
- **user uploads** are validated then **put to S3** (decision 4) — so those bytes
  are on S3 first and reach the local cache only on first read.

Local-first serves the common case with zero S3 latency/cost, while the S3
fallback means a **rebuilt/empty server** (or an incomplete local mirror) still
serves every score — it re-fetches from S3 on demand. This also keeps the local
disk disposable: losing it costs cache warmth, never data. `object_key` is the
same key in both stores, so the port needs no per-source branching.

**S3 changes role — from "backup" to "read fallback".** Today's crawler deploy
(`backend/deploy/DEPLOY.md` §11, `sync-scores.sh`) treats S3 as a pure off-box
**backup**, never read at runtime: the app serves strictly from the local mirror.
This change makes S3 a **runtime read fallback** (and the durable origin for user
uploads, which land on S3 first), so the storage port (task 2.5) must add the
S3-fetch-on-local-miss path the current pure-local serve does not have.

**Stale crawler docs to reconcile.** `DEPLOY.md` §11 and `sync-scores.sh` still
describe the pre-#82 key shape `object_key = "safe/<source>/<author>/<title>.mxl"`.
Commit #82 changed it to `<prefix>/<shard>/<uuid>.mxl`. Those docs (and the merge
logic in `sync-scores.sh`, which assumes `SCORES_DIR/ + object_key` resolves the
old per-title tree) MUST be updated to the UUID/shard layout as part of this
change's storage work, or the local-first resolution in 4b will not find the bytes.

**Open (for when this change is implemented):** whether the fallback pull is
write-through (cache the object locally on miss) or read-through-only (stream from
S3 without caching); and an optional cap/TTL on the local cache if disk pressure
matters. Not decided now — the bulk corpus is small (`.mxl` ≈ KBs) so caching
everything locally is fine at current scale.

### 5. New module + per-user data isolation

**Decision:** The backend module `score` **already exists** — the crawler created
it (`backend/score/`: `lib.rs` with `pub static MIGRATOR` + `SCHEMA = "score"` +
`connect()`, `repo.rs` with `CatalogRepo`/`FakeCatalogRepo`, `pg.rs` with
`PgCatalogRepo` on the runtime `sqlx::query(...).bind(...)` API, migrations
`0001_catalog.sql` + `0002_fingerprint.sql` for `score.catalog_scores`). Its
`lib.rs` doc-comment already anticipates this change: *"User uploads (user_scores +
a gRPC surface) are added to this same schema by the user-upload change."* So this
change **extends** the module, it does not create it:

- **New migration** `0003_user_scores.sql` (not `0001`) — `0001`/`0002` are taken.
- **New data-access types alongside the catalog ones** — `UserScoreRepo` +
  `FakeUserScoreRepo` in `repo.rs`, `PgUserScoreRepo` in `pg.rs`, following the
  same runtime-`query().bind()` pattern as `PgCatalogRepo`.
- **New gRPC surface** — `score-port/` (proto + `ScorePort` trait + domain
  structs) mirroring `backend/user-port/`, `score/src/module.rs` (logic),
  `score/src/grpc.rs` (tonic adapter reading `AuthIdentity` from request
  extensions). The current module is `anyhow`-based with **no tonic and no
  platform error type** in its `Cargo.toml` (it is written to directly by the
  crawler tool); the gRPC surface adds those deps.
- **Wire it in the composition root** `backend/server/src/main.rs` (own pool, run
  the existing `MIGRATOR`, `add_service`). The module is **not** wired into the
  server yet — only the crawler tool uses it via `connect()` on an admin/ingestion
  role. Migrations must have a **single applier** and share one `_sqlx_migrations`
  ledger in the `score` schema (the crawler already runs `MIGRATOR`); the server
  running the same `MIGRATOR` is idempotent by version.

**Decided — single shared ledger (Option A).** `_sqlx_migrations` lands in the
first schema of the connection's `search_path`. The `score_role` is pinned to
`search_path = score` and owns the schema, so the server's `MIGRATOR.run` records
the ledger at `score._sqlx_migrations` — the established `user`-module pattern.
The crawler today connects with an admin/ingestion role and pins **no**
search_path, so it would create a **second** ledger in `public` and re-apply
0001–0003 from scratch. Fix: **pin the crawler's ingestion connection to
`search_path = score`** (in `cymbra_score::connect()`, or run it as the
schema-owning `score_role`) so both converge on one ledger; the second runner then
applies only new versions (task 3.10). Belt-and-braces, the `0003` DDL is written
idempotently — `CREATE TABLE/INDEX IF NOT EXISTS`, uniqueness as guarded indexes,
fully-qualified names (task 3.3) — so even an accidental double-apply can't
hard-crash.

The module owns `user_scores` in its **own schema** with its own least-privilege
role (`ops-db-access` / module-isolation). Because each module role is confined
to its own schema, the `owner_id` column MUST be a **plain `UUID` (no
cross-schema DB-level FK)** — exactly how the `user` module stores identity.
`owner_id` is set from the caller's `AuthIdentity.user_id` (a UUID v7 in text
form, injected by `backend/platform/src/interceptor.rs`). Ids are UUID v7
generated app-side (`Uuid::now_v7()`), as elsewhere. Every query is scoped to the
authenticated caller's `owner_id`; owner-only deletion compares
`AuthIdentity.user_id` to the row's `owner_id` (mirror
`backend/platform/src/guard.rs`).

**Cross-schema erasure:** deleting a user account must also purge that user's
scores and their objects. This cannot be a DB cascade across schemas — extend the
existing `purge_user` worker job (`backend/worker/src/handlers.rs`, which already
runs under an `admin`/`admin_svc` pool) to also delete `user_scores` rows and
enqueue their object deletions, aligning with `user-account`'s "Delete account"
erasure.

**Why:** Preserves the codebase's module-isolation invariant
(`job-infrastructure`: "Producer can enqueue but not read others' data") and
keeps per-user authorization enforced at the data layer, not just the handler.

### 6. Data model — `score.user_scores`

Lives in the **same `score` schema** as `catalog_scores`, and deliberately mirrors
its column names/types so both tables read alike and could feed a shared query
later. Columns split into three provenance classes:

**User-owned inputs** (the only fields the client controls — decision 2b):
- `id uuid pk` — UUID v7, app-side (`Uuid::now_v7()`), native `UUID` type like
  `catalog_scores.id`.
- `owner_id uuid not null` — the caller's `AuthIdentity.user_id`. **Plain `UUID`,
  no cross-schema FK** (module-role isolation, exactly as `catalog_scores` avoids
  cross-schema FKs). Indexed (every query is scoped by it).
- `level text not null check (level in ('beginner','intermediate','advanced'))` —
  **same column name and vocabulary as `catalog_scores.level`** (not `difficulty`),
  mapping to the app's `PracticeLevel`.
- `level_source text not null default 'manual'` — mirrors `catalog_scores`; user
  uploads are always `'manual'`.
- `rights_basis text not null check (rights_basis in ('own_work','public_domain'))`
  — which basis the user attested: `own_work` (they are the author) or
  `public_domain` (public domain / free licence permitting availability).
- `rights_ack boolean not null` — the confirmation checkbox; must be true to insert.

**Server-derived from the parsed file** (decision 2b — client cannot set/alter):
- `title text`, `composer text` — from `ScoreSummary`.
- `title_norm text`, `work_key text` — normalised, reusing the crawler's derivation.
- `key_fifths integer not null default 0`, `time_sig text not null default ''`,
  `measure_count integer not null default 0`, `is_piano boolean not null default
  false` — parity with `catalog_scores`, read off the parsed `ScoreDocument`.
- `sha256 text not null` — content hash (same name as `catalog_scores.sha256`);
  `UNIQUE (owner_id, sha256)` so a user can't store the same file twice (the random
  `object_key` UUID would otherwise never collide). Not globally unique — two users
  may legitimately upload the same public-domain piece.
- `size_bytes bigint not null default 0` — **`BIGINT`**, matching `catalog_scores`
  (not `int`).

**Storage / lifecycle:**
- `object_key text not null unique` — `user-scores/{owner_id}/{uuid}.mxl`
  (decision 4).
- `created_at timestamptz not null default now()`.
- **Hard delete** (default per Open Questions — matches "librement supprimer"): no
  `deleted_at` tombstone; the row is removed and the object deletion is enqueued as
  an idempotent job.

Note the naming change vs the earlier draft: `user_id`→`owner_id`,
`difficulty`→`level` (+`level_source`), `content_hash`→`sha256`, `int`→`bigint`,
and the added server-derived metadata columns — all for one-to-one parity with the
sibling `catalog_scores` table in the same schema.

### 7. App architecture — Riverpod + reuse of existing seams

**Decision:** A `ScoreUploadNotifier` (`@riverpod`, Freezed state) drives the
three-step wizard state machine (`pickFile → validating → previewing →
confirming → submitting → done/error`). The screen is a `Navigator.push` from the
library (the app has no router — plain `MaterialPageRoute`, see
`apps/music/lib/screens/library_screen.dart`), gated on `SessionAuthenticated`
(`sessionNotifierProvider`, `apps/music/lib/state/session_notifier.dart`). File
picking via a new injectable `filePickerProvider` (adds a `file_picker` dep —
not currently in `pubspec.yaml`; faked in tests). Client validation via the
existing `notationEngineProvider` seam
(`apps/music/lib/services/notation_engine.dart`). Difficulty reuses the existing
`PracticeLevel { beginner, intermediate, advanced }` enum
(`apps/music/lib/state/score_catalog.dart`). The preview reuses the existing
engine + player: render horizontally with the `StaffPainter` (the existing
horizontal-scrolling synchronized view; `PartitionPainter` currently scrolls
vertically and is disabled on phones), and lock playback by driving
`playerProvider` at `speed = 1.0` with the score's own tempo
(`notationToTimedNotes`, `apps/music/lib/state/notation_playback.dart`) while
hiding the speed/Wait/practice controls in the transport bar. Backend calls go
through an injectable `scoreUploadService` provider wrapping a new gRPC stub via
`authedCall` (`apps/music/lib/services/grpc_client.dart`), faked in tests — per
`state-management`/`CLAUDE.md` (deps are providers, no constructor injection, no
`setState`).

**Why:** Matches mandated Riverpod 2 + Freezed patterns and keeps everything
testable without the native lib or a live backend.

### 8. Library integration for owned uploads

**Decision:** Extend `score-library` so, when signed in, the library also lists
the caller's contributed scores (fetched from the backend via `ListMyScores`) as
a distinct section, each openable in the player and carrying a delete action.
`CatalogEntry.assetPath` currently assumes a bundle path
(`apps/music/lib/state/score_catalog.dart`); uploaded scores need a byte-sourced
variant — add a backend-backed source paralleling
`apps/music/lib/services/score_asset_source.dart` that fetches the object bytes
through the backend, feeding the same `SelectedScore` → `NotationProvider` →
player path. Bundled catalog behavior is unchanged.

**Why:** The proposal requires owner deletion and implies the uploads are usable;
the library is the natural home and already groups by practice level. Alternative
(a separate "my uploads" screen only) duplicates listing/navigation — rejected.

## Risks / Trade-offs

- **Client/server validation drift** → Single shared `musicxml_core` validator
  used by both sides; server is authoritative.
- **Orphaned object storage** (process dies mid-write, or object-delete fails
  after row delete) → Row is source of truth; orphaned objects are invisible and
  reclaimed by an idempotent sweep/retry job; never leave a row pointing at a
  missing object.
- **Large / malicious uploads** (zip bombs in `.mxl`, huge XML) → Enforce
  server-side max message + max decompressed size + parse timeouts in
  `musicxml_core`; reject before storage. Decompress with bounded limits.
- **Legal exposure from the rights attestation** → The CGU attestation is
  mandatory: the user declares a basis (author or public domain / free licence)
  and confirms it, and both are persisted (`rights_basis`, `rights_ack`,
  `created_at`, user id) so contribution provenance is auditable — we record the
  **declared basis and the fact** it was confirmed at submission, not a versioned
  copy of the wording (Resolved Decisions). We do not verify the basis technically.
- **PII / privacy** → Object keys are namespaced per user; list/delete/read are
  authorization-scoped to the owner; deleting the account must cascade
  (align with `user-account` "Delete account" erasure).
- **Preview uses the real engine** → Restricting to horizontal + score-tempo is a
  UI gating concern; ensure the restricted mode can't leak practice controls or
  desync from the parsed tempo.

## Migration Plan

- Additive only: new module/schema/table (new Postgres migration), new gRPC
  service + `.proto` (regen client per `CLAUDE.md`), new app screen + providers,
  and a `score-library` extension. No changes to bundled-catalog data or existing
  tables.
- Rollout: ship backend (migration + service) first; the app feature is inert
  until the screen/entry point is enabled. Rollback = disable the app entry point;
  the additive table/service can remain unused.
- Object-store is **greenfield**: there is no S3 client, bucket config, or
  credentials in the backend today (every current "bucket" reference is
  job-dedup logic, not storage). Add an object-store client crate to the
  workspace matching the existing `tls-rustls`/no-OpenSSL stack (e.g.
  `aws-sdk-s3` or the `object_store` crate), new `CYMBRA_SCORE_S3_*` config keys
  (bucket, endpoint, region, credentials) plus the quota keys
  `CYMBRA_SCORE_UPLOAD_QUOTA_MAX` (default 5) and
  `CYMBRA_SCORE_UPLOAD_QUOTA_WINDOW_DAYS` (default 7, decision 9), all loaded via
  the typed `backend/platform/src/config.rs` `Config::from_env`, documented in
  `backend/.env.example` and `backend/docker-compose.yml` with a local
  MinIO/S3-compatible default for dev. Provision the bucket/prefix for user
  scores.

## Resolved Decisions (were open)

- **Delete → hard delete.** No `deleted_at` tombstone: on the user's request the
  row is removed and the object deletion is enqueued as an idempotent job
  (decision 4 / tasks 3.7, 4.1). Matches "librement supprimer" and avoids retaining
  data a user asked to remove.
- **CGU → user declares a rights basis (author OR public domain), records only
  the fact of confirming at submission, no versioning.** The upload step offers two
  bases — **author** (`own_work`) or **public domain / free licence**
  (`public_domain`) — and a confirmation checkbox. We persist `rights_basis` +
  `rights_ack = true` with `created_at` + `owner_id` — proof of the declared basis
  and that it was confirmed at upload time. **No** `cgu_version` column and no
  stored copy of the wording. Checkbox copy is fixed, localised (`app-localization`):
  **FR** "Je certifie être l'auteur de cette partition, ou qu'elle relève du domaine
  public (ou d'une licence libre en autorisant l'usage)" / **EN** "I certify that I
  am the author of this score, or that it is in the public domain (or under a free
  licence permitting its use)". The broader rights/liability clause it refers to
  lives in the site's Terms (CGU) pages (`cymbra-site`), not in this change — it
  covers both uploaded **scores** and uploaded **piano sounds** (soundfonts),
  requires a consent checkbox at **each** file upload, and makes the user solely
  responsible for the copyright and distribution rights of what they upload.
- **Quota → 5 uploads per rolling window, window length configurable.** The backend
  rejects an upload once the caller already has **5** contributed scores created
  within the last **N days** (rolling). **N is configurable** via
  `CYMBRA_SCORE_UPLOAD_QUOTA_WINDOW_DAYS` (default `7` = one week); the max count
  is `CYMBRA_SCORE_UPLOAD_QUOTA_MAX` (default `5`). Enforced server-side before
  storage (`COUNT` of `user_scores` scoped to `owner_id` with
  `created_at >= now() - make_interval(days => N)`). See decision 9.
- **Entry point → in the library.** A contribution button lives in the library
  (where the user already sees their contributed-scores section), gated on
  `SessionAuthenticated`, hidden/disabled when signed out (task 7.2).
- **Backend service → standalone.** A dedicated `ScoreService` (in `score-port/`),
  not methods bolted onto an existing music-scope service — consistent with the
  already-isolated `score` module (own schema, own role).

### 9. Per-user upload quota (rolling window, configurable length)

**Decision:** Before storing an upload the server counts the caller's existing
`user_scores` rows whose `created_at` falls within the last `N` days and rejects
the upload with a typed "quota exceeded" error once that count reaches the max
(default 5 per 7 days). `N` (window length in days) is configurable via
`CYMBRA_SCORE_UPLOAD_QUOTA_WINDOW_DAYS`; the cap via `CYMBRA_SCORE_UPLOAD_QUOTA_MAX`.
Both load through the typed `Config::from_env` (`backend/platform/src/config.rs`)
with the documented defaults, alongside the storage keys (Migration Plan).

**Interaction with hard delete (sub-decision):** because deletes are hard
(no tombstone), the windowed `COUNT` naturally excludes deleted scores — so
deleting a recent upload **frees a slot** within the window. This is acceptable for
the current intent (a light abuse/rate guard at <50-user scale), not a
delete-proof rate limit. If a delete-to-reset bypass ever matters, replace the
`COUNT`-over-rows with an append-only upload log (or a counter not cleared on
delete); explicitly out of scope now.

**Why:** A simple, cheap guard against a compromised/abusive account flooding
uploads, with the window tunable without a code change. A pure per-file size cap
(the earlier default) bounds one file's cost but not the count; a rolling per-user
cap bounds sustained volume.
