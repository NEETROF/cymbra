## Why

Today a signed-in user's uploaded score lands in their **private** `user_scores`
library and never reaches the public catalog or the moderation review queue — there
is simply no path from a private contribution into the catalog. The recently merged
SoundFont feature (`add-soundfont-moderation`, #168) established the pattern we want
for user-generated content: a private per-user library that is **owner-only and
unmoderated**, plus an **explicit, opt-in "propose to the public catalog"** action
that is the *only* way an item enters moderation. We want piano scores to behave the
same way: a user's score should land in review **only if** the user has deliberately
proposed it to the catalog.

## What Changes

- Add an **opt-in proposal** path for a user's private score: a new owner-scoped
  `ProposeScore` operation on `ScoreService` that takes an existing `user_scores`
  id plus a **licence declaration + right-to-distribute attestation** and creates a
  `pending` public-catalog entry attributed to the proposer. Mirrors soundfont
  `propose`.
- Establish the contract that **uploading a score keeps it private**: it is never
  auto-added to the public catalog and never enters the moderation review queue. The
  review queue is reached **only** through an explicit proposal.
- **Branch the proposed entry's initial status on the proposer's role** (plain user →
  `pending`; music-scope `admin` → `accepted`), never on a client-supplied value —
  mirroring the soundfont upload/propose status branch.
- **Detect duplicate content**: a proposal whose bytes match a non-`rejected` catalog
  entry is refused as a duplicate (reporting the existing id) rather than creating a
  second row; re-proposing an already-`pending`/`accepted` score is refused.
- **Reject with a reason + motivated re-proposal**: rejecting a proposal records a
  moderator **reason** surfaced back to the proposer. Because `catalog_scores.sha256` is
  `UNIQUE` (one row per content), re-proposing a **rejected** score **reopens** that same
  row (→ `pending`, re-attributed) and requires a non-empty **justification** shown to the
  moderator on re-review.
- Record **proposer attribution** on the catalog entry (`proposed_by`) and tag its
  **origin** (`source = 'user-proposal'`) so a user-proposed score is distinguishable
  from a crawler-ingested one.
- **Back office**: the moderation review queue **distinguishes** user-proposed scores
  from crawler-ingested ones and **shows the proposer's pseudo** (display name resolved
  via `UserPort`, exposed only on the privileged moderator/admin read — never to a normal
  caller).
- **Public contributor credit (opt-in)**: an `accepted` user-proposed score MAY carry a
  public "proposé par @pseudo" credit visible to any app user, included **only** when the
  proposer has opted into a **public** profile (fail-closed via `UserPort` visibility);
  the raw `proposed_by` id is never public.

The same attribution model is applied to the already-shipped SoundFont catalog in a
separate change (`add-soundfont-uploader-attribution`), which reuses the `UserPort` seam
this change wires into the music service.
- **App — two entry points**: offer the "Propose to the public catalog" action both (a)
  from the owner's contributions list and (b) as an opt-in step at the end of the upload
  wizard, immediately after a successful upload. Each contributed score surfaces its
  proposal state (not proposed / pending / accepted / rejected), the action is gated on
  the licence declaration + attestation, and it is hidden once submitted — mirroring the
  SoundFont management screen.

## Capabilities

### New Capabilities

- `score-catalog-proposal`: the opt-in path by which a user's private score enters the
  public catalog and its moderation review queue — the `ProposeScore` backend
  operation (licence + attestation, role-based initial status, content dedup,
  re-propose guard, proposer attribution) and the "private-by-default, review only on
  explicit proposal" contract.

### Modified Capabilities

- `score-upload`: the owner's contributions list additionally shows each score's
  proposal status and offers the propose action; reaffirms that an upload itself stays
  private and does not enter the catalog or review.
- `moderation-console`: the evaluate operation records a rejection reason; the review
  queue distinguishes user-proposed scores from crawler-ingested ones, shows the
  proposer's pseudo, and surfaces a reopened score's resubmission justification.

## Impact

- **Proto/API** (`backend/music/proto/score.proto`): new `ProposeScore` RPC on
  `ScoreService`; `ScoreRecord` gains an optional `proposal_status`.
- **Backend** (`backend/music`): new proposal handler in `grpc.rs`; propose logic +
  content dedup + re-propose guard; write a `catalog_scores` row from a `user_scores`
  row (copy bytes into the catalog object store, `source='user-proposal'`,
  `confidence='unverified'`), with proposer attribution. The score/catalog gRPC service
  gains a `UserPort` dependency (as `play_module` already has) to resolve `proposed_by`
  → display name for the privileged moderation read.
- **Proto** (`CatalogHit`): privileged-only `proposed_by` + `proposer_display_name`
  fields (moderator/admin reads only), plus a public `contributor_credit` field
  (populated only under the opt-in condition).
- **Migration** (`backend/music/migrations/0014_*.sql`): `catalog_scores.proposed_by`
  (nullable), `user_scores.proposed_catalog_id` (nullable) to link a private score to
  its catalog entry and drive the app's status + re-propose guard.
- **App** (`apps/music`): `score_upload_service` / contributed-scores state gain a
  `propose(...)` seam + proposal-status field; the contributions UI and the upload
  wizard's final step add the propose action + status tag (mirror of
  `imported_soundfonts` / `soundfonts_screen`).
- **App public credit** (`apps/music`): score cover/detail shows a "proposé par @pseudo"
  credit when the public `contributor_credit` is present.
- **Back office** (`apps/back-office`, Vue): the score review queue shows a user-proposed
  origin badge and the proposer's pseudo.
- **Moderation** (`score-moderation`): unchanged — a proposed `pending` catalog score
  automatically appears in the existing review queue.
- **No change** to the crawler-ingested catalog path or to the private-upload wizard's
  existing validation/attestation.
