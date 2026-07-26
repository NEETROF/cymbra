## 1. Roles & authorization (backend)

- [ ] 1.1 Add `moderator` as a recognized role value in the `music` scope (user module); confirm `effective_roles(user, "music")` includes it in the token.
- [ ] 1.2 Add a helper/guard `require_moderator_or_admin(identity)` (admin OR music-scoped moderator; `global/admin` passes) in `backend/platform/src/guard.rs`.
- [ ] 1.3 Widen #1's privileged status-filter guard and non-`accepted` fetch-bytes guard in `backend/music/src/grpc.rs` from admin-only to `require_moderator_or_admin`.
- [ ] 1.4 Add an ops bootstrap for the first `music/admin` (a `backend/scripts` command or documented SQL seed), consistent with the existing provision flow.

## 2. Evaluate & role-grant RPCs (backend)

- [ ] 2.1 Add `SetModerationStatus(score_id, status)` to the score proto; regenerate bindings.
- [ ] 2.2 Implement it: guard `require_moderator_or_admin`; single UPDATE setting `moderation_status` + `reviewed_by = caller` + `reviewed_at = now()`; reject unknown score; allow `pending`/`accepted`/`rejected`.
- [ ] 2.3 Add `GrantRole`/`RevokeRole(user_id, scope, role)` RPCs guarded by `require_admin`; granting `admin` requires the caller be `admin`; idempotent grant.
- [ ] 2.5 Add a `role_grants` audit table migration (target user, scope, role, action grant/revoke, acting admin, timestamp; append-only) and write to it on every grant/revoke.
- [ ] 2.4 Consume #2's `needs_review` flag (if present) to build the queue ordering; degrade gracefully when absent.
- [ ] 2.6 Add an optional structured `sort` to the search proto: a repeated `{ field, direction }` list (ordered = multi-key). Validate each `field` against an allow-list (substance/facet columns + moderation-oriented keys `status_rank`, `needs_review`); reject unknown fields. Gate moderation-oriented keys to `require_moderator_or_admin`. Apply server-side across the whole paginated set. **When `sort` is empty, preserve the current default ordering unchanged so the app hub is unaffected** (add a regression test asserting the hub's order without `sort` is identical).

## 3. Browser transport (backend)

- [ ] 3.1 Add `tonic-web` wrapping for the gRPC services and a `tower-http` `CorsLayer` restricted to the configured back-office origin(s); keep the native gRPC surface unchanged.
- [ ] 3.2 Add config: back-office origin allow-list + a back-office OIDC web client; add its audience mapping to `CYMBRA_ALLOWED_AUDIENCES`/OIDC config (BO targets the `music` audience — design D2).
- [ ] 3.3 Verify every gRPC-web-exposed method enforces the same auth + role guards as native gRPC.

## 4. Vue back-office SPA (new web app)

- [ ] 4.1 Scaffold a Vue 3 + Vite SPA (client-rendered, no SSR) as a new package/repo for `bo.cymbra.app`; wire Cymbra OIDC sign-in and a gRPC-web client generated from the protos.
- [ ] 4.2 Gate the app to `moderator`/`admin`: access-denied state for signed-in non-moderators; sign-in prompt when unauthenticated.
- [ ] 4.3 Build the catalog table: reuse the app hub filters (text/author/level/facets) + the BO-only moderation-status filter; show status per row.
- [ ] 4.4 Build the queue view: send the default review-priority `sort` list (e.g. `[{needs_review,desc},{status_rank,desc},{measure_count,desc},{staff_count,desc}]`) on every page request; a dedicated re-review filter; clicking a sortable column rebuilds the `sort` list sent to the API and re-queries from page 1 (no client-side sort). Keep the same `sort` across page changes.
- [ ] 4.5 Row detail: read-only preview rendering the score via the app's Rust notation/render engine compiled to **wasm** (fetch bytes → wasm render), so it matches the app; Accept/Reject actions calling `SetModerationStatus`; show reviewer/time after action.
- [ ] 4.6 Admin-only role management UI calling `GrantRole`/`RevokeRole`; surface the `role_grants` audit history.
- [ ] 4.7 Deploy config for `bo.cymbra.app` (reuse the marketing-site/Cloudflare Pages pattern).
- [ ] 4.8 Build a wasm render module from the app's Rust notation/render core (minimal `bytes → read-only rendered view` entry point); lazy-load it in the console and keep it isolated so a JS-renderer fallback stays possible if the wasm cost is too high.

## 5. Tests & verification

- [ ] 5.1 Rust: evaluate writes status + audit and is refused for non-moderator; grant/revoke gated by admin (and admin-grant requires admin); widened guards allow moderator, still block normal callers. `cargo llvm-cov ... --fail-under-lines 80`.
- [ ] 5.2 Rust/integration: gRPC-web method enforces auth/roles; CORS blocks a disallowed origin.
- [ ] 5.3 Vue app: unit/component tests for the table filters, status filter gating, queue ordering, and the accept/reject flow (its own test setup, outside the Flutter/Rust gates).
- [ ] 5.4 `cargo fmt`/`clippy` clean; regenerate proto bindings; `melos run analyze` unaffected (app unchanged).
- [ ] 5.5 `openspec validate add-moderation-back-office --strict` passes.

## 6. Rollout

- [ ] 6.1 Sequence with #1: ensure the console is deployable and the first admin seeded before/around #1's prod migration, so moderators can work the backlog and the hub is not empty indefinitely.
- [ ] 6.2 Document the moderator onboarding (seed first admin → admin grants moderators → moderators review the queue).
