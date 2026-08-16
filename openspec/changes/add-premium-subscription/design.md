## Context

Cymbra Music already contains the *gates* of a freemium product, each with its own seam:

- **Daily catalog access** (`music-score-daily-access`): `CatalogDailyAccess::caller_kind` asks a
  `SubscriptionSource` trait (`backend/music/src/catalog_daily_access.rs`) that is wired to
  `NoSubscriptions` in `backend/server/src/main.rs`; the proto already carries
  `CatalogAccessState.subscriber` / `upsell`, the Flutter state already parses both, and the
  localized upsell string says "coming soon". Flags `catalog.daily_access.*` (default off).
- **SoundFont bytes** (`soundfont-entitlement`): `soundfont_access::entitlement()` allows
  `free || own import || curation grant || music mod/admin`; the shop's `owned` column is the SQL
  twin of that union; the client mirrors it in `selected_piano.dart`. Migration
  `0016_curation_rewards.sql` reserves `redeemable = false` for "the future temporary-premium
  tier".
- **Private SoundFont library**: `USER_LIBRARY_MAX_FONTS = 5` hardcoded, mod/admin exempt.
- **Feature flags** (`cymbra-feature-flags`): `EvalContext { app, staff }`,
  `RolloutScope ∈ {global, staff_only}` (SQL CHECK), 10-minute identity-scoped snapshot in the
  app; the app never reads roles — it learns capabilities from flags and per-feature response
  fields only.
- **Accounts**: `user_account.users` + `user_roles(scope, role)`, admin guards, `role_grants`
  audit, `purge_user` erasure. No plan/tier column, no expiry on roles, no billing dependency
  anywhere (no `in_app_purchase`, no Stripe/Paddle client).
- **Distribution**: iOS/macOS App Store, Play Store, and **plain archives** for Linux and Windows
  attached to GitHub Releases (`release-build.yml`) — no store, hence no in-app purchase API, on
  desktop.

Constraints set by the product owner: no invoices, addresses or card data ever handled by
Cymbra; no subscription middleware taking a percentage (Apple/Google/MoR fees are accepted as
unavoidable); beta testers get access **without** becoming staff and **without** ever paying;
the beta must not undermine the paid tier.

## Goals / Non-Goals

**Goals:**

- One server-side answer to "effective plan for user U now", identical on every platform.
- Beta ⊂ premium by construction; premium always wins; a paid entitlement is never affected by a
  beta campaign ending; a beta tester can subscribe at any time and lands on premium immediately.
- Every existing gate consumes the plan through its existing seam; no gate is redeclared.
- Purchases on Apple, Google and the web all end in the same ledger row; management happens on
  the provider's portal; the backend stores identifiers only.
- Codes and grants mint free, campaign-bounded access only; redemption happens on the web.
- Ships fully dark; each channel is independently switchable; rollback is a flag flip.

**Non-Goals:**

- Selling premium for points, family sharing, lifetime purchases, regional pricing logic (the
  stores/MoR own pricing), promotional or win-back offers (store-native, later), the offline
  favorites cap (belongs to `add-offline-score-cache`), a Live plan (schema is ready, product is
  not), reader-app / external-link entitlement exceptions on iOS (jurisdiction-specific, later),
  a self-hosted invoicing or tax pipeline (explicitly refused).

## Decisions

### D1 — One new crate `cymbra-plans`, own `plans` schema, `product` column from day one

Entitlements, campaigns/codes and the three billing adapters live in `backend/plans`, modelled
on `cymbra-notifications` / `cymbra-discord`: **pure host-testable cores** (precedence, unlock
sets, provider-state mapping, code lifecycle) + thin adapters (Postgres repos, HTTP webhook
glue). Tables: `plans.plan_entitlements`, `plans.plan_campaigns`, `plans.access_codes`,
`plans.access_code_redemptions`, `plans.billing_events`, all with a `product` column
(`'music'` today) so a Live plan is a new row value, not a rename. `backend/music` consumes the
crate through two small ports: `SubscriptionSource` (already exists) and a new `PlanSource`
(`effective_plan(user_id) -> PlanSnapshot`), so `soundfont_access`, the shop and the library
quota depend on a trait, not on the crate's tables.

