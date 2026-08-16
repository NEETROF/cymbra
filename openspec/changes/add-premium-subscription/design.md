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
- Plan and beta membership are two independent axes: a beta never *is* a plan; a paid entitlement
  is never affected by anything a beta does; a beta tester can subscribe at any time.
- Betas are targeted and bounded: a premium trial ends N days after each tester's enrolment; a
  feature beta ends the day the operator closes it.
- Every existing gate consumes the plan through its existing seam; no gate is redeclared.
- Purchases on Apple, Google and the web all end in the same ledger row; management happens on
  the provider's portal; the backend stores identifiers only.
- Codes and grants mint free, campaign-bounded access only; redemption happens on the web.
- Ships fully dark; each channel is independently switchable; rollback is a flag flip.

**Non-Goals:**

- Selling premium for points, family sharing, lifetime purchases, regional pricing logic (the
  stores/MoR own pricing), promotional or win-back offers (store-native, later), a Live plan (schema is ready, product is
  not), reader-app / external-link entitlement exceptions on iOS (jurisdiction-specific, later),
  a self-hosted invoicing or tax pipeline (explicitly refused).

## Decisions

### D1 — One new crate `cymbra-plans`, own `plans` schema, `product` column from day one

Entitlements, campaigns/codes and the three billing adapters live in `backend/plans`, modelled
on `cymbra-notifications` / `cymbra-discord`: **pure host-testable cores** (precedence, unlock
sets, provider-state mapping, code lifecycle) + thin adapters (Postgres repos, HTTP webhook
glue). Tables: `plans.plan_entitlements`, `plans.beta_campaigns`, `plans.beta_memberships`,
`plans.access_codes`, `plans.access_code_redemptions`, `plans.billing_events`, all with a
`product` column (`'music'` today) so a Live plan is a new row value, not a rename.
`backend/music` consumes the crate through two small ports: `SubscriptionSource` (already
exists) and a new `PlanSource` (`snapshot(user_id) -> PlanSnapshot { plan, betas }`), so
`soundfont_access`, the shop and the library/upload quotas depend on a trait, not on the
crate's tables.

*Alternative considered*: a `plan` column on `user_account.users` or a `music/premium` role in
`user_roles`. Roles reuse grant RPCs and token claims for free, but have no expiry, no source, no
provider reference, and would put a commercial concept in the identity model — every gate would
have to special-case "role but expired". Rejected.

### D2 — Two axes: plan (`free`/`premium`) and beta memberships; guardrails stay plan-independent

**Plan.** `Plan ∈ {Free, Premium}`. A row grants premium from a source; the **effective plan** is
`Premium` iff at least one row is active now (`starts_at ≤ now < effective_end`, not revoked,
grace included), else `Free`. Unlocks are named keys (`catalog.unlimited`, `soundfonts.library`,
`soundfont_library.extended`, `scores.extended_quotas`, …) and **premium's set is fixed in code**
— it is what the store listing sells; changing it is a release, not a config edit. Consumers ask
`grants(unlock)`, never the plan name. `SubscriptionSource::has_active_subscription(u)` ≡
`effective_plan(u).grants(catalog.unlimited)` — the daily-access gate keeps its `Subscriber`
vocabulary.

**Beta membership.** An account belongs to zero or more **campaigns**. A campaign has a `kind`:

- `premium_trial` — "beta d'usage premium": enrolment creates a membership **and** a
  `premium` entitlement row (`source = code` or `admin`, `campaign_id` set) whose `ends_at` is
  `enrolled_at + duration_days` (campaign field, default **90**, fixed at creation). Closing
  enrolment stops new members; nobody is shortened. At most **one active premium trial per
  account** (a second trial campaign refuses an account whose trial is still running).
- `feature` — "beta par fonctionnalité" (`midi-drums`, …): enrolment creates a membership only,
  with **no end date**; feature flags rolled out with scope `beta:<campaign_key>` reach its active
  members (D6). The operator **closes** the campaign when the feature is stable → every membership
  ends at `closed_at`, early access stops for everyone at once. A feature beta may run for months.

