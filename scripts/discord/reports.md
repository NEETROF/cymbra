# Automated reports — field-by-field specification

The content contract for the `discord_digest` job (tasks 3.6–3.9 of
`openspec/changes/add-discord-notifications`). Every figure below names its source table and its
suppression rule, so the implementation has nothing left to invent.

**Discord limits that shape the format**: 2000 characters of message content, 4096 in an embed
description, 25 fields per embed, 6000 characters per embed total. A long ranking therefore goes
in the **description** (a 50-line list is ~2300 characters), never in 50 fields.

**Four global rules**, from the spec:

- **Nothing to say ⇒ nothing posted.** If every element of a report is zero or suppressed, the
  job posts **no message at all** — not zeroes, not a table of dashes. A channel repeating
  "0 players, 0 sessions" discourages the community it exists to grow. One substantive element is
  enough to publish, and an accepted catalog item counts on its own. The skip is logged with its
  reason, so silence stays distinguishable from a broken job.

- **No count of people.** No report states how many players, accounts or readers there were
  (maintainer decision, 2026-09-29): it publishes what was *done* — sessions, words read, reviews,
  ratings, pieces — never how many did it. Languages and sign-in methods appear as a ranking or as
  percentages, never as head counts.
- **Minimum contributors `k` — a flag, default `3`** (`discord.reports.min_contributors`,
  maintainer decision 2026-09-30): a figure whose contributing accounts are fewer than `k` is
  dropped from the message, so no published figure is one or two people's activity. Changing it
  from the back office takes effect at the next report, with no release; every rule below is
  written against `k`.
- **Naming gate**: a player is named only when their Discord opt-in is on **and** they are
  publicly listable (public profile + age-eligible). Otherwise the line reads `Anonymous` — the
  rank and the figure still show, so a board is never empty.

**Day boundary**: reports bucket by **UTC day**. `play_sessions` carries `tz_offset_minutes` for
the app's *local-day* heatmap; reusing that here would double-count players around midnight. A
**week** is the ISO week, Monday 00:00 to Sunday 24:00 UTC.

---

## 1. `#music-stats` — daily (Cymbra Music)

One embed per day. Title `Cymbra Music — <date>`, brand colour.

| Field | Source | Suppression |
|---|---|---|
| **Sessions** | `count(*)` in `music.play_sessions` over the UTC day | dropped if players `< k` |
| **Average accuracy** | `avg(overall_sync_pct)` in `music.play_sessions` | dropped if players `< k` |
| **Scores rated** | `count(*)` in `music.score_ratings` for the day | dropped if raters `< k` |
| **…of which reached consensus** | consensus/settlement state from the curation-rewards tables | omitted when 0 |
| **New in the catalog** | scores/soundfonts that became `accepted` that day, by title | never suppressed (an item is not a person) |
| **Top 10 pieces played** | `music.play_sessions` **joined to `music.catalog_scores`**, ranked by plays | pieces below `k` distinct players are dropped from the list |

No player count, and no "record of the day": records already have their own anonymous
announcement in `#music-leaderboards`, and naming players waits for the dedicated Discord consent.

Footer: none while no one is named.

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

Sessions  47          Average accuracy  78%
Scores rated  9  (3 reached consensus)

New in the catalog
• Gymnopédie No. 1 — Satie (score)
• Salamander Grand C5 (soundfont)

Top 10 pieces played
1. Gymnopédie No. 1 — 11 plays
2. Prélude in C — 8
…
```

---

## 2. `#id-stats` — weekly (Cymbra ID)

Weekly, not daily: the volume does not carry a daily report, and "1 new account today" reads
worse than saying nothing. Title `Cymbra ID — week of <date>`.

| Field | Source | Suppression |
|---|---|---|
| **Sign-in methods** | linked-identity providers of the accounts created during the week, as **percentages** | omitted entirely if those accounts `< k` |
| **Top languages** | the `locale` of the accounts created during the week, top 3, **names only**, most frequent first | omitted if those accounts `< k` |

No count of new accounts, verifications or linked identities: each one is a head count. A week
with no new account has nothing to say and posts nothing.

**Deliberately excluded: account deletions.** It is a legitimate metric — for the back office. In
a public community channel a churn number invites speculation and rewards nobody. Keep it in the
admin surface.

---

## 3. `#lingua-stats` — weekly (Cymbra Lingua)

Weekly, for the same reason as Cymbra ID. Title `Cymbra Lingua — week of <date>`.

**One call to the ops aggregate that already exists**: `LinguaAdminRepo::usage(from_day, to_day)`
over `lingua.daily_stats` (table from change `add-lingua-backend`, aggregate from the back-office
ops console, change `add-lingua-back-office`, [pg_admin.rs](../../backend/lingua/src/pg_admin.rs)).
Its totals are **one** `COUNT(DISTINCT user_id)` + `SUM(...)` over the whole window, so a reader
active on several days counts once; `by_language` is the same, per language over the window; only
`series()` groups by day, and the report does not need it. No account identifier comes back.

