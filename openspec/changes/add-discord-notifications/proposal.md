## Why

Cymbra has no place where its users meet each other: support and ideas live on GitHub
Discussions (a developer surface), and the app itself is single-player. Growing the user
community needs a Discord server that feels **alive without a human posting every day** —
which means the backend must be able to announce what happens in the product (a score
accepted into the catalog, a season record) and to act on Discord (grant a role, answer a slash
command). Release announcements are **not** part of this: a release is a CI event that
release-please already describes, so it is announced straight from the workflow that builds it
(`scripts/discord/release_announce.sh`), never through the backend.

Doing that naively would be harmful: publishing "player 1234 started playing" to a public
channel discloses pseudonymous personal data without consent, and a high-frequency event
stream would push the channel past Discord's rate limits and make it unreadable — the exact
opposite of the goal. This change therefore introduces the Discord surface **and** the
consent + rate discipline that make it safe.

## What Changes

- **New outbound port + crate** `cymbra-discord` (`backend/discord`), modelled on the
  existing `EmailSender` port: a `DiscordSender` trait (publish a message, grant/revoke a
  role), a webhook-backed implementation for announcements, a bot-REST implementation for
  roles, and a **pure host-testable core** for event selection and message rendering.
- **Announcements are enqueued, never inline**: producers enqueue a `discord_notify` job
  through the existing `Enqueuer` port **in the same transaction** as the domain write, so a
  rolled-back write can never produce a phantom announcement, and retries/DLQ come for free.
  A scheduled `discord_digest` job folds high-frequency activity into one daily message.
- **Two event tiers, one explicit deny-list**:
  - *immediate*: score/soundfont **accepted** into the public catalog, season record beaten;
  - *daily digest*: session counts, new-player counts, best tempo of the day;
  - **never announced**: sign-ins, `pending`/`rejected` moderation state, emails, raw ids.
- **A dedicated Discord-visibility consent**, separate from profile visibility. A player is
  named on Discord only when they opted in to Discord **and** are publicly listable
  (`listable_profiles`: public profile + age-eligible). Without the opt-in, the player only
  ever appears inside anonymous aggregates. Revocation is **forward-only** — already
  published messages cannot be retracted — and the app must say so.
- **A Cymbra Discord application without a gateway**: slash commands arrive on a signed
  interactions endpoint (`POST /discord/interactions`, Ed25519 verification) served by the
  existing HTTP server, and role grants are plain REST calls from the worker. No long-lived
  gateway process to host.
- **Kill-switch and per-category flags** through `FlagService`, so the feed can be muted or
  retuned from the back office without a redeploy.
- **The invite URL is never hardcoded**: the app opens a stable redirect
  (`cymbra.app/discord`) behind a runtime flag, so the invite can be rotated or the entry
  point withdrawn without shipping a release.
- **Beta access is claimed on Discord, never distributed as a list of codes**: a `/beta`
  slash command, restricted to a configured channel (and optionally a role), lets a member
  claim **one** beta access per Discord account per campaign — the channel decides the campaign
  (a premium trial such as `#beta-premium`, or a feature beta such as `#beta-midi-drums`). The handler asks the access-code
  capability (`music-access-codes`, introduced by `add-premium-subscription`) to mint a
  **single-use** code and answers with an **ephemeral** message holding the web redeem link;
  a member whose Cymbra account is already linked (D10) is granted directly and told so. The
  command mints only **free, campaign-bounded** access — never a discount and never a paid
  unlock — so it stays outside the stores' in-app-purchase rules; the store builds contain no
  code-entry field, redemption happens on the web.

## Capabilities

### New Capabilities
- `discord-announcements`: backend-driven publication of product events to a Discord server —
  event categories and the deny-list, the immediate/digest split, the consent + age gate on
  named announcements, per-player throttling, idempotent at-least-once delivery, and the
  kill-switch.
- `discord-bot-actions`: the Cymbra Discord application — signature-verified interactions
  endpoint for slash commands, Discord role grants/revocations driven by Cymbra account
  state, the `/beta` claim command (one free, campaign-bounded access per member, delivered
  ephemerally), with least-privilege bot permissions.
- `community-invite-entry`: the flag-gated in-app entry point to the community, resolved
  through a stable redirect rather than a hardcoded invite link.

### Modified Capabilities
- `public-player-profile`: adds a **Discord-visibility consent** that is independent of the
  public/private profile setting and **cumulative** with it — being publicly listable does
  not authorise a Discord announcement, and the Discord opt-in does not make a profile
  public. Fail-closed, forward-only on revocation, and erased with the account.

## Impact

- **New crate**: `backend/discord` (`cymbra-discord`) — a workspace member, so it counts
  toward `cargo llvm-cov --workspace --fail-under-lines 80`; the HTTP glue stays thin and is
  added to the coverage ignore regex, the selection/rendering core is fully tested.
- **Existing backend**: a job-name constant and `Channel` in `cymbra_jobs::registry`; two
  handlers plus an optional `discord` field in `WorkerCtx` (absent configuration = no-op,
  mirroring `storage`); one scheduled job; one new HTTP route on the server; producer call
  sites in the score/soundfont moderation and leaderboard-season paths.
- **Data**: one migration adding the Discord-consent column to the `user_account` schema,
  purged by the existing `purge_user` erasure job.
- **App (`apps/music`)**: a consent toggle plus its explanatory copy, the community entry
  point, and ARB strings for the four locales (en/fr/es/it).
- **Back office**: the new flags appear in the existing flags console; no new screen.
- **Legal**: the Discord publication and its irreversibility must be documented in
  `docs/legal/politique-de-confidentialite.md` and `docs/legal/privacy-policy.md`.
- **Secrets/ops**: per-channel webhook URLs, the bot token, and the application public key as
  environment variables documented in `backend/.env.example` — never in code. Deployment must
  register the interactions endpoint URL with the Discord application.
- **External dependency**: Discord's API and its rate limits (~5 requests/second per webhook,
  ~30 messages/minute per channel) become a runtime constraint; the digest exists to stay far
  below them.
- **Cross-change dependency**: the `/beta` command consumes the access-code issuing port
  defined by `add-premium-subscription` (`music-access-codes`: campaigns, single-use codes,
  one claim per external identity per campaign). Until that change lands, the command is
  registered but answers "beta not open" — the interactions endpoint, linking and roles do
  not depend on it. The Discord change records the `discord_user_id → code` claim so the
  cohort of beta testers is known without any Cymbra account link.
