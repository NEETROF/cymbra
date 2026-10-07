# lingua-sync Specification

## Purpose
TBD - created by archiving change add-lingua-connected-clients. Update Purpose after archive.
## Requirements
### Requirement: Optional account, local-first preserved
The extension and the container app SHALL remain fully usable without an account: highlighting, statuses, decks, review and local stats all work with no connection and no sign-in, and synchronisation SHALL start only after an explicit sign-in (opt-in). Signing out SHALL stop synchronisation without altering local state.

#### Scenario: Account-free use unchanged
- **WHEN** a user without an account reads a page with the extension installed
- **THEN** highlighting, the word popup and review work in full and no request is sent to the Cymbra backend

#### Scenario: Signing out loses nothing
- **WHEN** a signed-in user signs out of the extension
- **THEN** their local statuses, cards and stats stay intact and usable offline, and no further sync request is sent

### Requirement: Extension sign-in over gRPC-web bearer
The extension SHALL sign in over gRPC-web with a bearer `TokenPair` (never the `/web/auth` cookie surface): auth/refresh interceptors modelled on the back office, single-flight refresh with exactly one retry, the access token kept in `storage.session` and the refresh token in `storage.local`. The sign-in methods SHALL be those specified by `lingua-account` (at least Google OIDC through `chrome.identity.launchWebAuthFlow`, client id added to the `CYMBRA_GOOGLE_AUDIENCE` CSV, and email/password through `SignInLocal`).

#### Scenario: Google sign-in from the extension
- **WHEN** the user picks "Continue with Google" in the extension
- **THEN** the `launchWebAuthFlow` flow produces an id_token, `SignInOidc` returns a `TokenPair` for the `lingua` audience, and subsequent calls carry `Authorization: Bearer`

#### Scenario: Expired access token refreshed silently
- **WHEN** a sync call fails with UNAUTHENTICATED while a valid refresh token is present
- **THEN** the client refreshes exactly once (single-flight), replays the call with the new token, and the user sees no interruption

#### Scenario: Browser restart
- **WHEN** the user restarts their browser
- **THEN** the access token from `storage.session` is gone, the refresh token from `storage.local` obtains a new `TokenPair`, and the session continues without re-entering credentials

### Requirement: Apple and Google sign-in on Safari through the host app
On Safari, which has no `identity.launchWebAuthFlow`, the container app SHALL provide Sign in with Apple and Continue with Google natively (Sign in with Apple first) and SHALL hand the resulting id_token to its Safari extension through the App Group, readable once and for at most five minutes. The Safari extension SHALL exchange that id_token through `SignInOidc` against the `lingua` audience and SHALL own the resulting session like every other variant. The container app SHALL hold neither a session nor learning state, and the email flows SHALL stay in the extension.

#### Scenario: Apple sign-in from Safari
- **WHEN** the reader picks "Continue with Apple" in the Safari extension, completes the native Apple sheet in the container app and goes back to Safari
- **THEN** the extension exchanges the handed id_token through `SignInOidc`, is signed in to the same Cymbra account as with Apple on any other client, and schedules a sync

#### Scenario: A handed id_token is used once
- **WHEN** the extension has collected a handed id_token, or the token is older than five minutes
- **THEN** the App Group no longer holds it, and an expired token is discarded without any RPC

#### Scenario: Email flows stay in the extension
- **WHEN** a Safari reader signs up, verifies an email or resets a password
- **THEN** it happens in the extension's account page, with no native screen involved

### Requirement: Synchronisation keeps up with the reader
A signed-in device SHALL exchange with the server without the reader having to restart their browser: after a local mutation, when a Lingua surface opens (popup, in-page drawer, side panel), when a page loads or the reader returns to a tab, and when the extension's background wakes. Those triggers SHALL be throttled — at most one exchange every 10 s for a surface the reader opened, every 60 s for a page load — counted from the last successful one, which SHALL survive a background restart. A trigger SHALL be acknowledged only once its exchange is over, so the requesting surface keeps the background alive until then. Réglages SHALL show when this device last synced and SHALL offer a manual exchange, reporting a failure by category, never as a raw message. No exchange SHALL happen while signed out.

#### Scenario: A card captured on another device
- **WHEN** a reader adds a card on their phone, then opens the extension's panel on their Mac
- **THEN** the Mac exchanges with the server without being restarted, and the panel shows the new card

#### Scenario: Repeated openings
- **WHEN** the reader opens the popup three times within a few seconds
- **THEN** a single exchange runs

#### Scenario: A background the browser may suspend
- **WHEN** a surface triggers an exchange
- **THEN** it is told the exchange is done only when it is, and the exchange completes even though the browser suspends an idle background

