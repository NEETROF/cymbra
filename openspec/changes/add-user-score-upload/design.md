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
- Mandatory authorship **CGU checkbox** before upload; mandatory **difficulty**
  (Beginner / Intermediate / Advanced) before confirm.
- Server re-validates every upload with the **same** `musicxml_core` logic,
  stores the (canonical) bytes in the object store, and writes a DB record:
  owner, object key, difficulty, authorship flag, `created_at`.
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
`notationEngineProvider` seam) and the new `backend/score` module depend on the
**same** code. Add `.mxl` decoding there — read `META-INF/container.xml` →
rootfile → underlying XML — so both sides decode identically and validate the
same canonical XML. Extend the core with the "is piano / has playable notes"
check. (`zip` is not yet a workspace dependency.)

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

### 3. Upload transport: unary gRPC with size cap (revisit streaming later)

**Decision:** Add a new gRPC service (`ScoreUploadService` or fold into an
existing music-scope service) with `UploadScore(bytes, filename, difficulty,
authorship_ack) → score record`, `ListMyScores() → [record]`, and
`DeleteScore(id)`. Send the file as a bounded `bytes` field with a server-enforced
max message size (piano MusicXML is small — typically well under a few MB).

**Why:** Simplest correct path; matches the existing unary gRPC surface. If very
large `.mxl` files appear later, switch to client-streaming without changing the
DB/storage design. Alternative (pre-signed S3 PUT direct from client) was
rejected: it bypasses server-side validation-before-store and complicates the
"validated then stored" ordering the proposal requires.

### 4. Storage ordering — validate → put object → write DB row

**Decision:** On a valid upload: (1) generate an object key
(`user-scores/{user_id}/{uuid}.musicxml`, canonical decoded XML), (2) put the
object, (3) insert the DB record referencing that key inside the module's schema.
Deletion reverses it: delete row, then best-effort delete object (a reconciler /
orphan-sweep job can clean any object left if the process dies between steps).

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

**Open (for when this change is implemented):** whether the fallback pull is
write-through (cache the object locally on miss) or read-through-only (stream from
S3 without caching); and an optional cap/TTL on the local cache if disk pressure
matters. Not decided now — the bulk corpus is small (`.mxl` ≈ KBs) so caching
everything locally is fine at current scale.

### 5. New module + per-user data isolation

**Decision:** A new backend module `score` follows the existing per-module
hexagonal pattern — mirror `backend/user/` + `backend/user-port/`:
`score-port/` (proto + `ScorePort` trait + domain structs), `score/src/module.rs`
(logic), `repo.rs` (`ScoreRepo` trait + `FakeScoreRepo`), `pg.rs` (`PgScoreRepo`,
runtime `sqlx::query(...).bind(...)` API — **not** the compile-time `query!`
macros, matching `backend/user/src/pg.rs`), `grpc.rs` (tonic adapter reading
`AuthIdentity` from request extensions), `lib.rs` (`MIGRATOR`). It is wired in the
composition root `backend/server/src/main.rs` (own pool, own `MIGRATOR`,
`add_service`).

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

### 6. Data model — `user_scores`

Columns (final names in implementation):
- `id uuid pk`
- `user_id uuid not null` (FK/logical ref to account via `user` port)
- `object_key text not null unique`
- `title text` (extracted from MusicXML when present)
- `difficulty text not null check in (beginner, intermediate, advanced)`
- `authorship_ack boolean not null` (must be true to insert)
- `size_bytes int`, `content_hash text` (dedupe / integrity, optional)
- `created_at timestamptz not null default now()`
- `deleted_at timestamptz null` (soft delete) — TBD vs hard delete (see Open
  Questions)

Difficulty reuses the existing `score-library` practice-level vocabulary
(Beginner / Intermediate / Advanced) so uploaded scores slot into the same
grouping.

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
- **Legal exposure from the authorship attestation** → The CGU checkbox is
  mandatory and its acknowledgement is persisted (`authorship_ack`,
  `created_at`, user id) so contribution provenance is auditable. Wording is a
  product/legal input (see Open Questions). We do not verify authorship
  technically.
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
  (bucket, endpoint, region, credentials) loaded via the typed
  `backend/platform/src/config.rs` `Config::from_env`, documented in
  `backend/.env.example` and `backend/docker-compose.yml` with a local
  MinIO/S3-compatible default for dev. Provision the bucket/prefix for user
  scores.

## Open Questions

- **Soft vs hard delete**: keep a `deleted_at` tombstone (auditability, undo) or
  hard-delete row + object immediately? Default proposed: hard delete on the
  user's explicit request (matches "librement supprimer"), with the object
  removed via an idempotent job. Confirm.
- **CGU wording**: exact authorship-attestation text and whether it must be
  versioned (store which CGU version was accepted). Product/legal input needed.
- **Per-user quota / rate limit**: max number of uploads or total bytes per user?
  Default: a basic per-file size cap only, no count quota in this change.
- **Where the entry point lives** in navigation (a button on the library, a
  profile/account action, or both) — UX decision.
- **Backend service placement**: new standalone `ScoreUploadService` vs a method
  set on an existing music-scope service — confirm during proto design.
