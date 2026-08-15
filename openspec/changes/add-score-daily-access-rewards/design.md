## Context

Catalog scores are delivered over gRPC `ScoreService`. A player open pulls the raw
MusicXML via `GetCatalogScoreBytes` (`backend/music/src/grpc.rs:1047`), which today
does: identity → `guard_download` (the `catalog-access-limits` burst + engagement-aware
volume cap, `catalog_limits.rs:120`) → `ScoreModule::catalog_bytes_for_player`
(`module.rs:994`), which records the **coverage engagement signal** (post-play rating
eligibility) and then serves the bytes — honouring the offline cache's **conditional
fetch** (`if_none_match` → `unchanged: true`, `module.rs:925`). The app renders and
drives play-along on the client from that XML.

What exists now that the first draft (2026-08-05) assumed absent or different:

- **Audio synthesis exists server-side**: `backend/music/src/soundfont_synth.rs`
  (`render_preview_pcm`, `render_preview_wav`) + the pure `soundfont_preview.rs`
  (`SampleSequence`, `encode_preview`, `preview_object_key`). SoundFont previews are
  rendered **inline** at upload/regenerate and served over **HTTP**
  (`GET/POST /soundfonts/{id}/preview`, `backend/server/src/soundfont.rs:245`),
  authenticated, moderation-visible, no entitlement gate; `has_preview` is computed
  per row with `store.size(...)` in the font listing.
- **MusicXML → timed notes exists in Rust**: `crates/musicxml-core/src/playback.rs`
  (`schedule(&ScoreDocument) -> PlaybackSchedule { notes: Vec<TimedNote>, … }`),
  already a dependency of `cymbra-music`; today consumed only by the back-office wasm.
- **Points ledger** (`music.curation_points`): `award_kind ∈ {coverage, honesty,
  adjustment, redeem, performance, practice}`, plus `award_key` + `piece_id` (migration
  0025) with the unique partial index `(user_id, award_key)`. Debits: the shop redeem
  is guarded by the *grant* PK (`curation_grants`), the **streak freeze** is the atomic
  "lock → re-read balance → `INSERT … ON CONFLICT (user_id, award_key) DO NOTHING`"
  transaction (`pg_streak.rs:108`, key `streak_freeze:<day>`). Spendable balance =
  `SUM(amount)` (`pg_curation_rewards.rs:380`); lifetime excludes redeems.
