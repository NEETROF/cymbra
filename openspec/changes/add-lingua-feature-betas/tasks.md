# Tasks

## 1. Campaigns carry their product

- [x] 1.1 Add optional `product` to `CreateCampaignRequest` and `product` to `CampaignMsg` in `backend/plans/proto/plans.proto`. Verify: `buf breaking` against `main` passes (additive only).
- [x] 1.2 Add `product` to `NewCampaign`, validate it (empty → Music; Music or Lingua; Lingua refuses `premium_trial`), and write it in the `INSERT` of `backend/plans/src/pg.rs`. Verify: unit tests for each case, and a pg test that a Lingua campaign round-trips with its product.
- [x] 1.3 Replace `require_admin_in_scope(&id, "music")` in the campaign RPCs (create, enrol by handle, revoke membership, close, reopen, list) by the campaign's product; scope `can_enrol`'s one-trial rule to the campaign's product. Verify: tests that a Music-only admin cannot enrol into a Lingua campaign, a Lingua admin can, and a Music trial is not blocked by a Lingua membership.
- [x] 1.4 Update the module comments in `backend/plans` that describe campaigns as Music-only to say "the campaign's product". Verify: `cargo clippy --workspace --all-targets -- -D warnings`.

## 2. Flags evaluate the app's betas

- [x] 2.1 Add the app parameter to `PlanContextSource::plan_context` (`feature-flags/src/context.rs`), pass it from `GetEffectiveFlags`, and map it to a product in `server/src/flags.rs` (`music`/`lingua`/`live` → product, anything else → no betas). Update `NoPlanContext` and the mocks. Verify: mockall tests in `feature-flags/src/grpc.rs` — a Lingua token gets Lingua betas, a Music token Music betas, a `web` token none.
- [x] 2.2 Replace `campaign_exists(key)` by `campaign_app(key)` and refuse a `beta:<key>` write whose campaign belongs to another app (unless the flag is declared for every app), with its own cause. Verify: tests for accepted, wrong product, missing campaign and directory unreachable.
- [ ] 2.3 Prove Music is unchanged: the existing drums and plan tests pass untouched. Verify: `cargo test --workspace` and `cargo llvm-cov --workspace --fail-under-lines 80`.

## 3. Back office

- [x] 3.1 `CampaignsView.vue`: a product column, and a product selector on creation limited to the products the admin administers; Lingua offers only the `feature` kind. Strings in en/fr. Verify: unit tests and a Playwright e2e through the fake-client seam.
- [x] 3.2 `AccountPlanPanel.vue` and `UserDetailView.vue`: show each membership's product, offer only open campaigns of products the admin administers, and show the panel to an admin of Music or Lingua. Verify: unit tests for a Music-only, a Lingua-only and a both-scopes admin.
- [x] 3.3 `FlagDrawer.vue`: beta options limited to the open campaigns of the flag's app (all for a flag declared for every app), each labelled with its product; the "wrong product" refusal localised distinctly. Verify: unit tests and `yarn lint`, `yarn typecheck`, `yarn test`.

## 4. The extension reads its flags

- [x] 4.1 Add `backend/feature-flags/proto/flags.proto` to `tool/gen_proto.sh` and `flags` to `Clients` in `net/transport.ts`. Verify: `yarn gen:proto` and `yarn typecheck`.
- [x] 4.2 Implement the background flag store: one snapshot `{account, version, values}` under `cymbra-lingua-flags`, fetched on sign-in, start-up, wake and surface open (throttled to 5 minutes outside sign-in) with `known_version`; last-good kept on failure; ignored for another account; deleted on sign-out. Verify: tests with a mocked client for each trigger, the throttle, unchanged, failure, account switch and sign-out.
- [x] 4.3 Expose `flags:get` to surfaces and a small reader that defaults to `false`, re-reading on `chrome.storage.onChanged`. Verify: tests that an absent key, an absent snapshot and a signed-out state read `false`; `yarn check:variants`.
- [x] 4.4 Document the Firefox and Safari extension origins in `backend/.env.example` and `backend/deploy/.env.prod.example` (`CYMBRA_ALLOWED_WEB_ORIGINS`). Verify: a `GetEffectiveFlags` call from a Firefox build against a local backend succeeds. — Found while implementing: no origin is needed, the extension calls from its background with the backend in `host_permissions` (and Firefox's origin is random per install); the env examples now say so.

## 5. Integration

- [x] 5.1 Run `openspec validate add-lingua-feature-betas --strict`. Verify: it passes.
- [x] 5.2 On a local stack: create a Lingua feature campaign `lingua-smoke`, scope a test Lingua flag to it, enrol a **non-staff** account, open the extension. Verify: the flag reads `true` for that account and `false` for a non-enrolled non-staff account; closing the campaign turns it `false` at the next fetch; the Music app's flags are unchanged. — Done on the local stack through the RPCs the back office and the extension call (sign-up, Lingua-admin role, CreateCampaign, SetFlag, EnrolHandle, GetEffectiveFlags with a `lingua` token, CloseCampaign), with the test flag `lingua.sync.enabled` (on by default, so the beta turned it OFF for members): enrolled non-staff → the beta's value, non-enrolled non-staff → the default, closed campaign → the default again; a Music campaign refused for the Lingua flag (`foreign_campaign`), a Lingua trial and a Music campaign refused to a Lingua-only admin. The extension UI itself was not opened.
