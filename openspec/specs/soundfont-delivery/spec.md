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

The system SHALL maintain a server-owned mapping from each **SoundFont id** to its storage
key, its access tier (free/paid), its licence/attribution, its **moderation status** with
reviewer attribution (`reviewed_by`/`reviewed_at`), its **uploader** (`uploaded_by`), and
its exact-byte **content digest** (SHA-256). The delivery route, the moderation gate, and
the entitlement check SHALL resolve fonts through this mapping, so client-facing ids are
decoupled from bucket keys, the required attribution for redistributed (e.g. CC-BY) fonts
is recorded alongside the font, and identical content is detectable across uploads.

#### Scenario: Id resolves to a stored object

- **WHEN** the route receives a known SoundFont id
- **THEN** it resolves the id to the object's storage key and tier via the catalog before
  fetching

#### Scenario: Attribution is recorded for redistributed fonts

- **WHEN** a redistributed font requiring attribution (e.g. CC-BY) is in the catalog
- **THEN** its licence and required credit are recorded in the catalog entry

#### Scenario: Moderation and content identity are recorded

- **WHEN** a font is present in the catalog
- **THEN** its moderation status, reviewer attribution, uploader, and content digest are
  recorded alongside its storage key, tier, and licence

