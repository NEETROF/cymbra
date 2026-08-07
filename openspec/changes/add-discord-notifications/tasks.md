## 1. Crate skeleton + pure core (D1)

- [ ] 1.1 Create `backend/discord` (`cymbra-discord`) as a workspace member: `Cargo.toml` with workspace deps (`async-trait`, `serde`, `thiserror`, `tracing`, `reqwest`, `sqlx`), Apache header, and a `README.md` stating the port/core split
- [ ] 1.2 Define the event model in a pure module: `AnnouncementEvent` (kind + subject ids + dedup key) and `EventCategory` (immediate tiers + digest), with the **deny-list encoded in the type** so authentication and non-accepted moderation states are not representable
- [ ] 1.3 Define the `DiscordSender` port (`publish`, `grant_role`, `revoke_role`) + error type; add `#[automock]` for test doubles per the `rust-testing` convention
- [ ] 1.4 Implement pure `(product, category) → channel` routing over a configuration map with product-namespaced categories (`music.*`, `id.*`, later `live.*`); an unmapped pair resolves to "no channel" (no-op), never to a default channel, so one product can never post in another's section
- [ ] 1.5 Implement pure message rendering per event kind, locale-parameterised (D9), with a rendering test per kind asserting no id/email leaks into the body
- [ ] 1.6 Implement the pure throttle + aggregate-minimum arithmetic (per-player window, minimum player count below which a figure is omitted) with unit tests on the boundaries
- [ ] 1.7 Unit-test the pure core to the coverage bar and add the HTTP adapter paths to the `--ignore-filename-regex` list used by `cargo llvm-cov` (CI + CLAUDE.md snippet)

## 2. Consent + gate (D6)

- [ ] 2.1 Write `backend/user/migrations/0008_discord_visibility.sql`: additive `NOT NULL DEFAULT false` consent column on the user profile (renumber if `add-push-notifications` lands first)
- [ ] 2.2 Add the single fail-closed `UserPort` method returning the subset of ids **nameable on Discord** (consent ON **and** publicly listable **and** age-eligible, UTC with the existing one-day margin); implement it in the Postgres repo
- [ ] 2.3 Unit-test the gate exhaustively: consent-only, listable-only, both, unknown id, private profile, not-yet-eligible, eligible-today boundary — each asserting exclusion by default
- [ ] 2.4 Expose read/write of the consent on the account RPC surface (proto + service + tests); writing it MUST NOT change profile visibility, and vice versa
- [ ] 2.5 Verify the consent column is erased by the existing `purge_user` job and add a regression test for it

## 3. Publication pipeline (D4, D5)

- [ ] 3.1 Create `backend/discord/migrations/0001_init.sql`: `discord` schema + `published_announcements` (unique dedup key, status, timestamps) and the role the worker uses to write it
- [ ] 3.2 Implement the claim → post → mark-published sequence over that table, including re-claim of a stale claim after the grace period; integration-test the double-execution and crash-between-claim-and-post paths
- [ ] 3.3 Add `DISCORD_NOTIFY` and `DISCORD_DIGEST` name constants + `JobSpec`s + their `Channel` to `cymbra_jobs::registry`
- [ ] 3.4 Implement `WebhookDiscordSender` (reqwest, per-channel webhook URLs, no secret in logs or errors) behind the port
- [ ] 3.5 Add `discord: Option<Arc<dyn DiscordSender>>` to `WorkerCtx` and the `#[sqlxmq::job("discord_notify")]` handler: load event → re-evaluate flags + gate → render → publish; `None` sender or missing channel = successful no-op
- [ ] 3.6 Implement the `discord_digest` handler: one message **per product** into that product's stats channel, at that product's flag-driven cadence, from the existing daily aggregates, with the aggregate minimum applied; seed its schedule in `backend/jobs/migrations/`
- [ ] 3.7 Build the Music report content: active players, sessions, new accounts, scores rated (and how many reached consensus), catalog items accepted, top 10 pieces played — the top-pieces query MUST join `music.catalog_scores` and count only accepted catalog pieces, since `play_sessions.score_id` also holds **user** score ids and would otherwise publish a private upload's identity
- [ ] 3.8 Build the top-50 surfaces: weekly post in the product's leaderboard channel and an on-demand slash command, both reusing the same pure ranking core as the top 10
- [ ] 3.9 Test handler idempotency, the flag-disabled-after-enqueue path, the gate-revoked-after-enqueue path, per-product routing (a Music event never resolves an ID channel), and Discord-failure retry behaviour