*Alternative considered*: a `plan` column on `user_account.users` or a `music/premium` role in
`user_roles`. Roles reuse grant RPCs and token claims for free, but have no expiry, no source, no
provider reference, and would put a commercial concept in the identity model — every gate would
have to special-case "role but expired". Rejected.

### D2 — Plans are ranked, unlock sets are explicit, beta is a configurable *subset*

`Plan ∈ {Free = 0, Beta = 1, Premium = 2}`. A row grants a plan; the **effective plan** is the
highest-ranked row that is active now (`starts_at ≤ now < ends_at`, not revoked). Unlocks are
named keys (`catalog.unlimited`, `soundfonts.library`, `soundfont_library.extended`, …):

- **Premium's set is fixed in code** — it is what the store listing sells; changing it is a
  release, not a config edit.
- **Beta's set is a flag config** `plans.beta.features` (list of keys) whose evaluation is
  `intersect(configured, premium_set)`: beta can never grant an unlock premium does not have.
  Preview features for the beta cohort go through the flag rollout `beta_only` (D6), not through
  the unlock set, so "beta sees unreleased feature X" and "beta gets premium unlock Y" stay
  distinct and both stay under back-office control.
- **`SubscriptionSource::has_active_subscription(u)` ≡ `effective_plan(u).grants(catalog.unlimited)`**
  — the daily-access gate keeps its `Subscriber` vocabulary and never learns about beta; whether
  beta includes unlimited catalog is a config decision, off by default (beta testers exercise the
  quota path too — they are testers).

*Beta ↔ premium precedence rules* (the invariants the specs test):
1. premium row active ⇒ effective plan is premium regardless of beta rows;
2. a beta campaign closing or being revoked changes nothing for a user with an active paid row;
3. a beta user may purchase at any time; the paywall is not hidden for beta, it says "beta until
   X — subscribe to keep it"; after purchase, the beta row is simply outranked (kept for the
   cohort record);
4. codes/grants can mint `beta` or, for comps, `premium` — with an `ends_at` and a `source`
   that is never a store, so a comp is visibly not a sale;
5. an admin `premium` grant with no end (`ends_at = NULL`) is allowed only for the music-admin's
   own testing accounts and is flagged in the console; everything else expires.

### D3 — Ledger semantics: append-only rows, idempotent upserts by `(source, provider_ref)`

Each provider event upserts **one row per subscription** keyed by `(source, provider_ref)`
(Apple `originalTransactionId`, Google `purchaseToken`'s linked subscription id, MoR
`subscription_id`, code `code_id`, admin `grant_id`); the row's `ends_at` and `status` move
forward with events. Webhook payloads are stored in `billing_events` with the provider's event
id as unique key, so a replayed webhook is a no-op and a disputed row can be explained. Nothing is
ever deleted except by account erasure. Grace: a row in the provider's grace/billing-retry state
stays active until `ends_at + plans.grace_days` (flag), then lapses.

### D4 — Codes and campaigns: campaign owns the clock, code is single-use, one per account

`plan_campaigns(id, product, name, plan, ends_at, open, created_by)`; `access_codes(id,
campaign_id, code_hash, issued_by, issued_to_hint, max_uses = 1, uses, revoked_at)`;
`access_code_redemptions(code_id, user_id, redeemed_at)` unique on `(campaign_id, user_id)` so
one account consumes at most one code per campaign. Codes are ≥ 128-bit random, displayed once at
mint time, stored hashed; redemption is rate-limited per user and per IP through the existing
`ratelimit::check`. Redeeming writes an entitlement row `source = code, ends_at = campaign.ends_at`;
**moving the campaign end date moves every entitlement it produced** (the row stores
`campaign_id`, and effective `ends_at` = `COALESCE(row.ends_at_override, campaign.ends_at)`).
Closing a campaign stops new redemptions but does not shorten existing rows unless the operator
also moves the end date. Issuers implement one port (`AccessCodeIssuer::mint(campaign,
issued_by, hint)`); the back-office console and the Discord `/beta` command are two callers.

*Alternative considered*: per-code duration ("30 days from redeem"). Right for marketing later
(the column `ends_at_override` is there), wrong for a beta whose end you will move.

### D5 — Redemption on the web only; store builds carry no code entry