#### Scenario: Synchronising on demand
- **WHEN** the reader chooses « Synchroniser maintenant » in Réglages while the server is unreachable
- **THEN** they see a categorized message and the displayed last-sync time is unchanged

#### Scenario: Signed out
- **WHEN** any of those moments happens on a signed-out device
- **THEN** no request is sent

### Requirement: A word reclassified on another device converges here
Every decision about a word SHALL carry the time it was made, **including putting the word in the deck**, so two devices deciding differently converge on the later decision rather than on an undated one. When a pulled `known` or `ignored` wins for a word this device has carded, the card SHALL stop coming due here — the retirement the gesture performs on the device that decided, which that device could not perform for a card it did not hold. Retiring SHALL NOT date the card before its own last edit.

#### Scenario: Deck on one device, known on the other
- **WHEN** a reader puts a word in the deck on their Mac, then marks the same word known on their phone, and both devices sync
- **THEN** both show it as known and its card no longer comes up for review, on either device

#### Scenario: The deck wins when it comes last
- **WHEN** the word was marked known first and put in the deck afterwards
- **THEN** both devices show it as being learned, with its card

#### Scenario: A card edited after the status it retires
- **WHEN** a known dated before this device's last review of that card is pulled and wins the status
- **THEN** the card is retired and keeps its own, later date, so the retirement reaches the other devices

### Requirement: Local store merged at first sign-in
At the first sign-in of a device holding pre-account local state, the client SHALL push that state in full as operations preserving their original timestamps, then pull the merged state; the merge SHALL be the protocol's ordinary last-write-wins (no special case) and SHALL lose nothing: every local status, card or aggregate missing from the server is created.

#### Scenario: First sign-in after months of local use
- **WHEN** a user with 2,000 local statuses and 150 local cards signs in for the first time
- **THEN** all of it goes up with its original timestamps, and the merged state pulled back contains the union with any pre-existing server state

#### Scenario: A second device already used locally
- **WHEN** a second device with its own local state signs in to the same account
- **THEN** the two states are merged by last-write-wins and no card from either device is lost

### Requirement: The Claude Code plugin stays out of synchronisation
The Claude Code plugin's local store (`~/.lingua/`) SHALL NOT acquire any network path in this change: the ingestion's "no network connection" invariant SHALL remain tested and true, plugin data synchronisation being a later change.

#### Scenario: Ingestion still offline
- **WHEN** the `Stop` hook ingests a transcript while the user has a Lingua account signed in inside their extension
- **THEN** the ingestion opens no network connection and the plugin's store stays purely local

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
`DeckService` SHALL synchronise complete cards — lemma, surface form, source sentence, gloss, FSRS state — identified by a stable client id, last-write-wins per card (deletions included). The page a card was captured from stays on the device (`add-lingua-privacy-controls`): the card's `source` field is deprecated, sent empty and never stored. The contents of the `media` field SHALL NOT be synchronised in this change (the schema slot stays local).

#### Scenario: Card created on mobile, reviewed on desktop
- **WHEN** a card created while reading in Safari on iOS is synced and the user then opens the side panel on their Mac
- **THEN** the card appears there with its source sentence and FSRS state, and reviewing it on the Mac propagates back

#### Scenario: Card with an image
- **WHEN** a local card carries media and is synchronised
- **THEN** all of its fields go up except the media content, which stays on the originating device

### Requirement: Strict allow-list of what reaches the server
Only three families of data SHALL go up: lemma statuses, cards and stat aggregates. No browsing URL, no page text and no web reading history SHALL be transmitted or stored server-side — not even the page a card was captured from (`add-lingua-privacy-controls`). The local stack's exposure counters SHALL stay local in this change.

#### Scenario: Reading without capture
- **WHEN** a signed-in user reads ten pages without creating a card or touching a status
- **THEN** no data about those pages (URL, text, per-page counts) is transmitted to the server

#### Scenario: The card is the only exception
- **WHEN** the user creates a card from a page
- **THEN** that card's sentence goes up with it, and nothing else from the page goes up — not even its address

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

### Requirement: A card belongs to one studied language
`DeckService` SHALL carry the studied language of every card as a field of the card operation, SHALL read an absent or empty language as `en`, and SHALL identify a card on the server by (user, language, client id), so that the same client id in two languages is two cards. The server SHALL return each card with the language it stored.

#### Scenario: An installed client that sends no language
- **WHEN** a client built before this change pushes a card without a language
- **THEN** the server stores it as an English card, under the same key the client has always targeted, and last-write-wins applies as before

