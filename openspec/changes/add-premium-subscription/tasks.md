## 1. Crate skeleton + plan core (D1, D2)

- [ ] 1.1 Create `backend/plans` (`cymbra-plans`) as a workspace member: `Cargo.toml` with workspace deps (`async-trait`, `serde`, `thiserror`, `tracing`, `sqlx`, `reqwest`, `jsonwebtoken`), Apache header, `README.md` stating the core/adapter split and the "identifiers only, never billing PII" rule
- [ ] 1.2 Define the pure plan model: `Plan { Free, Beta, Premium }` (ordered), `Unlock` keys, `PREMIUM_UNLOCKS` const set, `beta_unlocks(configured) = configured ∩ PREMIUM_UNLOCKS`, `PlanSnapshot { plan, source, ends_at, campaign_ends_at, grants(unlock) }`
- [ ] 1.3 Implement pure `effective_plan(rows, now, grace_days)`: active-row filter (start/end/override/campaign end/revoked/grace), highest rank wins, premium > beta, no row ⇒ free — unit-test every precedence invariant of D2 (premium outranks beta, campaign end leaves paid rows alone, beta tester purchase, lapse + grace boundary, open-ended admin row)
- [ ] 1.4 Define the ports: `PlanSource::effective_plan(user_id)`, `EntitlementRepo` (upsert by `(source, provider_ref)`, list by user, revoke, purge), `AccessCodeIssuer::mint(campaign, issued_by, hint)`, `AccessCodeRepo`, `BillingEventRepo`; `#[automock]` per the `rust-testing` convention
- [ ] 1.5 Implement `SubscriptionSource` for the plan source (`has_active_subscription ≡ grants(catalog.unlimited)`) and a `plans.enabled`-aware wrapper that answers `Free` for everyone when the kill-switch is off
- [ ] 1.6 Add the HTTP/webhook adapter paths to the `cargo llvm-cov --ignore-filename-regex` list (CI + CLAUDE.md snippet) and keep every core module under test

## 2. Schema + repos (D1, D3, D4)

- [ ] 2.1 Write `backend/plans/migrations/0001_init.sql`: `plans` schema, `plan_entitlements(id, product, user_id, plan, source CHECK, provider_ref, campaign_id NULL, starts_at, ends_at NULL, ends_at_override NULL, status, revoked_at, created_at, updated_at, UNIQUE(source, provider_ref))`, `plan_campaigns(id, product, name, plan, ends_at, open, created_by, created_at)`, `access_codes(id, campaign_id, code_hash UNIQUE, issued_by, issued_to_hint, max_uses, uses, revoked_at, created_at)`, `access_code_redemptions(code_id, user_id, redeemed_at, UNIQUE(campaign_id, user_id))`, `billing_events(provider, event_id, received_at, payload_ref, applied, UNIQUE(provider, event_id))`, indexes on `user_id` and `ends_at`; a dedicated `plans_svc` role like `flags_svc`
- [ ] 2.2 Implement `PgEntitlementRepo` (upsert-forward semantics: never move `ends_at` backwards on a stale event, status transitions, list, revoke, purge) with integration tests including the stale-event and double-upsert paths
- [ ] 2.3 Implement `PgCampaignRepo` + `PgAccessCodeRepo`: create/close/move-end campaign, mint (128-bit random, hash at rest, clear text returned once), redeem in one transaction (code unused + campaign open + no prior redemption for `(campaign, user)` ⇒ mark used + insert redemption + upsert entitlement `source = code`), revoke code / revoke campaign codes; integration-test single-use, one-per-account-per-campaign, closed campaign, revoked code, moved end date reflected in effective end
- [ ] 2.4 Wire the purge: extend the existing account-erasure job to delete the user's rows in all `plans` tables and, for an active `web` row, call the provider cancel first; regression test

## 3. Wire the existing seams (D2, D6)

- [ ] 3.1 Replace `NoSubscriptions` in `backend/server/src/main.rs` with the plan-backed `SubscriptionSource`; keep the `NoSubscriptions` type for tests; assert daily access still ignores plan names
- [ ] 3.2 Extend `soundfont_access::entitlement()` (`backend/music/src/soundfont_access.rs`) with a `plan_unlocks_library: bool` input; supply it in the delivery route from `PlanSource` (false when `plans.enabled` is off); unit-test the new branch and the lapsed-plan refusal
- [ ] 3.3 Extend the reward shop `owned` expression (`backend/music/src/pg_curation_rewards.rs`, listing + redeem lookup) with the plan input; make `RedeemReward` refuse charging when the plan already unlocks the item and refuse `redeemable = false` items with an "included in premium" reason; tests
- [ ] 3.4 Replace `USER_LIBRARY_MAX_FONTS` in `backend/server/src/soundfont.rs` with a per-request `LibraryQuotaSource` (flags `plans.soundfont_library.max_fonts.{free,premium}` chosen by the `soundfont_library.extended` unlock, mod/admin exempt as before); typed error carries `upgrade_raises_limit`; tests for free/premium/exempt/flag-edited
- [ ] 3.5 Feature flags: add `plan: Plan` to `EvalContext` (`backend/feature-flags/src/context.rs`), `RolloutScope::{BetaOnly, PremiumOnly}` + `rollout_reaches` rules (beta_only ⊇ beta, premium, staff; premium_only ⊇ premium, staff), migration widening the `rollout_scope` CHECK, `parse_rollout_opt` in `grpc.rs`; populate `plan` in `GetEffectiveFlags` from `PlanSource` (Free when kill-switch off); unit-test the reach matrix
- [ ] 3.6 Register the new flag/config keys in `backend/feature-flags/src/registry.rs` with descriptions: `plans.enabled` (bool, default false, sensitive), `plans.beta.features` (string list, default `soundfonts.library`), `plans.grace_days` (int, 3), `plans.soundfont_library.max_fonts.free` (5) / `.premium` (50), `billing.apple.enabled`, `billing.google.enabled`, `billing.web.enabled` (bool, false, sensitive)