Apple 3.1.1 names "license keys" as a forbidden unlock mechanism; Play's policy is aligned. The
redeem page (`cymbra.app/redeem?code=…`, served by the backend behind the existing web session
cookie, or by the site calling `RedeemAccessCode`) is the only entry. Because the entitlement is
account-bound, it applies on the store builds within one plan refresh. Desktop and web builds may
deep-link the page; store builds must not render a code field or text inviting one.

### D6 — Feature flags gain a `plan` dimension, not a per-user table

`EvalContext { app, staff, plan }`; `RolloutScope` gains `beta_only` and `premium_only`
(`beta_only` reaches beta **and** premium and staff; `premium_only` reaches premium and staff).
The client snapshot key includes the plan so a purchase invalidates the cache. This is the
generic seam that makes every future feature plan-gatable from the flags console; per-user
targeting is still deliberately absent.

### D7 — Apple: StoreKit 2 through `in_app_purchase`, verification server-side, no middleware

App: `in_app_purchase` (official Flutter plugin; iOS + macOS via `in_app_purchase_storekit`)
behind an injectable `StoreClient` seam. On purchase/restore the app sends the **signed
transaction JWS** to `ReportStorePurchase`; the server verifies the JWS chain against Apple's
bundled root certificates, checks bundle id and environment, and upserts the row. Ongoing life
cycle comes from **App Store Server Notifications v2** (`POST /billing/apple/notifications`,
JWS-signed payload, same verifier). A reconciliation job calls the App Store Server API (JWT
signed with the ASC key) for rows nearing `ends_at` without a fresh event. Sandbox and production
are distinguished by environment claim, never by URL guessing.

*Alternative considered*: RevenueCat. Handles all of the above and unifies channels, at ~1% of
revenue and one more third party in the payment truth path. Refused by the owner; the JWS
verification is the only genuinely delicate piece and it is bounded.

### D8 — Google: Play Billing through `in_app_purchase`, RTDN + Play Developer API

Purchase token → `ReportStorePurchase` → server calls
`purchases.subscriptionsv2.get` (service-account JWT) to validate and read state, then
acknowledges the purchase server-side (mandatory within 3 days). RTDN arrives by Pub/Sub push
(`POST /billing/google/rtdn`, OIDC-token-verified push, message id as idempotency key); on each
notification the server re-reads the subscription state rather than trusting the notification
body.

### D9 — Web: one Merchant-of-Record provider, hosted checkout, HMAC webhook

Provider of the Paddle class (Paddle Billing first candidate; Lemon Squeezy equivalent) —
**not Stripe**, because with Stripe Cymbra is the seller of record (invoices, VAT/OSS, disputes).
`CreateWebCheckout` returns a hosted checkout URL carrying `custom_data.user_id`; the desktop app
opens it in the browser and shows "waiting for confirmation… / I've paid — refresh". Webhooks
(`POST /billing/web/webhook`, HMAC-verified, event id idempotency) upsert the row. "Manage
subscription" opens the provider's hosted portal URL fetched at request time. Cymbra stores the
provider customer/subscription ids and nothing else about the buyer.

### D10 — Channel-aware paywall; the plan is a `GetMyPlan` snapshot plus the flags dimension

`GetMyPlan` returns `{ effective_plan, source, ends_at, manage: {kind: apple|google|web|none},
beta_campaign_ends_at?, can_purchase_here: bool }`. `can_purchase_here` is decided **server-side
per audience/platform** so store builds never show a channel they must not: iOS/macOS → Apple,
Android → Google, Linux/Windows/web → web checkout. A user already premium via channel A sees
"managed on A" instead of a second purchase button on channel B (double subscription is the
number-one support ticket). The daily-access `upsell` field reuses the same decision. Refresh:
on app resume, after a purchase/restore, and by the flags poll (plan is part of the snapshot key).

### D11 — Back office: one `Plans` view on the existing role/audit patterns, music-admin only

Account lookup by handle → entitlement rows (source, plan, ends_at, provider ref hidden behind a
copy button), grant/revoke with reason (audited like `role_grants`), campaigns (create, move end
date, open/close), codes (mint N, show once, revoke), redemptions list (cohort export as CSV for
the "thank you" store offer). Guarded by `require_admin_in_scope("music")` — a moderator can
neither see who pays nor grant.

### D12 — Rollout: dark, then beta, then channels one at a time

