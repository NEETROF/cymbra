## Why

Feature betas — an operator enrols an account in a campaign, and a flag scoped `beta:<key>` opens
a feature to its members — exist and work for Music (the drums were shipped that way). Lingua
cannot use them: a campaign is always created as Music, a caller's betas are always read from the
Music snapshot, and the extension reads no flags at all.

`add-lingua-online-translation` wants exactly this mechanism: give « Traduction en ligne » to one
reader at a time, on request (for example on Discord), and withdraw it from everyone without a
release. This change makes feature betas work for Lingua, so that change only consumes them.

## What Changes

- **A campaign belongs to a product, chosen at creation.** `CreateCampaign` gains a product
  (Music or Lingua); the stored product is the chosen one instead of the column's default. A
  Lingua campaign is of kind `feature` only: a trial campaign grants a plan, and only Music has one.
  Creating, enrolling into, revoking from and closing a campaign require an admin of its product.
- **A caller's betas are those of the app the flags are evaluated for.** The flag service asks
  the plan snapshot of the caller's app (`lingua` for an extension token) instead of always
  Music's, so a Lingua member matches `beta:<lingua campaign>` and a Music member never reaches a
  Lingua flag through a Music campaign.
- **A beta scope names a campaign of the flag's app.** Writing `beta:<key>` on a Lingua flag is
  refused unless `<key>` is a Lingua campaign, and the console offers only those.
- **The back office shows and picks the product** on the campaigns page, in the enrol dialog of a
  user's page, and filters the flag drawer's beta options by the flag's app.
- **The extension reads its effective flags.** The background fetches `GetEffectiveFlags` on
  sign-in, at start-up, on wake and when a surface opens (throttled), keeps the last-good set
  keyed by account, clears it on sign-out, and lets surfaces read a key locally. Unknown or
  never-fetched keys read as off.
- **Firefox's extension origin** is documented next to Chrome's in the allowed web origins, so the
  flag read works there too.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `runtime-feature-flags`: the evaluation context carries the memberships **of the app being
  evaluated**; a beta scope must name a campaign of the flag's app.
- `music-plan-entitlements`: a campaign belongs to one product, and its memberships count only for
  that product.
- `music-access-codes`: the campaign-kinds requirement names the admin of the campaign's product
  instead of a music-scope admin, and restricts Lingua campaigns to `feature`.
- `feature-flags-admin`: a flag's beta options are the open campaigns of its app.
- `admin-plan-console`: campaigns and enrolments show and choose their product.
- `lingua-browser-extension`: the extension reads, caches and refreshes its effective flags.

The legacy `music-` names of the plan capabilities are kept, as the project rules ask; the
requirements touched here now say "a campaign's product" rather than Music.

## Impact

**Products.**
- **Cymbra ID / platform (plans, feature flags)** — new: a product on campaigns, betas evaluated
  per app, product-checked beta scopes. Consumed: the existing campaign kinds, enrolment by handle,
  membership revocation, the flag service and its integrity port.
- **Back office** — new: product column and selector on campaigns, product-aware enrol dialog,
  beta options filtered by app. Consumed: the existing campaign, plan and flag consoles.
- **Cymbra Lingua** — new: a flags client in the background and a local read for surfaces.
  Consumed: the account session and gRPC-web transport.
- **Music** — no behaviour change: every existing campaign is a Music campaign, and a Music token
  keeps reading Music betas.
- **Site, Live** — untouched.

**Contract.** Additive proto fields: `product` on `CreateCampaignRequest` and `CampaignMsg`
(empty means Music, so an older back office keeps working). No field removed or renumbered.

**Data.** No migration: `beta_campaigns.product` exists since `0001_init` with default `'music'`,
which is what every existing row holds.

**Relation to `harden-module-boundaries`.** That change states the product is carried on the
campaign and never inferred from the caller. This change keeps that: the campaign's product is
stored, and the caller's app only selects **which product's snapshot** a flag of that app is
evaluated against. It also unblocks part of its task 4.8 (product-aware back-office plan views)
for campaigns.
