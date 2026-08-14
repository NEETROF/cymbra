## Why

Badges today are a **curation-only** side-effect of `reward-unlocks`: seven static
milestones (`backend/music/src/curation_rewards_core.rs`, `BADGES`) measured against three
rating counters, rendered as a grid inside the curator section of the profile. A user who
plays every day but never rates a score opens their profile and sees seven grey padlocks —
the surface promises an achievement system they have no way to feed. The app already
records the activity that would earn badges (play sessions, leaderboard bests, season
standings, catalog and SoundFont contributions) but none of it counts.

The grid is also thin as a display: the wire carries only `threshold` + `earned`, so a
locked badge reads "🔒 25 aligned" instead of "12/25"; `curation_grants.granted_at` exists
but is never exposed, so there is no earned date, no "new" marker and no earned-first
ordering; all seven tiles share one `military_tech` icon with no description; and labels
are a hard-coded `switch` on the key in the widget, so **no badge can be added without an
app release**.

## What Changes

- Introduce a **badge registry** as a first-class concept, independent of the curation
  points economy: each badge declares a stable key, a **family**, a **metric**, a
  **threshold**, and (for tiered badges) its place in a **track**. The registry is the one
  place a badge is defined; nothing else enumerates badges.
- Extend badge metrics **beyond curation** to the counters that already exist in `music`:
  - **Play** — sessions recorded, distinct pieces played, high-accuracy sessions
    (`music.play_sessions`).
  - **Consistency** — distinct local days at the keyboard and the longest run of
    consecutive such days, counting **scoreless practice** (`practice_sessions`, a chosen
    measure range looped) alongside scored runs (`play_sessions.played_at` +
    `tz_offset_minutes`, the same local-day bucketing the activity heatmap uses).
  - **Ranking** — per-piece board placements and closed-season standings
    (`music.leaderboard_bests`, `music.global_season_snapshots`).
  - **Contribution** — accepted catalog proposals (`catalog_scores.proposed_by` +
    status) and accepted SoundFont contributions (`soundfonts.uploaded_by` +
    `moderation_status`).
  - **Curation** — the existing seven, migrated into the registry unchanged (same keys,
    same thresholds, already-granted rows keep working).
  - **Learning** — completed courses (`music.course_progress`, the per-user completion
    record `add-notation-courses` introduced; its `completed_at` is exactly the
    "earned once" signal, recorded but until now unconsumed).
- Send **progress, identity and provenance** on the wire: the user's current value for the
  metric (for `12/25`), the `granted_at` of an earned badge, the family, and the track +
  tier for tiered badges. Labels and descriptions become **server-provided localized
  text**, so a new badge no longer needs an app release.
- Rework the profile presentation: badges move out of the curator section into their own
  **Achievements** section grouped by family, earned-first, with a per-badge progress bar,
  a "new since last visit" marker, distinct iconography, and a **detail sheet** on tap
  (what it takes, where you stand, when you earned it). A tiered track (Curator I/II/III)
  renders as **one tile showing the current tier**, not three.
- Keep every `reward-unlocks` invariant: badges are **earned, never purchased**, never
  removed, and spending points never costs a badge. Awarding stays **idempotent and
  retroactive** — a badge defined today is granted to users who already qualify.

## Capabilities

### New Capabilities
- `achievement-badges`: the cross-domain badge registry (families, metrics, thresholds,
  tiered tracks), how counters are sourced from each domain, idempotent + retroactive
  awarding, the wire contract (progress, granted-at, localized identity), and the profile
  Achievements surface.

### Modified Capabilities
- `reward-unlocks`: badges stop being owned by the curation rewards capability. The
  "Badges earned by milestones" requirement narrows to the curation *counters* it
  contributes to the registry, and the curator-profile requirement stops carrying the badge
  grid (which moves to `achievement-badges`). The points economy, levels, shop and
  activity feed are unchanged.

## Impact

- **Backend (`cymbra-music`)**: new host-testable registry + evaluation module; new
  read-side counter queries against `play_sessions`, `leaderboard_bests`,
  `global_season_snapshots`, `catalog_scores`, `soundfonts`; `curation_rewards_core::BADGES`
  and `earned_badges` move into the registry; `curation_grants` (`grant_kind='badge'`)
  stays the durable grant store, so **no data migration** and no re-grant of existing
  badges.
- **API (`score.proto`)**: `CuratorBadge` gains progress/family/track/tier/granted-at and
  localized label + description; a badge read that is no longer curation-scoped needs its
  own RPC rather than riding `GetCuratorRewards`. Additive to the message; the new RPC is
  new surface.
- **App (`apps/music`)**: new Achievements section + detail sheet + notifier, driven by an
  injectable service seam; the hard-coded label/metric `switch` in
  `curator_rewards_section.dart` is deleted in favour of server text; existing badge grid
  removed from the curator section.
- **Out of scope (follow-up)**: public badge showcase and pinned badges — the
  `public-player-profile` spec already promises "level, badges" publicly and remains
  unimplemented.