- **Day conventions**: the play-award daily cap is the **server day**
  (`date_trunc('day', now())`, `pg_curation_rewards.rs:302` — "deliberately NOT the
  player's local day: a tz-keyed cap would let a farmer reset their allowance by
  changing the device clock's offset"); the practice award and the streak use the
  **player's local day** (`play_core::local_day`) because they are about *their* day.
- **Flags**: the music crate does not depend on `cymbra-feature-flags`; hot config is
  read at call time through a trait seam implemented at the composition root
  (`StreakConfigSource` → `backend/server/src/flags.rs:108`). Keys are declared in
  `backend/feature-flags/src/registry.rs` (`<area>.<sub>.<name>`, `.enabled` for
  booleans, safe default off, rollout global/staff-only).
- **Jobs**: `cymbra-jobs` `EnqueueRequest::for_job(spec, payload, …)` +
  `transactional_enqueue(&mut tx, …)` (`backend/jobs/src/engine.rs:74`), worker
  handlers in `backend/worker/src/handlers.rs`, specs in `jobs/src/registry.rs`.
- **Offline cache** (`add-offline-score-cache`): a cache hit is **authoritative and
  plays with no network round-trip** (`notation_notifier.dart:75`); a favourited piece
  is cached on its first successful open; online, a best-effort post-serve refresh
  re-calls `GetCatalogScoreBytes` with `if_none_match`.
- **Guests** never reach the catalog (`welcome-onboarding`: bundled scores only,
  `canUseOnlineServicesProvider` false); the hub, deck and community are sign-in gated.
- **Contributors**: an accepted user-proposed piece keeps its private `user_scores`
  row; the contributor opens it via `GetScoreBytes` (owner-scoped, ungated) unless they
  re-find it through catalog search (`catalogId` path). `catalog_scores.proposed_by`
  identifies them.
- **Back office** downloads catalog MusicXML through the same `GetCatalogScoreBytes`
  (audience `back-office`, exempt from the access limiter — `catalog_limits.rs:100`).
- **Redeem UX**: shop redeem is fire-and-observe with **no confirmation**; the streak
  freeze shows a confirmation dialog (`streak_listener.dart:103`) — the pattern this
  change follows for a *consumable* spend.

Adjacent prior art we build beside: `add-catalog-access-limits` (abuse cap — a
**separate** concern; both gate the same RPC, abuse first), `reward-unlocks` (which
already lists a future, non-redeemable "temporary premium access" shop item — the
natural placeholder for the subscription upsell).

## Goals / Non-Goals

**Goals:**
- A free user may fully open **N distinct catalog pieces per day**; re-opening a
  piece already opened today is free; the day resets on the server day.
- Opening an extra piece beyond the quota costs curation **points at open time**,
  after confirmation, and grants access **for today only** (consumable day-slot).
- A locked piece never ships its MusicXML — online, even from the offline cache — and
  is teased by a short **audio-only** server-rendered clip.
- A clean `SubscriptionSource` **seam** that later lifts the quota entirely, plus a
  recorded **upsell** hook at the quota/unlock moments.
- Everything tunable live (flags), off by default, staff-only rollout first.

**Non-Goals:**
- The subscription/billing system (only the boolean seam + upsell placeholder).
- Money, real DRM, per-user watermarking.
- Permanent ownership of scores (day-slots are consumable; contrast soundfont grants).
- Showing a locked piece's **notation** (audio-only teaser); server-side notation render.
- Gating bundled scores, a user's own uploads, the rating deck, or moderation reads.
- A compressed audio container (WAV, as the SoundFont previews).

## Decisions

### D1. Day boundary = server day; config through a flag-backed trait seam

The quota day is the **server day** (`date_trunc('day', now())`), the clock the
play-award daily cap already uses and for the same reason: the quota is the
monetisation ceiling, and a client-offset day would hand out N more free opens per
device-clock change (24 "days" per real day). The player-local day stays where it
belongs (practice award, streak — "their day"). To keep the reset humane the access
state carries `resets_at` (epoch ms) and the app shows "réinitialisé dans Xh";
re-opens being free removes the midnight cliff for a piece in progress.

Config comes from three declared flag keys (`registry.rs`, app `music`, safe defaults):

| key | type | default | role |
|---|---|---|---|
| `catalog.daily_access.enabled` | bool | `false` | kill-switch — off = gate absent, every open serves as today (data kept) |
| `catalog.daily_access.free_quota` | int | `3` | N distinct free opens per day (`0` = every catalog open costs points) |
| `catalog.daily_access.day_slot_cost` | int | `20` | points for one extra piece today (`0` = free unlock, effectively no gate) |

Read at call time through `DailyAccessConfigSource` (music-crate trait, implemented in
`backend/server/src/flags.rs` over the flag service like `StreakConfigSource`), so the
music crate stays flag-free and values hot-reload. Rollout scope **staff-only** first
(existing registry rollout), then global.

*Alternative recorded*: player-local day (friendlier reset). Rejected for the farming
hole above; revisit only if reset-time complaints outweigh it.

### D2. Daily-access model, storage and the pure decision core

**State**: `music.catalog_day_access (user_id UUID, catalog_id UUID FK → catalog_scores
ON DELETE CASCADE, day DATE, paid BOOL NOT NULL, opened_at TIMESTAMPTZ, PRIMARY KEY
(user_id, catalog_id, day))` + index `(user_id, day)`. A row = "this piece is open for
this user today"; `paid` marks a day-slot bought with points. Rows are small and pruned
by retention (D11).

**Pure core** (`backend/music/src/catalog_daily_access_core.rs`, coverage-included):

```
pub struct DayState { opened: BTreeSet<CatalogId>, free_used: u32 }
pub enum Open { Serve /* already open today, or gate off/exempt */, ServeFree /* consume a free slot */,
                Locked { cost: i64, free_left: u32 /* 0 */, upsell: bool } }
pub fn decide_open(piece: &CatalogId, day: &DayState, cfg: &DailyAccessConfig,
                   caller: CallerKind /* Exempt | Subscriber | Contributor | Regular */) -> Open
//  !cfg.enabled | caller ∈ {Exempt, Subscriber, Contributor} -> Serve
//  piece ∈ day.opened                                        -> Serve
//  day.free_used < cfg.free_quota                            -> ServeFree
//  else                                                      -> Locked{cost: cfg.day_slot_cost, upsell: true}
```

`Exempt` = back-office audience or music-scope moderator/admin (scope-checked like the
access limiter's exemption, `has_role_in_scope("music", …)`, not the scope-agnostic
`allow_unvalidated` test) — reviewing is work, not consumption.
`Contributor` = `catalog_scores.proposed_by == caller` (a contributor never pays to open
their own accepted piece from the catalog). `Subscriber` = `SubscriptionSource` (D6).

**Where the gate lives**: inside `ScoreModule::catalog_bytes_for_player`, **before**
the conditional-fetch short-circuit and **before** `record_engagement` — so (a) a
locked piece can never answer `unchanged` (which the app treats as "play from cache"),
and (b) a locked open records no coverage engagement (no bytes, no play). Order in the
handler stays: `guard_download` (abuse) → daily-access decision → bytes. On
`Serve`/`ServeFree` the piece is upserted into `catalog_day_access` (idempotent —
re-open is a no-op, `paid=false` unless a paid row already exists). `ListRatingDeck` /
`GetRatingPreviewBytes` (rater path) and `GetCatalogScore`/`SearchCatalog` (metadata)
are **not** gated (D10 acknowledges the deck as a bypass).

### D3. Contract: locked bytes response + a state read RPC

- `GetCatalogScoreBytesResponse` gains `optional CatalogAccessState access` and the
  locked case returns **`data` empty, `unchanged=false`** with `access.locked=true`
  (never a gRPC error — the app must branch on state, not on a status code, and the
  offline cache must not misread it as a transient failure).
- `CatalogAccessState { bool enabled; bool locked; int32 free_quota; int32 free_used;
  int64 resets_at_ms; int64 day_slot_cost; int64 spendable_balance; bool subscriber;
  bool upsell; }`.
- New `GetCatalogDailyAccess(GetCatalogDailyAccessRequest{}) → { CatalogAccessState
  state; repeated string opened_today; repeated string paid_today; }` — one read that
  lets the hub / library render "N free opens left · resets in Xh", mark opened-today
  cards, and show the cost on locked ones **without** probing bytes per card. `CatalogHit`
  gains `bool has_preview` (public; drives the audition control, no 404 round-trip).
- New `UnlockCatalogScoreForToday(catalog_id) → { CatalogAccessState state; bool
  unlocked; }` (D4). Locked pieces need no new error mapping in the app; insufficient
  balance is `FAILED_PRECONDITION` like the shop.

### D4. Paying with points = a confirmed, ledger-keyed consumable day-slot

On `Locked`, the app shows a **confirmation** ("débloquer pour X points aujourd'hui ?",
balance shown) — never a silent debit (the `practice-streak` freeze rule). On confirm,
`UnlockCatalogScoreForToday` runs one atomic transaction shaped exactly like
`PgStreakRepo::spend_and_restore`:

1. serialise per user (`pg_advisory_xact_lock(hashtext(user_id))` — there is no
   per-user row to `FOR UPDATE`), re-read `SUM(amount)`; if `< cost` → rollback,
   `unlocked=false`, nothing written;
2. `INSERT INTO music.curation_points (user_id, award_kind='redeem', amount=-cost,
   reward_key='score_day_slot', award_key='score_day_slot:<catalog_id>:<day>',
   piece_id=<catalog_id>) ON CONFLICT (user_id, award_key) WHERE award_key IS NOT NULL
   DO NOTHING` — the **ledger key** is the charge-once guard (two confirmations for the
   same piece and day charge once), not the read-then-write;
3. upsert `catalog_day_access (…, paid=true)`; commit.

This deliberately creates **no `curation_grants` row** (that means permanent
ownership) — the spend is permanent history, the access dies with the day, tomorrow
costs again. `piece_id` lets the activity feed and future analytics name the piece;
the app's activity list must label `reward_key=score_day_slot` (a redeem kind already
renders, but with a neutral label — see the "no raw kind in UI" rule). The pure
decision (`already open today?` / `enough balance?` / gate off) is a covered core; the
transaction is excluded glue.

