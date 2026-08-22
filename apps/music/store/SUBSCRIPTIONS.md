# Premium subscription — store & merchant-of-record setup

Checklist for the purchase channels of the Premium plan (changes:
add-premium-subscription, swap-store-billing-to-revenuecat). The backend stores
**identifiers only**; every channel is the merchant of record for its sales (Apple,
Google, Paddle). Nothing below is a secret — store credentials go **into RevenueCat**,
Cymbra's own secrets go in the deployment environment (see `backend/.env.example`,
"purchase channels").

The product ids offered by the paywall are the runtime config `plans.premium.products`
(default `premium_monthly`, `premium_yearly`): they must exist under the **same ids** on
every channel you enable.

## Store aggregator — RevenueCat (App Store + Google Play)

Since `swap-store-billing-to-revenuecat` the store channels go through RevenueCat:
the app purchases through its SDK, RevenueCat verifies with the store and tracks
renewals / grace / cancellations / refunds, and the backend learns store facts from
**one** webhook plus the customer API. Store consoles still hold the products; the
store **credentials** are configured in RevenueCat, **not** in our environment.

- [ ] RevenueCat project "Cymbra" with two apps: **App Store** (bundle
      `com.cymbra.music`, iOS + macOS) and **Play Store** (package `com.cymbra.music`).
      Note the project id (`CYMBRA_REVENUECAT_PROJECT_ID`, `VITE_REVENUECAT_PROJECT_ID`)
      and the two **public** SDK keys (`--dart-define=CYMBRA_RC_APPLE_KEY=…`,
      `CYMBRA_RC_GOOGLE_KEY=…` at build time — empty ⇒ the app's store client is a no-op).
- [ ] Entitlement **`premium`** attached to `premium_monthly` and `premium_yearly` on
      each store (products imported into RevenueCat once they exist in the consoles).
      Offerings / Paywalls are **not** used: the app lists products by id from
      `plans.premium.products`.
- [ ] Project settings → **Restore behavior: "Keep with original App User ID"** — a
      receipt bound to another Cymbra account is refused by the SDK
      (`receiptAlreadyInUse` → "linked to another account"), never migrated.
- [ ] Webhook (Integrations → Webhooks): URL `https://api.cymbra.app/billing/revenuecat/webhook`
      (staging: the staging host), **Authorization header value** generated there →
      `CYMBRA_REVENUECAT_WEBHOOK_SECRET`; environment "Production + Sandbox" (the
      backend applies sandbox only where `CYMBRA_REVENUECAT_ALLOW_SANDBOX=true`).
      "Send test event" must answer 200.
- [ ] Secret **v1 API key** (Project → API keys) → `CYMBRA_REVENUECAT_API_KEY`
      (customer reads for `SyncStorePlan` / reconciliation, customer deletion on erasure).
- [ ] Legal: sign the RevenueCat DPA (Account → Legal); RevenueCat is listed as a
      sub-processor in the privacy policies. The SDK is configured with the Cymbra
      account id only — no attributes, no email, no advertising ids.
- [ ] Dashboards: RevenueCat Overview / Charts are where active subscriptions, MRR
      and monthly revenue (current month included) are read — per store, product,
      country. Optionally set Charts' net-revenue estimate to the Small Business
      Program rate once enrolled.

### Apple — App Store Connect (iOS + macOS)

- [ ] Agreements: Paid Apps agreement active, banking + tax forms complete.
- [ ] Subscription group **"Cymbra Premium"** with two auto-renewable subscriptions:
      `premium_monthly` (1 month), `premium_yearly` (1 year); localized display names
      (en/fr/es/it), prices per territory, review screenshot + notes.
- [ ] **In-App Purchase key** (Users and Access → Integrations → In-App Purchase):
      generate, upload the `.p8` + key id + issuer id **to RevenueCat** (App Store app
      → In-app purchase key configuration). Nothing goes in our environment.
- [ ] App Store Server Notifications **v2**: production and sandbox URLs set to the
      URL RevenueCat shows for the app (App Store app → App Store Server Notifications),
      not to our backend.
- [ ] Sandbox tester accounts (one per locale) for TestFlight; verify purchase,
      renewal (accelerated), grace / billing retry, cancel, refund via the sandbox
      request, and "restore purchases" on a second device — each visible as a
      RevenueCat customer event AND as a `plans` row through the webhook.
- [ ] Review notes: no external purchase link, no code entry field, no discount copy in
      the app; the trial is a free, campaign-bounded access (not an introductory offer).

### Google — Play Console (Android)

- [ ] Merchant account set up; app on the closed/internal track with billing enabled.
- [ ] Subscription products `premium_monthly`, `premium_yearly`, each with one base plan
      (auto-renewing, monthly / yearly), localized listing, prices per country.
- [ ] Service account (Google Cloud) with the Play Developer API enabled and, in the
      Play Console, **View financial data + Manage orders** permission → its JSON key
      is uploaded **to RevenueCat** (Play app → Service account credentials). Nothing
      goes in our environment.
- [ ] Real-time developer notifications: Play Console → Monetization setup → topic
      name = the Pub/Sub topic RevenueCat provides for the app (Play app → Google
      Real-Time Developer Notifications); "Send test notification" green in RevenueCat.
- [ ] License testers for the test track; verify purchase, RTDN-driven renewal, cancel,
      on-hold (billing issue → grace), pause / resume, refund/void, and restore on a
      second device — each visible in RevenueCat AND as a `plans` row.

## Web — Paddle Billing (Linux, Windows, site)

- [ ] Paddle account verified (website review: terms, privacy, refund policy pages
      published on cymbra.app), payout account, sandbox account for staging.
- [ ] Product "Cymbra Premium" with two **prices** whose ids are configured in
      `plans.premium.products` for the web channel (Paddle price ids, e.g. `pri_…`)
      — or map `premium_monthly` / `premium_yearly` to those ids in the flags config
      before enabling `billing.web.enabled`.
- [ ] Notification destination (webhook) `https://api.cymbra.app/billing/web/webhook`
      subscribed to `subscription.*`, `transaction.completed`, `adjustment.created`
      → its endpoint secret in `CYMBRA_PADDLE_WEBHOOK_SECRET`.
- [ ] API key → `CYMBRA_PADDLE_API_KEY`; hosted checkout page
      `CYMBRA_PADDLE_CHECKOUT_PAGE=https://cymbra.app/checkout` — the site page hosting
      the Paddle.js overlay (`apps/site`, change add-site-account-pages); it reads
      `_ptxn` and returns to `https://cymbra.app/checkout/done`. Approve the domain
      `cymbra.app` in Paddle (Checkout → Website approval) and set the site's
      `PUBLIC_PADDLE_ENV` + `PUBLIC_PADDLE_CLIENT_TOKEN` (client-side token).
- [ ] Web sign-in for the site: `web` in `CYMBRA_ALLOWED_AUDIENCES`,
      `https://cymbra.app` in `CYMBRA_WEB_ORIGINS`, the site origin on the Google web
      client (Authorized JavaScript origins) and on the Apple Services ID (domain
      `cymbra.app`, Return URLs `/redeem`, `/account`) — the same client ids as the
      back office, set as `PUBLIC_GOOGLE_CLIENT_ID` / `PUBLIC_APPLE_CLIENT_ID`.
- [ ] Verify in sandbox from a Windows/Linux build: checkout opens in the browser
      on `/checkout`, completes, lands on `/checkout/done`; the webhook activates the
      plan, "I've paid — refresh" shows Premium; `/account` opens the portal
      (management, cancel, refund); and the iOS/Android builds show "managed on the
      web" instead of a second purchase.
- [ ] Beta codes: mint codes in the back office (Plans → campaign → codes), redeem
      one on `https://cymbra.app/redeem?code=…` (sign-in required), see it in the
      app after a refresh; the store builds have no code field and no `/redeem` link.

## Rollout order (design D12)

1. Everything dark: `plans.enabled` off, every `billing.*.enabled` off.
2. `plans.enabled` on with a staff-only trial campaign; then the community beta
   (Discord `/beta`) and the first feature beta.
3. Staging with `CYMBRA_REVENUECAT_ALLOW_SANDBOX=true`: sandbox purchases through
   RevenueCat on iOS / macOS / Android (webhook + `SyncStorePlan` + reconciliation);
   then production: `billing.apple.enabled` (TestFlight, production environment)
   → `billing.google.enabled`; then web once the Paddle review and the legal pages
   are live (Paddle is routed into RevenueCat at that point — see the change's §8).

Rollback at any step: flip the flag off — a disabled store hides its purchase button
and the aggregator webhook acknowledges-and-ignores its events; `plans.enabled` off
restores the pre-plan behaviour for everyone.
