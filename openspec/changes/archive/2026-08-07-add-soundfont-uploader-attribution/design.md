## Context

`add-soundfont-moderation` (#168, merged) gave the public SoundFont catalog
(`music.soundfonts`) a moderation lifecycle and recorded `uploaded_by` (the contributor's
`user_id`, a plain UUID — no cross-schema FK, matching every `music` table). That id is
persisted but never resolved to a human name and never surfaced.

The sibling change `add-score-catalog-proposal` establishes the attribution model for
user-proposed **scores**: a privileged, unconditional console pseudo (`proposed_by` →
display name) plus an opt-in **public** contributor credit gated on the proposer's
public-profile visibility, both resolved through `cymbra_user_port::UserPort`
(`get_player_profile` returns `{ handle, display_name, visibility }`, fail-closed
`Private`). This change applies the identical model to SoundFonts, reusing that seam.

## Goals / Non-Goals

**Goals:**
- Resolve `uploaded_by` → uploader pseudo on the privileged (moderator/admin) soundfont
  read, so the review queue names the contributor.
- Add an opt-in public contributor credit on an `accepted` public soundfont, gated on the
  uploader's public-profile opt-in (fail-closed), additive to the licence attribution.
- Keep scores and soundfonts on one attribution model / one `UserPort` seam.

**Non-Goals:**
- No SoundFont data-model change (no migration) — `uploaded_by` already exists.
- No change to the licence attribution (sample author), the private soundfont library, the
  moderation lifecycle, or public visibility gating.
- No public exposure of the raw `uploaded_by` id.

## Decisions

### Two fields, two gates, one port

Mirror the score design exactly:
- **`uploader_display_name`** — on the **admin** soundfont listing (privileged,
  moderator/admin only); always the pseudo, for audit/accountability; **unconditional**
  (independent of the uploader's profile visibility); never populated for a normal caller.
- **`contributor_credit`** — on the **public** `ListSoundFonts` / delivery surfaces;
  populated only when the font is `accepted`, has an `uploaded_by`, and that uploader's
  profile is `Public` (via `UserPort::get_player_profile`, **fail-closed** — omit on
  private/unresolvable/handle-less). Never the raw id. This is separate from the licence
  `attribution` field (the sample author), which always shows.

A seeded/bundled font with no `uploaded_by` (e.g. `upright-piano-kw`) yields neither field.

### Reuse the UserPort seam

The soundfont handlers resolve identities through the same `cymbra_user_port::UserPort`
the music service already uses for play/leaderboards, doubled with `MockUserPort` in tests.
If the soundfont gRPC/delivery path does not already hold a `UserPort`, it gains one wired
in `server` (and the HTTP delivery route in `backend/server` gets the same resolver). No
denormalised snapshot on the soundfont row — resolution is at read time, so renames and
visibility changes are reflected.

### Rejection reason + motivated re-proposal (parity with scores)

Mirror `add-score-catalog-proposal`'s moderator↔uploader loop on `music.soundfonts`:
- **`review_reason`** — set when a moderator **rejects** (via `SetSoundFontModerationStatus`,
  extended with an optional reason; a non-`rejected` decision clears it). Surfaced to the
  uploader through the private-library proposal status (the app already resolves a private
  font's proposal status by matching `content_sha256` to a catalog font — that resolution
  now also carries the reason).
- **`resubmission_note`** — the uploader's mandatory justification when reopening a
  `rejected` font (propose route gains the field). Surfaced to the moderator on the
  privileged review-queue read.

**Reopen semantics.** Unlike scores, `soundfonts.content_sha256` is **not** unique, so a
rejected font and a new one *could* coexist. For a consistent, auditable loop we still
**reopen** the matched `rejected` row (status → `pending`, `uploaded_by` re-attributed,
`review_reason` cleared, `resubmission_note` stored) rather than pile up rejected duplicates
— the same behaviour users see for scores. The existing non-`rejected` dedup is unchanged.

## Risks / Trade-offs

- **N id→profile lookups per listing page** → Mitigation: resolve in a single batched
  `UserPort` call per page (dedupe ids); the privileged admin page is already paginated and
  small, and the public listing is short (catalog of grands). Avoid a per-row round-trip.
- **Uploader made their profile public, then private again** → the credit disappears on the
  next read (resolution is live + fail-closed). Intended — the public credit tracks the
  current opt-in, matching the score behaviour.
- **Ordering vs `add-score-catalog-proposal`** → both wire the same `UserPort` into the
  music service. Mitigation: make the wiring idempotent/tolerant so whichever change lands
  first adds it and the other reuses it; neither hard-depends on the other.

## Open Questions

- Should the public soundfont credit also appear on the in-player "current instrument"
  attribution, or only in the picker/catalog list? (Leaning: picker/catalog list in this
  change; in-player is a fast follow if wanted.)