## 4. Producers (D4)

- [ ] 4.1 Enqueue `discord_notify` on soundfont acceptance, **inside the existing transaction**, and assert no announcement is enqueued for propose/reject transitions
- [ ] 4.2 Enqueue `discord_notify` on score-proposal acceptance, same transactional guarantee and same negative assertions
- [ ] 4.3 Enqueue `discord_notify` when a season record is beaten, reusing the existing season-best ingest hook
- [ ] 4.4 Add a rollback test per producer proving a rolled-back domain write leaves no enqueued announcement
- [ ] 4.5 Assert no producer performs Discord I/O on the request path (the `Enqueuer` port is the only seam used)

## 5. Flags (D8)

- [ ] 5.1 Register the `discord.enabled` kill-switch (**default off**) and one flag per product-namespaced category (`discord.music.*`, `discord.id.*`) plus each product's report cadence (daily/weekly) in the flag definitions
- [ ] 5.2 Read the flags at publication time in both handlers; kill-switch off suppresses every category, a category flag off suppresses only its own
- [ ] 5.3 Add the flag descriptions/copy so they are self-explanatory in the back-office flags console (no new screen)

## 6. Bot: interactions endpoint + roles (D3, D10)

- [ ] 6.1 Add `POST /discord/interactions` to the existing HTTP server with Ed25519 verification over `timestamp || body` **before** any parsing with side effects; `401` on missing/invalid signature, `PONG` on `PING`
- [ ] 6.2 Test the endpoint: valid signature accepted, tampered body rejected, stale timestamp rejected, missing headers rejected, `PING` answered — and assert no side effect on every rejection path
- [ ] 6.3 Implement the account-link slash command: short-lived one-time code issued by the app, resolved by the handler to store the member's Discord user id (single-use, expiring, rate-limited)
- [ ] 6.4 Implement `BotRestDiscordSender` role grant/revoke (idempotent: granting an existing role or revoking an absent one succeeds) with the bot token from the environment
- [ ] 6.5 Drive the role from account state (grant on qualifying, revoke on losing it) via a job, and test the idempotent repeat
- [ ] 6.6 Apply the D6 gate to any player-data answer a command returns, and return the same neutral "not available" answer for private, not-eligible, and unknown players
- [ ] 6.7 Document all Discord secrets in `backend/.env.example` (per-channel webhook URLs, bot token, application public key) and assert missing config disables rather than degrades

## 7. App: consent toggle + community entry point

- [ ] 7.1 Add the Discord-visibility toggle to the account settings via a Riverpod notifier calling the account service (UI never calls the service directly), default off
- [ ] 7.2 Write the toggle copy stating **before** opting in that already-published Discord messages stay published, and add ARB strings for `en`/`fr`/`es`/`it`
- [ ] 7.3 Add the flag-gated community entry point opening the stable `cymbra.app/discord` redirect through the existing launcher seam, with no invite code anywhere in the app
- [ ] 7.4 Widget-test the toggle (optimistic state, failure path shows a localized message, never a raw error) and the entry point (hidden when the flag is off, target asserted through the injected launcher)

## 8. Legal + docs

- [ ] 8.1 Document the Discord publication, the consent, and its forward-only irreversibility in `docs/legal/politique-de-confidentialite.md` and `docs/legal/privacy-policy.md`
- [ ] 8.2 Write `backend/discord/README.md`: event categories, the deny-list, the gate, the flags, and the operational runbook (rotate a webhook, hard-kill, read the DLQ)

## 9. Verification

- [ ] 9.1 `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo llvm-cov --workspace --fail-under-lines 80` green with the new crate included
- [ ] 9.2 `melos run analyze`, `dart run custom_lint`, `dart format`, and `flutter test --coverage` green with the app changes
- [ ] 9.3 `openspec validate add-discord-notifications --strict` passes
- [ ] 9.4 Manual: deploy dark (flags off, no config) and confirm every announcement path is inert and the interactions route rejects unverified requests
- [ ] 9.5 Manual: configure a test Discord server, enable one immediate category, verify the message content, then verify the digest and the throttle over a day
