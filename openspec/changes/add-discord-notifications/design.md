## Context

Cymbra's user-facing community surface today is GitHub Discussions — a developer surface. A
Discord server is the growth lever, but it only stays alive if the product feeds it. The
backend already has everything needed to feed it safely:

- a transactional job platform (`cymbra-jobs`: `Enqueuer`/`EnqueueRequest`, `JobSpec` registry,
  channels, retries, DLQ, scheduler) and a long-running consumer (`cymbra-worker`);
- an outbound-port precedent for third-party I/O (`EmailSender` in `cymbra-platform`, with the
  SMTP implementation behind the trait);
- a fail-closed public-listing gate (`UserPort::listable_profiles` — public profile **and**
  age-eligible, `share_eligible_from` / `min_public_sharing_age` default 16);
- a runtime flag service (`FlagService`) already wired into the worker context and the back
  office.

Constraints that shape the design:

- **Privacy.** Naming a player in a public Discord channel publishes pseudonymous personal
  data. Cymbra's own model already treats public listing as opt-in, age-gated and fail-closed;
  Discord must be at least as strict, and a Discord message is **not retractable**.
- **Rate.** Discord allows roughly 5 requests/second per webhook and ~30 messages/minute per
  channel, and a channel that receives hundreds of machine messages a day is abandoned by
  humans. Event volume must be shaped, not forwarded.
- **Coverage.** A new workspace member is pulled into
  `cargo llvm-cov --workspace --fail-under-lines 80`, so the crate must be mostly pure code
  with a thin, ignore-listed I/O edge.
- **`cymbra-notifications` is not on `main`** (branch `add-push-notifications`). This change
  must not depend on it.

## Goals / Non-Goals

**Goals:**

- The backend can publish product events to Discord channels, transactionally and idempotently.
- No player is ever named without a dedicated, revocable, age-gated consent.
- High-frequency activity reaches Discord as an aggregate, never as a stream.
- The whole surface can be muted from the back office without a redeploy, and hard-killed by
  revoking a webhook URL in Discord.
- Slash commands and Discord role grants are possible without hosting a gateway process.

**Non-Goals:**

- Reading Discord messages, presence, or voice state (that would require a gateway).
- Moderating the Discord server from Cymbra, or mirroring Discord content into the app.
- Any per-user push/notification targeting — that is `cymbra-notifications`' job, deliberately
  kept separate.
- Provisioning the Discord server itself (channels, roles, onboarding, AutoMod). That is
  server-configuration work, tracked separately.
- Announcing anything about content that is not publicly accepted, or about authentication.

## Decisions

### D0 — A dedicated `cymbra-discord` crate, not an extension of `cymbra-notifications`

`backend/discord` owns the Discord protocol. Push notifications target **one user across their
devices** (per-user consent, device tokens, per-category user preferences); Discord **broadcasts
to one server** (per-server webhooks, one channel per category, no user targeting). The
selection logic has almost nothing in common, and `cymbra-notifications` lives on an unmerged
branch — folding into it would couple this change to that branch's fate.

*Alternative considered*: a second "channel" inside the notifications dispatch. Rejected: it
would force a per-user shape onto a broadcast surface and create a merge dependency.

### D1 — Outbound port + pure core, mirroring `EmailSender`

`DiscordSender` (trait) exposes `publish(channel, message)` and `grant_role`/`revoke_role`.
Implementations: `WebhookDiscordSender` (announcements) and `BotRestDiscordSender` (roles) —
both thin `reqwest` adapters. Everything that decides *whether* and *what* to publish lives in a
pure `discord_core` module: event → category, category → channel, gate application, message
rendering, throttle/aggregate-minimum arithmetic. Tests double the port with `mockall`
generated mocks (per the `rust-testing` convention); the HTTP adapters go into the
`--ignore-filename-regex` list like `api/audio.rs` and the SMTP sender.

**Categories are product-namespaced** (`music.*`, `id.*`, later `live.*`), because the Discord
server groups channels **by product**: statistics live in the section of the product they
describe (`#music-stats`, `#id-stats`), not in one global stats section. The routing key is
therefore `(product, category) → channel`, which keeps a second product from ever landing in the
first one's channels and makes "mute one product's feed" expressible (D8). An unmapped pair
stays a no-op — never a fallback to some default channel, which would silently cross products.

### D2 — Webhooks for announcements, bot REST for roles

Webhooks need no bot membership, are scoped to a single channel, and can be revoked
individually — a leaked webhook can post to one channel and is rotated in seconds. Roles cannot
be granted by a webhook, so role operations use the bot token.

*Alternative considered*: posting all messages with the bot token. Rejected: broader blast
radius for one secret, and it requires channel-level permission management the webhook model
avoids.

### D3 — No gateway: HTTP interactions endpoint with Ed25519 verification

