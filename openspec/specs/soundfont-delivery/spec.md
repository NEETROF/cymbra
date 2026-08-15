# soundfont-delivery Specification

## Purpose
Deliver playback SoundFonts to authenticated clients from a dedicated, private
object-store bucket via an authenticated, range-capable backend route, gated by a
per-font entitlement check and resolved through a server-owned font catalog so that
free fonts ship today and paid fonts can be added later without re-architecting delivery.
## Requirements
### Requirement: SoundFonts stored in a dedicated private object-store bucket

The system SHALL store playback SoundFonts in a **dedicated, private** object-store
bucket, separate from the catalog scores bucket, with its own credentials scope, ACL, and
local warm cache. SoundFont objects MUST NOT be publicly readable, and the SoundFont store
MUST be independent of the scores store so that neither bucket's access policy or cache is
affected by the other. Access to the bucket SHALL require credentials — there is no public
URL for a SoundFont object.

#### Scenario: SoundFonts are isolated from scores

- **WHEN** the backend is configured with a SoundFont bucket and a scores bucket
- **THEN** they are distinct buckets with independent access policies and warm caches, and
  writing or reading a SoundFont never touches the scores store

#### Scenario: SoundFont objects are not public

- **WHEN** a client requests a SoundFont object directly from the object store without
  backend credentials
- **THEN** the object store denies it — the bytes are reachable only through the backend

#### Scenario: Feature is gated by configuration

- **WHEN** the SoundFont bucket is not configured
- **THEN** the SoundFont delivery route is disabled and reports that it is unavailable,
  without affecting any other backend surface

### Requirement: Authenticated SoundFont delivery route

The backend SHALL expose an authenticated HTTP route that streams a SoundFont's bytes to a
client, addressed by a stable **SoundFont id**. The route SHALL require a valid
authenticated session (the same identity used by the other authenticated surfaces); an
unauthenticated request MUST be rejected. The route SHALL stream the object body and
support **range requests** so large fonts (tens to hundreds of MB) are delivered
efficiently without buffering the whole file, reading through the local-first warm cache
with the private bucket as origin. A request for an unknown SoundFont id MUST return
not-found. The bytes SHALL be served from the backend/API origin (never a direct
browser-to-bucket fetch).

The route SHALL additionally enforce the moderation gate for public catalog fonts: to a
normal (non-moderator, non-admin) caller it SHALL serve only an `accepted` font — a
`pending` or `rejected` catalog font MUST be treated as not-found — while a music-scope
moderator/admin MAY fetch a catalog font of any moderation status (to audition it). A
font in a user's **private library** SHALL be served only to its owner and is not subject
to moderation.

#### Scenario: Authenticated client fetches a font

- **WHEN** an authenticated client requests an existing `accepted` SoundFont by id
- **THEN** the backend streams the SoundFont bytes from the SoundFont store (warm cache,
  else the private bucket) with a success status

#### Scenario: Range request is honoured

- **WHEN** a client requests a byte range of a SoundFont
- **THEN** the backend serves only that range, so the font can be fetched/resumed in parts
  rather than buffered whole

#### Scenario: Unauthenticated request is rejected

- **WHEN** an unauthenticated caller requests a SoundFont
- **THEN** the request is rejected (unauthorized) and no bytes are served

#### Scenario: Unknown font id

- **WHEN** an authenticated client requests a SoundFont id that does not exist
- **THEN** the backend responds not-found and streams nothing

#### Scenario: Unvalidated catalog font hidden from a normal caller

- **WHEN** a normal caller requests a `pending` or `rejected` catalog font by id
- **THEN** the backend responds not-found and streams nothing

#### Scenario: Moderator auditions an unvalidated catalog font

- **WHEN** a music-scope moderator/admin requests a `pending` or `rejected` catalog font
- **THEN** the backend streams its bytes so the reviewer can audition it

#### Scenario: Private font served only to its owner

- **WHEN** a user requests a private-library font they own
- **THEN** the backend streams it, whereas any other caller receives not-found

### Requirement: Per-font entitlement check, ready for paid fonts

Every SoundFont delivery request SHALL pass through a **per-font entitlement check** that
decides whether the requesting identity may receive that specific font. A SoundFont SHALL
carry an access tier: **free** fonts are granted to any authenticated identity, while a
**paid** font is granted only to an identity entitled to it. The check SHALL be a distinct,
testable decision point (not hardcoded inline), so that introducing paid fonts later is a
matter of adding font entries and an entitlement source — not re-architecting delivery. A
request for a font the identity is not entitled to MUST be refused (forbidden) and no bytes
served.

#### Scenario: Free font is granted to any signed-in user

- **WHEN** an authenticated identity requests a font whose tier is free
- **THEN** the entitlement check passes and the font is delivered

#### Scenario: Paid font requires entitlement

- **WHEN** an authenticated identity requests a font whose tier is paid and the identity is
  not entitled to it
- **THEN** the entitlement check fails, the request is refused (forbidden), and no bytes are
  served

#### Scenario: Entitlement is enforced before any bytes are read

- **WHEN** a delivery request is refused by the entitlement check
- **THEN** the backend does not read or stream the object, and the refusal is independent of
  whether the object exists

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

### Requirement: Authenticated SoundFont upload route

The private SoundFont store SHALL have an authenticated **upload** counterpart to its
download route, so a font's bytes can be placed into the store through the backend
(never a direct browser-to-bucket write). The upload SHALL require a music-scope
moderator/admin identity; any lesser or unauthenticated caller MUST be refused before
any store write. The route SHALL accept the font body (with its metadata) up to a
**configured maximum size**, rejecting an over-large body, and SHALL validate that the
body is a real SoundFont before storing it, rejecting an invalid body without writing.
The bytes SHALL be written through the backend/API origin into the same private store
the delivery route reads.

#### Scenario: Admin uploads a font

- **WHEN** a music-scope admin uploads a valid `.sf2` through the upload route
- **THEN** the backend streams it into the private SoundFont store and it becomes
  fetchable by the delivery route

#### Scenario: Unauthorized upload is refused

- **WHEN** an unauthenticated caller, or one lacking music-scope moderator/admin,
  attempts to upload a font
- **THEN** the request is refused before any store write and nothing is stored

#### Scenario: Invalid upload body is rejected

- **WHEN** an authorized admin uploads a body that is not a valid SoundFont
- **THEN** the route rejects it and stores nothing

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

