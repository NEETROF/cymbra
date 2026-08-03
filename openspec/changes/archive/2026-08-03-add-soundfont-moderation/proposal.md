## Why

Today the SoundFont catalog is either curated by admins/moderators (auto-published)
or imported by a user as a device-local file. There is no review gate for
contributed fonts and no way for a user's imported fonts to follow them across
devices. Because serving a font to *another* user is redistribution of third-party
sample data, uncontrolled contribution is a licensing liability. This change brings
SoundFonts to parity with score moderation and cleanly separates two needs that are
currently conflated: a user's *private* library (convenience, no redistribution) and
a *public* catalog contribution (redistribution, must be reviewed).

## What Changes

- **Public catalog moderation.** Every `music.soundfonts` row carries a
  `pending`/`accepted`/`rejected` status with reviewer attribution, mirroring score
  moderation. Only `accepted` fonts are visible/served to normal callers; a
  moderator/admin can see and audition any status.
- **Upload branching by trust.** An **admin** upload is auto-`accepted`; a
  **moderator** or **user** upload lands `pending`. Back-office uploads stay
  restricted to admin/moderator; the in-app upload path is open to any authenticated
  user (into their private library, not the public catalog).
- **Uploader recorded.** Each font records its `uploaded_by` (for ownership of a
  private font, audit, and per-user quota). No self-review guard in v1: a moderator
  may review any font — the 4-eyes rule is deferred until contribution volume and
  moderator count warrant it.
- **Private per-user library.** A user importing a `.sf2` in the app uploads it to a
  **private, per-user** store, synced across that user's devices, unmoderated and
  visible only to them. **BREAKING (behaviour):** app-imported fonts move from
  device-local to server-backed private storage.
- **Opt-in proposal to the catalog.** A user can propose one of their private fonts
  to the public catalog; it enters as `pending` with a **mandatory explicit licence
  declaration + right-to-distribute attestation** captured at proposal time (a
  soundfont is third-party samples, so the CGU "I am the author" attestation is not
  sufficient on its own).
- **Back-office review queue.** The Sound fonts screen gains status badges, a
  `pending` queue, accept/reject actions with audition-before-decision, honoring the
  self-review guard.
- **Delivery/listing gate.** The delivery route and `ListSoundFonts` return
  `accepted`-only for normal callers; a new `AdminListSoundFonts` (or extended
  admin listing) exposes all statuses; a new `SetSoundFontModerationStatus` RPC
  stamps the decision.

## Capabilities

### New Capabilities
- `soundfont-moderation`: the `pending`/`accepted`/`rejected` lifecycle on the
  public soundfont catalog — status + reviewer attribution + `uploaded_by`, the
  server-side read gate (accepted-only for the public), upload status-branching by
  role, and the `SetSoundFontModerationStatus` decision RPC.
- `user-soundfont-library`: a private, per-user soundfont store synced across a
  user's devices (unmoderated, owner-only visibility), plus the opt-in
  propose-to-catalog action that captures a licence declaration + attestation and
  submits the font as `pending`.

### Modified Capabilities
- `soundfont-delivery`: the delivery route (`GET /soundfonts/:id`) and the public
  `ListSoundFonts` now enforce accepted-only for normal callers; admin listing
  exposes every status; the upload route (`POST /soundfonts/:id`) accepts any
  authenticated caller with status branching (admin→accepted, else→pending) and
  keeps back-office-origin uploads restricted to admin/moderator.
- `moderation-console`: the back-office moderation surface gains a soundfont review
  queue — status badges, and accept/reject with audition-before-decision.

## Impact

- **DB**: new migration on `music.soundfonts` (`moderation_status`, `reviewed_by`,
  `reviewed_at`, `uploaded_by`, status-check + index; backfill bundled
  `upright-piano-kw`→`accepted`, others→`pending`); new per-user private soundfont
  table + private storage keying.
- **Backend (Rust)**: `FontEntry`/`SoundFontRepo` gain status + `uploaded_by` and
  `set_moderation_status`; delivery `decide()` gains the accepted-only gate;
  upload handler gains role-based status branching and audience/origin restriction;
  private-library repo + endpoints; per-user upload quota/abuse limits.
- **API (proto)**: `ListSoundFonts` gated; `AdminListSoundFonts` (all statuses);
  `SetSoundFontModerationStatus`; private-library list/import/propose RPCs or routes.
  Regenerate the bridge/clients.
- **Back-office (Vue)**: Sound fonts screen review queue + actions.
- **Flutter app**: private-library import→server + cross-device sync; propose flow
  with licence/attestation form; download list already accepted-only via the gated
  `ListSoundFonts`.
- **Ops/cost**: user uploads consume the private OVH bucket — quota + size limits.
- **Interop**: `backend/scripts/seed_soundfonts.sh` uploads with an admin token →
  its fonts remain auto-`accepted`; no change needed.
