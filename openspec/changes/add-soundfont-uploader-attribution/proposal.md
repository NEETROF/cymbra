## Why

The public SoundFont catalog already records who contributed each font (`uploaded_by`,
change `add-soundfont-moderation`), but that id is never surfaced: the back-office
soundfont review queue shows no contributor pseudo, and an accepted community font carries
no public "proposé par" credit. The sibling change `add-score-catalog-proposal` introduces
exactly this attribution model for user-proposed scores (console pseudo + opt-in public
credit); this change brings SoundFonts to the **same** model so both content types behave
identically. No new SoundFont data model is required — only resolving the existing
`uploaded_by` through the user directory.

## What Changes

- **Console attribution**: the privileged (music-scope moderator/admin) soundfont listing
  resolves `uploaded_by` to the uploader's **pseudo** (display name) via `UserPort`, so the
  soundfont review queue identifies the contributor by name rather than by raw id.
- **Public contributor credit (opt-in)**: an `accepted` public soundfont carries a public
  "proposé par @pseudo" credit, included **only** when the uploader has opted into a
  **public** profile (fail-closed via `UserPort` visibility). This is distinct from and
  additive to the existing **licence attribution** (the sample author), which is unchanged.
- The raw `uploaded_by` id is **never** exposed on any public path; a seeded/bundled font
  with no `uploaded_by` carries neither pseudo nor credit.
- **Proto**: privileged `uploader_display_name` on the admin soundfont listing; public
  `contributor_credit` on `ProtoSoundFont`.
- **App**: the soundfont picker/catalog entry shows the public contributor credit when
  present.
- **Back office**: the soundfont review queue shows the uploader's pseudo, captures a
  **rejection reason** on reject, and surfaces a reopened font's resubmission justification.
- **Rejection reason + motivated re-proposal** (parity with `add-score-catalog-proposal`):
  rejecting a soundfont records a moderator reason surfaced to the uploader; re-proposing a
  **rejected** font **reopens** that row (→ `pending`, re-attributed) and requires a
  non-empty **justification** shown to the moderator on re-review.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities

- `soundfont-moderation`: the privileged admin/console read resolves `uploaded_by` to the
  uploader's pseudo; an `accepted` public soundfont carries an opt-in public contributor
  credit gated on the uploader's public-profile visibility (fail-closed), separate from the
  font's licence attribution; the evaluate operation records a **rejection reason**; and
  re-proposing a `rejected` font **reopens** its row and requires a **justification**.
- `user-soundfont-library`: the opt-in proposal carries a **resubmission justification**
  when reopening a rejected font.
- `moderation-console`: the back-office soundfont review queue shows the uploader's pseudo,
  captures a rejection reason on reject, and surfaces a reopened font's resubmission note.

## Impact

- **Proto** (`backend/music/proto/score.proto`): `uploader_display_name` on the admin
  soundfont listing message; `contributor_credit` on `ProtoSoundFont`.
- **Backend** (`backend/music` + `backend/server` delivery): the soundfont handlers resolve
  `uploaded_by` via the same `UserPort` seam used by `play_module` — pseudo on the
  admin/console read (unconditional, privileged-only), opt-in credit on the public
  `ListSoundFonts` / delivery route (gated on `Public` visibility, fail-closed). If the
  soundfont gRPC service does not yet hold a `UserPort`, it gains one (mock in tests).
- **App** (`apps/music`): the soundfont picker/catalog surfaces `contributor_credit` when
  present (+ localized strings).
- **Back office** (`apps/back-office`, Vue): the soundfont review queue row shows the
  uploader's pseudo.
- **Migration** (`backend/music/migrations/0015_soundfont_review_feedback.sql`, see
  `migrations-note.md`): `soundfonts.review_reason` + `resubmission_note` (additive; no
  backfill). `uploaded_by` already exists; no change to licence attribution or the private
  library schema.
- **Proto/HTTP**: `SetSoundFontModerationStatus` gains an optional rejection `reason`; the
  private-font propose route gains an optional `resubmission_note`; the private-library
  status surface carries the rejection reason.
- **Relationship**: shares the `UserPort` attribution mechanism with
  `add-score-catalog-proposal`; the two are independent (either can land first — whichever
  wires `UserPort` into the music service first, the other reuses it).
