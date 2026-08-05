## Context

The SoundFont catalog lives in `music.soundfonts` (change: add-soundfont-catalog-db)
and is served by the delivery route `GET /soundfonts/:id` and the gRPC
`ListSoundFonts`; admin CRUD + upload arrived with add-soundfont-back-office-management
(`POST /soundfonts/:id`, admin/moderator only). Scores already have a
`pending`/`accepted`/`rejected` moderation lifecycle (`score-moderation`,
`moderation-access-control`, `moderation-console`) with server-side read gating and
reviewer attribution — this change mirrors that proven shape onto soundfonts.

The playback engine (`rustysynth 1.3`, in-app and in the wasm renderer) only reads
uncompressed SF2; compressed SF3 is rejected. Fonts are third-party sample data, so
serving one to *another* user is redistribution — the reason for the CC0/CC-BY
curation discipline. Prod is small (<50 users; no admin bootstrapped yet), so the
design favours the simplest correct mechanism and defers scale-only machinery.

## Goals / Non-Goals

**Goals:**
- Public catalog parity with score moderation: `pending`/`accepted`/`rejected`,
  reviewer attribution, server-side accepted-only gate, admin/moderator can audition
  any status.
- Trust-based upload branching: admin→`accepted`, moderator/user→`pending`.
- A private, per-user soundfont library synced across a user's devices, unmoderated
  and owner-only — decoupled from the public catalog so private convenience carries
  no redistribution risk.
- An opt-in path for a user to propose a private font to the public catalog, gated by
  an explicit licence declaration + right-to-distribute attestation.
- Content-identity detection so the same font isn't stored/listed twice.
- Back-office review queue for soundfonts.

**Non-Goals:**
- No self-review (4-eyes) guard in v1 — record `uploaded_by`, but any moderator/admin
  may review any font; defer the guard until moderator count/volume warrant it.
- No perceptual/near-duplicate detection — exact-byte identity only.
- No SF3 support (engine limitation); no paid-font entitlement changes (the existing
  entitlement seam is untouched).
- No content-addressed object sharing/de-dup of bytes in storage (an optimisation);
  v1 detects identity at the metadata layer only.

## Decisions

**1. Mirror score moderation columns rather than invent a new shape.**
Add to `music.soundfonts`: `moderation_status TEXT NOT NULL DEFAULT 'pending' CHECK IN
('pending','accepted','rejected')`, `reviewed_by UUID`, `reviewed_at TIMESTAMPTZ`,
plus `uploaded_by UUID`, and an index on `moderation_status`. Backfill the bundled
`upright-piano-kw`→`accepted` (it ships in-app), everything else→`pending`.
*Alternative considered:* a separate `soundfont_moderation` table — rejected as
needless join complexity; the catalog row is 1:1 with its status, exactly like scores.

**2. Read gate server-side, accepted-only for the public.** `decide()` (delivery) and
`ListSoundFonts` return/serve only `accepted` to a normal caller; a
moderator/admin-authorised path (`AdminListSoundFonts` + admin-scoped delivery) sees
any status so a reviewer can audition. Enforced server-side so an unaware client still
can't reach an unvalidated font (mirrors the score gate).

**3. Upload branches on role, not on a status parameter.** The client never sets the
status; the server derives it: an `admin` identity → `accepted`; any other
authenticated identity (moderator or plain user) → `pending`. Back-office-origin
uploads stay restricted to admin/moderator; app-origin uploads are open to any
authenticated user but target the **private library**, not the public catalog. Origin
is distinguished by token audience (app=`music`, back-office=`back-office`).

**4. Private library is a separate table + storage prefix, keyed by user.** A new
`music.user_soundfonts` (owner `user_id`, label, object_key, `content_sha256`,
`size_bytes`, timestamps) with objects under a per-user prefix in the private bucket.
No moderation, owner-only listing/delivery. This is the "synced across devices" store.
*Alternative considered:* reuse `music.soundfonts` with an `owner_id` + a `private`
flag — rejected because the gating, listing, and lifecycle differ enough that one
table would be a tangle of conditionals; two tables keep each read path simple.