### D5. Offline cache contract: online the server decides first, offline is grace

The encrypted favourites cache serves a hit with **no network**. Left as is, a
favourite opened once would escape the quota forever (favourite → open → cached →
free every day). Contract:

- **Online**: before playing a cached favourite, the app performs the conditional
  fetch **first** (`if_none_match=<cached etag>`, cheap: `unchanged` skips storage) —
  it doubles as the access decision. `unchanged`/bytes → play (and refresh as today);
  `access.locked` → the app shows the locked flow and does **not** play the cached
  copy (the copy is kept: access is per-day, not revoked). The served decision counts
  as today's open exactly like a full fetch.
- **Offline** (no network / server unreachable): the cached favourite plays. This is
  the cache's purpose; the loophole requires airplane mode, only covers pieces already
  cached through a legitimately served open, and the sessions still sync as activity.
  Recorded as an accepted soft-limit trade-off (the quota is a product limit, not a
  security boundary — the egress cap is `catalog-access-limits`).

*Alternative recorded*: mirror the day set locally and gate offline too. Rejected
as over-engineering for a soft limit; revisit if offline abuse shows in analytics.

### D6. Subscription seam + upsell placeholder

`SubscriptionSource { fn has_active_subscription(&self, user_id) -> bool }` (music-crate
trait, server implementation returns `false`, documented for billing). When true,
`decide_open` short-circuits to `Serve` (unlimited, no day rows needed). The
`Locked` state carries `upsell=true` so the client nudges at exactly those moments;
today the app's placeholder reuses the shop's existing "temporary premium access —
coming later" item copy (`reward-unlocks`), wired to the real offer later. Keeping
`subscriber`/`upsell` in the contract now avoids a proto change when billing lands.

