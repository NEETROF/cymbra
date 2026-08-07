# Automated reports — field-by-field specification

The content contract for the `discord_digest` job (tasks 3.6–3.8 of
`openspec/changes/add-discord-notifications`). Every figure below names its source table and its
suppression rule, so the implementation has nothing left to invent.

**Discord limits that shape the format**: 2000 characters of message content, 4096 in an embed
description, 25 fields per embed, 6000 characters per embed total. A long ranking therefore goes
in the **description** (a 50-line list is ~2300 characters), never in 50 fields.

**Two global rules**, from the spec:

- **Aggregate minimum `k` (default 5)**: a figure covering fewer than `k` distinct players is
  replaced by `—`, so a small count cannot implicitly name one person.
- **Naming gate**: a player is named only when their Discord opt-in is on **and** they are
  publicly listable (public profile + age-eligible). Otherwise the line reads `Anonymous` — the
  rank and the figure still show, so a board is never empty.

**Day boundary**: reports bucket by **UTC day**. `play_sessions` carries `tz_offset_minutes` for
the app's *local-day* heatmap; reusing that here would double-count players around midnight.

---

## 1. `#music-stats` — daily (Cymbra Music)

One embed per day. Title `Cymbra Music — <date>`, brand colour.

| Field | Source | Suppression |
|---|---|---|
| **Players who played** | `count(distinct user_id)` in `music.play_sessions` over the UTC day | `—` if `< k` |
| **Sessions** | `count(*)` in `music.play_sessions` | shown with the above |
| **Average accuracy** | `avg(overall_sync_pct)` in `music.play_sessions` | `—` if players `< k` |
| **Scores rated** | `count(*)` in `music.score_ratings` for the day | `—` if raters `< k` |
| **…of which reached consensus** | consensus/settlement state from the curation-rewards tables | omitted when 0 |
| **New in the catalog** | scores/soundfonts that became `accepted` that day, by title | never suppressed (an item is not a person) |
| **Top 10 pieces played** | `music.play_sessions` **joined to `music.catalog_scores`** | pieces below `k` distinct players are dropped from the list |
| **Record of the day** | `music.leaderboard_bests` rows whose `achieved_at` falls in the day | name via the gate, else `Anonymous` |

Footer: `Figures covering fewer than 5 players are hidden · Names appear only with the player's opt-in`

**The join is mandatory, not an optimisation.** `play_sessions.score_id` is an opaque `TEXT` that
holds *either* a catalog id *or* a **user** score id ([0010_play_sessions.sql](../../backend/music/migrations/0010_play_sessions.sql)).
Ranking straight off that column would publish the title of somebody's private upload. Restrict
to accepted catalog pieces.

**Not available, do not promise it**: *time played*. The summary tier of `play_sessions` has no
duration column — duration lives inside the `session_result` JSONB, which the retention job
prunes. Any "minutes played" figure would silently degrade to partial data as history ages.

**Example**

```
Cymbra Music — 7 August 2026

Players who played  12          Sessions  47          Average accuracy  78%
Scores rated  9  (3 reached consensus)

New in the catalog
• Gymnopédie No. 1 — Satie (score)
• Salamander Grand C5 (soundfont)

Top 10 pieces played
1. Gymnopédie No. 1 — 11 plays
2. Prélude in C — 8
…

Record of the day
Tempo · Prélude in C · 138 BPM · Anonymous

Figures covering fewer than 5 players are hidden · Names appear only with the player's opt-in
```

---

## 2. `#id-stats` — weekly (Cymbra ID)

Weekly, not daily: the volume does not carry a daily report, and "1 new account today" reads
worse than saying nothing. Title `Cymbra ID — week of <date>`.

| Field | Source | Suppression |
|---|---|---|
| **New accounts** | `user_account` rows created during the week | `—` if `< k` |
| **Email verifications completed** | verification state transitions in the week | `—` if `< k` |
| **Sign-in methods** | linked-identity providers, as **percentages** | omitted entirely if accounts `< k` |
| **Identities linked** | link operations during the week | `—` if `< k` |
| **Top languages** | the account `locale` column, top 3 | omitted if accounts `< k` |

**Deliberately excluded: account deletions.** It is a legitimate metric — for the back office. In
a public community channel a churn number invites speculation and rewards nobody. Keep it in the
admin surface.

---

## 3. `#music-leaderboards` — weekly (Cymbra Music)

One embed, ranking in the **description**.

- **Top 50 pieces of the week** — same query as the daily top 10, one week window, same
  catalog-only join. ~2300 characters, comfortably inside the 4096 limit.
- **Top players** — blocked on the global board. `music.leaderboard_bests` is keyed
  `(user_id, catalog_score_id, mode)`, i.e. **per-piece** bests; the difficulty-weighted global
  ranking lives on the unmerged `add-global-leaderboard` branch. Until it lands, publish
  **records set this week** (piece + mode + figure, name via the gate) instead of a global top 10.
- **Season** — current 30-day window, days remaining, and the leader when the gate allows.

## 4. `/top50` — on demand

The same ranking as §3, answered by the interactions endpoint. **Ephemeral by default** (visible
only to the requester) so a pull does not push 50 lines into the channel for everyone. The
ranking core is shared with the digest — one implementation, three surfaces.

---

## Cadence, per product

Each product carries its own cadence flag (`discord.music.*`, `discord.id.*`). Start Music
**weekly** too if the first week's numbers look thin, then switch to daily from the back office —
no redeploy. Suppressed and throttled figures are counted in the logs, so a quiet report is
distinguishable from a broken one.
