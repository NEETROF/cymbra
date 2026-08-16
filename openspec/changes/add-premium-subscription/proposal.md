## Why

Cymbra Music has every gating primitive a freemium product needs — a daily catalog quota with a
`SubscriptionSource` seam that still answers `false`, a SoundFont `entitlement()` union, a
runtime-flag system, a points shop whose migration already reserves `redeemable = false` for
"the future premium tier" — but **no notion of a plan on the account, no way to sell one, and no
way to run a targeted, time-bounded beta**. Launching a community beta and then a paid tier
needs a single server-side answer to "what plan does this user have, from which source, until
when" and "which betas is this user in", identical on iOS, Android, macOS, Linux and Windows,
without the developer ever touching an invoice, an address or a card number, and without a
subscription middleware taking a cut.

## What Changes

- **Two independent axes on the account, both server-side.**
  - **Plan** — `free` or `premium`. A multi-source, expiring entitlement ledger records *who*,
    *from which source* (`apple`, `google`, `web`, `code`, `admin`), *until when*, plus the
    provider's opaque reference. The effective plan is `premium` when any row is active; a paid
    row is never touched by anything a beta does. Premium's unlock set (unlimited catalog opens,
    the whole SoundFont library, larger private quotas) is fixed in code — it is what is sold.
    No personal billing data is stored: identifiers only.
  - **Beta membership** — the account belongs to zero or more **beta campaigns**. Two kinds:
    **`premium_trial`** ("beta d'usage premium") grants premium for a fixed number of days
    **from each tester's own enrolment** (default 90), enrolment can be closed without
    shortening anyone; **`feature`** ("beta par fonctionnalité", e.g. `midi-drums`) grants no
    plan but makes the member reachable by feature flags rolled out to that campaign, and has
    **no end date — the operator closes it when the feature is stable**, which ends early access
    for everyone at once. A user can be in several betas; at most one premium trial is active per
    account at a time.
- **Existing seams are wired, not redeclared**: `SubscriptionSource` (daily catalog access) is
  implemented over the ledger; `soundfont_access::entitlement()` and its SQL twin in the reward
  shop gain a plan disjunct; the private SoundFont library cap and the score upload quota become
  per-plan runtime config instead of constants; the feature-flag `EvalContext` gains the plan and
  the beta memberships, with rollouts `premium_only` and `beta:<campaign>`. Anti-abuse guardrails
  (download burst/volume, enumeration, auth throttles) stay **plan-independent**.
- **Access codes** for beta enrolment and comps: high-entropy, single-use, bound to a campaign,
  redeemed on the **web** (never through a code-entry field in a store build), one redemption
  per account per campaign. Nominative enrolment/grants by handle (source `admin`) cover testers
  who are not on Discord. Codes and grants mint **free access only** — never a price or a
  discount. The Discord `/beta` command (`add-discord-notifications`) is one issuer.
- **Three purchase channels, one entitlement**: Apple (StoreKit 2, verified server-side through
  App Store Server Notifications v2 + signed transactions), Google (Play Billing, verified through
  Real-Time Developer Notifications + the Play Developer API) and a **web Merchant-of-Record
  checkout** (Paddle-class provider) for Linux/Windows and the site. Every channel ends in the same
  ledger row, so a subscription bought anywhere is active everywhere; managing or cancelling it
  happens on the provider's own portal. No RevenueCat-style intermediary.
- **A paywall and plan-status surface in the app** that is **channel-aware per platform**: store
  builds show only their store's purchase flow (and no external-purchase or code hint), desktop
  builds open the web checkout in the browser, every build shows the current plan, its source and
  end date, the active betas, and a "manage" action that opens the right provider portal. The
  existing daily-access upsell hook stops saying "coming soon".
- **A back-office plan console**: look up an account's entitlements and betas, grant/revoke by
  handle, create campaigns of either kind, close enrolment or close a feature beta, mint or revoke
  codes, see who joined what, export a campaign's members.
- **Everything ships dark**: a `plans.enabled` kill-switch (default off) plus one flag per
  purchase channel; with all flags off the app is exactly today's app.

## Capabilities

### New Capabilities
- `music-plan-entitlements`: the `free`/`premium` plan, the multi-source expiring entitlement
  ledger, effective-plan resolution, the fixed premium unlock set, lapse and grace, beta
  memberships as a separate axis (kinds `premium_trial` and `feature`, per-tester duration vs
  operator-closed), exposure of plan + memberships to the app, kill-switch, erasure with the
  account.
- `music-access-codes`: beta campaigns (kind, plan effect, per-tester duration, enrolment
  close, feature close), single-use high-entropy codes bound to a campaign, one redemption per
  account per campaign, one active premium trial per account, web-only redemption, issuer port
  (back office, Discord `/beta`), revocation, and the "free access only, never a discount"
  invariant.
- `music-subscription-billing`: the three purchase channels (Apple, Google, web MoR) — server-side
  verification of every purchase event, idempotent webhook ingestion, mapping of provider states
  (active, grace, billing retry, cancelled, refunded, revoked) onto entitlement rows,
  cross-platform activation, provider-hosted management, and the "no personal billing data"
  invariant.
- `music-premium-paywall`: what a free user sees at each locked surface and in account settings,
  the channel-aware purchase entry per platform, restore/refresh of the plan, the premium-trial
  variant ("trial until X — subscribe to keep it"), the betas list, and store-compliance rules
  (no external purchase steering, no code entry, no discount talk in store builds).