`plans.enabled` (default off) gates every plan-aware decision back to today's behaviour;
`billing.apple.enabled`, `billing.google.enabled`, `billing.web.enabled` gate the paywall's
purchase button per channel and the webhook routes' acceptance (routes always answer 2xx to
avoid provider retries storms but ignore payloads when disabled — logged). Sequence: land dark →
enable `plans.enabled` with a staff-only beta campaign → open the community beta (Discord `/beta`)
→ Apple sandbox → Apple prod → Google → web.

## Risks / Trade-offs

- **Apple JWS verification done by hand** (no official Rust library) → bundle Apple's root CAs,
  verify the x5c chain + signature + `bundleId` + `environment` in a pure core with fixture JWS
  from Apple's sandbox; a verification failure is a hard reject with a metric, never a fallback to
  "trust the client".
- **Double subscription across channels** → `can_purchase_here` is false when a paid row from
  another source is active; the paywall shows where it is managed; support runbook documents the
  provider-side refund path.
- **Beta undermining premium** → beta set is an intersection with the premium set, default
  excludes `catalog.unlimited`; the beta campaign has an end date from creation; comps are
  visibly non-store; the console flags open-ended grants.
- **A leaked code list** → codes are minted one at a time, hashed at rest, single-use, one per
  account per campaign, rate-limited; a leak costs at most one seat per code until the campaign
  end; whole-campaign revocation exists.
- **Store rejection** → no code entry, no external purchase link, no discount copy in store
  builds; the redeem page is web-only; the "manage" action opens the store's own management for
  store rows.
- **Webhook route is public** → signature/OIDC/HMAC verification before any side effect,
  idempotency by provider event id, always-2xx-when-disabled to avoid retry storms, rate-limited.
- **Provider state drift** (missed webhook) → reconciliation job on rows nearing expiry, and
  `ReportStorePurchase`/restore lets the client re-assert a transaction at any time.
- **Grace and lapse behaviour surprises** → `plans.grace_days` flag, lapse degrades to free
  without deleting anything: favorites, cache, imported fonts stay; re-download of premium fonts
  is refused by `entitlement()` exactly like any locked font.
- **Coverage** → new workspace member; cores fully tested, webhook/HTTP glue in the ignore regex.
- **Legal** → subscription terms and privacy update are part of the change (Impact); no launch of
  a channel before its terms are published.

## Migration Plan

1. Land the crate, migrations (`plans` schema, additive), the flag keys and the CHECK widening on
   `feature_flags.flag_overrides.rollout_scope`; everything off. Behaviour is unchanged.
2. Wire the seams (`SubscriptionSource`, `PlanSource` in `entitlement()`, shop `owned`, library
   quota, `EvalContext.plan`) — all no-ops while `plans.enabled` is off.
3. Ship the back-office console; create a staff campaign; grant the owner's accounts.
4. Ship the app (paywall hidden while `plans.enabled` is off; plan status shows "free").
5. Enable `plans.enabled` → beta campaign open → Discord `/beta` (other change) → observe.
6. Apple sandbox with `billing.apple.enabled` on a TestFlight build; then production; then
   Google; then web after the MoR account and terms are ready.

**Rollback**: `plans.enabled` off restores today's behaviour instantly (gates fall back to
`NoSubscriptions`-equivalent answers, paywall hidden). Channel flags off stop new purchases and
ignore webhooks (logged). Schema is additive; rows are inert when the code is reverted.

## Open Questions

- Which MoR (Paddle Billing vs Lemon Squeezy) — decide on fees, payout currency and the quality of
  their sandbox; the adapter is one module either way.
- Does the beta unlock set include unlimited catalog from day one, or only SoundFonts + preview
  features? Default in this design: **not** included (testers exercise the quota path).
- Prices and periods (monthly/yearly) — a store/MoR configuration, not code; but the
  `plans.premium.periods` shown on the paywall must match, so a config key or a hard list?
- Whether the redeem page lives on the Astro site (calls `RedeemAccessCode` with the web
  session) or is served by the backend (`GET /redeem`), given the site is a separate repo.
- Should the "thank you" for beta testers be a comp (`premium`, source `admin`, N months) or a
  store offer code campaign? Both are possible; the cohort export makes either easy.
