# add-lingua-back-office — Cymbra Lingua: the back office's admin section (OPS)

## Why

Once the Lingua backend is in place (`add-lingua-backend`: the `lingua` schema, sync,
`StatsService`), running the product is blind: there is no way to know how many accounts
use Lingua, at what rate, on which languages, nor which data-pack versions are in
circulation. The Vue back office is already the admin console for every product (music,
plans, flags, usage) — Lingua should plug into it, not invent a console of its own.

The scope is **OPS only, by product decision**: aggregates, the pack registry, flags.
**No per-account support view** — Lingua data (words encountered, decks, review history)
describes what a person reads; that is sensitive by nature, and the administrator has no
operational need to see it. That limit is a spec requirement, not an omission.

## What Changes

- **A new `admin-lingua-console` capability**: the back office's "Lingua" screen and the
  admin RPCs that serve it.
- **New admin protos in `backend/lingua/proto`** (a dedicated `lingua_admin.proto`,
  service `LinguaAdminService`): `AdminGetLinguaUsage` (aggregate tiles),
  `AdminGetLinguaUsageSeries` (per-day series), `AdminListDataPacks` (the pack registry).
  Behind the existing **strict** auth interceptor, gated by
  `require_admin_in_scope(id, "lingua")` — the pattern of `backend/music`'s and
  `backend/analytics`'s admin RPCs, with the **lingua** scope made explicit (the
  separation-of-powers lesson: a `music/admin` does not administer Lingua by accident).
- **Backend implementation** in `backend/lingua`: SQL aggregates over the `lingua` schema
  (the `lingua_svc` pool), no new telemetry — we aggregate what sync already holds. No
  response carries an account identifier.
- **The pack registry is a committed manifest**: `scripts/lingua-data` emits a
  `packs-manifest.json` (pack_version, analyzer_version, the L2→L1 pair, CI build date,
  size, NOTICE), embedded by the backend at compile time and served as-is. Pack
  distribution stays **bundled in the extension** in v1: the registry is informational
  and paves the way for a future OTA.
- **The "Lingua" back-office screen** (`apps/back-office`, route `/lingua`), following the
  `vue-frontend-architecture` skill strictly: a Pinia store behind the
  `api()`/`setClientsForTest` seam, async state as exhaustively matched `Async<T>` unions,
  no API call inside a component. The visual pattern of the `/usage` screen (tiles + time
  series + breakdowns). The nav entry and the route are gated by
  `adminScope: "lingua"` (precedent: `/takedowns`).
- **Lingua flags: no new UI.** The keys are declared in the backend registry (`KeyDef`,
  app `lingua`) and administered through the **existing** `/flags` console.
- UI vocabulary: as everywhere in Lingua, the word "lemma" never appears on screen (say
  "words learned", "distinct words", "dictionary form").

## Capabilities

### New Capabilities
- `admin-lingua-console`: the back office's Lingua section — `lingua`-scoped
  authorisation, usage aggregates (tiles + series + breakdown by studied language), a
  hard privacy rule (aggregates only, never per-account data), a read-only registry of
  pack versions, flags through the existing console, and localised async states.

### Modified Capabilities
_None. The `/flags` console (`feature-flags-admin`) is consumed as-is — the Lingua keys
are only backend registry declarations, already covered by `runtime-feature-flags`. Back
office authentication (`back-office-admin-session`) and the scoped-role model are
consumed, not modified (the `lingua` scope is one more value in an existing list, not new
behaviour)._

## Impact

- **Products**: **back office** = all the new front end (screen, store, nav, en/fr i18n);
  **Lingua backend** = new protos + admin RPCs in `backend/lingua` (depends on
  `add-lingua-backend`, which creates the crate, the schema and sync); **ID** = consumed
  (`back-office` audience, scoped roles, `require_admin_in_scope`); Music / Live / site:
  untouched.
- **Dependency**: `add-lingua-backend` must be implemented first (the `lingua` schema, the
  `lingua_svc` pool, `StatsService`). If that change has not already declared the `lingua`
  scope (`SCOPES`/`APP_SCOPES` in `backend/platform` + role assignment), this change does
  it.
- **Tree**: `backend/lingua/proto/lingua_admin.proto` + `src` (aggregates, gating);
  `scripts/lingua-data` (manifest emission); `apps/back-office/src` (stores/lingua.ts,
  views/LinguaView.vue, transport, router, i18n); `apps/back-office/e2e/lingua.spec.ts`.
- **CI**: no new unit — `backend/lingua` is covered by the `rust` lane (workspace) and
  `apps/back-office` by `back-office-check`; `ci-units` unchanged. The new proto falls
  under the `proto` workflow's `buf breaking` gate (a new file: it passes). Coverage
  ≥ 80% on both sides; the thin adapters (`pg*.rs`, `grpc.rs`) follow the existing
  exclusion convention, and the aggregation logic stays host-tested.
- **Out of scope (explicitly)**: a per-account support view (rejected for privacy — not
  "later", rejected), pack OTA (the registry prepares it, does not ship it), writing the
  registry from CI to production, a new flags UI, real-time aggregates.
