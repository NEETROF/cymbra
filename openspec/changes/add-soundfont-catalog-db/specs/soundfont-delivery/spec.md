## MODIFIED Requirements

### Requirement: Server-owned SoundFont catalog

The system SHALL maintain a server-owned mapping from each **SoundFont id** to its
storage key, its access tier (free/paid), and its licence/attribution, **persisted
in a database table** as the single source of truth. The delivery route and the
entitlement check SHALL resolve fonts through this persisted catalog, so
client-facing ids are decoupled from bucket keys and the required attribution for
redistributed (e.g. CC-BY) fonts is recorded alongside the font. Adding a font
SHALL be a **data change** — inserting a catalog row and uploading its object — not
a code change. The catalog SHALL be seeded with the free default font so it is
present without any manual step.

#### Scenario: Id resolves to a stored object via the persisted catalog

- **WHEN** the route receives a known SoundFont id
- **THEN** it resolves the id to the object's storage key and tier by reading the
  persisted catalog before fetching

#### Scenario: Attribution is recorded for redistributed fonts

- **WHEN** a redistributed font requiring attribution (e.g. CC-BY) is in the catalog
- **THEN** its licence and required credit are recorded in the catalog row

#### Scenario: A font is added as data, not code

- **WHEN** a new font's row is inserted into the catalog and its object uploaded to
  the bucket
- **THEN** the delivery route resolves and serves that font with no code change or
  redeploy

#### Scenario: Unknown id is not in the catalog

- **WHEN** the route receives an id with no matching catalog row
- **THEN** it resolves to nothing and the request is not-found, before any object
  access

## ADDED Requirements

### Requirement: Authenticated SoundFont catalog listing

The backend SHALL expose an **authenticated** endpoint that lists the available
SoundFonts from the persisted catalog — each with its id, display label, access
tier, licence, and attribution — so a client can discover exactly which fonts exist
on the server. An unauthenticated request MUST be rejected. The listing SHALL
reflect the catalog rows, so a font that is not in the catalog is never listed and a
newly added row appears without a client update.

#### Scenario: Authenticated client lists the catalog

- **WHEN** an authenticated client requests the SoundFont listing
- **THEN** the backend returns every catalog font with its id, label, tier, licence,
  and attribution

#### Scenario: Unauthenticated listing is rejected

- **WHEN** an unauthenticated caller requests the SoundFont listing
- **THEN** the request is rejected and no catalog is returned

#### Scenario: Listing reflects the catalog

- **WHEN** a font row is added to (or removed from) the catalog
- **THEN** the listing includes (or omits) it with no client-side change

### Requirement: Clients present only server-listed downloadable fonts

A client that offers **downloadable** SoundFonts SHALL source that list from the
authenticated catalog listing, and SHALL NOT present a downloadable font that is
absent from the listing. Fonts the client bundles locally, or that the user imports
from their own device, are independent of the listing and remain available. If the
listing cannot be obtained, the client SHALL present only its bundled and imported
fonts rather than any hardcoded or stale downloadable entry.

#### Scenario: Only listed fonts are offered for download

- **WHEN** the client builds its selectable-font catalog
- **THEN** the downloadable fonts it offers are exactly those returned by the
  listing, and none that the listing does not contain

#### Scenario: Listing unavailable degrades to local fonts

- **WHEN** the catalog listing cannot be fetched (offline, unauthenticated, or the
  backend is unavailable)
- **THEN** the client still offers its bundled default and any user-imported fonts,
  and offers no downloadable fonts

#### Scenario: Bundled default is not duplicated by the listing

- **WHEN** the listing contains the same font the client already bundles as its
  default
- **THEN** the client presents that font once (the bundled entry), not twice