Membership is active iff the campaign is not closed, `ends_at` is null or in the future, and the
row is not revoked. Memberships are the **cohort record**: who joined which beta and when — the
list to announce to, thank, and export.

**Invariants (what the specs test):**
1. a paid row is never created, shortened or ended by any campaign operation;
2. a premium-trial tester who purchases has two premium rows; the later `ends_at` governs; the
   trial ending changes nothing for them;
3. a member of a feature beta keeps early access whether free, trialling or paying — and a paying
   user outside it never sees the unfinished feature (they pay for stability);
4. codes/grants mint **free** access only (trial or membership); a comp is a `premium` row with a
   non-store `source` and an `ends_at`, visibly not a sale;
5. an admin `premium` grant with no end (`ends_at = NULL`) is allowed only for the music-admin's
   own testing accounts and is flagged in the console;
6. **security guardrails ignore both axes**: `catalog-access-limits` (download burst,
   engagement-aware volume, enumeration cap) and the auth throttles apply to free, trial and
   premium alike; a plan changes *product* limits (quota, library size, fonts), never *security*
   ceilings.

*Alternative considered*: a ranked `beta` plan between free and premium with a configurable
unlock subset. It forces precedence rules ("premium beats beta", "beta ⊂ premium") that only exist
because beta was a rank, and it cannot express "in the MIDI-drums beta but on the free plan".
Rejected in favour of the two axes.

### D3 — Ledger semantics: append-only rows, idempotent upserts by `(source, provider_ref)`

Each provider event upserts **one row per subscription** keyed by `(source, provider_ref)`
(Apple `originalTransactionId`, Google `purchaseToken`'s linked subscription id, MoR
`subscription_id`, code `code_id`, admin `grant_id`); the row's `ends_at` and `status` move
forward with events. Webhook payloads are stored in `billing_events` with the provider's event
id as unique key, so a replayed webhook is a no-op and a disputed row can be explained. Nothing is
ever deleted except by account erasure. Grace: a row in the provider's grace/billing-retry state
stays active until `ends_at + plans.grace_days` (flag), then lapses.

### D4 — Codes and campaigns: the campaign owns the effect and the clock, the code is single-use

`beta_campaigns(id, product, key, name, kind, duration_days NULL, enrollment_closes_at NULL,
closed_at NULL, created_by)`; `beta_memberships(campaign_id, user_id, enrolled_at, ends_at NULL,
revoked_at, source)` unique on `(campaign_id, user_id)`; `access_codes(id, campaign_id,
code_hash, issued_by, issued_to_hint, max_uses = 1, uses, revoked_at)`;
`access_code_redemptions(code_id, user_id, redeemed_at)`. Codes are ≥ 128-bit random, displayed
once at mint time, stored hashed; redemption is rate-limited per user and per address through the
existing `ratelimit::check` and refuses without revealing whether a code exists.

