# add-lingua-native-language-server — the server records the language of a gloss and of a day, before any client sends one

## Why

Change 10 of the [language matrix programme](../../../docs/lingua/language-matrix-programme.md),
the seventh of stage 1, and the one the programme's rule « server first » is about. Decisions M4
and M14, restated: cards record the language of their gloss, and synced daily statistics carry
the reader's native language — both server first, both absent meaning `fr`, both behind a
capability flag the client waits for.

Today `lingua.cards` stores a card's gloss with no language, and every stored gloss is French:
the only packs that ever shipped are glossed in French. A default of `fr` therefore labels every
existing row exactly, as the card-language precedent's `en` did for the studied language
(`add-lingua-card-language`, migration 0005). The label must be in production before any client
that can create a non-French gloss syncs: once an English gloss is stored under no label, no
default can tell it apart.

`lingua.daily_stats` is keyed by (reader, day, studied language, device) and carries no native
language, so the back office can break usage down by studied language only. A reader of English
who studies Spanish and a reader of French who studies Spanish are one row of `es` each. The
native language is new collected data: the privacy policy and the App Store answers change with
the release that sends it (M14), which is change 12, not this one.

## What Changes

- **Additive `.proto` only**, in `cymbra.lingua.v1`:
  - `CardOp.gloss_language = 12`, a string; empty means `fr`.
  - `PullCardsRequest.any_gloss_language = 3`, a bool: this client reads a card's gloss language
    and accepts every one. Unset, the pull returns cards glossed in French only: a client that
    predates labels never holds a gloss it would write back unlabelled.
  - `DailyStat.native_language = 8`, a string; empty means `fr`.
  - `GetDataStateResponse.language_labels = 3`, a bool: this server stores both labels. A server
    that predates the field answers `false` (proto3 default), as `card_language` does.
- **Migration 0006**, idempotent, rewriting no row: `cards.gloss_language TEXT NOT NULL DEFAULT
  'fr'`, `daily_stats.native_language TEXT NOT NULL DEFAULT 'fr'`. The keys do not change: a card
  is still (reader, studied language, client id) — `client_id` is the lemma, so two devices of
  one account with different native languages write one card row, and the label travels with the
  write that wins. A day's statistic is still (reader, day, studied language, device); the native
  language is a value of the row, the device's for that day.
- **Normalised on receipt**, with the one normaliser the server has, given its default: the
  studied language reads an empty value as `en`, a gloss or native language as `fr`. The
  requirement that says every language value reads `en` when empty is modified to say so.
- **A client that predates labels pulls French-glossed cards only.** Installed clients push their
  whole deck at every sync, with no gloss language, and a pull filters by studied language alone;
  without a filter, an old device that pulled a card glossed in English would write it back
  relabelled `fr` the next time it graded it. The pull withholds such cards from a client that does
  not say it reads labels, as it withholds non-English cards from one that names no language.
- **Returned as stored**: a pulled card carries its gloss language. The consolidated statistics
  read (`GetStats`, the reader's own screen) is unchanged.
- **Back office and worker: nothing to change.** The usage breakdown by pair is the optional
  `add-admin-lingua-pair-usage`; the purge job lists tables, not columns.
- **Not in this change: no client sends either field.** Installed extensions and the Apple app
  keep working: they send nothing new, and read nothing new.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

`lingua-sync` and `lingua-stats` are held by no open change:

- `lingua-sync`: ADDED *A card carries the language of its gloss* (wire field, default `fr`,
  stored and returned; withheld from a client that predates labels), *The server states that it
  stores the language of glosses and of statistics* (the capability flag); MODIFIED *Language
  values are normalised on receipt* — one sentence: the studied language reads `en` when empty, a
  gloss or native language `fr`; both scenarios kept.
- `lingua-stats`: ADDED *A daily statistic carries the native language of its device* (wire
  field, default `fr`, a value of the row).

*The backup records the reader's language profile* (`lingua-decks-review`) says the profile is
never sent; a day's native language is a value the client will send with its statistics
(change 12), which rewords that requirement then. *Lingua's privacy disclosures match what it
collects* (`lingua-privacy`) is held by the parked `add-lingua-remote-translation`; the
disclosures move with the release that sends (change 12, with change 31), not with this server
release, which collects nothing new from any client.

## Impact

- **Products.** Backend only: `backend/lingua` (three proto files, migration 0006, `deck.rs`,
  `pg_deck.rs`, `deck_grpc.rs`, `stats_core.rs`, `stats.rs`, `pg_stats.rs`, `stats_grpc.rs`,
  `data_grpc.rs`, `language_core.rs`, tests). Consumed, not redeclared: the Lingua pool and
  migrator wiring in `backend/server`, the purge job in `backend/worker`. Cymbra ID, Music,
  Live, the back office and the site are untouched; the extension's and the back office's
  generated stubs follow at their next build, with nothing to commit.
- **Contract.** `cymbra.lingua.v1` gains four fields; the `proto` workflow reports no break.
  Installed clients send no gloss or native language (read as `fr`), ignore the flag, and pull
  French-glossed cards only, which is every card they have ever pulled.
- **Deployment order.** This backend release is deployed to production, and checked from
  outside, **before** any client able to create a non-French gloss or to send a native
  language is built for a store (changes 11, 12). Rollback is a revert while no label other than
  `fr` is stored, which is the case until change 12 ships: the columns and the flag are
  additive, and a rolled-back server answers `language_labels: false`. After change 12 ships, a
  rolled-back server would write a gloss without its label; the runbook step for that case is in
  the design's migration plan.
- **Data.** The migration is rehearsed on a copy of production before the release; it adds two
  defaulted columns and rewrites no row.
- **Not here.** Review showing the current pack's gloss when the card's is in another language
  (change 11); the client sending and reading the labels, statistics v4 with the native
  language, the policy and App Store wording (changes 12, 31); the usage breakdown by pair in
  the console (optional `add-admin-lingua-pair-usage`).
