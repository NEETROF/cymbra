## Why

Cymbra Music has every gating primitive a freemium product needs — a daily catalog quota with a
`SubscriptionSource` seam that still answers `false`, a SoundFont `entitlement()` union, a
runtime-flag system, a points shop whose migration already reserves `redeemable = false` for
"the future premium tier" — but **no notion of a plan on the account, no way to sell one, and no
way to hand one to a beta tester**. Launching a community beta and then a paid tier needs a single
server-side answer to "what plan does this user have, from which source, until when" that works
identically on iOS, Android, macOS, Linux and Windows, without the developer ever touching an
invoice, an address or a card number, and without a subscription middleware taking a cut.

## What Changes

- **A server-side plan model** (`free` < `beta` < `premium`) with a multi-source, expiring
  entitlement ledger. An entitlement row records *who*, *which plan*, *from which source*
  (`apple`, `google`, `web`, `code`, `admin`), *until when*, plus the provider's opaque
  reference. The effective plan is the highest-ranked active row; **premium always wins over
  beta**, and a paid entitlement is never touched by the end of a beta campaign. Premium's unlock
  set is fixed in code (it is what is sold); beta's is a **configurable subset** of premium — beta
  can never unlock more than premium, only less, plus preview features via a plan-scoped flag
  rollout. No personal billing data is stored: identifiers only.
- **Existing seams are wired, not redeclared**: `SubscriptionSource` (daily catalog access) is
  implemented over the ledger; `soundfont_access::entitlement()` and its SQL twin in the reward
  shop gain a plan disjunct; the private SoundFont library cap becomes a per-plan config instead
  of a hardcoded constant; the feature-flag `EvalContext` gains a `plan` dimension and a
  plan-scoped rollout so any future feature can be gated per plan from the back office.
- **Access codes and campaigns** for beta testers and comps: a campaign carries the plan and an
  **end date the operator can move**; codes are high-entropy, single-use, bound to a campaign,
  claimed on the **web** (never through a code-entry field in a store build) and consumed once
  per account per campaign. Nominative grants (by handle, source `admin`) cover testers who are
  not on Discord. Codes and grants mint **free access only** — never a price or a discount.
  The Discord `/beta` command (`add-discord-notifications`) is one issuer of such codes.
- **Three purchase channels, one entitlement**: Apple (StoreKit 2, verified server-side through
  App Store Server Notifications v2 + signed transactions), Google (Play Billing, verified through
  Real-Time Developer Notifications + the Play Developer API) and a **web Merchant-of-Record
  checkout** (Paddle-class provider) for Linux/Windows and the site. Every channel ends in the same
  ledger row, so a subscription bought anywhere is active everywhere; managing or cancelling it
  happens on the provider's own portal. No RevenueCat-style intermediary.
- **A paywall and plan-status surface in the app** that is **channel-aware per platform**: store
  builds show only their store's purchase flow (and no external-purchase or code hint), desktop
  builds open the web checkout in the browser, every build shows the current plan, its source and
  end date, and a "manage" action that opens the right provider portal. The existing daily-access
  upsell hook stops saying "coming soon".
- **A back-office plan console**: look up an account's entitlements, grant/revoke by handle,
  create campaigns and move their end date, mint or revoke codes, see who redeemed what.
- **Everything ships dark**: a `plans.enabled` kill-switch (default off) plus one flag per
  purchase channel; with all flags off the app is exactly today's app.

## Capabilities

### New Capabilities
- `music-plan-entitlements`: the plan model (`free`/`beta`/`premium`), the multi-source expiring
  entitlement ledger, effective-plan resolution and precedence (premium > beta > free, paid never
  overwritten by a beta end), the unlock sets (premium fixed, beta configurable subset), lapse and
  grace handling, exposure of the effective plan to the app, and erasure with the account.
- `music-access-codes`: campaigns (plan + movable end date + open/closed), single-use
  high-entropy codes bound to a campaign, one redemption per account per campaign, web-only
  redemption, issuer port (back office, Discord `/beta`), revocation of a code or a whole
  campaign, and the "free access only, never a discount" invariant.
- `music-subscription-billing`: the three purchase channels (Apple, Google, web MoR) — server-side
  verification of every purchase event, idempotent webhook ingestion, mapping of provider states
  (active, grace, billing retry, cancelled, refunded, revoked) onto entitlement rows,
  cross-platform activation, provider-hosted management, and the "no personal billing data"
  invariant.