That query needs one addition, in the **totals** statement of `usage()` (the `by_language` breakdown
is untouched) and still identifier-free: a **contributor
count per summed column**, `COUNT(DISTINCT user_id) FILTER (WHERE <column> > 0)`. Without it the
only available gate is the active-account count, and five active accounts of which one reviewed
would publish that one person's review total. Extend `usage()`; do not write a second query.

| Field | Source | Suppression |
|---|---|---|
| **Words read** | `Usage.words_read` — occurrences read in passages actually seen, **not** distinct words (definition from `refine-lingua-reading-stats`) | dropped if accounts that read `< k` |
| **Words learned** | `Usage.words_learned` | dropped if accounts that learned a word `< k` |
| **Reviews done** | `Usage.reviews` | dropped if accounts that reviewed `< k` |
| **Languages studied** | `Usage.by_language`, ranked by accounts, **names only** (never a count, never its per-language sums) | see below |

`Usage.active_accounts` is read but **never published**.

**Languages studied** is rendered only when **two or more** languages each reach `k` accounts. The
repository returns languages alphabetically, so the pure core drops those under `k`, sorts by
`active_accounts` descending (language ascending on ties), keeps at most three and prints their
**names only** (`English · Spanish`) — the per-language account count orders the list and is never
shown. There is no `other` line. Today the extension syncs every day as `en`
([sync.ts](../../apps/lingua-extension/src/sync/sync.ts)), so the row is omitted: a lone `English`
says nothing.

Footer: `Counts signed-in accounts whose activity this week has reached the server`

**What is counted, honestly.** A daily stat is computed on the device and reaches the server only
when a signed-in extension syncs, so:

- signed-out readers and devices kept local are **not** counted — the back office states the same
  bias on screen, and the report has to say it too (the footer);
- stats from an extension older than `refine-lingua-reading-stats` are dropped at sync, so an
  outdated extension counts nowhere until it updates;
- the figures are **lower bounds**: a day synced after its week's report went out is not added
  later, since a published report is never re-emitted (D5). The Lingua week is therefore reported
  one day after it closes (see *Cadence*), which shrinks that loss; a device offline for longer is
  still missed.

**Deliberately not published: new words seen and comprehension.** `Usage.new_words_seen` exists,
but its change decided the counter is for the back office only (maintainer decision,
`refine-lingua-reading-stats`, 2026-09-27). Publishing it is that capability's decision to
revisit, not this report's to take.

**Nobody is ever named here, and the naming gate never applies.** Lingua has no public profile, and
the report reads only the ops aggregate, whose own privacy allow-list returns counts and never an
account identifier (`backend/lingua/src/admin.rs`). `word_statuses` and `cards` are per-account
rows and stay out of any report. A single heavy reader can still dominate a sum such as Words
read; that is accepted, because nothing published says who they are.

**Expect silence at first.** With `k = 3` a figure is published once three synced accounts
contributed to it in the week; until then the report posts nothing ("nothing to say ⇒ nothing
posted"), rather than zeroes or dashes.

**Erasure needs no special handling**: `LinguaDataService.EraseMyData` and the account erasure job
both delete the user's `daily_stats` rows, so later reports stop counting them, and an
already-published aggregate carries no identity to retract.

---

## 4. `#music-leaderboards` — weekly (Cymbra Music)

One embed, ranking in the **description**.

- **Top 50 pieces of the week** — same query as the daily top 10, one week window, same
  catalog-only join. ~2300 characters, comfortably inside the 4096 limit.
- **Season** — the current 30-day window and the days remaining.
- **Top players — not before the naming consent (tranche 4).** A top 10 of the season's
  **global board** (difficulty-weighted best-N, `add-global-leaderboard`: `music.global_season_bests`,
  read through `backend/music/src/pg_global_leaderboard.rs`'s core, **not** `music.leaderboard_bests`,
  which holds per-piece bests) is only worth publishing with names; ten `Anonymous` lines are not.

## 5. `/top50` — on demand (bot, not in v1)

The same ranking as §4, answered by the interactions endpoint. **Ephemeral by default** (visible
only to the requester) so a pull does not push 50 lines into the channel for everyone. The
ranking core is shared with the digest — one implementation, three surfaces.

---

## Cadence, per product

Each product carries its own cadence flag (`discord.music.*`, `discord.id.*`, `discord.lingua.*`).
Defaults: Music **daily**, ID and Lingua **weekly**. Each run reports only a closed period, and the
Lingua week is due one day after it closes, since its figures arrive by device sync. Start Music
**weekly** too if the first week's numbers look thin, then switch to daily from the back office —
no redeploy. Suppressed and throttled figures are counted in the logs, so a quiet report is
distinguishable from a broken one.
