# Automated reports — field-by-field specification

The content contract for the `discord_digest` job (tasks 3.6–3.9 of
`openspec/changes/add-discord-notifications`). Every figure below names its source table and its
suppression rule, so the implementation has nothing left to invent.

**Discord limits that shape the format**: 2000 characters of message content, 4096 in an embed
description, 25 fields per embed, 6000 characters per embed total. A long ranking therefore goes
in the **description** (a 50-line list is ~2300 characters), never in 50 fields.

**Three global rules**, from the spec:

- **Nothing to say ⇒ nothing posted.** If every element of a report is zero or suppressed, the
  job posts **no message at all** — not zeroes, not a table of dashes. A channel repeating
  "0 players, 0 sessions" discourages the community it exists to grow. One substantive element is
  enough to publish, and an accepted catalog item counts on its own. The skip is logged with its
  reason, so silence stays distinguishable from a broken job.

- **Aggregate minimum `k` (default 5)**: a figure covering fewer than `k` distinct players is
  replaced by `—`, so a small count cannot implicitly name one person.
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
| **Active accounts** | `Usage.active_accounts` | `—` if `< k` |
| **Words read** | `Usage.words_read` — occurrences read in passages actually seen, **not** distinct words (definition from `refine-lingua-reading-stats`) | `—` if accounts that read `< k` |
| **Words learned** | `Usage.words_learned` | `—` if accounts that learned a word `< k` |
| **Reviews done** | `Usage.reviews` | `—` if accounts that reviewed `< k` |
| **Languages studied** | `Usage.by_language`, **accounts only** (never its per-language sums) | see below |

**Languages studied** is rendered only when **two or more** languages each reach `k` accounts. The
repository returns languages alphabetically, so the pure core drops those under `k`, sorts by
`active_accounts` descending (language ascending on ties) and keeps at most three. There is **no
`other` line**: `by_language` holds one distinct count per language, and summing several of them
counts an account once per language it studies, so no honest remainder can be built from it.
Active accounts minus the sum of the lines shown is at best a lower bound on the accounts outside
them, and negative when an account studies two shown languages; it is accepted because it names
neither a language nor a person. Today the extension syncs every day as
`en` ([sync.ts](../../apps/lingua-extension/src/sync/sync.ts)), so the row is omitted: a single
`en — N` would only repeat Active accounts.

Footer: `Counts signed-in accounts whose activity this week has reached the server · figures covering fewer than 5 accounts are hidden`

**What is counted, honestly.** A daily stat is computed on the device and reaches the server only
when a signed-in extension syncs, so:

- signed-out readers and devices kept local are **not** counted — the back office states the same
  bias on screen, and the report has to say it too (the footer);
- an account that only reviewed cards that week is active without having read, hence
  "Active accounts", not "readers";
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

**Expect silence at first, and let it be silent.** With `k = 5` the whole section is suppressed
until five synced accounts are active in the same week, and "nothing to say ⇒ nothing posted" then
keeps the channel empty rather than publishing zeroes.

**Erasure needs no special handling**: `LinguaDataService.EraseMyData` and the account erasure job
both delete the user's `daily_stats` rows, so later reports stop counting them, and an
already-published aggregate carries no identity to retract.

---

## 4. `#music-leaderboards` — weekly (Cymbra Music)

One embed, ranking in the **description**.

- **Top 50 pieces of the week** — same query as the daily top 10, one week window, same
  catalog-only join. ~2300 characters, comfortably inside the 4096 limit.
- **Top players** — top 10 of the current season's **global board** (difficulty-weighted
  best-N, `add-global-leaderboard`, now on `main`: `music.global_season_bests`, read in
  `backend/music/src/pg_global_leaderboard.rs`). Read it through that existing core rather than
  re-deriving the aggregation — `music.leaderboard_bests` is keyed
  `(user_id, catalog_score_id, mode)`, i.e. **per-piece** bests, and is the wrong source. Names
  via the gate, else `Anonymous`.
- **Season** — current 30-day window, days remaining, and the leader when the gate allows.

## 5. `/top50` — on demand

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