### D7. Score audio preview: job at accept, `musicxml-core` schedule, HTTP delivery

- **Sequence** (pure, covered): `preview_sequence(schedule: &PlaybackSchedule,
  max_ms) -> SampleSequence` — take `schedule.notes` (from
  `cymbra_musicxml_core::playback::schedule(&doc)`, the same timing the app uses),
  clip to the first `max_ms` (`catalog.preview.max_ms` flag/int, default 30 000),
  truncating held notes at the boundary; deterministic. Reuse `render_preview_pcm` +
  `encode_preview` unchanged (mono 16-bit 44.1 kHz WAV, ≈2.6 MB per 30 s — accepted;
  size is the trade-off for a 30 s teaser, and why a codec is a later option).
- **Font**: `catalog.preview.soundfont_id` (string flag) names an **accepted** catalog
  SoundFont whose bytes are read from the SoundFont store; unset/unknown → render
  fails → preview absent (feature dormant, not broken).
- **Render is a job**, not inline in the accept RPC: `SetModerationStatus(accepted)`
  enqueues `score_preview_render {catalog_id}` **in the same transaction** as the status
  write (`cymbra_jobs::transactional_enqueue` on the status-write connection — the job
  exists iff the accept commits; the music Pg repo takes the `cymbra-jobs` dependency
  for this one call, and the music DB role needs EXECUTE on `jobs.enqueue`, the
  SECURITY DEFINER entry point), channel `Channel::parallel("music", "preview")`,
  retries per policy. The worker: load
  MusicXML from the score store → parse → schedule → sequence → render → `put` at
  `catalog-preview/{catalog_id}.wav` (score store, beside the bytes) → set
  `catalog_scores.preview_rendered_at = now()`. Failure logs, retries, then leaves the
  piece without a preview (recoverable). Accepting is **never** blocked on the preview
  (unlike SoundFonts, where a human auditions before accept — here the piece is
  already accepted content and the clip is derived).
- **`preview_rendered_at`** is the "has a sample" truth for listings (`CatalogHit.
  has_preview`, the admin "no sample" filter) — a per-row `store.size` like the font
  listing does not scale to the corpus.
- **Delivery**: `GET /scores/{catalog_id}/preview` on the server (same shape and
  middleware as `serve_preview` for fonts): authenticated, moderation-visible
  (accepted for normals; any status for moderator/admin), **no quota / points gate**,
  `404` when absent. `POST /scores/{catalog_id}/preview` = admin/moderator
  **regenerate**, rendered **inline** (immediate feedback for the back-office action),
  overwrites the object and stamps `preview_rendered_at`.
- **Backfill**: an ops path enqueues `score_preview_render` for every accepted piece
  with `preview_rendered_at IS NULL` (a `--enqueue-missing-previews` mode of the
  existing maintenance binary, or a scheduler entry) — the corpus is thousands of
  pieces; the back-office filter is for spot checks, not the backfill.

### D8. App surfaces (Riverpod, listeners, no await-and-branch)

- `CatalogService` gains `dailyAccess()`, `unlockForToday(id)`; bytes result carries
  `access`. A `ScorePreviewService` twin of `SoundFontPreviewService` (`GET
  /scores/{id}/preview` with bearer, 404 → `false`) + `soundClipPlayerProvider` for
  playback.
