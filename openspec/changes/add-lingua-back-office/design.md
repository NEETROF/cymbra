# Design — add-lingua-back-office

## Context

The local stack (`add-lingua-analysis` → `add-lingua-agent`) ships a local-only product;
`add-lingua-backend` adds the backend (`backend/lingua`: schema, sync, `StatsService`).
This change closes the operations loop: what the administrator sees of Lingua, and above
all what they do **not** see. A user decision settled upfront: OPS scope only — packs +
flags + aggregates — **no per-account support view**, for privacy (a person's reading
data is sensitive).

Inherited constraints: the `vue-frontend-architecture` skill (the `api()` seam,
exhaustive `Async<T>`, no API call inside a component, an e2e fake client, aligned en/fr
i18n); the separation-of-powers memory (never a flat role check on a console token: a
`back-office` token unions the roles of every scope); the coverage convention (thin
adapters excluded, logic host-tested); and the UI rule "never 'lemma' on screen".

## Goals / Non-Goals

**Goals:**
- An administrator **of the lingua scope** sees aggregated usage (active accounts, words
  learned per day, reviews per day, breakdown by studied language), the registry of pack
  versions, and manages Lingua flags from the existing `/flags` console.
- Privacy is guaranteed **by the schema**: no Lingua admin RPC can carry per-account data.

**Non-Goals:**
- A per-account support view (rejected, not deferred), user search, individual
  drill-down.
- Pack OTA (the registry is read-only, distribution stays bundled in the extension).
- A new flags UI, new telemetry in the Lingua clients, real-time aggregates (daily is
  enough).

## Decisions

### D1 — One capability, and it is `admin-*`: `admin-lingua-console`
The screen belongs to the **admin** product (precedents: `admin-plan-console`,
`admin-account-directory`), even though the RPCs live in `backend/lingua`: the contract
describes what the console allows and forbids. Rejected alternative: a `lingua-admin`
capability on the Lingua product side — the `lingua-*` domain describes the learner
experience; mixing OPS into it would blur the boundary `config.yaml` establishes.

### D2 — OPS-only scope: privacy is a requirement, not an absence
The admin sees **aggregates only**. The strongest way to guarantee that is structural:
the response messages of the Lingua admin protos **carry no account identifier field**,
and the backend only builds `COUNT`/`SUM` grouped by day/language. A leak would require
changing the `.proto` — visible in review and at the `buf breaking` gate. Rejected
alternative: a per-account support view behind a stronger gate — even gated, it creates
the endpoint we would regret; an explicit user decision.

### D3 — Authorisation: `require_admin_in_scope(id, "lingua")`, never the flat set
The console token unions the roles of every scope (the separation-of-powers lesson: 25
sites out of 39 tested the flat set — an admin of one product was an admin of
everything). Every Lingua admin RPC requires `admin` **in the `lingua` scope**
(`backend/platform/src/guard.rs`), with the `global/admin` break-glass included — the
pattern already applied by `backend/analytics` (scope `music`) and by `backend/music`'s
admin RPCs. On the front end the route carries
`meta: { admin: true, adminScope: "lingua" }` (precedent: `/takedowns`) and the nav link
is hidden outside the scope — UX only, since every RPC is re-gated server-side.
Prerequisite: the `lingua` scope declared in `SCOPES`/`APP_SCOPES` (and assignable
through the roles console) — normally done by `add-lingua-backend`; otherwise this change
does it (checked in task 2.1).

### D4 — Aggregates computed on sync data, no new telemetry
`AdminGetLinguaUsage` (tiles) and `AdminGetLinguaUsageSeries` (per-day points) aggregate
the `lingua` schema in SQL (the `lingua_svc` pool — the privilege comes from the pool, not
from the file): active accounts = accounts that synced within the window; words learned =
dated transitions to `known`; reviews = the synced review log; breakdown = the studied
languages of active profiles. An accepted bias, and one that is **displayed**: a device
that does not sync is invisible (an on-screen "synced accounts" note). Rejected
alternatives: wiring in `cymbra-analytics` (app-side events, HMAC per period — designed
for the Music app, redundant here since sync already holds the state); new
extension→backend telemetry (a new privacy surface for zero need).

### D5 — Pack registry: a committed manifest, embedded at compile time
`scripts/lingua-data` emits a `packs-manifest.json` (pack_version, analyzer_version, the
L2→L1 pair, CI build date, size, NOTICE) — small, deterministic, diffed in review like
any pack change. The backend embeds it (`include_str!`) and `AdminListDataPacks` serves it
as-is. Rejected alternatives: a `lingua.data_packs` table fed by CI — that demands an
authenticated CI→production write path for release data that lives perfectly well in the
repo; manual entry in the back office — guaranteed drift. On the day OTA lands, the
manifest moves to DB/object storage and this RPC changes source without changing shape.

### D6 — Flags: registry keys, zero UI
The Lingua flags are `KeyDef`s with app `lingua` in the backend registry
(`backend/feature-flags/src/registry.rs`); the existing `/flags` console lists and
administers them with its current gating. Nothing else. (Any future hardening of
`/flags`'s write gating by scope is a separate `feature-flags-admin` piece of work, out of
scope here.)

### D7 — Front end: the `/usage` pattern, identically
A `stores/lingua.ts` Pinia store behind `api()` (clients added in `lib/transport.ts`,
doubled in the e2e seam), resources as `Async<T>` (`report`, `series`, `packs`) matched
with `match(...).exhaustive()`, window filters (30 days by default), and a `LinguaView.vue`
view reusing `UsageView`'s tile/chart/table components. An RPC error becomes a localised
message inside the union (never a raw gRPC code). i18n: `en.json`/`fr.json` aligned, no
label containing "lemma".

## Risks / Trade-offs

- [Small cohorts: a "breakdown by language" with n=1 all but names someone] → an internal
  console gated on admin-lingua, a risk accepted in v1; if Lingua opens up to
  less-privileged operators, add a display floor ("< 5") — an open question.
- ["Sync-only" bias in the aggregates] → accepted and labelled on screen; it is the direct
  counterpart of "no new telemetry" (D4).
- [`add-lingua-backend` not merged] → this change does not compile without the
  `backend/lingua` crate; the implementation order is sequential, though the specs can be
  ratified in parallel.
- [A committed manifest ≠ the real state of the stores] → the registry describes what CI
  built, not what each store published; accepted (informational) and written into the UI;
  OTA will bring server truth.
- [Removing an admin RPC later = a `buf` FILE break] → the three RPCs are born in a
  dedicated, deliberately minimal proto file; any extension goes through additive fields.
- [The e2e seam may not know how to simulate a scoped admin outside lingua] → extending
  `fixtures.ts`/`e2e-seam.ts` (roles per scope) is part of the e2e task, not a surprise.

## Migration Plan

Nothing to migrate: everything is additive (a new proto file, a new service, a new
screen). Rollback before release = remove the route and do not mount the service. After
release, removing an RPC is an intentional, marked break (`feat(lingua)!:` — the `proto`
gate's pattern). The aggregates create no new data; the pack manifest is a build artefact
versioned with the repo.

## Open Questions

- A display floor for small cohorts ("< 5") from v1, or only if the console opens beyond
  lingua admins?
- Default window: 30 days (aligned with `/usage`) — confirm that Lingua's seasonality does
  not call for 7 days by default.
- `AdminGetLinguaUsage` and `AdminGetLinguaUsageSeries`: would one combined RPC do? Settle
  at implementation by looking at what the screen actually calls (the current split mimics
  `usage.proto`, which has proven itself).