Redeeming a code = **enrolling** in its campaign: one transaction marks the code used, inserts the
membership, and — for `premium_trial` — upserts the premium row with `ends_at = now +
duration_days`. Enrolment is refused when the campaign is closed, enrolment is closed, the
account is already a member, or (trial) another premium trial is active for the account.
Revoking a code stops its redemption only; revoking a membership ends it (and its trial row) now.
Issuers implement one port (`AccessCodeIssuer::mint(campaign, issued_by, hint)`); the back-office
console and the Discord `/beta` command (campaign chosen by the channel's configuration) are two
callers. Nominative enrolment by handle from the console goes through the same enrolment path
with `source = admin` and no code.

*Alternative considered*: a campaign-wide movable end date for trials. Right when the whole beta
should end together, but the owner wants each tester to get the same N days from *their* start —
so the clock is per membership; the operator's lever is closing enrolment. Feature betas are the
opposite (no clock, closed by hand), which is why `kind` exists rather than a nullable date.

### D5 — Redemption on the web only; store builds carry no code entry

Apple 3.1.1 names "license keys" as a forbidden unlock mechanism; Play's policy is aligned. The
redeem page (`cymbra.app/redeem?code=…`, served by the backend behind the existing web session
cookie, or by the site calling `RedeemAccessCode`) is the only entry. Because the entitlement is
account-bound, it applies on the store builds within one plan refresh. Desktop and web builds may
deep-link the page; store builds must not render a code field or text inviting one.

### D6 — Feature flags gain `plan` and `betas`, not a per-user table

`EvalContext { app, staff, plan, betas: Set<campaign_key> }`. `RolloutScope` gains `premium_only`
(reaches effective-plan premium and staff) and `beta:<campaign_key>` (reaches active members of
that campaign and staff — **not** premium payers outside it). The stored `rollout_scope` is a
string validated by pattern (`global | staff_only | premium_only | beta:[a-z0-9-]+`) instead of the
current two-value CHECK. The client snapshot key includes plan and memberships so a purchase, an
enrolment, a lapse or a beta closing invalidates the cache. This is the generic seam that makes
every future feature plan- or beta-gatable from the flags console; per-user targeting is still
deliberately absent. Closing a feature beta therefore needs no flag edit: members simply stop
matching; when the feature is stable the operator moves the flag to `global` (or `premium_only`).

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

`GetMyPlan` returns `{ effective_plan, source, ends_at, trial: {campaign, ends_at}?,
betas: [{campaign, kind, joined_at}], manage: {kind: apple|google|web|none},
can_purchase_here: bool }`. `can_purchase_here` is decided **server-side
per audience/platform** so store builds never show a channel they must not: iOS/macOS → Apple,
Android → Google, Linux/Windows/web → web checkout. A user already premium via channel A sees
"managed on A" instead of a second purchase button on channel B (double subscription is the
number-one support ticket). The daily-access `upsell` field reuses the same decision. Refresh:
on app resume, after a purchase/restore, and by the flags poll (plan is part of the snapshot key).

### D11 — Back office: one `Plans` view on the existing role/audit patterns, music-admin only

Account lookup by handle → entitlement rows (source, plan, ends_at, provider ref hidden behind a
copy button) and its beta memberships, grant/revoke and enrol/unenrol with reason (audited like
`role_grants`), campaigns (create with kind + duration, close enrolment, close a feature beta),
codes (mint N, show once, revoke), members list per campaign (export as CSV for announcements or
the "thank you" store offer). Guarded by `require_admin_in_scope("music")` — a moderator can
neither see who pays nor grant.

### D12 — Rollout: dark, then beta, then channels one at a time

`plans.enabled` (default off) gates every plan-aware decision back to today's behaviour;
`billing.apple.enabled`, `billing.google.enabled`, `billing.web.enabled` gate the paywall's
purchase button per channel and the webhook routes' acceptance (routes always answer 2xx to
avoid provider retries storms but ignore payloads when disabled — logged). Sequence: land dark →
enable `plans.enabled` with a staff-only trial campaign → open the community trial and the first feature
beta (Discord `/beta`)
→ Apple sandbox → Apple prod → Google → web.

### D13 — Withdrawal on lapse: server rotates the cache secret, app purges plan-only files

Offline content is a **lease**, not a gift: without withdrawal, a 90-day trial is "cache 300
favourites, drop to free, play offline forever" — the offline cache deliberately bypasses the daily
quota when offline, so the loophole would be structural. Hence (a) `offline.cache` is a premium
unlock (free = catalog online only; bundled demos and own uploads offline for everyone), and (b)
when the effective plan drops to `free` **past grace**, the server rotates the user's offline
cache secret once (the seam already specified by `backend-offline-key`) and the entitlement gate
keeps refusing premium `.sf2` bytes. The app, at its next connection, deletes premium SoundFonts it
no longer owns and cached catalog scores, keeps everything the user owns, and shows a localized
notice; the plan status has announced the "rights end on <date>" beforehand. As a local belt the
app stops opening plan-only content after `ends_at + grace` from its last snapshot while offline.

