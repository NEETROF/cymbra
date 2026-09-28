# cymbra-discord

Discord announcements from the backend (change: `add-discord-notifications`).
Release announcements are **not** here — CI posts them (`scripts/discord/release_announce.sh`).

## What is announced

| Event | Category | Channel | Content |
|---|---|---|---|
| A score accepted into the public catalog | `music.catalog` | `#scores-and-soundfonts` | title, composer |
| A SoundFont accepted into the public catalog | `music.catalog` | `#scores-and-soundfonts` | name, keyboard/drum kit, licence, credit |
| A season record beaten on a catalog piece | `music.records` | `#music-leaderboards` | piece, mode, figure — **never the player** |

**Catalog acceptances are grouped.** Each acceptance enqueues its job 10 minutes later; the
job announces, in **one** message, every item accepted in the last 24 hours that nobody
announced yet. Accepting 500 scores in a row yields one or two messages ("🎼 500 new scores
in the catalog", the first ten named, "…and 490 more"); every later job of the burst finds
nothing left. A lone acceptance keeps its detailed message. Each item is still announced
once in its life, whichever message carries it.

A record is announced only when a result beats a best held by **another** player: the first
result on a piece is not a record (the start of a season would flood the channel), and a
holder improving their own record has not beaten anyone. At most one record message per
piece, mode and UTC day.

**The deny-list is the type.** `AnnouncementEvent` has no variant for authentication,
pending or rejected content, or anything naming a player, so no producer can enqueue one.
Naming players needs a dedicated Discord consent — a later tranche.

## How it works

```
music write ──(same transaction, savepoint)──▶ discord_notify job ──▶ Publisher
                          (catalog: +10 min)                            │
  kill-switch → category flag → channel configured
  → catalog: every recent accepted item not yet announced / record: its piece
  → claim the dedup keys → one post → settle them all
```

- **Port / core split.** `event`, `routing`, `render`, `flags` and `publish` are pure and
  unit-tested against `mockall` doubles of the `ports`. `webhook.rs` and `pg.rs` are thin
  I/O, excluded from the coverage gate; `pg.rs` is covered by `#[ignore]`d integration
  tests (`tests/pg_it.rs` here, `backend/music/tests/discord_producers_it.rs` for the
  producers).
- **Producers enqueue an event, not a message.** The worker re-reads the subject and the
  flags when it posts, so an item rejected again, or a flag turned off, after the enqueue
  publishes nothing.
- **Best-effort enqueue.** `pg::enqueue_notify` runs in a savepoint of the domain
  transaction: the job exists iff the write commits, and a failed enqueue never aborts the
  moderation decision or the season best.
- **At most once.** Each announcement claims its dedup key in `jobs.discord_announcements`
  before posting. The table lives in `jobs` because the worker (`worker_svc`, owner of that
  schema) is its only reader and writer.
- **Retries.** Only a refused connection, a `429` or a `5xx` returns `Err` (the job retries
  with backoff; the claim is released). A `400/401/403/404`, or a timeout after the request
  left (it may have been posted), is recorded as `failed` and not retried — a deleted
  webhook cannot burn the retry budget, and a retry cannot double-post.
- **Pools.** The ledger uses the worker's queue pool (`worker_svc`); the subject reads use
  `admin_svc`, the worker's cross-schema reader, with schema-qualified names.
- **Escaping.** User-supplied titles are escaped (no links, headings or mentions), and every
  post disables mention parsing and link embeds.

## Configuration

| Setting | Where | Default |
|---|---|---|
| `DISCORD_WEBHOOK_SCORES_AND_SOUNDFONTS`, `DISCORD_WEBHOOK_MUSIC_LEADERBOARDS` | worker env (from `scripts/discord/.webhooks.env`) | unset ⇒ job is a no-op |
| `CYMBRA_DISCORD_LOCALE` | worker env | `en` (`fr` supported) |
| `discord.enabled` | back-office flags | **off** |
| `discord.music.catalog`, `discord.music.records` | back-office flags | **off** |

## Runbook

- **Turn it on**: put the two webhooks in the worker environment, restart the worker, then
  enable `discord.enabled` and **one** category in the back office. Watch the volume for a
  few days before enabling the next.
- **Mute**: turn `discord.enabled` (everything) or a category flag off. Jobs already queued
  post nothing.
- **Hard stop / leaked webhook**: delete the webhook in Discord (Server settings →
  Integrations → Webhooks). Posts then fail with `404` and are recorded, not retried.
  Re-run `scripts/discord/provision.sh` to create a new one and update the environment.
- **What failed**:
  `SELECT dedup_key, reason, settled_at FROM jobs.discord_announcements WHERE status = 'failed' ORDER BY settled_at DESC;`
  Retries in progress and the dead letters are in the back-office Jobs console
  (`discord_notify`).