Slash commands arrive as signed HTTP POSTs on `POST /discord/interactions`, served by the
existing axum/tonic server (Caddy already routes by path). Verification is Ed25519 over
`timestamp || body` against the application public key, **before** any parsing that could have a
side effect; `PING` answers `PONG`. This adds no deployable and no persistent connection.

*Alternative considered*: a `serenity`/`twilight` gateway process. Rejected: a second
long-running service to deploy and monitor, justified only by message/presence reads that are
explicit non-goals.

### D4 — Producers enqueue an **event descriptor**; the worker renders at publication time

Producers enqueue `discord_notify` through `Enqueuer` inside the domain transaction. The payload
is the **event identity** (kind, subject ids, dedup key) — *not* a rendered message. This is a
deliberate deviation from the `verification_email` job, where the producer renders the body: an
email is addressed to someone who already consented, whereas a Discord announcement must
re-evaluate consent, listability, flags and throttle **at publication time** (a spec
requirement). A pre-rendered message would freeze a consent snapshot that may be stale by the
time the job runs.

Consequence: the worker needs read access to the data it renders from (music aggregates, user
profiles) through the existing in-process module/port seams — no new cross-schema writes.

### D5 — Idempotency by dedup key in a `discord` schema

`backend/discord/migrations/0001_init.sql` creates a `discord` schema with a
`published_announcements` table keyed by a **unique dedup key** derived from event identity
(e.g. `score_accepted:<id>`), plus a status and timestamps — the same "own crate, own schema,
own migrations" shape as `cymbra-analytics`. Publication is: claim the key
(`INSERT … ON CONFLICT DO NOTHING`; zero rows ⇒ already handled ⇒ stop), post, mark published. A
crash between claim and post leaves a stale claim that becomes re-claimable after a configured
grace period, so at-least-once delivery converges without ever double-posting in the normal
case.

*Alternative considered*: relying on the job engine's exactly-once illusion. Rejected: sqlxmq is
at-least-once by contract; the spec forbids duplicate messages.

**Failure classification drives the retry.** The handler returns `Err` — which is what makes the
job engine retry with its backoff — only for failures that a later attempt can fix: a network
error, a timeout, a `429`, or a `5xx` from Discord. A `400` (malformed embed), a `404` (the
webhook was deleted in the Discord UI) or a `401/403` are **terminal**: the handler marks the
claim failed with the reason and returns `Ok`, so the job stops instead of burning its retry
budget on something no attempt will fix. Terminal failures are logged at `error` and counted, so
a deleted webhook surfaces as an alert rather than as silence. The retry itself can never
double-post: the claim row already holds the dedup key, so a re-attempt of a request that
actually reached Discord stops at the claim.

### D6 — One port method returns the **already-gated** subset

Rather than exposing the Discord consent as a readable flag that every call site must combine
with `listable_profiles`, `UserPort` gains a single fail-closed method returning the subset of
ids that may be **named on Discord** (consent on **and** publicly listable **and** age-eligible,
evaluated in UTC with the existing one-day margin). Call sites cannot assemble the two checks
incorrectly, which is the same reasoning that produced `listable_profiles`.

Storage: an additive column on the user profile in the `user_account` schema, migration
`backend/user/migrations/0008_discord_visibility.sql`, `NOT NULL DEFAULT false`. It is covered by
the existing `purge_user` erasure job.

### D7 — Two tiers, one digest, one throttle, one aggregate minimum

`discord_notify` handles the immediate tier (accepted catalog item, season record). **Releases
are not in it**: release-please already produces the version and its notes in CI, so the
announcement is posted by the workflow that builds the release
(`scripts/discord/release_announce.sh`), after the platform jobs have attached their artifacts —
announcing at release-creation time would link a page with no downloads. Routing that through
the backend would mean opening an authenticated ingress for CI and would gain nothing; the
trade-off accepted is that the back-office kill-switch does not cover release announcements,
whose off switch is the repository secret or the job itself.
`discord_digest` is a **scheduled** job (seed in `backend/jobs/migrations/…_seed_discord_digest_schedule.sql`,
following the existing schedule seeds) that aggregates the previous closed period and posts one
message **per product**, into that product's stats channel. The **cadence is per product and
flag-driven**, not global: a product with little traffic reports weekly and one with real volume
reports daily, so a channel never publishes "3 players today". Long rankings are **pulled, not
pushed** — the digest carries a top 10, the full top 50 goes to the product's leaderboard channel
on a weekly cadence and to a slash command on demand (D3). Named announcements are throttled per player over a flag-configured window; aggregate
figures below a flag-configured minimum player count are omitted, so an aggregate can never
implicitly identify one person. Suppressions are counted/logged — never silent.