## 4. gRPC surface: `PlanService` (D10)

- [ ] 4.1 Add `backend/plans/proto/plans.proto`: `GetMyPlan` (returns plan, source, ends_at, campaign_ends_at, manage kind, can_purchase_here, purchase channel), `RedeemAccessCode`, `CreateWebCheckout`, `ReportStorePurchase` (Apple JWS | Google token), `RestorePurchases`; admin: `LookupEntitlements`, `GrantPlan`, `RevokeEntitlement`, `CreateCampaign`, `UpdateCampaign`, `MintCodes`, `RevokeCodes`, `ListRedemptions`, `ExportRedemptions`
- [ ] 4.2 Implement `GetMyPlan`: `can_purchase_here` decided from the token audience/platform hint and the presence of an active paid row from another source; `plans.enabled` off ⇒ `free`, no purchase; tests for beta tester, store subscriber on another platform, kill-switch off
- [ ] 4.3 Implement `RedeemAccessCode` with `ratelimit::check` per user and per address **before** any lookup, neutral refusal for unknown/revoked/used, and the web-only surface (reject when the caller audience is a store build); tests
- [ ] 4.4 Implement the admin RPCs behind `guard::require_admin_in_scope("music")` with audit rows (acting admin, target, action, reason) mirroring `role_grants`; open-ended grant requires an explicit `confirm_open_ended` flag; store/web rows are not revocable; tests for moderator rejection and each mutation's audit
- [ ] 4.5 Regenerate the Dart gRPC stubs (`melos gen-grpc`) and the back-office client (`yarn gen`)

## 5. Apple channel (D7)

- [ ] 5.1 Implement the pure JWS verifier: x5c chain validation against bundled Apple root CAs, ES256 signature, `bundleId`, `environment`, `expiresDate`; fixture-test with Apple sandbox JWS samples (valid, tampered, wrong bundle, expired chain)
- [ ] 5.2 Implement the pure Apple state mapper: `notificationType/subtype` (+ transaction/renewal info) → entitlement transition (renew, grace, billing retry, cancel, refund, revoke, plan change); table-driven tests
- [ ] 5.3 `POST /billing/apple/notifications`: verify → `billing_events` idempotency → map → upsert; `billing.apple.enabled` off ⇒ 200 + logged skip; tests for replay, invalid signature (no side effect), disabled channel
- [ ] 5.4 `ReportStorePurchase` (Apple arm) + `RestorePurchases`: verify each JWS, upsert by `originalTransactionId`; App Store Server API client (JWT signed with the ASC key) for `get_all_subscription_statuses` used by reconciliation; secrets documented in `backend/.env.example`

## 6. Google channel (D8)

- [ ] 6.1 Play Developer API client (service-account JWT): `purchases.subscriptionsv2.get` + `acknowledge`; pure mapper from `subscriptionState`/`linkedPurchaseToken` to entitlement transitions; table-driven tests
- [ ] 6.2 `ReportStorePurchase` (Google arm): validate token via the API, upsert by the subscription's stable id, acknowledge server-side; test the unacknowledged and refunded paths
- [ ] 6.3 `POST /billing/google/rtdn`: verify the Pub/Sub push OIDC token, `messageId` idempotency, re-read state from the API (never trust the notification body), upsert; disabled ⇒ 200 + skip; tests

## 7. Web Merchant-of-Record channel (D9)

- [ ] 7.1 Choose the provider (Paddle Billing vs Lemon Squeezy — record the decision in design.md Open Questions) and implement `WebBillingProvider` port: `create_checkout(user_id) -> url`, `portal_url(customer_id)`, `cancel(subscription_id)`; mock for tests
- [ ] 7.2 `CreateWebCheckout`: only when `billing.web.enabled`, only for non-store audiences, custom data = user id; `POST /billing/web/webhook`: HMAC verification, event-id idempotency, pure mapper (created/updated/past_due/cancelled/refunded → transitions), upsert; tests incl. replay and bad signature
- [ ] 7.3 Reconciliation job (`plans_reconcile`, scheduled via `cymbra_jobs::registry`): for rows ending within N days without an event in the last M days, re-read provider state (Apple statuses, Google get, web get) and upsert; test the "missed renewal repaired" and "refund caught" cases

