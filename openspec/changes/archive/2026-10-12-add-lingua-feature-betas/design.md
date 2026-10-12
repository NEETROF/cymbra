## Context

See proposal.md for why. The pieces, as they are on `main`:

- **Products.** `cymbra_platform` has `MUSIC_SCOPE`, `LIVE_SCOPE`, `LINGUA_SCOPE` and
  `APP_SCOPES` (`backend/platform/src/lib.rs`). Tokens carry an audience: `music`, `lingua`,
  `live`, `web` (the site), `back-office`.
- **Campaigns.** `Campaign.product` is read from `beta_campaigns.product` and
  `core::snapshot(…, product)` keeps only the memberships of that product
  (`backend/plans/src/core.rs`). But `NewCampaign` has no product and the `INSERT`
  (`backend/plans/src/pg.rs`) never writes one, so every campaign is `'music'` (column default from
  `0001_init`). Admin RPCs (`create_campaign`, `EnrolHandle`, `RevokeMembership`, close/reopen) are
  gated by `require_admin_in_scope(&id, "music")`.
- **Flag evaluation.** `GetEffectiveFlags` builds `EvalContext::for_identity(&id.audience, id)`
  and asks `PlanContextSource::plan_context(user_id)` for `(premium, betas)`
  (`backend/feature-flags/src/grpc.rs`). Its only production implementation calls
  `snapshot(user_id, MUSIC_SCOPE)` (`backend/server/src/flags.rs`), whatever the audience.
- **Scope integrity.** `CampaignDirectory::campaign_exists(key)` checks existence only, with no
  product (`feature-flags/src/context.rs`, implemented in `server/src/flags.rs`).
- **Back office.** `CampaignsView.vue` (create form: key, name, kind), `AccountPlanPanel.vue`
  (enrol, revoke) and `FlagDrawer.vue` (beta options = all open campaigns) know no product; the
  pages are shown to Music admins only.
- **Extension.** No flags client: `tool/gen_proto.sh` builds auth, user and Lingua protos only;
  `net/transport.ts` `Clients` has no flag service. The flag service is mounted behind the
  optional interceptor and the gRPC-web CORS layer covers the whole router, so the extension
  origin can already reach it.
- **Music app reference.** `packages/cymbra_flags` already implements the client contract of
  `runtime-feature-flags` (one snapshot, known version, per-identity cache, last-good on failure).

## Goals / Non-Goals

**Goals:**
- An operator can run a feature beta for Lingua end to end: create it, enrol and revoke accounts,
  scope a Lingua flag to it, close it.
- The extension shows what its flags say, and only the server decides access.
- Music behaves exactly as today.

**Non-Goals:**
- Lingua trials and Lingua access codes. A Lingua campaign is `feature` only.
- Live campaigns. The product list is open to Live later; nothing here enables it.
- A per-product back-office section for plans. Campaigns stay on their current page, which gains
  a product column; the broader per-product console is `harden-module-boundaries` task 4.8.
- The Discord `/beta` command (`add-discord-notifications`). Enrolment on request stays manual.

## Decisions

### D1. The flag's app chooses the snapshot; the campaign carries the product
`PlanContextSource::plan_context(user_id, app)` gains the app the context is built for, and the
server implementation maps it to a product: `music` → Music, `lingua` → Lingua, `live` → Live,
anything else → no betas and not premium. `GetEffectiveFlags` passes the app it already derives
from the audience.
- **Why:** `harden-module-boundaries` requires the product to be carried by the campaign and never
  inferred from the caller. It still is: the caller's app only says **which product's flags are
  being evaluated**, and therefore which product's memberships are relevant. A Music member
  reading Lingua flags is simply not a Lingua member.
- **Alternative — one snapshot with every product's betas:** `beta:<key>` would match across
  products; the leak `harden-module-boundaries` closed would reopen.
- **Server-side evaluators** that build a context by hand (the drums gate in
  `music/src/module.rs`) already pass Music explicitly and are untouched. The Lingua relay of
  `add-lingua-online-translation` will build its context the same way, for Lingua.

### D2. Product on creation, Music by default, feature only for Lingua
`CreateCampaignRequest.product` (new, optional string) and `CampaignMsg.product` (new). Empty
means Music, so a back office that predates the field keeps creating Music campaigns. The server
validates the product against `APP_SCOPES` minus Live, refuses `premium_trial` for Lingua, and
writes the product in the `INSERT`. No migration: the column exists and every row is `'music'`.
Keys stay globally unique (the existing `UNIQUE`), so `beta:<key>` is never ambiguous.

### D3. Admin of the campaign's product
Every campaign RPC checks `require_admin_in_scope(&id, campaign.product)` instead of the literal
`"music"`: create (on the requested product), enrol by handle, revoke a membership, close and
reopen. Listing campaigns returns those of the products the caller administers. The one-trial-at-
a-time rule (`can_enrol`) is checked against the memberships of the campaign's product.

### D4. Integrity per app
`CampaignDirectory::campaign_exists(key)` becomes `campaign_app(key) -> Option<product>`. A write
of `beta:<key>` on a flag of app A is accepted when the campaign exists and its product is A, or
when the flag is declared for every app. Otherwise it is refused with a cause distinct from
"no such campaign", which the console localises. Stored scopes are not rewritten.

### D5. The extension's flag client mirrors `cymbra_flags`
- `tool/gen_proto.sh` adds `backend/feature-flags/proto/flags.proto`; `Clients` gains `flags`.
- The background owns one snapshot `{ account, version, values }` in `chrome.storage.local` under
  `cymbra-lingua-flags`. It is fetched on sign-in (immediately), at start-up, on wake and on a
  surface opening, throttled to once every 5 minutes outside sign-in, with `known_version`.
- A snapshot whose `account` is not the signed-in account is ignored; sign-out deletes it.
- Surfaces read through a runtime message (`flags:get`), answered from memory; an absent key, an
  absent snapshot or a signed-out state reads `false`. They re-read on `chrome.storage.onChanged`.
- No new permission: the extension has no `alarms` permission and does not need one.
- **Alternative — a Lingua-specific "access" RPC per feature:** each gated feature would add its
  own endpoint and its own cache. One flag read serves every future Lingua gate.

### D6. Firefox's origin
`CYMBRA_ALLOWED_WEB_ORIGINS` already admits the Chrome extension origin. The env examples gain the
Firefox `moz-extension://` origin and the Safari host's origin, so the flag read (like every RPC)
works on all three browsers.

## Risks / Trade-offs

- **[`plans.enabled` is a Music key]** It gates every snapshot; turned off, Lingua betas vanish
  too. → It is on in production; documented next to the Lingua flags. Splitting it per product is
  out of scope.
- **[Staff sees every beta]** `beta:<key>` matches staff of the app. → Verification of a Lingua
  beta is done from a non-staff account (task 5.2), as the drums beta taught.
- **[A stale extension snapshot]** An enrolment shows within one throttle period, or at the next
  sign-in. → The server re-evaluates on every gated call, so a stale snapshot can only show a
  feature that then answers "unavailable", never grant access.
- **[Older back office]** One that does not send a product creates Music campaigns. → That is
  today's behaviour.

## Migration Plan

1. Deploy the backend (additive proto, no migration). Music is unchanged.
2. Deploy the back office with the product selector.
3. Release the extension with the flags client. With no Lingua flag scoped to a beta, nothing
   visible changes.

Rollback: redeploy the previous backend. Lingua campaigns created meanwhile become invisible to
evaluation (their memberships never reached Music) and harmless.