- `catalogDailyAccessProvider` (keepAlive, identity-scoped, refreshed on open/unlock/
  resume) feeds: a hub/library header chip ("N ouvertures gratuites · réinit. dans
  Xh"), an opened-today mark on `ScoreCard`, and the locked cost. Card slots are
  contended (bottom-left = offline/status tag): the access mark goes on the cover
  overlay, not a new slot.
- **Open flow**: `openScore` (`open_score.dart`) already pre-flights the load; on
  `access.locked` the notation notifier exposes `ScoreLoadFailure.locked` (not an
  error snackbar) and the opener presents an **unlock sheet**: title/composer, "écouter
  un extrait" (audition; greyed when `!has_preview`), "débloquer pour X pts
  aujourd'hui" (disabled with the shortfall when balance < cost), the upsell
  placeholder line. Confirm → `unlockNotifier.unlock(id)` (fire-and-observe); a
  dedicated listener widget reacts to the unlock state: success → re-select the score
  (now served) and bump `rewardRevisionProvider`; failure → localized snackbar.
- **Card audition** (validated during the manual pass): a small labelled
  "▶ Extrait / ■ Stop" pill in the cover's bottom-left slot (the trophy-badge
  pattern — a secondary tappable control, the card's main tap still opens), shown
  only when `hasPreview`, so a user hears a piece *before* spending a free open.
  One `ScorePreviewPlayback` notifier (keepAlive) owns the audition for the cards
  AND the unlock sheet: fetch (session cache by id) → play through the clip player →
  a timer sized from the WAV header stops it after **one** pass (the engine's clip
  player loops by design for the 2.4 s SoundFont phrase — wrong for a 30 s teaser);
  one clip at a time; `openScore` and app pause stop it.
- Analytics (client taxonomy, `usage_actions.dart`): `catalog_quota_reached`,
  `catalog_day_slot_unlock`, `catalog_preview_audition`.
- l10n: en/fr/it/es for every new string.

### D9. Back office

Mirror the SoundFonts pattern: `regenerateScorePreview(id)` as an injectable transport
(`POST /scores/{id}/preview`, `setRegenerateScorePreviewForTest`), a dedicated
`Async<void>` `preview` ref in the catalog store (optimistic `hasPreview=true`),
"Generate sample" in the score detail / table row actions, and a `hasPreview:
"" | "yes" | "no"` filter on `FiltersBar` backed by an admin search param
(`preview_rendered_at IS NULL`). Two BO locales (en/fr).

### D10. Anti-abuse ordering and acknowledged bypasses

- `guard_download` (burst + engagement-aware volume) runs **before** the freemium
  decision: a locked probe still counts one unit (rate is rate), and a scraper is
  stopped by the abuse cap regardless of quota. Two cores, two concerns, ordered.
- `GetRatingPreviewBytes` returns full bytes of pending/accepted un-rated pieces to any
  signed-in rater. It stays ungated (rating is the product; the deck only lists
  un-rated pieces and egress is capped) — an acknowledged bypass for a determined
  client, consistent with "soft product limit, not a security boundary".

### D11. Data lifecycle

- Account purge (`purge_user` job) deletes the user's `catalog_day_access` rows (the
  ledger rows follow the existing rewards purge). Piece deletion cascades.
- Retention: prune `catalog_day_access` older than 30 days from the existing
  prune schedule (rows are only ever read for the current day).
- The preview object is deleted with the score object (same store, same purge path).

## Risks / Trade-offs

- **Freemium on the core loop.** Capping daily practice can hurt engagement — and
  engagement is what mints points (play rewards). Mitigation: off by default,
  staff-only rollout, live `free_quota`/`day_slot_cost`, re-opens free, audio teaser +
  points path keep momentum; watch `catalog_quota_reached` vs `catalog_day_slot_unlock`
  in BO /usage.
- **Server-day reset feels arbitrary in some time zones.** Mitigated by the visible
  reset countdown and free re-opens; the alternative (player-local day) is a farming
  hole the play cap already rejected.
- **Offline grace is a loophole.** Bounded to already-cached favourites in airplane
  mode; accepted (D5).
- **Two "limit" systems on one RPC.** Abuse (`catalog-access-limits`) and freemium
  both gate `GetCatalogScoreBytes`. Distinct cores, abuse first, both flag-killable.
- **Audio-only teaser is thin for scores; clips are WAV.** ≈2.6 MB per 30 s clip
  across the corpus is real storage/egress; default 30 s, length is a flag; a codec
  and/or a lower sample rate are follow-ups if cost shows.
- **Preview render depends on a configured font.** Unset → no previews (feature
  dormant, not broken); the BO filter and backfill make gaps visible and recoverable.
- **Points as a recurring sink.** Scores now consume points daily alongside pianos and
  streak freezes. Day-slot cost is a flag; balance earn/spend with the same levers as
  `music-play-rewards` (daily cap 40, practice 3/day) — a default cost of 20 ≈ half a
  good day of play.
