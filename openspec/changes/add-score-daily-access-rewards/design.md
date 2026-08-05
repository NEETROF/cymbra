## Context

Catalog scores are delivered over gRPC `ScoreService`. A player open pulls the raw
MusicXML via `GetCatalogScoreBytes` ([backend/music/proto/score.proto:479](backend/music/proto/score.proto));
the app renders notation + drives play-along from that XML on the client. There is
**no** server-side notation rendering (that lives in the client / back-office wasm
painter), and rendering a server-side *image* of the score was considered and
**dropped** (poor visual quality). The backend also has no audio synthesis today,
but `add-soundfont-entitlement-previews` introduces a headless `rustysynth` render
path — this change reuses it to render a score's audio.

Adjacent prior art we build on / beside:
- `add-curation-rewards`: an append-only points **ledger**, a spendable balance, and
  `curation_grants` (permanent ownership rows).
- `add-catalog-access-limits` ([backend/music/src/catalog_limits.rs](backend/music/src/catalog_limits.rs)):
  per-user egress limits (anti-scraping). This change is a **separate, gamified**
  quota; the two coexist (abuse-prevention vs freemium product limit).
- `play_core.rs`: already truncates activity to a **date using the client's UTC
  offset** — the same day-boundary convention the daily quota uses.

## Goals / Non-Goals

**Goals:**
- A free user may fully open **N distinct catalog pieces per day**; re-opening a
  piece already opened today is free; the day resets at the client-offset midnight.
- Opening an extra piece beyond the quota costs curation **points at open time**
  (with confirmation) and grants access **for today only** (consumable day-slot, not
  a permanent grant).
- A locked piece never ships its MusicXML; it is teased by a short **audio-only**
  server-rendered clip.
- A clean `has_active_subscription` **seam** that (later) lifts the quota entirely,
  plus a recorded **upsell** hook at the quota/unlock moments.

**Non-Goals:**
- The subscription/billing system (only the boolean seam + upsell placeholder).
- Money, real DRM, per-user watermarking.
- Permanent ownership of scores (day-slots are consumable; contrast soundfont grants).
- Showing a locked piece's **notation** (image render dropped; audio-only teaser).
- Server-side notation rendering; changing how free/accessible pieces play.

## Decisions

### 1. Daily-access model (host-testable core)
State per user per **local day**: the **set of catalog piece ids opened today**
(`opened_today`) and how many of those were **free-quota** opens vs **paid**
day-slots. A pure core decides each open:

```
enum Open { ServeFree, ServePaidToday, NeedsPoints{cost}, NeedsSubscription, Serve /*already today*/ }
fn decide_open(piece_id, opened_today, free_quota, free_used, is_subscriber) -> Open
//  piece_id ∈ opened_today                    -> Serve         (re-open, free)
//  is_subscriber                              -> ServeFree     (unlimited; add to set)
//  free_used < free_quota                     -> ServeFree     (consume a free slot; add to set)
//  else                                       -> NeedsPoints   (offer the paid day-slot / upsell)
```

The **day key** is `date(now, client_utc_offset)` (reuse the `play_core` helper).
`opened_today` is read for the caller+day; on a served open the piece id is recorded
into it (idempotent — recording an id already present is a no-op, so re-opens don't
consume quota). The core is coverage-included; the DB reads/writes and the gRPC
handler are the excluded glue.

`free_quota` is a **config/flag value** (via `cymbra-feature-flags`) so it is tunable
without a deploy, and can be `0`/disabled to turn the whole gate off (kill-switch),
mirroring other rollout flags.

### 2. Paying with points = a consumable day-slot, not a grant
When `decide_open` returns `NeedsPoints`, the app shows a **confirmation**
("débloquer pour X points aujourd'hui ?"). On confirm, a single atomic operation:
- writes a **ledger debit** (`-cost`, reason `score_day_slot`, referencing the piece
  + day) — the *spend* is permanent history, exactly like a soundfont redemption;
- records the piece id into `opened_today` as a **paid** slot (the *access* is
  ephemeral — it lives in the day state and is gone at midnight);
- rejects if the spendable balance is insufficient (checked against the ledger sum,
  as `add-curation-rewards` already does).

This deliberately does **not** create a `curation_grants` row (that means permanent
ownership). Re-opening the piece **the same day** is free (it's in `opened_today`);
**tomorrow** it costs again. The debit decision (enough balance? already paid today?
idempotency) is a covered core; the transaction is excluded glue.

### 3. Subscription seam + upsell (future)
The gate calls `has_active_subscription(caller) -> bool`, implemented now as a stub
returning `false` (no billing yet). When true, `decide_open` short-circuits to
unlimited free opens. Additionally, the design records that once subscriptions exist,
the `NeedsPoints`/quota-reached responses carry an **upsell** signal so the client can
nudge the user to subscribe at exactly those moments; today that is a
placeholder/no-op flag on the response, wired to the real offer later. Keeping both
in the response contract now avoids a proto change when billing lands.

### 4. Score audio preview (reuse the soundfont render engine)
A locked piece is teased by a short **audio-only** clip:
- **Rendered at acceptance**: when a catalog score is accepted (the moderation
  transition that makes it playable), render a short clip — synthesize the first
  ~N seconds of the piece's MIDI with a **default soundfont** via the backend
  `rustysynth` path from `add-soundfont-entitlement-previews` — and store it as a
  **public** object (`score-preview/{catalogId}.wav`). Reuses `render_preview_pcm` /
  `encode_preview`; the only new pure piece is turning the score's MusicXML→MIDI
  events into the render sequence (a covered helper), bounded to N seconds.
- **Delivery**: a new RPC (e.g. `GetScorePreviewBytes(catalogId)`) serves the public
  clip with **no quota/entitlement gate** (hearing it is the point), moderation
  visibility only; not-found if absent.
- **Back-office fallback**: an admin action re-renders + overwrites the clip (for
  pieces accepted before this change / failed renders), same shape as the soundfont
  "Generate sample".
- **App**: a locked piece's play button auditions this clip; an absent clip greys the
  control.

### 5. Where the gate lives
`GetCatalogScoreBytes` (player open, accepted-only) is the single choke point.
`ListRatingDeck` / `GetRatingPreviewBytes` (moderation deck) are **not** gated — they
are a moderator/rater path, not a player open. `GetCatalogScore` (metadata) stays
open so the catalog can show titles + access state.

## Risks / Trade-offs

- **Freemium on the core loop.** Capping daily practice can hurt engagement — and
  engagement is what mints points. Mitigation: `free_quota` is a live flag (start
  generous, tune with data; kill-switch to 0 disables the gate), re-opens are free,
  and the audio teaser + points path keep momentum.
- **Day-boundary correctness.** A client-supplied offset can be spoofed to farm free
  opens across "days". Mitigation: reuse the vetted `play_core` date logic; the quota
  is a soft product limit, not a security boundary — the real anti-abuse cap is
  `add-catalog-access-limits`, which still applies underneath.
- **Two "limit" systems on one RPC.** `add-catalog-access-limits` (abuse) and this
  (freemium) both gate `GetCatalogScoreBytes`. Mitigation: distinct concerns, distinct
  cores; abuse-limit first (hard cap), then freemium quota (product), clearly ordered.
- **Audio-only teaser is thin for scores.** No notes shown. Accepted: users usually
  know the piece by title/composer, and the audio + metadata is enough to drive an
  unlock; richer previews are a later option.
- **Points as a recurring sink.** Scores now consume points daily alongside
  soundfonts. Mitigation: day-slot cost is a flag; balance the earn/spend with the
  same levers as `add-curation-rewards`.
