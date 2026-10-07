# Design — add-lingua-native-language-server

## Context

See proposal.md (Why). The precedent is `add-lingua-card-language` (archived 2026-10-07): an
additive field with a default, a migration that rewrites nothing, one normaliser applied at the
edge, a capability bool on `GetDataStateResponse`, and a release checked from outside before any
client is built. This change follows it field for field. Where things are today:

| Where | What |
|---|---|
| `backend/lingua/proto/deck.proto` `CardOp` | fields 1–11; `gloss = 6`; `language = 11` (studied, empty = `en`); next free 12 |
| `deck.proto` `PullCardsRequest` | `cursor = 1`, `languages = 2` (empty = English only: the precedent's pull filter); next free 3 |
| `backend/lingua/proto/stats.proto` `DailyStat` | fields 1–7; `optional uint32 unknown_seen = 7` (presence marks an up-to-date client); next free 8 |
| `backend/lingua/proto/lingua_data.proto` `GetDataStateResponse` | `erased_at = 1`, `bool card_language = 2`; next free 3 |
| `backend/lingua/migrations/0001…0005` | `lingua.cards` (PK reader, language, client_id; `gloss TEXT` nullable), `lingua.daily_stats` (PK reader, day, language, device_id) |
| `backend/lingua/src/language_core.rs` | `normalise` (trim, lowercase, primary subtag, 8 bytes, `en` when empty); host-tested |
| `deck.rs`, `stats.rs`, `stats_core.rs` | `Card`, `DailyStat`; module tests on hand fakes (`FakeDeckRepo`, `FakeStatsRepo`) |
| `pg_deck.rs`, `pg_stats.rs`, `*_grpc.rs`, `data_grpc.rs` | the Postgres and gRPC adapters, excluded from the coverage gate; `data_grpc.rs` answers `card_language: true` |
| `backend/lingua/tests/privacy_allow_list.rs` | every proto field name checked against forbidden substrings |
| `backend/lingua/tests/pg_*_it.rs` | `#[ignore]` Postgres tests, run by `backend-it` with the migrator |

## Goals / Non-Goals

**Goals:**
- Every stored gloss is labelled `fr` exactly, before any other label can exist.
- A day's statistic can say its device's native language, so usage can be broken down by pair.
- A client can tell, before pushing, whether the server stores both.
- Nothing an installed client sends is refused or changed.

**Non-Goals:**
- Any client change: changes 11 and 12.
- The usage breakdown by pair in the console: `add-admin-lingua-pair-usage`, optional.
- A language-aware key for cards or statistics: `client_id` is the lemma (one card row for one
  reader's lemma, whatever the native), and a device has one native language a day.
- A runtime feature flag: the capability is a property of the deployed server build, not a
  switch; `card_language` set the pattern (its D4).

## Decisions

### D1 — Two additive string fields, empty meaning `fr`

`CardOp.gloss_language = 12` and `DailyStat.native_language = 8`. An installed client sends
neither; proto3 reads an absent string as empty, and the server reads empty as `fr`, the native
language of every pack ever shipped (M22: existing installs stay French). The precedent's
`language = 11` reads empty as `en` for the same reason on the studied side. *Language values
are normalised on receipt* is modified by one sentence to say which default applies to which
value; its scenarios stand.

Alternative: a `PushCardsV2`, or the native language on the request rather than on each
statistic. A request-level language would make one push carry one native for every stat in it,
which is true today and false the day a backup from another native is restored; the row is the
unit the back office reads.

### D2 — Migration 0006 adds two defaulted columns and no key

`ALTER TABLE lingua.cards ADD COLUMN IF NOT EXISTS gloss_language TEXT NOT NULL DEFAULT 'fr'`,
and the same for `lingua.daily_stats.native_language`. Postgres adds a defaulted column without
rewriting rows. Existing rows read `fr`, exactly (M4).

The keys stay. A card is (reader, studied language, client id), and `client_id` is the lemma:
two devices of one account with different native languages write the same row, last write wins
on `updated_at` with the device tie-break, and the gloss language travels with the gloss that
won — a card holds one gloss, in one language, and says which. A statistic is (reader, day,
studied language, device): a device has one native language, and if it changes during a day the
last upsert of that day sets the row's value, as it sets its counts.

Alternative: the native language in the statistics' key. A native change within a day would
make two rows for one device, double-counting its day in the console; and no reader changes
native twice a day.

### D3 — One normaliser, given its default, in the measured module

`language_core::normalise_or(raw, default)`; `normalise(raw)` is `normalise_or(raw, "en")`, and
two named readers, `gloss_language(raw)` and `native_language(raw)`, are `normalise_or(raw,
"fr")`: trimmed, lowercased, primary subtag, capped, never refused. The gRPC edge calls the
named readers (`deck_grpc::from_proto`, `stats_grpc::from_proto`), as it calls `normalise` for
the studied language; the readers live in `language_core.rs`, which the coverage gate measures,
and their tests are the normalisation scenarios' tests (the adapters are excluded from the gate).

### D4 — One capability bool, `language_labels = 3`

`GetDataStateResponse.language_labels` is `true` on this server build. It says one thing: this
server stores the gloss language of cards and the native language of statistics. It is not a
version number (the precedent's D4). A client that holds a non-French gloss pushes it only when
the flag is true (change 12); a server rolled back answers `false` and the client withholds. The
name is the flag's own, as `card_language` is beside `language`: a bool and a string of one name
in one package would read alike in generated code.

Alternative: two bools. The two columns ship in one migration, in one release; a client that
could see one without the other would be reading a server that never existed.

### D7 — A client that predates labels pulls French-glossed cards only

`PullCardsRequest.any_gloss_language = 3`. A client that sets it receives every card of its
languages, whatever the gloss language (change 11 shows a gloss the reader can read). A client
that leaves it unset — every installed one — receives the cards glossed in `fr` only, withheld
not consumed, as cards of other studied languages are withheld from a client that names none
(the precedent's D3; the cursor returned is the highest sequence among the cards returned).

Why: an installed client pushes its whole deck at every sync with no gloss language, and reviews
rewrite `updated_at`. Had it pulled a card glossed in English, its next grade would push that
English gloss unlabelled, and the edge would store it as `fr` — the one thing M4 forbids. Every
card an installed client has ever pulled is glossed in French, so it loses nothing.

### D5 — Returned as stored; the consolidated read unchanged

`PullCardsResponse` cards carry `gloss_language` as stored, so a device of another native
learns the label (change 11 shows the current pack's gloss when the label differs).
`ConsolidatedStat` (the reader's own statistics screen) is unchanged: it sums a reader's devices
per (day, studied language), and the native language is not a dimension of what the reader
sees.

### D6 — Where the logic lives, and how it is tested

`Card` and `DailyStat` gain a `String` field each, read by the edge through the named readers
of D3. `DeckRepo::changes_since` takes the pull's `any_gloss_language`, and `DeckModule`
passes it (D7). Module tests on the hand fakes (`deck.rs`, `stats.rs`) cover the round trip, the
label travelling with the winning write, and the pull filter; `language_core.rs` tests cover the
default and the normalisation (`gloss_language`, `native_language`); `tests/convergence.rs`
carries the label through two devices; `tests/privacy_allow_list.rs` passes with the new names;
the `#[ignore]` Postgres tests apply 0006 over 0005, read `fr` on a row written before it, and
filter a pull. The adapters stay excluded from the coverage gate, as the repository's rule has
them.

## Risks / Trade-offs

- **A non-French gloss stored under `fr`** → two rules close it: no client sends a non-French
  gloss before this server answers `language_labels: true` (change 12 checks the flag), and a
  client that does not read labels never pulls a non-French gloss it could write back (D7).
- **A rollback after change 12 ships** → a rolled-back server writes a gloss and leaves its
  label; the runbook resets `gloss_language` to `fr` for the rows updated during the rollback
  (migration plan). Before change 12 ships, no label other than `fr` exists and a rollback is a
  plain revert.
- **A proto break** → three added fields, no renumbering; the `proto` workflow runs `buf
  breaking` against the target branch.
- **A migration that locks** → two `ADD COLUMN … DEFAULT` on Postgres 11+, no rewrite, one short
  access-exclusive lock on each table; the previous container is stopped before the new one
  boots, so only the worker's jobs could hold a lock; rehearsed on a copy of production (task 5.1).
- **A test that proves less than its name** → the Postgres tests run on a fresh database, so a
  row "written before 0006" is a row written by SQL that names no label after it; the behaviour
  of rows that existed before the migration is Postgres's catalogue default, rehearsed in 5.1 and
  stated as such in the tests.
- **A name the privacy allow-list refuses** → `gloss_language` and `native_language` contain
  none of its substrings; the test runs in CI.

## Migration Plan

1. Merge and release the backend (release-please).
2. Rehearse 0006 on a copy of production; replay a sync from the published extension and the
   Apple app against it: no request refused, cards and statistics unchanged.
3. Deploy; the migrator runs 0006 at boot.
4. Verify from outside: `GetDataState` answers `language_labels: true`; a card pushed without
   a gloss language comes back with `fr`; a pull without `any_gloss_language` returns the same
   cards as before; a statistic pushed without a native language is stored.
5. Only then may `add-lingua-card-gloss-language` and `add-lingua-native-language-sync-client`
   be built for a store.

Rollback: the previous image refuses to boot while the ledger holds a migration it does not
know (`sqlx::migrate!` validates the applied versions and stops the whole server on a missing
one — true of every migration before this one, and stated here because this design claimed a
plain revert). The runbook is: `DELETE FROM lingua._sqlx_migrations WHERE version = 6`, then
redeploy the previous image; the columns stay, unread, and the flag answers `false`. Until
change 12 ships, that is all. After it, a rolled-back server writes glosses without their label:
on rolling forward again, `UPDATE lingua.cards SET gloss_language = 'fr' WHERE updated_at >=
<rollback>` restores the invariant — every unlabelled write during the rollback came from a
client that pushes French glosses or holds the flag false.