**An empty report is not published at all.** The rendering core answers "is there anything to
say?" *before* the sender is called: if every publishable element is zero or suppressed, the job
completes successfully having posted nothing. This is a **product** rule, not a technical one — a
channel showing "0 players, 0 sessions" week after week actively discourages the community it
exists to grow, and a dash-filled table is worse than silence. The corollary is that the
`aggregate minimum` and this rule compose: with `k = 5`, a period where 3 players played yields
all-suppressed figures, hence no message. A report is published as soon as **one** substantive
element survives — an accepted catalog item counts, since an item is not a person and is never
suppressed. Every skip is logged and counted with its reason, so "nothing happened" stays
distinguishable from "the digest is broken", which is exactly the failure mode silence would
otherwise hide.

### D8 — Flags: kill-switch **defaults off**, one flag per category

`FlagService` carries a global `discord.enabled` (default **off**, so the code deploys dark) plus
one flag per **product-namespaced** category (`discord.music.daily_report`,
`discord.id.weekly_report`, …), read at publication time. The namespacing means one product's feed
can be muted without touching another's — the operational unit matches the server's sections. Turning the kill-switch off suppresses jobs
that are already enqueued.

### D9 — Announcement language: one configured server locale, default English

Discord channels are shared by all locales, so announcements are rendered in a single
configured locale (default English) rather than per-user. The rendering core takes the locale as
input, so a second server or a localized channel later needs no structural change. App-side
copy (consent toggle, community entry point) stays fully localized in the four app locales.

### D10 — Account↔Discord link by one-time code, not OAuth

Granting a Discord role requires knowing the member's Discord user id. v1 links accounts with a
**one-time code**: the app shows a short-lived code, the player runs the link slash command in
Discord, and the interaction handler resolves the code to the account and stores the Discord
user id. No OAuth client, no redirect, no browser round-trip.

*Alternative considered*: Discord OAuth2 (`identify`) from the site or app. Better UX, but it
adds an OAuth client, a redirect surface and secret handling for a v1 whose only consumer is one
role. Reconsider when roles multiply.

## Risks / Trade-offs

- **Channel flooding kills the community** → digest tier + per-player throttle + aggregate
  minimum + kill-switch defaulting to off; volume is reviewed after the first enabled category
  before enabling the next.
- **Re-identification through small aggregates** ("1 new player today" next to a visible
  arrival) → configured minimum player count below which a figure is omitted.
- **Leaked webhook URL lets anyone post as Cymbra** → treated as a secret (env only, never
  logged, never in error messages), one URL per channel so the blast radius is one channel,
  rotation is a config change with no redeploy.
- **The interactions endpoint is public by construction** → signature verification before any
  side effect, `401` on failure, and no state mutation on unverified input; it is also a
  denial-of-service surface, so it stays cheap and rate-limited.
- **Publishing personal data is legally irreversible** → dedicated opt-in, forward-only
  revocation stated in the app copy *before* opting in, privacy documentation updated in both
  languages, consent erased with the account.
- **Discord is a third party** (availability, ToS, rate-limit changes) → announcements are
  best-effort via retries and DLQ; no product behaviour depends on Discord succeeding.
- **Stale rendering data** → the worker reads at publication time, so a message can describe
  state that changed seconds later; acceptable for announcements, and the consent gate is
  re-checked at that same moment.
- **Migration number collision** with the unmerged `add-push-notifications` branch, which also
  adds `backend/user/migrations/0008_*` → whichever lands second is renumbered on rebase (this
  has already happened once in this repo, `0012 → 0015`).
- **Coverage**: a new workspace member drags the global gate; the pure core must be thoroughly
  tested and the two HTTP adapters added to the ignore regex in the same change, or CI fails.

## Migration Plan

1. Land the code with `discord.enabled` **off** and no Discord configuration in the
   environment: every announcement path is a no-op, the interactions route rejects everything it
   cannot verify. Fully inert.
2. Apply the additive migrations (`discord` schema, user consent column, digest schedule seed).
   All additive; no destructive step, no backfill.
3. Create the Discord application and webhooks; put the webhook URLs, bot token and application
   public key in the deployment environment; register the interactions endpoint URL with the
   application.
4. Enable **one** immediate category, observe message volume and content for a few days.
5. Enable the digest, then the remaining categories one at a time.
6. Ship the app-side consent toggle and community entry point once the server is presentable.

**Rollback**: flip `discord.enabled` off (no redeploy). For a hard stop, delete the webhooks in
Discord — the sender degrades to a logged failure and the DLQ absorbs it. The schema additions
are inert if the code is reverted; the consent column keeps its `false` default.

## Open Questions

- Which slash commands ship in v1 beyond linking? A stats/leaderboard lookup is the obvious
  candidate, but each command adds a public disclosure surface subject to D6's gate.
- Which channel receives which category, and how many channels the server starts with — depends
  on the server structure, which is decided outside this change.
- Should the season-record announcement name the player at all, or only the piece and the
  figure? Naming is the point of a community feed, but the piece-only variant needs no consent
  at all and could ship before the toggle.
- Does the digest belong in a public channel or a low-traffic "stats" channel? A daily bot
  message in the main channel competes with human conversation.