- `admin-plan-console`: the back-office screen and gRPC surface for entitlement/membership
  lookup, nominative grants/enrolments/revocations, campaign and code management, member
  export, and the audit trail — admin (music scope) only.

### Modified Capabilities
- `music-score-daily-access`: the "Subscription bypass seam and upsell hook" requirement is
  fulfilled by the effective plan (`premium` ⇒ `Subscriber`), and the upsell carries a real,
  platform-appropriate call to action instead of a placeholder.
- `soundfont-entitlement`: raw `.sf2` bytes are also served when the caller's effective plan is
  `premium`; the client-side lock mirror follows the same rule.
- `reward-unlocks`: a shop item is `owned` for a premium caller; `redeemable = false` items are
  presented as "included in premium" rather than "coming later"; points still cannot buy premium.
- `user-soundfont-library`: the private-library count quota is resolved per plan at request time
  (free keeps today's value) instead of a hardcoded constant; moderator/admin exemption is kept.
- `backend-score-storage`: the per-user rolling upload quota and a new private-library size cap
  are resolved per plan at request time (free keeps today's values); proposing to the catalog
  stays open to every plan.
- `runtime-feature-flags`: the evaluation context gains the effective plan and the set of active
  beta campaigns; rollouts `premium_only` and `beta:<campaign>` join `global`/`staff_only`; the
  client snapshot is keyed by plan and memberships as well as identity.
- `feature-flags-admin`: the flags console can pick the plan- and beta-scoped rollouts.
- `admin-account-directory`: `ListAccounts` accepts an explicit `ids` set so the back office can
  filter the accounts directory by plan / beta after resolving the criterion through the plan
  service — the identity service stays product-agnostic.

## Impact

Products (consumed vs new):

- **Cymbra ID (consumed only)** — `user_id`, handle lookup, `(scope, role)` for the admin
  guards, and the existing account-erasure job, which must purge the new tables. No new identity
  concept: neither a plan nor a beta membership is a role.
- **Cymbra Music (new)** — a new backend crate `backend/plans` (`cymbra-plans`): entitlement
  ledger + effective-plan core, campaigns/memberships/codes, one adapter per purchase channel
  (pure state mapping core + thin HTTP/webhook glue), the `SubscriptionSource` implementation and
  a `PlanSource` port consumed by `backend/music` (`soundfont_access`, reward shop, library and
  upload quotas). New HTTP routes on the existing server: Apple ASN v2, Google RTDN push, web-MoR
  webhook, and the web redeem/checkout return pages. New gRPC: `PlanService` (`GetMyPlan`,
  `RedeemAccessCode`, `CreateWebCheckout`, `ReportStorePurchase`, `RestorePurchases`, admin RPCs).
  App (`apps/music`): `in_app_purchase` (official Flutter plugin, StoreKit 2 / Play Billing)
  behind an injectable seam, paywall + plan/beta status + manage action, plan-aware unlock
  mirrors, ARB strings for en/fr/es/it. Postgres: new `plans` schema (`plan_entitlements`,
  `beta_campaigns`, `beta_memberships`, `access_codes`, `access_code_redemptions`,
  `billing_events`), additive, with a `product` column so Live can reuse it later without a
  rename.
- **Platform (consumed + one modification)** — feature flags: `EvalContext.{plan, betas}` +
  `RolloutScope::{PremiumOnly, Beta(campaign)}` (SQL CHECK widened to a pattern), new keys
  `plans.enabled`, `plans.grace_days`, `plans.premium.products`,
  `plans.soundfont_library.max_fonts.{free,premium}`, `plans.scores.upload_quota.{free,premium}`,
  `plans.scores.library_max.{free,premium}`, `billing.{apple,google,web}.enabled`; jobs: a
  reconciliation job (re-check provider state for entitlements nearing expiry) and the purge
  hook; rate limiting on `RedeemAccessCode` and the webhook routes via the existing
  `ratelimit::check`.
- **Back office (new screen + directory columns)** — `Plans` view: account lookup, grants,
  campaigns (both kinds), codes, members; the accounts directory gains plan/beta badges and
  filters (premium, premium trial, beta by campaign) via `ListAccountIdsByPlan` +
  `GetPlansForAccounts`; consumes the existing roles/audit patterns (`RolesView`, `role_grants`)
  and the flags console for the new keys.
- **Cymbra Live** — untouched; the `product` column and the crate are ready for it.
- **Legal/ops** — terms of sale (subscription, renewal, refunds handled by the store/MoR; beta
  access free, time-bounded, revocable), privacy policy (provider identifiers stored, no card
  data), App Store Connect / Play Console product setup, MoR account and webhook secrets in the
  environment (`backend/.env.example`), Apple root certificates bundled for JWS verification.
- **Coverage** — `cymbra-plans` is a workspace member; its cores (effective plan, membership
  lifecycle, state mapping, code lifecycle) are host-testable, the HTTP/webhook glue is added to
  the `--ignore-filename-regex` list like the other adapters.
- **Cross-change** — `add-discord-notifications` consumes `music-access-codes` through an issuer
  port (`/beta`, campaign chosen by the channel's configuration); the in-flight
  `add-offline-score-cache` may later add a per-plan favorites cap on the same ledger —
  deliberately out of scope here.
