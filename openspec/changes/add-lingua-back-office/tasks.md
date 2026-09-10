# Tasks — add-lingua-back-office

_Prerequisite: `add-lingua-backend` implemented (the `backend/lingua` crate, the `lingua` schema, the `lingua_svc` pool, sync + `StatsService`)._

## 1. Admin protos (backend/lingua/proto)

- [ ] 1.1 Create `backend/lingua/proto/lingua_admin.proto`: service `LinguaAdminService` with `AdminGetLinguaUsage` (window → tiles: active accounts, words learned, reviews, breakdown by studied language), `AdminGetLinguaUsageSeries` (window + dimension → per-day points), `AdminListDataPacks` (→ the pack registry)
- [ ] 1.2 Privacy invariant at the schema level: **no account identifier field** in any response message — review the `.proto` against that criterion + a header comment stating it
- [ ] 1.3 Confirm the `proto` workflow (`buf breaking` gate) does cover `backend/lingua/proto/**` through its `backend/*/proto/**` glob; a new file ⇒ the gate passes

## 2. Backend — authorisation and aggregates

- [ ] 2.1 The `lingua` scope: check that `add-lingua-backend` declared `lingua` in `SCOPES`/`APP_SCOPES` (`backend/platform/src/lib.rs`) and in role assignment (`backend/user/src/module.rs`); otherwise add it here — with the "a `music/admin` does not hold `lingua`" test
- [ ] 2.2 Gating: every `LinguaAdminService` RPC calls `cymbra_platform::guard::require_admin_in_scope(&id, "lingua")`; the service is mounted in `backend/server` behind the **strict** interceptor (the `UsageServiceServer` pattern), inert without `CYMBRA_LINGUA_DATABASE_URL`
- [ ] 2.3 Gating tests (the `backend/analytics/src/grpc.rs` pattern): `lingua/admin` accepted, `global/admin` accepted (break-glass), `music/admin` refused (`PermissionDenied`), a legacy flat token refused
- [ ] 2.4 SQL aggregates over the `lingua` schema (the `lingua_svc` pool): active accounts (synced within the window), words learned per day (dated transitions to `known`), reviews per day (the synced log), breakdown by studied language — queries grouped by day, counts only
- [ ] 2.5 Host-testable split: shaping the aggregates (windows, series, percentages) in a pure, tested module; the thin Postgres adapter follows the coverage exclusion convention (`pg*.rs`)

## 3. Pack registry

- [ ] 3.1 `scripts/lingua-data`: emit a **committed** `packs-manifest.json` (pack_version, analyzer_version, the L2→L1 pair, CI build date, size, NOTICE); a reproducible build ⇒ an empty diff when the pack has not changed
- [ ] 3.2 Backend: embed the manifest (`include_str!`), parse it at startup (build/boot fails if it is invalid), serve it as-is through `AdminListDataPacks`; test against a fixture manifest
- [ ] 3.3 CI: check that the committed manifest matches the built pack (the pack build fails if the manifest is stale)

## 4. Lingua flags (no new UI)

- [ ] 4.1 Declare the Lingua keys in the backend registry (`KeyDef`, app `lingua`, safe defaults, `doc`) — at minimum the sync kill-switch; verify they show up in the existing `/flags` console with no further change
- [ ] 4.2 Back-office i18n: flag descriptions in `flag-descriptions.ts` (en/fr aligned)

## 5. Back office — store, screen, navigation

- [ ] 5.1 `lib/transport.ts`: the `LinguaAdminService` client added to `createClients` (typed in `Clients`), so it is reachable through `api()` only
- [ ] 5.2 `stores/lingua.ts` (Pinia): `Async<T>` resources (`report`, `series`, `packs`), window filters (30 days by default, the `stores/usage.ts` pattern), `load`/`loadPacks` through `run` — no scattered `loading`/`error` refs, errors carried into the union via `humanError`
- [ ] 5.3 `views/LinguaView.vue`: tiles + time series + breakdown by language (reusing `UsageView`'s chart/table components), a read-only pack table (expandable NOTICE), and the "synced accounts" note (the sync-only bias); `match(...).exhaustive()` on every resource; no API call inside the component
- [ ] 5.4 Route `/lingua` with `meta: { admin: true, adminScope: "lingua" }` (the `/takedowns` precedent); the `auth` store's scope type extended with `lingua`; the nav link visible only to admins of the scope
- [ ] 5.5 i18n: en/fr labels aligned; lint/greps for "no UI string contains 'lemma'" (say "words learned", "distinct words")

## 6. Front-end tests

- [ ] 6.1 vitest on `stores/lingua.ts` through `setClientsForTest`: success (tile/series/pack mapping), RPC failure ⇒ a localised `error` inside the union, filters applied to the requests
- [ ] 6.2 e2e seam (`lib/e2e-seam.ts` + `e2e/fixtures.ts`): `LinguaAdminService` fakes + seed data; add a scoped-role `loginAs` if needed (lingua admin vs music-only admin)
- [ ] 6.3 `e2e/lingua.spec.ts` (the `usage.spec.ts` pattern): a lingua admin sees tiles + series + packs; a moderator sees no link and is redirected; a music-only admin is redirected (the `adminScope` gate)

## 7. Gates and finishing

- [ ] 7.1 `cargo fmt --all --check` + `clippy -D warnings` + `cargo llvm-cov --workspace --fail-under-lines 80` (thin adapters under the existing exclusion convention — no new entry unless genuinely needed)
- [ ] 7.2 Back-office `yarn`: vitest green, Playwright green, `back-office-check` passing; `ci-units` unchanged (no new unit) — confirm with `python3 scripts/check_ci_units.py --list`
- [ ] 7.3 Final privacy review: re-read every admin proto response message (no account identifier), every SQL query (aggregates only), the screen (no per-account search)
- [ ] 7.4 Final `openspec validate add-lingua-back-office --strict` + spec updates if implementation moved a contract
