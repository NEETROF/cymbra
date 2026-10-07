## Context

`backend/lingua` syncs three families of data. Statuses and declared levels are keyed by
`(user_id, language, lemma)` / `(user_id, language)`, daily stats by
`(user_id, day, language, device_id)`; the language is an opaque `TEXT` the server never
normalises (`known_words_grpc.rs`, `stats_grpc.rs`). Cards are the exception: `CardOp`
(`backend/lingua/proto/deck.proto`) has ten fields and no language, `lingua.cards` is keyed
`(user_id, client_id)` (`migrations/0001_lingua.sql`), and the upsert conflicts on that key
(`pg_deck.rs`). The published clients mint `client_id` from the lemma
(`crates/lingua-wasm/src/lib.rs`, `export_card_ops`), push their whole deck on every sync,
and stamp every pulled card `en` (`apps/lingua-extension/src/sync/sync.ts`, `fetchCards`),
because the wire has nothing else to offer.

The Spanish programme needs cards keyed by language before any client can create one.
This change is the server half; the client half (`add-lingua-language-sync-client`) ships
in a later release and checks the server first.

Constraints: `buf breaking` with the FILE rule set on every proto pull request; one
change = one pull request; `lingua-sync` and `lingua-stats` live only in open changes, so
deltas are ADDED only; Postgres adapters (`pg_*.rs`) and tonic adapters (`*_grpc.rs`) are
coverage-excluded, so every rule must live in a host-tested module.

## Goals / Non-Goals

**Goals:**

- A card has a studied language on the wire and in its key; the same lemma in two
  languages is two cards.
- Installed clients (extension ≤ 1.4.0, Apple app 1.3.x) keep working with no change and
  never receive a non-English card.
- Language values received by the module are normalised once, the same way, everywhere.
- A future client can tell whether the server it talks to understands card languages.

**Non-Goals:**

- No client change of any kind (extension, Apple app, agent plugin).
- No per-language filter on `GetStats` beyond the existing `language` field (`""` still
  means every language; a multi-language selector is not needed by the programme).
- No language on `lingua.data_erasures` (erasure stays whole-Lingua, per user).
- No allow-list of languages: the server accepts any well-formed code, so a third
  language needs no backend release.

## Decisions

### D1 — Language on `CardOp` as an additive field, empty meaning `en`

`string language = 11` is appended after `device_id = 10`. An absent or empty value is
read as `en` on receipt and written as the stored value on return. Installed clients send
nothing and keep being understood; they also keep reading what they receive as `en`,
which is exactly what the server now guarantees them (D3).

*Rejected — a new message or a `PushCardsV2` RPC.* Nothing the FILE rules refuse is
needed; a second RPC would leave the first one to rot.

*Rejected — encoding the language inside `client_id` (`es:lemma`).* Installed clients
fall back from `lemma` to `client_id` when they apply a card (`lingua-wasm`,
`apply_card_ops`), so a compound id would be read as a lemma by any client that ignores
the field. The key change on the server (D2) makes the prefix unnecessary; the client
change may still choose to mint distinct ids, but the server does not depend on it.

### D2 — `(user_id, language, client_id)` as the primary key, migration 0005

`lingua.cards` gains `language TEXT NOT NULL DEFAULT 'en'` and the primary key is
replaced by `(user_id, language, client_id)`. Existing rows read `en`, the value every
installed client would have sent, so the migration rewrites nothing and keeps the
LWW semantics for every card that exists today. The `cards_cursor (user_id, seq)` index and
the `lingua.change_seq` sequence do not move: the change sequence stays per user.

The upsert conflicts on the new key. A push of the same `client_id` in two languages
therefore yields two rows — which is the point — and the tie-break rule
(`updated_at`, then `device_id`) is unchanged.

*Rejected — keep the key and rely on the client namespacing ids.* It would make the
server's correctness depend on every client's minting rule, including the agent plugin's.

### D3 — The pull filters by the languages the client accepts; empty means English only

