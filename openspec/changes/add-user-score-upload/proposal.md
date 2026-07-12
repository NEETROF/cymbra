## Why

Today the app can only play scores from the bundled, read-only catalog
(`score-library`). Signed-in users have no way to bring their own piano pieces
into Cymbra. This change lets an authenticated user contribute their own piano
MusicXML score — validated on both the client and the server, attributed to them
with a date, stored durably, and removable by its owner — unlocking a personal,
growing repertoire beyond the shipped catalog.

## What Changes

- A new multi-step **contribution screen** in the app, reached only when signed
  in, with three stages:
  1. **Upload** — pick a `.musicxml`/`.xml` file or a zipped `.mxl`; an
     authorship **CGU checkbox** ("I am the author of this score") MUST be
     checked before the file can be submitted.
  2. **Verification** — the file is decoded and validated locally, then rendered
     as a **horizontal-only** partition that can be played back **only at the
     score's own tempo** (no tempo control, no practice modes) so the user can
     confirm it is correct.
  3. **Confirmation** — the user MUST choose a **difficulty level** (Beginner /
     Intermediate / Advanced) before finalizing the upload.
- **Client-side validation**: reject files that are not decodable piano MusicXML
  (unsupported format, corrupt zip, unparseable, no notes) before any upload.
- **Server-side validation**: the backend re-validates every uploaded file
  (never trusting the client) before persisting it.
- **Durable storage**: validated score bytes are stored in the object store
  (S3), and a database record attributes the score to the uploading user with a
  creation date, chosen difficulty, and the authorship attestation.
- **Owner-only deletion**: a user can freely delete a score they uploaded
  (removing both the DB record and the stored object); they cannot delete
  scores belonging to other users or the bundled catalog.
- The user's own uploaded scores become **listable and playable** from the
  library, alongside the bundled catalog, with a delete affordance.

## Capabilities

### New Capabilities

- `score-upload`: App-side, multi-step contribution flow for a signed-in user —
  file picking (plain + zipped MusicXML), the mandatory authorship CGU gate,
  client-side decode/validation, the horizontal-only tempo-locked verification
  preview, the mandatory difficulty selection, submission to the backend, and
  the owner-facing "my uploads" list with delete.
- `backend-score-storage`: Backend (gRPC) surface that receives an uploaded
  score, re-validates it server-side, persists the bytes to the object store,
  records ownership + upload date + difficulty + authorship attestation in
  Postgres, lists a user's own contributed scores, and enforces owner-only
  deletion of both the record and the stored object.

### Modified Capabilities

- `score-library`: The library additionally surfaces the signed-in user's own
  contributed scores (loaded from the backend/object store rather than the asset
  bundle), each openable in the player and deletable by its owner. The
  bundled-catalog behavior is unchanged.

## Impact

- **App (`apps/music`)**: new contribution screen + Riverpod state, a
  file-picker dependency, reuse of the existing MusicXML decode FFI seam for
  client-side validation and the existing partition/playback widgets (in a
  restricted, tempo-locked, horizontal-only mode), new backend client calls
  (gRPC), and library changes to list/delete owned uploads. Requires the user to
  be authenticated (OIDC) — gated on `user-account` / `account-access`.
- **Backend (`backend/`)**: a new gRPC service + `.proto`, server-side MusicXML
  validation (reusing the Rust `musicxml_core` logic shared with the engine),
  object-store (S3) put/delete, a new Postgres migration for the contributed
  scores table (owner FK, object key, difficulty, authorship flag, timestamps),
  and authorization checks tying every operation to the resolved user identity
  (`backend-auth`).
- **Shared Rust**: `.mxl` (zipped MusicXML) decoding may need to be added to the
  shared `musicxml_core` logic so client and server validate identically.
- **Data / storage**: new object-store bucket/prefix for user scores; new DB
  table. No changes to existing catalog data.
- **Out of scope (assumptions)**: uploaded scores are **private to their
  owner** — no public sharing, discovery, or moderation in this change; no
  quota/rate-limit tuning beyond a basic size cap; no editing of an uploaded
  score (delete + re-upload instead).