#### Scenario: The same lemma in two languages
- **WHEN** a client pushes a Spanish card and an English card whose client ids are both `son`
- **THEN** the server holds two cards, and an update to one leaves the other untouched

#### Scenario: A card comes back with its language
- **WHEN** a client that accepts Spanish pulls its cards
- **THEN** every Spanish card returned carries `es` and every English card carries `en`

### Requirement: Non-English cards reach only the clients that accept them
A pull SHALL name the languages the client accepts; the server SHALL return only cards in those languages and SHALL treat an empty list as English only, so that a client that predates card languages never receives a card it would mis-file. The cursor returned SHALL be the highest change sequence among the cards returned, or the request's cursor when none was; the server SHALL keep no memory of what it withheld, so a client that later accepts more languages can pull again from an earlier cursor.

#### Scenario: An installed client never sees a Spanish card
- **WHEN** an account holds Spanish cards and a client built before this change pulls with its usual request
- **THEN** it receives the English cards only, and its cursor advances no further than the last English card returned

#### Scenario: A client that accepts both languages
- **WHEN** a client pulls with `languages` set to `en` and `es`
- **THEN** it receives the cards of both languages in sequence order, tombstones included

#### Scenario: Widening the accepted set
- **WHEN** a client that used to pull English only pulls again from an earlier cursor with `es` added
- **THEN** it receives the Spanish cards it had never been sent, and re-receiving English cards it already holds changes nothing (idempotent last-write-wins)

### Requirement: The server states that it understands card languages
The data-state read every sync begins with SHALL say whether the server understands card languages, so that a client can withhold its non-English cards from a server that would store them under the old key.

#### Scenario: A client checks before pushing
- **WHEN** a client that holds Spanish cards reads the data state and the server answers that it understands card languages
- **THEN** the client may push them; had the answer been absent or negative, it pushes its English cards only

### Requirement: Language values are normalised on receipt
Every language value the Lingua services receive — on statuses, declared levels, daily stats and cards — SHALL be normalised the same way before it is stored or used as a filter: trimmed, lowercased, reduced to its primary subtag, bounded in length, and read as `en` when empty. The server SHALL NOT refuse a request over its language value.

#### Scenario: Regional and upper-case codes
- **WHEN** one device pushes a status for `es-ES` and another for `ES`
- **THEN** both land on the same (user, `es`, lemma) key

#### Scenario: A language the server has never seen
- **WHEN** a client pushes a card in `pt`
- **THEN** it is stored as a `pt` card and returned only to clients that accept `pt`

### Requirement: A device pulls the cards of its reader's languages
A device SHALL accept the reader's studied languages that its package ships, or the default pair's language when none is, and SHALL name them in every card pull. It SHALL file each pulled card under the language the server returns, reading an empty value as English. It SHALL NOT apply pulled statuses or declared levels in a language it does not accept.

#### Scenario: Every reader today
- **WHEN** a reader who studies English alone syncs
- **THEN** the card pull names English, and every card and status is applied as before

#### Scenario: A Spanish card comes back as Spanish
- **WHEN** a device that accepts English and Spanish pulls a card the server returns as Spanish
- **THEN** the card is filed under Spanish, next to an English card with the same spelling

#### Scenario: A status in a language the device does not accept
- **WHEN** a device that accepts English alone pulls a Spanish status
- **THEN** the status is not applied, and no Spanish pack is loaded

### Requirement: A device pushes a non-English card only to a server that keys cards by language
A device SHALL send every card's language with it. It SHALL push a card in a language other than English only when the data state read at the start of the sync says the server understands card languages, and SHALL otherwise push its English cards only.

#### Scenario: A server that understands card languages
- **WHEN** a device holding English and Spanish cards syncs with a server that answers it understands card languages
- **THEN** both are pushed, each with its language

#### Scenario: A server that predates card languages
- **WHEN** the same device syncs with a server whose data state does not say it understands card languages
- **THEN** only the English cards are pushed

### Requirement: Widening a device's languages pulls again from the start
A device SHALL remember the languages of its last successful sync, reading a device that never stored them as having accepted English alone. When it accepts a language it did not, it SHALL pull statuses and cards again from the start, so it receives what the server withheld or it skipped before. Narrowing the languages SHALL NOT reset anything.

#### Scenario: Adding Spanish
- **WHEN** a device that synced in English alone starts accepting Spanish
- **THEN** its next sync pulls statuses and cards from the start, and the cards it already holds stay as they are

#### Scenario: Updating the extension
- **WHEN** a device that synced before this change, with no stored languages, syncs in English alone
- **THEN** its cursors are kept and nothing is pulled again

