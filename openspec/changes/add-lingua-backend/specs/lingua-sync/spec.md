# lingua-sync — Cymbra ID account and synchronisation: server and protocol

## ADDED Requirements

### Requirement: The `lingua` audience admitted by configuration alone
Lingua tokens SHALL be issued and refreshed under the `lingua` audience, admitted by adding `lingua` to the `CYMBRA_ALLOWED_AUDIENCES` configuration list — with no new identity code, no new role and no new scope (the platform's `SCOPES`/`APP_SCOPES` are unchanged).

#### Scenario: Issuing a lingua token
- **WHEN** a client calls `SignInLocal` or `SignInOidc` with the `lingua` audience on a server whose `CYMBRA_ALLOWED_AUDIENCES` contains `lingua`
- **THEN** a `TokenPair` is issued for the `lingua` audience and refresh works for that audience

#### Scenario: Unconfigured audience refused
- **WHEN** a client requests the `lingua` audience on a server whose configuration does not list it
- **THEN** sign-in is refused by the existing audience check

### Requirement: General gRPC-web origins without widening the console list
The server SHALL admit product browser origins on the gRPC-web surface through a new `CYMBRA_ALLOWED_WEB_ORIGINS` list (including `chrome-extension://<id>`), served as a union with `CYMBRA_BACK_OFFICE_ORIGINS` by tonic's CORS layer; the console list SHALL NOT be widened and the new list SHALL NOT be credentialed (bearer only, no cookies). An empty list SHALL remain the default (no product origin admitted).

#### Scenario: The extension's preflight is accepted
- **WHEN** the published extension issues a gRPC-web call and its `chrome-extension://<id>` origin is listed in `CYMBRA_ALLOWED_WEB_ORIGINS`
- **THEN** the CORS preflight and the call succeed, without that origin appearing in `CYMBRA_BACK_OFFICE_ORIGINS`

#### Scenario: Unknown origin blocked
- **WHEN** a web page from an unlisted origin attempts a gRPC-web call
- **THEN** the browser blocks the call by CORS, and the auth interceptor remains the authorisation authority for any call that does reach the server

### Requirement: Isolated backend module, inert without configuration
The Lingua backend SHALL be a `backend/lingua` crate modelled on `backend/music`: a `lingua` Postgres schema owned by a `lingua_svc` role with a pinned `search_path`, its own migrations, an injected `UserPort` for any account need (never a read of another schema), and `cymbra.lingua.v1` protos in `backend/lingua/proto`. Without `CYMBRA_LINGUA_DATABASE_URL`, the Lingua services SHALL NOT be wired and the server SHALL start normally.

#### Scenario: Server without a lingua database
- **WHEN** the server starts without `CYMBRA_LINGUA_DATABASE_URL`
- **THEN** it serves the other modules normally and logs that the Lingua services are disabled

#### Scenario: Schema isolation
- **WHEN** a query from the Lingua module runs
- **THEN** it runs on the `lingua_svc` pool, whose `search_path` resolves the `lingua` schema only

### Requirement: Status synchronisation by op-log and last-write-wins
`KnownWordsService` SHALL synchronise statuses per (language, lemma): the client pushes an outbox of timestamped operations (idempotent batches, offset resumption), the server resolves last-write-wins per lemma (most recent timestamp, deterministic per-device tie-break), and the client pulls changes by delta cursor; bootstrap or an invalid cursor SHALL go through a snapshot guarded by ETag/version (an "unchanged" response with no body when the ETag matches).

#### Scenario: A status propagates between two devices
- **WHEN** the user marks `seldom` as known on their Mac and then syncs their iPhone
- **THEN** the iPhone receives the `known` status for `seldom` through the cursor pull

#### Scenario: Conflict resolved by the most recent gesture
- **WHEN** two offline devices set different statuses on the same lemma and then sync
- **THEN** the status with the most recent timestamp wins on the server and both devices converge on it

#### Scenario: An interrupted push resumes without duplicates
- **WHEN** an outbox push is interrupted and then replayed in full
- **THEN** the server state is identical to that of a single push (per-batch idempotence)

### Requirement: Complete card synchronisation, media excluded
`DeckService` SHALL synchronise complete cards — lemma, surface form, source sentence, explicitly captured source, gloss, FSRS state — identified by a stable client id, last-write-wins per card (deletions included). The contents of the `media` field SHALL NOT be synchronised in this change (the schema slot stays local).

#### Scenario: Card created on mobile, reviewed on desktop
- **WHEN** a card created while reading in Safari on iOS is synced and the user then opens the side panel on their Mac
- **THEN** the card appears there with its source sentence and FSRS state, and reviewing it on the Mac propagates back

#### Scenario: Card with an image
- **WHEN** a local card carries media and is synchronised
- **THEN** all of its fields go up except the media content, which stays on the originating device

### Requirement: Strict allow-list of what reaches the server
Only three families of data SHALL go up: lemma statuses, cards and stat aggregates. No browsing URL, no page text and no web reading history SHALL be transmitted or stored server-side — the only source that goes up is the one carried by a card the user explicitly created. The local stack's exposure counters SHALL stay local in this change.

#### Scenario: Reading without capture
- **WHEN** a signed-in user reads ten pages without creating a card or touching a status
- **THEN** no data about those pages (URL, text, per-page counts) is transmitted to the server

#### Scenario: The card is the only exception
- **WHEN** the user creates a card from a page
- **THEN** that card's sentence and source go up with it, and nothing else from the page goes up

### Requirement: Lingua data purged on account deletion
Account deletion (`DeleteAccount`) SHALL purge all of the user's `lingua.*` data through the existing `purge_user` job, extended (worker handler + admin role `search_path` including `lingua`), idempotently; an account with no Lingua data SHALL be a no-op for that step.

#### Scenario: Deleting an account that has Lingua data
- **WHEN** a user with server-side statuses, cards and stats deletes their account
- **THEN** the `purge_user` job erases all of their rows in the `lingua` schema's tables

#### Scenario: Replayed purge
- **WHEN** the `purge_user` job is replayed for the same user
- **THEN** it succeeds without error and without further effect

### Requirement: Platform consumed without redeclaration
Lingua SHALL consume the platform as-is: feature-flag evaluation SHALL derive the app from the token audience (`lingua` automatically, no new mechanism), usage events SHALL go through the existing analytics service (`platform` = `web` for the extension, `ios`/`macos` for the app), and the `cymbra.lingua.v1` services SHALL be reachable over the existing gRPC/gRPC-web route with no reverse-proxy change (no new HTTP route).

#### Scenario: Flag evaluated for the lingua audience
- **WHEN** a client signed in with a `lingua` token evaluates a flag scoped to the `lingua` app
- **THEN** the flag is evaluated in the `lingua` context with no extra configuration on the flags side

#### Scenario: gRPC-web call routed with no Caddy change
- **WHEN** the extension calls `/cymbra.lingua.v1.KnownWordsService/…` through the production reverse proxy
- **THEN** the call reaches tonic through the existing default branch, CORS preflight included
