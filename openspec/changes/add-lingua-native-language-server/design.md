# Design — add-lingua-native-language-server

## Context

See proposal.md (Why). The precedent is `add-lingua-card-language` (archived 2026-10-07): an
additive field with a default, a migration that rewrites nothing, one normaliser applied at the
edge, a capability bool on `GetDataStateResponse`, and a release checked from outside before any
client is built. This change follows it field for field. Where things are today:

| Where | What |
|---|---|
| `backend/lingua/proto/deck.proto` `CardOp` | fields 1–11; `gloss = 6`; `language = 11` (studied, empty = `en`); next free 12 |
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
`language = 11` reads empty as `en` for the same reason on the studied side.

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

### D3 — One normaliser, given its default

`language_core::normalise_or(raw, default)`; `normalise(raw)` is `normalise_or(raw, "en")`.
A gloss or native language is normalised with `"fr"`: trimmed, lowercased, primary subtag,
capped, never refused. Applied at the gRPC edge for cards (`deck_grpc::from_proto`) and
statistics (`stats_grpc::from_proto`), as the studied language is.

### D4 — One capability bool, `native_language = 3`

`GetDataStateResponse.native_language` is `true` on this server build. It says one thing: this
server stores the gloss language of cards and the native language of statistics. It is not a
version number (the precedent's D4). A client that holds a non-French gloss pushes it only when
the flag is true (change 12); a server rolled back answers `false` and the client withholds.

Alternative: two bools. The two columns ship in one migration, in one release; a client that
could see one without the other would be reading a server that never existed.

### D5 — Returned as stored; the consolidated read unchanged

`PullCardsResponse` cards carry `gloss_language` as stored, so a device of another native
learns the label (change 11 shows the current pack's gloss when the label differs).
`ConsolidatedStat` (the reader's own statistics screen) is unchanged: it sums a reader's devices
per (day, studied language), and the native language is not a dimension of what the reader
sees.

### D6 — Where the logic lives, and how it is tested

`Card` and `DailyStat` gain a `String` field each, defaulted by the edge. Module tests on the
hand fakes (`deck.rs`, `stats.rs`) cover the default, the normalisation and the round trip;
`language_core.rs` tests cover `normalise_or`; `tests/convergence.rs` carries the label through
two devices; `tests/privacy_allow_list.rs` passes with the new names; the `#[ignore]` Postgres
tests apply 0006 over 0005 and read `fr` on a row written before it. The adapters stay
excluded from the coverage gate, as the repository's rule has them.

## Risks / Trade-offs

- **A non-French gloss stored under `fr`** → impossible by order: no client sends a gloss
  language before this server answers `native_language: true` (change 12 checks the flag), and
  no shipped pack is glossed in anything but French.
- **A proto break** → three added fields, no renumbering; the `proto` workflow runs `buf
  breaking` against the target branch.
- **A migration that locks** → two `ADD COLUMN … DEFAULT` on Postgres 11+, no rewrite; rehearsed
  on a copy of production (task 5.1).
- **A name the privacy allow-list refuses** → `gloss_language` and `native_language` contain
  none of its substrings; the test runs in CI.

## Migration Plan

1. Merge and release the backend (release-please).
2. Rehearse 0006 on a copy of production; replay a sync from the published extension and the
   Apple app against it: no request refused, cards and statistics unchanged.
3. Deploy; the migrator runs 0006 at boot.
4. Verify from outside: `GetDataState` answers `native_language: true`; a card pushed without
   a gloss language comes back with `fr`; a statistic pushed without a native language is
   stored.
5. Only then may `add-lingua-card-gloss-language` and `add-lingua-native-language-sync-client`
   be built for a store.

Rollback: revert and redeploy; the columns stay, unread, and the flag answers `false`.
