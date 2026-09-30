# add-lingua-card-language — a deck card carries its studied language, server first

## Why

Lingua is about to study a second language (Spanish, ES→FR, at the English perimeter).
Statuses, declared levels and daily stats already carry a language on the wire and in
their keys; **cards do not**: `CardOp` has no language, `lingua.cards` is keyed
`(user_id, client_id)`, and installed clients mint `client_id` from the lemma
(`crates/lingua-wasm/src/lib.rs`). A Spanish card and an English card spelled the same
(*son*, *red*, *pie*, *once*) would overwrite each other, and — worse — every published
client stamps **every pulled card `en`** (`apps/lingua-extension/src/sync/sync.ts`,
`fetchCards`) and re-uploads its whole deck on each sync: one Spanish card reaching an
installed build would come back to the server as an English card, and propagate.

So the server must learn the language of a card **before any client exists that can
create a non-English card**, and must keep non-English cards away from clients that do
not say they understand them. This change is that server brick, deployable on its own
and inert for today's clients. It is the first change of the Spanish programme and the
only one with a deployment ordering constraint on the backend.

## What Changes

- **Additive `.proto` only** (`backend/lingua/proto`, `buf breaking` FILE rules, no
  breaking marker):
  - `CardOp.language = 11` — the card's studied language; **empty means `en`**, so an
    installed client that never sets it keeps working unchanged.
  - `PullCardsRequest.languages = 2` (repeated) — the languages the client accepts;
    **empty means `["en"]`**, so an installed client only ever receives English cards.
  - `GetDataStateResponse.card_language = 2` (bool) — the server understands card
    languages; a future client checks it before pushing a non-English card, as a guard
    against a rollback of the backend.
- **Migration `0005`**: `lingua.cards` gains `language TEXT NOT NULL DEFAULT 'en'` and
  its primary key becomes `(user_id, language, client_id)`; existing rows stay valid as
  English. Idempotent and schema-qualified like 0001–0004.
- **One language normaliser** for every language value the module receives (statuses,
  declared levels, daily stats, cards): trim, lowercase, primary subtag only
  (`es-ES` → `es`), bounded length, empty → `en`. Applied in the gRPC adapters; the
  server never refuses an installed client over its language value.
- **Deck module**: LWW per `(user, language, client_id)`; `pull_cards` filters by the
  accepted languages; the cursor keeps its meaning (max sequence of what was returned).
- **Back office and worker**: nothing to change (the console never lists cards; the
  purge deletes by user).
- **Not in this change**: no client sends a language yet, no card is created in another
  language, nothing moves in the extension, the Apple app or the agent plugin. The
  clients follow in `add-lingua-language-sync-client` once the server is deployed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

`lingua-sync` and `lingua-stats` exist only inside the open change `add-lingua-backend`
(and `add-lingua-connected-clients`), and `lingua-decks-review`'s *Card schema with
provenance* is MODIFIED by the open change `add-lingua-reader`. This change therefore
**adds requirements only** and never rewrites a held one; `.openspec.yaml` declares
`archiveAfter: add-lingua-backend, add-lingua-connected-clients`.

- `lingua-sync`: ADDED — *A card belongs to one studied language* (wire field, default
  `en`, key on the server), *Non-English cards reach only the clients that accept them*
  (pull filter defaulting to English, capability flag), *Language values are normalised
  on receipt*.
- `lingua-decks-review`: ADDED — *A card is keyed by its studied language* (the same
  lemma in two languages is two cards).
- `lingua-stats`: ADDED — one scenario-level requirement stating that the language of a
  daily stat is normalised the same way (no new field).

## Impact

- **Products.** Backend only (`backend/lingua`: proto, migration, `deck.rs`, `pg_deck.rs`,
  `deck_grpc.rs`, `known_words_grpc.rs`, `stats_grpc.rs`, a new host-tested
  `language_core.rs`, tests). Consumed, not redeclared: the Lingua database pool and
  migrator wiring in `backend/server`, the `purge_user` job in `backend/worker`. Nothing
  changes for Cymbra ID, Music, Live, the back office or the site.
- **Contract.** `cymbra.lingua.v1` gains three fields; the `proto` workflow reports no
  break. Installed extension builds (1.4.0 and earlier) and the Apple app keep working:
  they send no language (read as `en`) and declare no accepted language (served English
  only).
- **Deployment order.** This backend release is deployed to production **before** any
  client able to create a non-English card is built for a store. Rollback is a revert:
  the column and the flag are additive; a rolled-back server serves English cards under
  the old key semantics, and a client that checked `card_language` stops pushing
  non-English cards.
- **Data.** The migration is rehearsed on a copy of production before the release; it
  rewrites no row.
