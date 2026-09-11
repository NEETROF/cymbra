# admin-lingua-console Specification

## Purpose
TBD - created by archiving change add-lingua-back-office. Update Purpose after archive.
## Requirements
### Requirement: Scoped authorisation of the Lingua admin RPCs
Every Lingua administration RPC (`LinguaAdminService`) SHALL require the `admin` role **in the `lingua` scope** (`require_admin_in_scope`), with the `global/admin` break-glass still accepted; a token without that scoped role — including a console token holding `admin` in another scope — SHALL be refused with `PermissionDenied`. The service SHALL be mounted behind the strict authentication interceptor (a valid token required, the `back-office` audience admitted).

#### Scenario: Admin of another product refused
- **WHEN** a holder of `music/admin` (with no role in the `lingua` scope) calls `AdminGetLinguaUsage`
- **THEN** the response is `PermissionDenied`, even though their flat role set contains `admin`

#### Scenario: Lingua admin and break-glass accepted
- **WHEN** a holder of `lingua/admin` or of `global/admin` calls a `LinguaAdminService` RPC
- **THEN** the call is authorised

### Requirement: Screen reserved for admins of the lingua scope
The back office SHALL show the "Lingua" navigation entry and serve the `/lingua` route only to administrators of the `lingua` scope (`meta: { admin: true, adminScope: "lingua" }`); any other profile SHALL be redirected without a raw error. This route gate is a UX convenience: every RPC stays independently gated server-side.

#### Scenario: Moderator with neither the link nor access
- **WHEN** a non-admin moderator signed in to the console navigates to `/lingua`
- **THEN** the "Lingua" link is absent from the navigation and the route redirects them away from `/lingua`

#### Scenario: Music-only admin redirected
- **WHEN** an administrator holding `admin` only in the `music` scope navigates to `/lingua`
- **THEN** the route redirects them (they lack the `lingua` scope), exactly as the server would if they called the RPCs directly

### Requirement: Lingua usage aggregates
The "Lingua" screen SHALL present, over a filterable date window (30 days by default), aggregates served by the Lingua backend's admin RPCs: tiles (active synced accounts, words learned, reviews), per-day time series, and a breakdown by studied language. The aggregates are computed server-side over the existing synchronisation data — no new telemetry — and the screen states that only accounts that sync are counted. Labels are plain-language: no UI string contains the word "lemma" (say "words learned", "distinct words").

#### Scenario: Consulting a window
- **WHEN** a lingua admin opens the screen with the default window
- **THEN** the tiles, the per-day series and the breakdown by studied language for that window are displayed, together with the "synced accounts" note

#### Scenario: Plain-language vocabulary
- **WHEN** the screen's UI strings (en and fr) are run through the vocabulary lint
- **THEN** no occurrence of "lemma" is found

### Requirement: Privacy — aggregates only, guaranteed by the schema
The Lingua console SHALL present aggregates only: no `LinguaAdminService` RPC SHALL return data attributable to an individual account (words encountered, decks, an account's statistics or activity), no response message in `lingua_admin.proto` SHALL carry an account identifier field, and the screen SHALL offer no per-account search or view. This limit is the scope of the Lingua admin product, not a temporary restriction.

#### Scenario: The response schema cannot leak an account
- **WHEN** the response messages of `lingua_admin.proto` are inspected
- **THEN** none carries an account identifier field or per-account content — a leak would require a `.proto` change, visible in review and at the proto gate

#### Scenario: No per-account view in the console
- **WHEN** a lingua admin browses the "Lingua" screen
- **THEN** no per-account search, no link to an account and no individual detail is offered

### Requirement: Registry of data-pack versions
The screen SHALL display a **read-only** registry of published data packs — `pack_version`, `analyzer_version`, the L2→L1 pair, CI build date, size, NOTICE — served by `AdminListDataPacks` from the manifest produced by the pack pipeline and versioned with the repo. Pack distribution SHALL stay bundled in the extension in v1: the registry is informational and paves the way for a future OTA, it drives no distribution.

#### Scenario: Consulting the registry
- **WHEN** a lingua admin opens the screen's packs section
- **THEN** every published pack appears with its version, its `analyzer_version`, its L2→L1 pair, its CI build date, its size and its NOTICE available to read, with no write or publish action

### Requirement: Lingua flags through the existing console
Lingua feature flags SHALL be keys declared in the backend registry (`KeyDef`, app `lingua`) and administered by the existing `/flags` console with its current gating; this change SHALL introduce no new flags interface.

#### Scenario: A lingua key shows up in /flags
- **WHEN** a flag key is declared in the backend registry with the `lingua` app
- **THEN** it is listed and administrable in the existing `/flags` console, with no new screen or component

### Requirement: Localised async states on the screen
Every asynchronous resource on the screen (aggregates, series, packs) SHALL be modelled as an exhaustively matched `Async<T>` discriminated union, loaded exclusively through a Pinia store behind the `api()` seam; an RPC failure SHALL yield a localised error message inside the union — never a raw gRPC code or exception on screen, and never an API call from a component.

#### Scenario: An aggregates RPC fails
- **WHEN** `AdminGetLinguaUsage` fails during loading
- **THEN** the screen renders the error state with a localised message, the technical cause being logged only

