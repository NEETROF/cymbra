## ADDED Requirements

### Requirement: Browser clients read and act on their plan through bearer-authenticated JSON routes

The backend SHALL expose JSON routes for browser clients — `GET /web/plans/me`,
`POST /web/plans/redeem`, `POST /web/plans/checkout`, `GET /web/plans/portal` — that require a
valid short-lived access token in the `Authorization: Bearer` header (the token the web sign-in
hands the browser), reject a missing or invalid token with `401` before any side effect, and
delegate each call to the plan service without adding business rules. Responses SHALL carry the
same fields as the gRPC surface (`me` ≡ `GetMyPlan` for platform `web`); errors SHALL be
`{ "error": "<neutral message>" }` with the same neutral wording as the RPC.

#### Scenario: Missing bearer is refused before any read

- **WHEN** a request reaches `/web/plans/me` without a valid bearer
- **THEN** the answer is `401` and no plan data is read

#### Scenario: `me` mirrors the RPC for the web platform

- **WHEN** a signed-in web session calls `GET /web/plans/me`
- **THEN** the JSON carries the plan, source, ends_at, trial, betas, managed_on, can_purchase_here and products exactly as `GetMyPlan(platform = web)` would

### Requirement: Redemption over the web API keeps the RPC's rules

`POST /web/plans/redeem` SHALL apply the same per-account and per-address rate limit before any
lookup, the same neutral refusal for unknown / revoked / spent codes, and the same
one-per-campaign / one-active-trial rules as `RedeemAccessCode`; it SHALL be accepted only for
a `web` (non-store) audience token.

#### Scenario: Successful redemption returns the campaign

- **WHEN** a signed-in web session posts a valid, unused code of an open campaign
- **THEN** `200` with the campaign key, name, kind and end date, and the account is enrolled

#### Scenario: Neutral refusal and throttle

- **WHEN** repeated invalid codes are posted from one account or address
- **THEN** each answer is the same neutral error and, past the limit, a rate-limit error before any lookup

### Requirement: Checkout and portal are gated like the app

`POST /web/plans/checkout` SHALL refuse when `plans.enabled` or `billing.web.enabled` is off, when
the product is not offered, or when the account already holds an active paid row from another
channel; otherwise it SHALL return the hosted checkout URL bound to the account.
`GET /web/plans/portal` SHALL return the provider-hosted portal URL fetched at request time for an
active `web` row and refuse otherwise (store rows are managed on the store).

#### Scenario: Checkout refused when subscribed elsewhere

- **WHEN** an account with an active `apple` row posts a checkout request
- **THEN** the answer is a refusal naming that the subscription is managed elsewhere and no checkout is created

#### Scenario: Portal only for web subscribers

- **WHEN** a web subscriber calls `/web/plans/portal`
- **THEN** a fresh provider portal URL is returned; a non-web subscriber gets a refusal

### Requirement: Cross-origin access is restricted to the configured web origins

The routes SHALL be served with a CORS policy allowing only the configured web origins
(`CYMBRA_WEB_ORIGINS`, a superset of the back-office origins) with the `Authorization` header;
any other origin MUST NOT be allowed.

#### Scenario: Site origin allowed, others not

- **WHEN** a preflight comes from `https://cymbra.app` and from an unknown origin
- **THEN** the first is allowed and the second is not
