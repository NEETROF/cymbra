# Private Score Catalog v1

## Why

`music.user_scores` is already a private, server-side, per-user score library
(owner-scoped, SHA-256 dedup, OVH private bucket) — but the product on top of it
is a one-file-at-a-time contribution wizard with no organisation. Users who want
their personal repertoire in Cymbra must upload files one by one and get a flat
list. Meanwhile the rights attestation only offers `own_work | public_domain`,
so a user storing a score they legally bought or transcribed **for personal use**
is forced to misdeclare — which weakens the attestation's protective value for
everyone. This change turns the existing storage into an actual *private
catalog*: bulk import, collections, an honest `private_use` rights basis that is
structurally locked out of any sharing, and the minimal notice-and-takedown
surface that makes the hosting-provider (hébergeur) status defensible.

Out of scope, deliberately: the public catalog (unchanged), organisations /
établissements, any web upload surface.

## What Changes

- **Batch import**: the contribution flow accepts multi-selection; files are
  validated and uploaded one by one over the existing single-upload backend
  operation, with a per-file result board (imported / duplicate / invalid) and a
  single attestation step covering the batch. No new backend upload endpoint.
  The rolling upload quota is plan-resolved (`free` = 5 / 7 days today), so a
  batch is bounded by the caller's plan: the flow surfaces the remaining
  allowance up front and reuses the quota refusal's existing upsell signal
  rather than inventing one. This change does **not** alter quota values.
- **Collections**: users can create named collections in their private library,
  assign/remove scores, and filter the library by collection. Collections are
  server-persisted and sync across devices (same pattern as
  `saved-catalog-library`).
- **`private_use` rights basis**: third attestation option, "for my strictly
  personal use". A `private_use` score can never be proposed to the public
  catalog — hidden client-side, **rejected server-side**. Existing
  `own_work`/`public_domain` behavior is unchanged.
- **Minimal takedown**: an admin (music scope) can look up a user's private
  scores and remove one with a mandatory reason, leaving an audit trail — the
  in-repo half of a notice-and-takedown procedure. The intake side (contact
  point, CGU clause for illicit-content reports, CGU wording for the new basis)
  lands in `apps/site`, in the same repo.

Impacted products: **Music** (Flutter app + backend `music` module) and
**back-office** (takedown lookup/removal). No Cymbra ID or Live changes;
platform is consumed as-is (auth, roles).

Note on legacy names: this change touches `score-upload` and
`backend-score-storage`, which would normally trigger their `music-*` rename,
but the in-flight `add-offline-score-cache` change also deltas
`backend-score-storage`; renaming it here would break that change. Renaming only
`score-upload` would split a pair that belongs to the same domain and lands in
the same change, so both renames are deferred together.

## Capabilities

### New Capabilities

- `music-private-collections`: named collections over the private score library —
  create/rename/delete, membership, filtering, cross-device sync.
- `music-batch-score-import`: multi-file selection and sequential batch upload in
  the app with per-file outcomes and one batch-level attestation.
- `music-user-score-takedown`: admin-scoped lookup and reasoned removal of a
  private user score with audit trail (back-office surface + backend RPCs).

### Modified Capabilities

- `score-upload`: attestation gains the `private_use` basis; the contribution
  entry point supports multi-selection; propose-to-catalog affordances are
  absent for `private_use` scores.
- `backend-score-storage`: server accepts and persists the `private_use` basis;
  any proposal path MUST reject a `private_use` score regardless of client
  behavior.
- `score-catalog-proposal`: `ProposeScore` refuses a score whose **stored** basis
  is `private_use`, even when the proposal declares an otherwise-permissive
  licence.

## Impact

- **DB**: `music.user_scores.rights_basis` CHECK gains `'private_use'`; new
  tables for collections + membership; new audit table (or reuse of the existing
  moderation audit pattern) for takedown.
- **Backend** (`backend/music`): upload validation accepts the new basis;
  ProposeScore guard; new collection CRUD + list-filter RPCs; new admin
  lookup/remove RPCs (music admin scope).
- **App** (`apps/music`): contribution wizard batch mode + result board;
  library collections UI (create/assign/filter); attestation copy for
  `private_use`; propose affordance gated by basis. New strings FR/EN.
- **Back-office** (`apps/back-office`): user-score lookup + remove-with-reason
  action, music admin scope.
- **Site** (`apps/site`): CGU/Terms additions in both languages — the personal-use
  basis in Annex A.2 and a new A.3 on reporting illegal content and removal.
- **Interaction with shipped work**: the propose flow (`score-catalog-proposal`)
  is live, and it captures its **own** licence declaration at proposal time —
  which is precisely why the guard reads the stored basis instead of trusting
  the proposal payload. The plan layer (`music-plan-entitlements`) already
  resolves the upload quota per plan and is consumed as-is.