- `music-premium-paywall`: what a free user sees at each locked surface and in account settings,
  the channel-aware purchase entry per platform, restore/refresh of the plan, the beta-tester
  variant ("beta until X — subscribe to keep it"), and store-compliance rules (no external
  purchase steering, no code entry, no discount talk in store builds).
- `admin-plan-console`: the back-office screen and gRPC surface for entitlement lookup,
  nominative grants/revocations, campaign and code management, and the audit trail — admin
  (music scope) only.

### Modified Capabilities
- `music-score-daily-access`: the "Subscription bypass seam and upsell hook" requirement is
  fulfilled by the effective plan (a plan whose unlock set contains unlimited catalog access is a
  `Subscriber`), and the upsell carries a real, platform-appropriate call to action instead of a
  placeholder.
- `soundfont-entitlement`: raw `.sf2` bytes are also served when the caller's effective plan
  includes the SoundFont library unlock; the client-side lock mirror follows the same rule.
- `reward-unlocks`: a shop item is `owned` when the caller's plan includes the SoundFont unlock;
  `redeemable = false` items are presented as "included in premium" rather than "coming later";
  points still cannot buy premium.
- `user-soundfont-library`: the private-library count quota is resolved per plan at request time
  (free keeps today's value) instead of a hardcoded constant; moderator/admin exemption is kept.
- `runtime-feature-flags`: the evaluation context gains a `plan` dimension and a plan-scoped
  rollout (`beta_only`, `premium_only`) next to `global`/`staff_only`; the client snapshot is
  keyed by plan as well as identity.
- `feature-flags-admin`: the flags console can pick the plan-scoped rollouts.

## Impact

Products (consumed vs new):

- **Cymbra ID (consumed only)** — `user_id`, handle lookup, `(scope, role)` for the admin
  guards, and the existing account-erasure job, which must purge the new tables. No new identity
  concept: a plan is **not** a role.
- **Cymbra Music (new)** — a new backend crate `backend/plans` (`cymbra-plans`): entitlement
  ledger + effective-plan core, campaigns/codes, one adapter per purchase channel (pure state
  mapping core + thin HTTP/webhook glue), the `SubscriptionSource` implementation and a
  `PlanSource` port consumed by `backend/music` (`soundfont_access`, reward shop, library quota).
  New HTTP routes on the existing server: Apple ASN v2, Google RTDN push, web-MoR webhook, and the
  web redeem/checkout return pages. New gRPC: `PlanService` (`GetMyPlan`, `RedeemAccessCode`,
  `CreateWebCheckout`, `ReportStorePurchase`, admin RPCs). App (`apps/music`): `in_app_purchase`
  (official Flutter plugin, StoreKit 2 / Play Billing) behind an injectable seam, paywall + plan
  status + manage action, plan-aware unlock mirrors, ARB strings for en/fr/es/it. Postgres: new
  `plans` schema (`plan_entitlements`, `plan_campaigns`, `access_codes`, `access_code_redemptions`,
  `billing_events`), additive, with a `product` column so Live can reuse it later without a
  rename.
- **Platform (consumed + one modification)** — feature flags: `EvalContext.plan` +
  `RolloutScope::{BetaOnly, PremiumOnly}` (SQL CHECK widened), new keys `plans.enabled`,
  `plans.beta.features`, `plans.grace_days`, `plans.soundfont_library.max_fonts.{free,premium}`,
  `billing.{apple,google,web}.enabled`; jobs: a reconciliation job (re-check provider state for
  entitlements nearing expiry) and the purge hook; rate limiting on `RedeemAccessCode` and the
  webhook routes via the existing `ratelimit::check`.
- **Back office (new screen)** — `Plans` view: account lookup, grants, campaigns, codes,
  redemptions; consumes the existing roles/audit patterns (`RolesView`, `role_grants`) and the
  flags console for the new keys.
- **Cymbra Live** — untouched; the `product` column and the crate are ready for it.
- **Legal/ops** — terms of sale (subscription, renewal, refunds handled by the store/MoR),
  privacy policy (provider identifiers stored, no card data), App Store Connect / Play Console
  product setup, MoR account and webhook secrets in the environment (`backend/.env.example`),
  Apple root certificates bundled for JWS verification.
- **Coverage** — `cymbra-plans` is a workspace member; its cores (precedence, state mapping,
  code lifecycle) are host-testable, the HTTP/webhook glue is added to the `--ignore-filename-regex`
  list like the other adapters.
- **Cross-change** — `add-discord-notifications` consumes `music-access-codes` through an issuer
  port (`/beta`); the in-flight `add-offline-score-cache` may later add a per-plan favorites cap
  on the same ledger — deliberately out of scope here.
