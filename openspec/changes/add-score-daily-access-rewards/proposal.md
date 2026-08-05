## Why

The score catalog is our most valuable asset, yet today any authenticated user can
open (and thereby download the raw MusicXML of) unlimited accepted catalog pieces
via `GetCatalogScoreBytes`. We want a **freemium** shape: everyone can practise a
handful of pieces a day for free, deeper access is earned (curation points) or
bought (a coming monthly subscription), and a locked piece never ships its
MusicXML — it is teased by a short **server-rendered audio clip** so the user knows
what they'd unlock. This extends the reward economy (`add-curation-rewards`) from
soundfonts to the core content, and reuses the backend audio-render engine
introduced by `add-soundfont-entitlement-previews`.

## What Changes

- **Daily free quota on full opens** — a user may open **N distinct catalog pieces
  per day** for free (full play-along). The set of pieces opened *today* is
  remembered, so **re-opening a piece already opened today is free** (leaving and
  coming back mid-day never re-charges). The day resets at local midnight (the
  client-offset date, as `play_core.rs` already does for play activity).
- **Points unlock an extra piece for the day** — opening a piece **beyond** the free
  quota costs curation points, **at open time and with an explicit confirmation**.
  Spending adds that piece to today's access set (a *consumable day-slot*, **not** a
  permanent grant): the points spend is a normal ledger debit, but the access it
  buys expires at midnight like the free quota.
- **Subscription bypass (seam only)** — the quota gate consults
  `has_active_subscription(caller)`; a subscriber has **unlimited** opens. The
  subscription system does not exist yet, so this returns `false` for now — the seam
  is in place so the future billing work plugs in one function without touching the
  quota logic. The design SHALL also record that, once subscriptions exist, the
  quota-reached / unlock moments become the **upsell surface**: the user is nudged to
  subscribe (a placeholder hook now, wired to the real offer later).
- **Locked pieces are audio-teased, never downloadable** — when a piece is not
  accessible (over quota, unpaid, unsubscribed), `GetCatalogScoreBytes` refuses the
  MusicXML; the app instead plays a short **audio-only** preview clip rendered
  server-side. The clip is **pre-rendered when a catalog piece is accepted** (reusing
  the backend `rustysynth` render path) and stored as a **public** object; a
  back-office "Generate sample" fallback regenerates it for pieces accepted before
  this change or after a failed render.
- **App & back-office surfacing** — the catalog shows a piece's access state
  (free-quota-remaining / owned-today / locked-needs-points / subscriber) with a
  strong unlock affordance; the locked play button auditions the audio clip; the
  back-office score screen gets the "Generate sample" action.

Out of scope: the subscription/billing system itself (only the seam); real money;
per-piece permanent ownership of scores (day-slots are consumable); showing the
notation of a locked piece (image render was dropped — audio-only teaser); changing
how accepted/free pieces already play.

## Capabilities

### New Capabilities
- `score-daily-access`: the freemium gate on `GetCatalogScoreBytes` — a per-day set
  of opened pieces, an N/day free quota, a points-funded consumable day-slot for an
  extra piece (spend at open, with confirmation), and a `has_active_subscription`
  bypass seam. Re-opening a piece already opened today is free; the set resets at the
  client-offset midnight.
- `score-audio-preview`: the audio-only teaser for a locked piece — pre-rendered at
  acceptance via the backend audio-render engine, stored as a public object,
  regenerable from the back office, served openly, and played by the app when a piece
  is locked.

### Modified Capabilities
<!-- None. The daily gate and audio teaser are new capabilities layered over the
     existing catalog delivery; the reward economy (curation-rewards) is reused, not
     changed. -->

## Impact

- **Depends on** `add-curation-rewards` (points ledger + spendable balance) and
  `add-soundfont-entitlement-previews` (the backend `rustysynth` render engine — the
  score audio clip reuses it).
- **Backend**: a daily-access core (quota + today's set + reset) gating
  `GetCatalogScoreBytes`; a points-debit for a day-slot (ledger write + day-slot
  record); `has_active_subscription` stub seam; score audio-preview render at accept +
  public storage + delivery RPC + admin regenerate. New per-user/day state
  (opened-today set, day-slots).
- **App** (`apps/music`): access-state on catalog items, unlock confirmation flow,
  locked-piece audio audition, "free opens left today" affordance.
- **Back office**: "Generate sample" action + a "no sample" filter (list pieces
  missing a preview) on the score admin screen.
- **Coverage**: Rust ≥ 80% for the daily-access core (quota/set/reset, day-slot debit
  decision) and the render/sample helpers (host-testable); gRPC/route/synth glue
  coverage-excluded as usual. App ≥ 80% via fakes; the Vue action under its test setup.