`PullCardsRequest.languages` (repeated) names the languages a client can hold. The server
returns only cards whose language is in that set, and treats an empty list as `["en"]`:
every installed client, which sends nothing, receives English cards only. This is the
protection against the re-upload loop described in the proposal, and it costs the old
client nothing it had.

The cursor keeps its meaning — the highest sequence among the cards returned, or the
request's cursor when nothing was returned. Cards filtered out are not "consumed": a
client that later accepts a new language re-pulls from an earlier cursor (the client
change decides when; the server needs no memory of what it withheld).

*Rejected — gating on a client version header.* There is no such header in the
transport, and a declared set is what the reader's choice of languages naturally maps to.

*Rejected — doing nothing because the installed base is small.* The pollution is
self-propagating: each sync re-uploads the whole deck, so one mis-stamped card would
overwrite the Spanish card on the server and reach every other device.

### D4 — A capability flag on `GetDataStateResponse`

`bool card_language = 2` is set to `true` by this server. Every sync already starts by
reading `GetDataState` (the erasure mark), so a future client learns, at no extra call,
whether it may push a non-English card. A rolled-back backend answers `false` (proto3
default), and the client withholds its non-English cards rather than have them stored
under the old key. The flag is the only new thing the data service says; it is not a
version number and is not meant to grow into one.

### D5 — One normaliser, host-tested, applied at the edge

`language_core::normalise(raw) -> String`: trim, lowercase, keep the primary subtag
(`es-ES`, `es_419`, `spa-ESP` → `es`, `spa`), cap the length at 8 bytes, empty → `en`.
Applied by the three gRPC adapters to statuses, declared levels, daily stats and cards on
the way in, and to `GetStatsRequest.language` (an empty value still means every
language, so the normaliser is applied only when the field is non-empty). It refuses
nothing: an unknown code is stored as normalised and served back. Storing `es` and `ES`
under one key is the goal; deciding which languages exist is not the server's job.

Only new writes are normalised; rows that already exist were all written as `en` by the
published clients, so nothing needs rewriting.

### D6 — Where the logic lives

`Card` gains `language`; `DeckRepo::changes_since` takes the accepted languages;
`DeckModule::pull_cards` defaults an empty set to `["en"]` and computes the cursor. The
fake repo in `deck.rs` tests reproduces the composite key so the LWW tests cover the
two-language case on the host. `pg_deck.rs` only carries the SQL (`language = ANY($3)`),
and the Postgres integration test seeds cards in two languages to prove the key and the
filter against a real database.

## Risks / Trade-offs

- [An installed client re-uploads English cards under the old semantics after the
  migration] → it sends no language, the server reads `en`, and the row it targets is the
  same one it targeted before (`en` is the migrated value). Rehearsed on a copy of
  production with the published builds (extension 1.4.0, Apple 1.3.x) before the release.
- [A client sends a language the server has never seen] → stored as normalised, served
  only to clients that accept it. No error path, nothing to deploy for a third language.
- [The filter hides cards from a client that widens its accepted set later] → by design;
  the client change resets or rewinds its cursor when its set grows, and this server keeps
  no per-client memory, so a rewind is always safe (idempotent LWW).
- [Primary-key replacement locks `lingua.cards` for the duration] → the table is small
  (one row per card per user) and the migration runs at server start-up as the previous
  four did; the rehearsal measures it.
- [Coverage] → the normaliser, the default-to-English rules and the composite-key LWW are
  in host-tested modules; `pg_deck.rs` and `deck_grpc.rs` stay thin and excluded.

## Migration Plan

1. Merge; `buf breaking` reports no break (three added fields), no `!` marker.
2. Backend release; the migrator applies `0005` at start-up. Nothing else deploys.
3. Verify from outside: a pull with no `languages` on a test account returns the same
   cards as before; `GetDataState` answers `card_language: true`.
4. Only then may `add-lingua-language-sync-client` be built for a store.
5. Rollback: revert the pull request and release. The column and the flag are inert for
   the old code; a card stored in another language is simply not addressable by the old
   key until the change is redeployed. No data is lost.

## Open Questions

None. The client-side identity of non-English cards (bare lemma or prefixed id) is the
client change's decision; this server works with either.