## 8. App: plan state, paywall, unlock mirrors (D5, D10)

- [ ] 8.1 Add `in_app_purchase` (+ `in_app_purchase_storekit` for iOS/macOS, `in_app_purchase_android`) behind an injectable `StoreClient` seam (provider), with a no-op implementation for Linux/Windows/tests
- [ ] 8.2 Add `PlanService` client + `planProvider` (Riverpod, `@riverpod` + Freezed `PlanSnapshot`), refreshed on resume, after purchase/restore, on explicit request, and included in the flags identity key (`cymbra_flags` snapshot keyed by plan); failed refresh keeps last-known
- [ ] 8.3 Build the paywall screen: benefits list, channel-aware purchase button (Apple/Google via `StoreClient`, desktop opens `CreateWebCheckout` URL through the launcher seam + "I've paid — refresh"), "managed on <channel>" state, restore action on store builds, beta variant ("beta until <date>"); no code field, no external link, no discount copy on store builds; localized errors only
- [ ] 8.4 Add the plan status section in account settings: plan, source, end/renewal date, manage action (store management deep link / web portal URL), campaign detail for beta rows
- [ ] 8.5 Replace the placeholders: daily-access lock sheet upsell CTA (`catalog_unlock_sheet.dart`, `catalogUnlockUpsell` string), SoundFont picker lock ("included in premium" for `redeemable = false`, plan-unlocked fonts selectable — update `selected_piano.dart` mirror), private-library quota refusal upsell
- [ ] 8.6 Desktop deep links: handle `cymbra.app/redeem` and checkout return through the existing loopback/launcher patterns where available; store builds ignore them
- [ ] 8.7 ARB strings for en/fr/es/it (paywall, plan status, beta variant, upsells, restore, errors)
- [ ] 8.8 Widget/notifier tests: paywall per platform (mocked `StoreClient`, mocked plan service), hidden purchase when managed elsewhere, beta variant, restore flow, offline keeps last plan, no code field on store builds

## 9. Back office: `Plans` view (D11)

- [ ] 9.1 Add the `plans` store (Pinia, `Async<T>` unions) over the admin RPCs behind the injectable client seam; role-gate the route to music admins
- [ ] 9.2 Build the view: handle lookup → entitlement rows + effective plan; grant/revoke dialogs with reason (open-ended confirmation); campaigns table (create, move end date, open/close); mint N codes (shown once, download once); revoke; redemptions list + CSV export
- [ ] 9.3 Flags console: rollout dropdown gains `beta_only` / `premium_only` with a plan-scoped marker; new keys visible with descriptions; `plans.enabled` and `billing.*.enabled` marked sensitive
- [ ] 9.4 Component tests + a Playwright e2e on the fake-client seam (lookup, grant, campaign move, mint codes shown once)

## 10. Legal, ops, docs

- [ ] 10.1 Update terms (`docs/legal/`): subscription, renewal, cancellation and refunds handled by the App Store / Google Play / web provider; beta access is free, time-bounded and revocable
- [ ] 10.2 Update privacy policies: provider identifiers stored, no card/address data, erasure behaviour
- [ ] 10.3 Document all secrets and setup in `backend/.env.example` + `backend/plans/README.md`: ASC key + issuer/key ids, Apple root CAs bundle, Google service account + Pub/Sub topic, web provider API key + webhook secret, product ids per channel; runbook: rotate a webhook secret, disable a channel, refund path, cohort export
- [ ] 10.4 Store setup checklist (App Store Connect subscription group + products, Play Console base plans, MoR products) recorded in `apps/music/store/`

## 11. Verification

- [ ] 11.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo llvm-cov --workspace --fail-under-lines 80` green with `cymbra-plans` included
- [ ] 11.2 `melos run analyze`, `dart run custom_lint`, `dart format`, `flutter test --coverage` green
- [ ] 11.3 Back office: `yarn lint`, `yarn test`, e2e green (own `BO_E2E_PORT`)
- [ ] 11.4 `openspec validate add-premium-subscription --strict` passes
- [ ] 11.5 Manual — dark deploy: all flags off ⇒ app identical to before, webhook routes acknowledge and ignore, `GetMyPlan` says free
- [ ] 11.6 Manual — beta: create a staff campaign, mint a code, redeem on the web, verify beta plan on iOS/Android/macOS/Linux/Windows builds of the same account, move the campaign end date and see it reflected, close the campaign
- [ ] 11.7 Manual — precedence: beta tester purchases in Apple sandbox ⇒ premium; campaign closes ⇒ still premium; refund in sandbox ⇒ back to beta/free
- [ ] 11.8 Manual — channels: Apple sandbox (purchase, renew, grace, restore on second device), Google test track (purchase, RTDN, acknowledge), web provider sandbox from a Windows build (checkout, webhook, portal, cancel); confirm no double purchase offered cross-channel
