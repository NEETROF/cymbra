## 1. Roles & authorization (backend)

- [x] 1.1 Add `moderator` as a recognized role value in the `music` scope (user module); confirm `effective_roles(user, "music")` includes it in the token.
- [x] 1.2 Add a helper/guard `require_moderator_or_admin(identity)` (admin OR music-scoped moderator; `global/admin` passes) in `backend/platform/src/guard.rs`.
- [x] 1.3 Widen #1's privileged status-filter guard and non-`accepted` fetch-bytes guard in `backend/music/src/grpc.rs` from admin-only to `require_moderator_or_admin`.
- [x] 1.4 Add an ops bootstrap for the first `music/admin` (a `backend/scripts` command or documented SQL seed), consistent with the existing provision flow.

## 2. Evaluate & role-grant RPCs (backend)

- [x] 2.1 Add `SetModerationStatus(score_id, status)` to the score proto; regenerate bindings.
- [x] 2.2 Implement it: guard `require_moderator_or_admin`; single UPDATE setting `moderation_status` + `reviewed_by = caller` + `reviewed_at = now()`; reject unknown score; allow `pending`/`accepted`/`rejected`.
- [x] 2.3 Add `GrantRole`/`RevokeRole(user_id, scope, role)` RPCs guarded by `require_admin`; granting `admin` requires the caller be `admin`; idempotent grant.
- [x] 2.5 Add a `role_grants` audit table migration (target user, scope, role, action grant/revoke, acting admin, timestamp; append-only) and write to it on every grant/revoke.
- [x] 2.4 Consume #2's `needs_review` flag (if present) to build the queue ordering; degrade gracefully when absent.
- [x] 2.6 Add an optional structured `sort` to the search proto: a repeated `{ field, direction }` list (ordered = multi-key). Validate each `field` against an allow-list (substance/facet columns + moderation-oriented keys `status_rank`, `needs_review`); reject unknown fields. Gate moderation-oriented keys to `require_moderator_or_admin`. Apply server-side across the whole paginated set. **When `sort` is empty, preserve the current default ordering unchanged so the app hub is unaffected** (add a regression test asserting the hub's order without `sort` is identical).
- [x] 2.7 Close the seam with #2 now that `add-app-score-rating` has shipped (2.4 built the graceful-when-absent ordering; #2 was archived after — this wires the live path so the promise "flagged scores surfaced at the top" is met end-to-end):
  - [x] 2.7a Replace the `needs_review` sort-key no-op with a correlated SQL predicate mirroring `is_flagged_for_review` (score has ≥ `min_count` ratings AND average effective value ≤ `review_threshold`, thresholds from `RatingConfig::default()` so the queue sort and the module's `needs_review` share one source). Update the now-stale "inert until #2" no-op tests/comments.
  - [x] 2.7b Add a privileged `review_queue` mode to `SearchCatalog` (proto + `CatalogQuery`/`CatalogSearchParams`): the work list = `pending` scores PLUS `accepted` scores flagged for re-review, overriding the single-status filter (design D-queue: "queue = pending + accepted flagged needs_review"). Gate it to `require_moderator_or_admin` like the status filter; the single-status WHERE stays for the app hub. Fake mirrors it via the shared ratings view.
  - [x] 2.7c Expose the flag per row: add `needs_review` + `moderation_status` to the `CatalogHit` proto/domain, computed in the search adapter only in a privileged context (gated so the app hub pays nothing). Module test: review-queue returns pending + flagged accepted, flagged first, with `needs_review`/status populated, and the hub search never exposes the flag.

## 3. Browser transport (backend)

- [x] 3.1 Add `tonic-web` wrapping for the gRPC services and a `tower-http` `CorsLayer` restricted to the configured back-office origin(s); keep the native gRPC surface unchanged.
- [x] 3.2 Add config: back-office origin allow-list + a back-office OIDC web client; add its audience mapping to `CYMBRA_ALLOWED_AUDIENCES`/OIDC config (BO targets the `music` audience — design D2).
- [x] 3.3 Verify every gRPC-web-exposed method enforces the same auth + role guards as native gRPC.

## 4. Vue back-office SPA (new web app)

> Built as `apps/back-office/` (Vue 3 + Vite + TS, Connect gRPC-web). Vitest
> (50 tests) + `yarn build` green. CF Pages deploy (4.7) now wired; the wasm
> notation renderer (4.8) stays deferred; 4.5 ships the preview shell + accept/reject.

- [x] 4.1 Scaffold a Vue 3 + Vite SPA (client-rendered, no SSR) as a new package/repo for `bo.cymbra.app`; wire Cymbra OIDC sign-in and a gRPC-web client generated from the protos. (Connect gRPC-web client generated from the protos via `pnpm gen`; local sign-in fully wired + `SignInOidc` exchange; the Google GIS button is a config-gated seam.)
- [x] 4.2 Gate the app to `moderator`/`admin`: access-denied state for signed-in non-moderators; sign-in prompt when unauthenticated.
- [x] 4.3 Build the catalog table: reuse the app hub filters (text/author/level/facets) + the BO-only moderation-status filter; show status per row. (Status is a single-value selector — the server filter is single-status — so rows show the active status.)
- [x] 4.4 Build the queue view: send the default review-priority `sort` list (e.g. `[{needs_review,desc},{status_rank,desc},{measure_count,desc},{staff_count,desc}]`) on every page request; a dedicated re-review filter; clicking a sortable column rebuilds the `sort` list sent to the API and re-queries from page 1 (no client-side sort). Keep the same `sort` across page changes. (Server-side sort only; the re-review filter was inert until #2 supplied `needs_review` data — now wired: see 4.9.)
- [x] 4.9 Wire the queue to the live re-review signal (now that #2 shipped — completes 4.4/2.7): the queue view sends `review_queue: true` so it lists `pending` + community-flagged `accepted` scores, with "Priority order" (`needs_review`-first) genuinely surfacing the flagged re-reviews above the pending backlog (previously a no-op in a pending-only list). The table shows each row's OWN moderation status (mixed queue) plus a "Re-review" badge on flagged rows, using the new `CatalogHit.needs_review`/`moderation_status`. Vitest covers the badge/per-row status and the `review_queue` forwarding.
- [~] 4.5 Row detail: read-only preview rendering the score via the app's Rust notation/render engine compiled to **wasm** (fetch bytes → wasm render), so it matches the app; Accept/Reject actions calling `SetModerationStatus`; show reviewer/time after action. (Preview shell + bytes fetch + Accept/Reject/Re-queue via `SetModerationStatus` DONE; the **wasm notation render** and reviewer/time display are deferred — the latter needs `reviewed_by`/`reviewed_at` on `CatalogHit`.)
- [x] 4.6 Admin-only role management UI calling `GrantRole`/`RevokeRole`; surface the `role_grants` audit history.
- [x] 4.7 Deploy config for `bo.cymbra.app` (reuse the marketing-site/Cloudflare Pages pattern). — CI build + `wrangler pages deploy` in `.github/workflows/back-office-deploy.yml` (build in Actions since the Pages image can't run `yarn gen`/protoc), plus `public/_redirects` (SPA fallback) and `public/_headers` (prod HSTS + `frame-ancestors`). Dormant until the `CF_PAGES_PROJECT` repo var + Cloudflare secrets are set; README "Deploy" documents the setup.
- [ ] 4.8 Build a wasm render module from the app's Rust notation/render core (minimal `bytes → read-only rendered view` entry point); lazy-load it in the console and keep it isolated so a JS-renderer fallback stays possible if the wasm cost is too high. — DEFERRED (isolated seam `ScorePreview.vue` in place)

## 5. Tests & verification

- [x] 5.1 Rust: evaluate writes status + audit and is refused for non-moderator; grant/revoke gated by admin (and admin-grant requires admin); widened guards allow moderator, still block normal callers. `cargo llvm-cov ... --fail-under-lines 80`. (workspace coverage 90.95%)
- [~] 5.2 Rust/integration: gRPC-web method enforces auth/roles; CORS blocks a disallowed origin. (auth/role enforcement is the SAME interceptor + handler guards as native gRPC — covered by unit tests; a live gRPC-web/CORS transport integration test is deferred with the Vue/integration slice.)
- [x] 5.3 Vue app: unit/component tests for the table filters, status filter gating, queue ordering, and the accept/reject flow (its own test setup, outside the Flutter/Rust gates). (Vitest: 19 tests across auth/role gating, catalog status-filter + queue sort + accept/reject, role admin + audit, and the table/filters components.)
- [x] 5.4 `cargo fmt`/`clippy` clean; regenerate proto bindings; `melos run analyze` unaffected (app unchanged).
- [x] 5.5 `openspec validate add-moderation-back-office --strict` passes.

## 6. Rollout

> Console is deployable (§4.7); onboarding + sequencing documented in the
> back-office README. The one remaining deferral in this change is the wasm
> notation renderer (§4.8).

- [x] 6.1 Sequence with #1: ensure the console is deployable and the first admin seeded before/around #1's prod migration, so moderators can work the backlog and the hub is not empty indefinitely. — console is now deployable (4.7); sequencing captured in the README "Moderator onboarding" runbook (seed admin around the catalog-moderation rollout so the queue is populated).
- [x] 6.2 Document the moderator onboarding (seed first admin → admin grants moderators → moderators review the queue). — README "Moderator onboarding" section.
