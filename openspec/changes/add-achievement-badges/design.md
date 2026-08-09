## Context

Badges exist today as a leaf of the curation points economy (change `add-curation-rewards`,
capability `reward-unlocks`):

- **Definition**: `const BADGES: [BadgeDef; 7]` in `backend/music/src/curation_rewards_core.rs`
  — key + metric + threshold, over three counters (`rating_count`, `aligned_count`,
  `first_rater_count`).
- **Evaluation**: `CurationRewardsModule::grant_due_badges()` diffs `earned_badges(counts)`
  against the granted set and inserts the newcomers. It runs **on read** (`rewards()`), on
  rating ingest, and after each settlement.
- **Storage**: `music.curation_grants (user_id, grant_kind, key, granted_at)`,
  `PRIMARY KEY (user_id, key)` — the insert is the idempotency guard.
- **Wire**: `CuratorBadge { key, metric, threshold, earned }` inside `CuratorRewards`.
- **UI**: `_BadgeGrid` / `_BadgeTile` in `apps/music/lib/widgets/curator_rewards_section.dart`,
  with `_badgeLabel` / `_metricLabel` `switch`es hard-coding the seven keys.

Constraints that shape this design:

- **Counter sources verified on `main`**: `music.play_sessions` (0010, with
  `tz_offset_minutes` for local-day bucketing), `music.leaderboard_bests` (0015),
  `music.global_season_snapshots` (0018), `music.catalog_scores.proposed_by` + status
  (0014), `music.soundfonts.uploaded_by` + `moderation_status` (0013),
  `music.score_ratings` (0009). All live in the `music` schema — no cross-schema reach.
- **Courses have no server-side progress**: `music.courses` (0019) stores opaque manifests
  served by `ListCourses`/`GetCourse`; there is no per-user completion record and no course
  UI in `apps/music` on `main`. A learning badge has nothing to count.
- **`reward-unlocks` invariants are load-bearing**: badges are earned, never purchased;
  once earned, kept; spending points never removes one.
- Coverage gate ≥ 80% both ecosystems, so evaluation logic must live in host-testable pure
  modules (the `*_core.rs` convention), with I/O behind a trait doubled by `mockall`.

## Goals / Non-Goals

**Goals:**
- One registry that owns every badge definition, across domains, with families and tiered
  tracks — nothing else enumerates badges.
- Badges earnable from playing, consistency, ranking and contribution, not only curation.
- The wire carries enough to render a rich grid: current progress, earned date, family,
  track/tier, and localized label + description.
- Adding a badge requires **no app release**.
- Awarding stays idempotent and becomes retroactive by construction.
- The seven existing badges keep their keys and thresholds; already-granted rows keep
  working with **no data migration**.

**Non-Goals:**
- Public badge showcase / pinned badges on the public profile (follow-up; the
  `public-player-profile` spec already promises it and is unimplemented).
- A learning badge (blocked on server-side course progress).
- Badge-driven rewards: a badge grants no points and unlocks no content. It stays a mark.
- A back-office CRUD for badges.
- Changing the points economy, levels, shop or activity feed.

## Decisions

### D1 — The registry lives in `cymbra-music`, not a new crate

New modules mirroring the existing split: `badges_core.rs` (pure — registry, families,
tracks, threshold evaluation, streak computation), `badges.rs` (the `BadgeRepo` trait +
`BadgeCounters` DTO), `badges_module.rs` (orchestration: fetch counters, grant, project).

*Alternative rejected*: a dedicated `cymbra-badges` crate. Every counter source is a table
in the `music` schema and there is exactly one consumer; a crate boundary would buy a trait
seam we already get from `BadgeRepo` while adding a workspace member to keep in sync.
Revisit only if a second bounded context (e.g. a live/ensemble app) needs badges.

### D2 — Grant-on-read, not per-domain ingest hooks and not a worker job

`GetAchievements` recomputes counters and grants any newly-due badge before projecting the
response — the pattern `grant_due_badges()` already uses.

*Alternative rejected — hook every domain ingest* (play session recorded, best updated,
proposal accepted): spreads badge knowledge across five modules, couples unrelated write
paths to the registry, and gives no retroactivity for a badge defined later.
*Alternative rejected — a periodic worker job*: needs an all-users scan on a cadence,
delays the grant, and buys nothing here because the only consumer of a grant is the screen
that would have triggered it.

The cost of grant-on-read is that a user who never opens the screen never gets a grant row.
D3 makes that harmless.

### D3 — `earned = granted OR counter ≥ threshold`; the grant row records *when*, not *whether*

Deriving `earned` purely from live counters would let a badge disappear: `leaderboard_bests`
rows cascade-delete when a catalog piece is purged, a re-opened catalog proposal leaves
`accepted`, and play-session retention will eventually prune rows. Deriving it purely from
the grant table would make a badge depend on having opened the screen.

