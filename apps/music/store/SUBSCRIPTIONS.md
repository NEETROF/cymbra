# Premium subscription — store & merchant-of-record setup

Checklist for the purchase channels of the Premium plan (change:
add-premium-subscription). The backend stores **identifiers only**; every channel is
the merchant of record for its sales (Apple, Google, Paddle). Nothing below is a
secret — secrets go in the deployment environment (see `backend/.env.example`,
"purchase channels").

The product ids offered by the paywall are the runtime config `plans.premium.products`
(default `premium_monthly`, `premium_yearly`): they must exist under the **same ids** on
every channel you enable.

## Apple — App Store Connect (iOS + macOS)

- [ ] Agreements: Paid Apps agreement active, banking + tax forms complete.
- [ ] Subscription group **"Cymbra Premium"** with two auto-renewable subscriptions:
      `premium_monthly` (1 month), `premium_yearly` (1 year); localized display names
      (en/fr/es/it), prices per territory, review screenshot + notes.
- [ ] App Store Server Notifications **v2**: production URL
      `https://api.cymbra.app/billing/apple/notifications`, sandbox URL the staging
      host; both `POST`, no auth header (payloads are signed JWS).
- [ ] App Store Server API key (In-App Purchase role): key id + issuer id + `.p8` →
      `CYMBRA_APPLE_ASC_KEY_ID`, `CYMBRA_APPLE_ASC_ISSUER_ID`, `CYMBRA_APPLE_ASC_KEY_PEM`.
- [ ] `CYMBRA_APPLE_BUNDLE_ID=com.cymbra.music`; staging sets
      `CYMBRA_APPLE_ALLOW_SANDBOX=true`, production **must not**.
- [ ] Sandbox tester accounts (one per locale) for TestFlight; verify purchase, renewal
      (accelerated), grace / billing retry, cancel, refund via the sandbox request, and
      "restore purchases" on a second device.
- [ ] Review notes: no external purchase link, no code entry field, no discount copy in
      the app; the trial is a free, campaign-bounded access (not an introductory offer).

## Google — Play Console (Android)

- [ ] Merchant account set up; app on the closed/internal track with billing enabled.
- [ ] Subscription products `premium_monthly`, `premium_yearly`, each with one base plan
      (auto-renewing, monthly / yearly), localized listing, prices per country.
- [ ] Real-time developer notifications: Pub/Sub topic + a **push** subscription to
      `https://api.cymbra.app/billing/google/rtdn` authenticated with an OIDC token
      (audience = that URL, service account = the push account) →
      `CYMBRA_GOOGLE_RTDN_AUDIENCE`, `CYMBRA_GOOGLE_RTDN_SA_EMAIL`.
- [ ] Service account with the Play Developer API enabled and **View financial data /
      Manage orders** permission → `CYMBRA_GOOGLE_SA_EMAIL`, `CYMBRA_GOOGLE_SA_KEY_PEM`;
      `CYMBRA_GOOGLE_PACKAGE_NAME=com.cymbra.music`.
- [ ] License testers for the test track; verify purchase (server acknowledges), RTDN
      delivery, cancel, on-hold, refund/void, and restore on a second device.

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
      `CYMBRA_PADDLE_CHECKOUT_PAGE=https://cymbra.app/checkout` (a page hosting the
      Paddle.js overlay that reads `_ptxn`); optional `CYMBRA_PADDLE_SUCCESS_URL`.
- [ ] Verify in sandbox from a Windows/Linux build: checkout opens in the browser,
      webhook activates the plan, "I've paid — refresh" shows Premium, portal
      management, cancel, refund; and that the iOS/Android builds show "managed on
      the web" instead of a second purchase.

## Rollout order (design D12)

1. Everything dark: `plans.enabled` off, every `billing.*.enabled` off.
2. `plans.enabled` on with a staff-only trial campaign; then the community beta
   (Discord `/beta`) and the first feature beta.
3. `billing.apple.enabled` on sandbox (TestFlight) → production; then Google; then web
   once the Paddle review and the legal pages are live.

Rollback at any step: flip the flag off — a disabled channel hides its purchase button
and its notification route acknowledges-and-ignores; `plans.enabled` off restores the
pre-plan behaviour for everyone.
