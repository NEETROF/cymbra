# Tasks — add-private-score-catalog

## 1. Database & rights basis

- [x] 1.1 Migration `0031_private_score_catalog.sql` (idempotent): widen the
  `user_scores.rights_basis` CHECK to include `'private_use'`; create
  `music.user_score_collections` + `music.user_score_collection_items` (FKs ON
  DELETE CASCADE, unique `(owner_id, lower(name))`, PK `(collection_id,
  user_score_id)`); create `music.user_score_takedowns` audit table
- [x] 1.2 Accept `private_use` in backend upload validation (basis whitelist) —
  persist unchanged otherwise; unit tests: accepted+persisted, unknown basis
  still rejected, missing confirmation still rejected
- [x] 1.3 Proposal guard in the live `ProposeScore` handler: reject when the
  **stored** row's basis is `private_use`, read from the DB; tests must cover a
  proposal that declares a permissive licence over a `private_use` score (the
  declaration must not win) and confirm no catalog row, no bytes copy, no
  moderation entry is created

## 2. Collections — backend

- [x] 2.1 Repository: create/rename/delete collection (case-insensitive name
  conflict → distinguishable error), add/remove membership (idempotent,
  owner-validated), list collections, list scores filtered by collection —
  owner-scoped everywhere; mockall-tested
- [x] 2.2 gRPC: proto messages + service methods for collection CRUD,
  membership, and filtered listing; auth = app audience, owner from
  AuthIdentity; handler tests (happy path, cross-owner rejected, name conflict)
- [x] 2.3 Regenerate clients (`melos run gen-grpc`) and Rust build green

## 3. Takedown — backend

- [x] 3.1 Admin lookup RPC (music admin scope): paged search by owner id/handle
  and/or title fragment, minimal metadata fields, no bytes; non-admin rejected;
  tests
- [x] 3.2 Admin remove RPC: mandatory non-empty reason; write audit row (admin,
  owner, score id, sha256, title, reason, timestamp) **before** deleting DB row
  then S3 object; tests: audit-first ordering, missing reason rejected, owner
  list no longer returns the score

## 4. App — attestation & gating

- [x] 4.1 Add the `private_use` basis to the attestation step: FR/EN copy
  stating the score stays private forever and can never be proposed; submit
  gating unchanged; widget tests
- [x] 4.2 Hide share/propose affordances on `private_use` rows in the owner's
  score list (basis exposed in list payload if not already); widget tests both
  bases

## 5. App — batch import

- [x] 5.1 Multi-select in the file picker seam; route: 1 file → existing
  wizard, >1 → batch flow
- [x] 5.2 Batch flow (Riverpod notifier + Freezed state): one attestation + one
  difficulty up-front (start blocked until both), then sequential uploads via
  the existing upload service; per-file outcome accumulation
  (imported / duplicate / invalid / quota); failure isolation (continue on
  error); notifier unit tests with mocked upload service
- [x] 5.3 Quota pre-check: resolve the caller's **plan-based** remaining
  allowance, warn when the selection exceeds it before any upload starts, and
  reuse the existing typed quota refusal (which already carries the upsell
  signal) rather than inventing a new one; test the warning path and a batch
  larger than the free allowance
- [x] 5.4 Result board UI with localized, non-technical outcome strings (FR/EN);
  widget tests incl. mixed-outcome batch

## 6. App — collections

- [x] 6.1 Collections service (gRPC seam) + notifier: load, create, rename,
  delete, assign/remove, filter state; mockito-tested (conflict error surfaced
  as localized message)
- [x] 6.2 Library UI: collection filter (all ↔ one collection), create/rename/
  delete flows, add-to-collection action on a score; widget tests for
  filtering and the collision error path

## 7. Back-office — takedown surface

- [x] 7.1 Store (Pinia) with `Async<T>` unions for lookup + removal behind the
  injectable client seam; unit tests
- [x] 7.2 Screen gated to music-scope admins: search form, results table,
  removal dialog with mandatory reason + explicit irreversible confirmation;
  Playwright e2e on the fake-client seam (lookup, refusal without reason,
  successful removal)

## 8. Quality gates & external follow-ups

- [ ] 8.1 `build_runner`, `melos run analyze`, `dart run custom_lint`,
  `cargo fmt --check` + `clippy -D warnings`, coverage ≥ 80 % (Rust + Flutter)
- [ ] 8.2 `openspec validate add-private-score-catalog --strict` passes
- [ ] 8.3 MANUAL (cymbra-site repo): CGU additions — `private_use` wording,
  illicit-content reporting clause, notice-and-takedown contact in the mentions
  légales; verify the app's legal links still resolve