Mechanics: a nullable `withdrawn_at` on the entitlement row + a daily sweep job
(`plans_withdraw`) **and** the same check on `GetMyPlan`/`GetOfflineCacheKey`; whichever sees
"no active row ∧ latest `effective_end` < now ∧ not yet withdrawn" rotates once and stamps —
idempotent across devices and retries. Nothing runs while a row is in grace/billing retry.
Re-subscription needs no repair (favorites intact, content re-fetched). Feature betas grant no
content, so closing one withdraws nothing.

*Alternative considered*: trusting the client to delete on its clock. Gameable and time-zone
sensitive; kept only as the offline belt. *Alternative*: keeping a small free offline cache
(N favourites). Rejected by the owner — premium needs tangible privileges and the rule stays
explainable ("offline = premium").

## Risks / Trade-offs

- **Apple JWS verification done by hand** (no official Rust library) → bundle Apple's root CAs,
  verify the x5c chain + signature + `bundleId` + `environment` in a pure core with fixture JWS
  from Apple's sandbox; a verification failure is a hard reject with a metric, never a fallback to
  "trust the client".
- **Double subscription across channels** → `can_purchase_here` is false when a paid row from
  another source is active; the paywall shows where it is managed; support runbook documents the
  provider-side refund path.
- **Betas undermining premium** → a trial is premium for a fixed N days per tester and one trial
  per account at a time; a feature beta grants no plan at all; comps are visibly non-store; the
  console flags open-ended grants; a paying user never receives unfinished features.
- **Trial testers never exercise the free path** (quota, paywall, upsell) → run a small
  `feature`-kind campaign on the free plan for a handful of testers, or revoke your own grant from
  the console before release.
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
- **Grace and lapse behaviour surprises** → `plans.grace_days` flag; withdrawal only past
  grace, announced in advance with the date, never silent, never touching owned content;
  re-subscription restores without repair.
- **Withdrawal wiping a paying user by mistake** (missed renewal webhook) → withdrawal keys on
  "no active row past grace", the reconciliation job runs before the sweep, and a re-asserted
  store transaction re-activates immediately; the cache is re-fetched, nothing owned was lost.
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
5. Enable `plans.enabled` → trial campaign + first feature beta open → Discord `/beta` (other
   change) → observe.
6. Apple sandbox with `billing.apple.enabled` on a TestFlight build; then production; then
   Google; then web after the MoR account and terms are ready.

**Rollback**: `plans.enabled` off restores today's behaviour instantly (gates fall back to
`NoSubscriptions`-equivalent answers, paywall hidden). Channel flags off stop new purchases and
ignore webhooks (logged). Schema is additive; rows are inert when the code is reverted.

## Open Questions

- Which MoR — **leaning Paddle Billing** for an EU-first audience (mature subscription
  lifecycle, EU local payment methods, established VAT handling); Polar (EU company, cheaper) to
  be checked as a challenger; Lemon Squeezy as fallback given its post-acquisition uncertainty.
  Fees, payout currency and sandbox quality to be verified at signing; the adapter is one module
  either way.
- ~~Beta as a plan~~ — **decided**: two axes (D2); trials = premium N days per tester (default
  90); feature betas closed by hand; guardrails plan-independent.
- Prices and periods live in the stores/MoR; the app only needs *which* products to offer —
  runtime config `plans.premium.products` (leaning) rather than a hard list.
- Whether the redeem page lives on the Astro site (calls `RedeemAccessCode` with the web
  session) or is served by the backend (`GET /redeem`), given the site is a separate repo.
- Should the "thank you" for beta testers be a comp (`premium`, source `admin`, N months) or a
  store offer code campaign? Both are possible; the cohort export makes either easy.