So `earned` is the **union**: the badge shows earned if it has ever been granted *or* the
counter currently clears the threshold. The grant row is the durable memory of the date;
the live counter is what makes a fresh badge retroactive without a backfill. Displayed
progress is **clamped to the threshold** once earned, so an earned badge never renders as
`3/20`. `granted_at` is `null` for a badge earned by counter but not yet granted (it is
granted in the same call, so this is only visible to a caller that reads without granting).

### D4 — One repo call returns all counters; the streak is computed in Rust

`BadgeRepo::counters(user_id) -> BadgeCounters` issues the handful of user-scoped
aggregates once per read, rather than one query per badge. Every query is covered by an
existing index (`play_sessions_user_played_idx` on `(user_id, played_at)`,
`leaderboard_bests` PK, `global_season_snapshots` PK, `catalog_scores_proposed_by_idx`,
`soundfonts_moderation_status_idx`).

Distinct-local-days and longest-consecutive-day-run are **not** SQL window functions: the
repo returns the distinct local days (`played_at` shifted by `tz_offset_minutes`, the same
bucketing the activity heatmap uses) and `badges_core` folds them into a count and a longest
run. Pure, host-testable, and it keeps the streak rule in the module that owns badge
semantics.

*Trade-off*: a full-history day list per read. Bounded by retention and one indexed
user-scoped scan; if it ever hurts, add a short-TTL per-user cache before touching the
shape.

### D5 — Tiered tracks: the server sends the whole ladder, the client collapses it

The registry gains `track` + `tier`, so `curator_1/2/3` is one track at tiers 1/2/3. The
response carries **every** badge including the tiers already surpassed, and the client grid
renders one tile per track (highest earned tier, progress toward the next) while the detail
sheet shows the full ladder.

*Alternative rejected*: collapsing server-side. It would hide the ladder the detail sheet
needs and bake a presentation choice into the API.

### D6 — Identity is an inline-localized map on the wire, resolved client-side

Label and description ship as `{en, fr, es, it}` maps, exactly the pattern
`music.courses.title_json` already established on this service; the client picks its active
locale with an `en` fallback. This deletes `_badgeLabel` / `_metricLabel` from the widget
and satisfies "a new badge needs no app release".

*Alternative rejected — resolve server-side from the account locale*: adds a `UserPort`
call to a hot read, breaks when the account locale is unset, and resolves the *account*
language when what matters is the language currently displayed (they differ; see
`account-language-sync`).

### D7 — The registry stays static Rust; a new badge is a backend release

`BADGES` becomes `REGISTRY` — still a `const` array, now with family, track, tier and the
localized maps. Milestones are product decisions, not per-environment configuration.

*Alternative rejected*: a DB-backed registry with back-office CRUD. It buys same-day badge
authoring at the cost of a table, an admin surface, validation of user-authored thresholds,
and a second source of truth for something that changes a few times a year. "No app
release" is the requirement; "no backend release" is not.

### D8 — A dedicated `GetAchievements` RPC; `CuratorRewards.badges` stays populated but deprecated

Badges are no longer curation-scoped, so they get their own read rather than growing
`GetCuratorRewards`. `CuratorRewards.badges` keeps returning the curation subset, marked
deprecated, so an app version already in users' hands keeps rendering its grid; the field
is dropped in a later change once those versions age out.

## Risks / Trade-offs

- **A new counter aggregate on a hot screen** → all queries are user-scoped and index-backed
  and issued once per read (D4); measure before caching.
- **Play-session retention silently lowering a play counter** → D3's union makes an earned
  badge permanent; only an *unearned* badge could stall, which is the honest reading of
  "you have not done this yet".
- **Grid inflation**: five families × tiers is a lot of tiles for a new user → the grid
  groups by family and collapses tracks (D5); a family with zero progress renders as a
  single collapsed row rather than a wall of padlocks.
- **Two badge surfaces during rollout** (deprecated `CuratorRewards.badges` + the new
  section) → the new app build removes the curator-section grid in the same change, so no
  user ever sees both.
- **Localized maps drift** (a badge added with `en` only) → the fallback is defined and the
  registry is a single `const`, so a missing translation is visible in review, not at
  runtime.
- **Learning family declared but empty** → it ships as a family with no entries and no UI
  affordance; the section simply does not render a family that has no badges.

## Migration Plan

No data migration. `music.curation_grants` remains the grant store and existing
`grant_kind='badge'` rows are re-read by the registry under the same keys, so every badge
already earned stays earned on first read. Rollback is a backend revert: the grant rows the
new registry wrote for non-curation badges are inert to the old code (it only ever reads the
seven keys it knows).

## Open Questions

- **Exact thresholds per family.** The curation seven are fixed (they must not move). The
  play / consistency / ranking / contribution milestones need a first pass tuned against
  real `play_sessions` volume rather than guessed — to be set when writing the registry, and
  cheap to adjust later since a raised threshold cannot un-earn a badge (D3).
- **Whether the "new since last visit" marker reuses `curatorActivitySeenProvider`** (the
  existing seen-timestamp mechanism for the activity feed) or gets its own persisted
  timestamp. Leaning to its own key: the two surfaces are now independent.
