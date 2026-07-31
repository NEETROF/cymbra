## 1. Platform: scoped identity + guard (additive, ship first)

- [ ] 1.1 Extend `Claims` in `backend/platform/src/token.rs` to carry roles with their scope (per-scope grouping) while keeping the existing flat `roles`; update `new_claims`/`sign`/verify round-trip
- [ ] 1.2 Extend `AuthIdentity` in `backend/platform/src/identity.rs` with a per-scope role view and `has_role_in_scope(scope, role)` (= `global` roles ∪ `scope` roles); keep `has_role`/`roles` intact
- [ ] 1.3 Add `require_admin_in_scope(id, scope)` to `backend/platform/src/guard.rs` (and a helper to compute the caller's authorized scopes); keep `require_admin`/`require_moderator_or_admin` unchanged
- [ ] 1.4 Unit-test identity + guard: `has_role_in_scope` truth table, `global/admin` passes every scope, `music/admin` fails `live`, non-admin fails all (mockall/hand fixtures per `rust-testing`)

## 2. User + auth: per-scope roles and the back-office audience

- [ ] 2.1 Add `scoped_effective_roles(user_id)` to the user port + module (`backend/user/src/module.rs`) returning roles grouped by scope across `global/music/live`; back it with a repo query (`pg.rs`) and the in-memory fake (`repo.rs`)
- [ ] 2.2 In `backend/auth/src/module.rs` `issue()`, mint a multi-scope token for the `back-office` audience via `scoped_effective_roles`; keep the single-scope path for `music`/`live`; ensure `refresh()` re-resolves roles and preserves the audience
- [ ] 2.3 Add `back-office` to allowed audiences (`backend/platform/src/config.rs` default + `CYMBRA_ALLOWED_AUDIENCES`); confirm `check_audience` accepts it
- [ ] 2.4 Tests: back-office token carries per-scope roles; app-audience tokens unchanged; refresh preserves audience and reflects a role change

## 3. User gRPC: scope-matched grant/revoke + per-scope directory

- [ ] 3.1 Switch `grant_role`/`revoke_role` (`backend/user/src/grpc.rs`) from `require_admin` to `require_admin_in_scope(&id, &r.scope)`
- [ ] 3.2 Change `list_accounts` to compute the caller's authorized scopes and pass them to the read path; deny (`PERMISSION_DENIED`) when the caller is admin in no scope
- [ ] 3.3 Replace the hardcoded `r.scope = 'music'` join in `backend/user/src/pg.rs` `ListAccounts` with `r.scope = ANY($scopes)`, aggregating roles per scope; mirror in the in-memory fake (`repo.rs`)
- [ ] 3.4 Reshape `AccountRow.roles` in `backend/user-port/proto/user.proto` to per-scope (`roles_by_scope`); update port `AccountSummary` types; regenerate protos/bridge
- [ ] 3.5 Tests: `music/admin` grant→`live` denied; `global/admin` grant→`live` allowed; `global/admin` grant→`global` allowed; `music/admin` grant→`global` denied; directory hides `live`/`global` roles for a `music/admin`; `global/admin` sees `global/music/live`; non-admin refused

## 4. Back office (Vue): audience, store, RolesView

- [ ] 4.1 Set `AUDIENCE = "back-office"` in `apps/back-office/src/stores/auth.ts`; derive `isAdmin`/authorized scopes from the per-scope token; keep `/roles` route- and server-guarded on holding `admin` in ≥1 scope
- [ ] 4.2 Make `grant`/`revoke` in `apps/back-office/src/stores/roles.ts` take an explicit required `scope` (drop the `"music"` default); wire the regenerated per-scope `AccountRow` through the catalog/roles store using the `Async<T>` union + injectable client seam
- [ ] 4.3 Update `apps/back-office/src/views/RolesView.vue`: scope selector shown only when >1 scope is authorized (a `global/admin` gets `global`/`music`/`live`); table shows the selected scope's roles per row; grant/revoke target the selected scope, including granting `global` roles; a single-scope admin never sees other scopes
- [ ] 4.4 Update i18n strings (`en.json`/`fr.json`) that pin "music scope"; keep localized error/empty messages (no raw gRPC strings)
- [ ] 4.5 Playwright e2e (fake-client seam): single-scope admin sees only their scope; multi-scope admin switches scope; grant/revoke reflects in the row

## 5. Verify + ship

- [ ] 5.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo llvm-cov --workspace --fail-under-lines 80` green
- [ ] 5.2 `melos run analyze` + `dart run custom_lint` clean; back-office unit/e2e pass; coverage ≥ 80%
- [ ] 5.3 Grep for other consumers of the old flat `AccountRow.roles` / `ListAccounts`; confirm none break
- [ ] 5.4 `openspec validate scope-aware-role-admin --strict` passes
- [ ] 5.5 Deploy note: set `CYMBRA_ALLOWED_AUDIENCES` to include `back-office` (with `music`,`live`) before the back office deploys; roll out backend→config→front in order
