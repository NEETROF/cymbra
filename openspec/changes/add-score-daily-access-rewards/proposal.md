## Why

The score catalog is our most valuable asset, yet today any authenticated user can
open (and thereby download the raw MusicXML of) unlimited accepted catalog pieces
via `GetCatalogScoreBytes`. We want a **freemium** shape: everyone can practise a
handful of pieces a day for free, deeper access is earned (curation points) or
bought (a coming monthly subscription), and a locked piece never ships its
MusicXML — it is teased by a short **server-rendered audio clip** so the user knows
what they'd unlock.

The reward economy this plugs into has grown since the idea was first written
(2026-08-05): points are now earned by rating (`curation-rewards`), by **playing and
practising** (`music-play-rewards`), spent in the piano shop (`reward-unlocks`,
`soundfont-pricing`) and on a **confirmed streak freeze** (`practice-streak`), and
the backend already renders audio previews headlessly (`soundfont-preview`). This
change adds the core content as the next points sink and reuses those pieces
instead of introducing parallel mechanisms.

## What Changes

- **Daily free quota on full opens** — a user may open **N distinct catalog pieces
  per day** for free (full play-along). The set of pieces opened *today* is
  remembered, so **re-opening a piece already opened today is free** (leaving and
  coming back mid-day never re-charges). The day is the **server day**, the same
  clock as the play-award daily cap — deliberately *not* the client-offset day, which
  a device-clock change would reset (see design D1). The app shows when the quota
  resets.
- **Points unlock an extra piece for the day** — opening a piece **beyond** the free
  quota costs curation points, **at open time and with an explicit confirmation**
  (the streak-freeze pattern: never a silent debit). Spending adds that piece to
  today's access set (a *consumable day-slot*, **not** a permanent grant): the spend
  is a normal ledger debit, idempotent on a ledger key, but the access it buys
  expires with the day like the free quota.
- **Who is never gated** — the gate applies to the authenticated **player-open** of a
  *catalog* piece only. Bundled scores, a user's own uploads (`GetScoreBytes`), the
  contributor of an accepted user-proposed piece opening their own contribution, and
  the back-office / moderation audiences are outside the quota by construction.
- **Subscription bypass (seam only)** — the gate consults a `SubscriptionSource`
  seam; a subscriber has **unlimited** opens. No billing exists yet, so it returns
  `false`; the seam is in place so the future billing work plugs in one implementation
  without touching the quota logic. The quota-reached / unlock moments are recorded
  as the future **upsell surface** (the reward shop already lists a "temporary premium
  access — coming later" item; the app's placeholder points at it).
- **Locked pieces are audio-teased, never downloadable** — when a piece is not
  accessible (over quota, unpaid, unsubscribed), the bytes RPC refuses the MusicXML and
  answers with the locked state (cost, remaining quota, reset time); the app instead
  plays a short **audio-only** preview clip rendered server-side. The clip is rendered
  by a **background job enqueued when a catalog piece is accepted** (reusing the
  backend `rustysynth` render path and the shared `musicxml-core` playback schedule),
  stored beside the score bytes and served over the same authenticated HTTP shape as
  the SoundFont previews; a back-office "Generate sample" action re-renders on demand
  and an ops backfill enqueues the already-accepted corpus.
- **Offline cache stays honest** — a cached favourite is playable **offline** (that is
  the cache's purpose), but **online** the server decides first: the app asks before
  playing from cache and a locked answer never plays the cached bytes (design D5).
- **App & back-office surfacing** — the hub shows today's access state (free opens
  left, opened-today marks, locked/unlock cost, reset time); the locked open flow
  offers the audition + the confirmed unlock; the back-office score screen gets the
  "Generate sample" action and a "no sample" filter.

Out of scope: the subscription/billing system itself (only the seam); real money;
per-piece permanent ownership of scores (day-slots are consumable); showing the
notation of a locked piece (image render was dropped — audio-only teaser); changing
how bundled / user-upload / accessible pieces already play; a compressed audio
container for the clip (WAV like the SoundFont previews; a codec is a later option).

## Capabilities

### New Capabilities
- `music-score-daily-access`: the freemium gate on the catalog player-open — a
  per-user per-day set of opened pieces, an N/day free quota, a points-funded
  consumable day-slot for an extra piece (confirmed spend, ledger-keyed idempotency),
  a `SubscriptionSource` bypass seam, the exempt audiences, and the online/offline
  contract with the encrypted favourites cache.
- `music-score-audio-preview`: the audio-only teaser for a locked piece — rendered by
  a job at acceptance via the backend audio-render engine + `musicxml-core` schedule,
  stored beside the score bytes with a DB "rendered" marker, regenerable from the back
  office, backfillable, served openly (moderation visibility only), and played by the
  app when a piece is locked.

### Modified Capabilities
<!-- None. The gate and teaser layer over the existing catalog delivery; the reward
     economy (curation-rewards / reward-unlocks / music-play-rewards) and the flag,
     job and storage platforms are consumed, not changed. -->

## Impact

**Products** (per `openspec/config.yaml`):

| Product | Consumed (unchanged) | New in this change |
|---|---|---|
| **Cymbra ID** | identity/roles (`AuthIdentity`, `BACKOFFICE_AUDIENCE`, music-scope roles), account purge | none (purge job must cover the new per-user day-access rows) |
| **Platform** | `runtime-feature-flags` (typed config, staff-only rollout, backend enforcement), `job-infrastructure` (transactional enqueue, worker handler), object storage, `observability` | 3 flag keys, 1 job spec, 1 migration |
| **Cymbra Music** | `curation-rewards` ledger + `reward-unlocks` spendable balance, `music-play-rewards` day/cap conventions, `practice-streak` confirmed-spend pattern, `soundfont-preview` render engine + HTTP preview shape, `shared-musicxml-crate` playback schedule, `catalog-access-limits` (abuse cap, stays first), `offline-score-cache`, `score-catalog-proposal` (`proposed_by`), `feature-usage-analytics` (client actions) | the two capabilities above; proto: locked bytes response, `GetCatalogDailyAccess`, `UnlockCatalogScoreForToday`, `CatalogHit.has_preview`; HTTP `GET/POST /scores/{id}/preview` |
| **Back office** | catalog store/filters, `soundfonts` "Generate sample" pattern (`Async<T>` + injectable transport) | "Generate sample" on scores + "no sample" filter |
| **Cymbra Live** | — | none |

- **Depends on** archived changes (now specs): `curation-rewards`, `reward-unlocks`,
  `soundfont-preview`, `music-play-rewards`, `runtime-feature-flags`; and in-flight
  `add-catalog-access-limits` (implemented), `add-offline-score-cache` (implemented),
  `add-practice-streak` (implemented) for the patterns it mirrors.
- **Coverage**: Rust ≥ 80% for the daily-access core (decision, day-slot debit
  decision), the preview sequence builder and the flag/subscription seams
  (host-testable); gRPC/route/synth/worker glue coverage-excluded as usual. App ≥ 80%
  via mockito mocks over the new services/notifiers; the Vue action + filter under
  Vitest through the client seam.