**5. Propose = copy private→public as `pending` with captured licence + attestation.**
Proposing does not move bytes ownership; it creates a public-catalog row (status
`pending`, `uploaded_by` = the proposer) referencing the same content, and records the
mandatory licence string + an explicit right-to-distribute attestation flag/timestamp.
The CGU "I am the author" attestation is insufficient for third-party samples, so the
proposal form captures a distinct licence declaration.

**6. Content identity via exact-byte SHA-256.** Compute `content_sha256` over the
uploaded `.sf2` bytes; store it on both the private and public rows. On upload/propose,
look up the hash first:
- **Private library:** importing bytes already in the user's library returns the
  existing entry (idempotent) instead of a second copy.
- **Public catalog:** a byte-identical font already present is refused as a duplicate
  (HTTP 409 / a typed gRPC error) reporting the existing id, so the catalog never lists
  the same font twice.
This runs *before* the `id`-primary-key check, so identity — not just id — is the real
uniqueness guard. *Alternative considered:* audio-fingerprint/perceptual hashing —
rejected as disproportionate; exact-byte catches the actual "same file re-uploaded"
case and is O(bytes) with no new deps.

**7. `SetSoundFontModerationStatus(id, status)` RPC** stamps `reviewed_by` = caller +
`reviewed_at` = now, authorised for music-scope moderator/admin (reusing the
`moderation-access-control` role checks). Rejected fonts keep their row + object (audit
trail), simply un-served/un-listed — mirroring scores.

## Risks / Trade-offs

- **Redistribution / licensing liability of user contributions** → Public exposure is
  opt-in and always `pending`; the proposal captures an explicit licence + attestation;
  moderators reject anything not clearly libre; private-library fonts are never served
  to anyone but their owner.
- **Duplicate detection misses re-encoded/near-identical fonts** → Accepted trade-off;
  exact-byte SHA-256 catches the common case, and a reviewer catches the rest during
  audition. Documented as a Non-Goal.
- **A rejected font's hash could permanently block a legitimate re-upload** → v1 dedups
  the public catalog against non-`rejected` rows only, so a `rejected` id doesn't lock
  out a later corrected/relicensed submission; an admin still sees the prior rejection
  via `uploaded_by`/audit. (Open question below.)
- **User uploads grow the private OVH bucket** → per-user quota (count + total bytes)
  and the existing 400 MiB per-file cap; reject over quota with a typed error.
- **Backfilling existing catalog rows to `pending` hides them** → Intended (parity with
  scores). Only the bundled default is pre-`accepted`; operators re-validate the rest
  via the back-office queue (the `seed_soundfonts.sh` admin uploads are auto-accepted).
- **BREAKING: app-imported fonts move device-local → server** → One-time client
  migration/upload of any existing local imports; until then treat local + server as a
  union in the picker.

## Migration Plan

1. Ship the additive migration (`music.soundfonts` columns + index; new
   `music.user_soundfonts`; `content_sha256` columns). Additive + reversible at the
   schema level; backfill statuses as above.
2. Deploy backend with the read gate, upload branching, private-library endpoints,
   dedup, and the new RPCs; regenerate proto/bridge/clients.
3. Back-office review queue; then the Flutter private-library + propose flow.
4. Rollback: revert app/back-office first (server tolerates old clients); the migration
   is reversible (drop columns/table) if needed before wide adoption.

## Open Questions

- Public-catalog dedup scope vs `rejected`: dedup against non-rejected only (proposed),
  or against all with an admin override to re-open a rejected hash?
- Per-user private-library quota values (max fonts, max total MiB) — pick defaults.
- Should an `accepted` public font that a user also holds privately be de-listed from
  their private library, or shown in both? (Proposed: show in both; they are distinct
  stores.)
