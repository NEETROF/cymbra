# Spec Delta

## ADDED Requirements

### Requirement: Public latest-release view for download pages

The backend SHALL expose an unauthenticated, read-only HTTP endpoint that returns an informational description of the latest downloadable desktop release for a product and channel: its version, release date, optional notes link, and, per `<os>-<arch>` target, the install kind, download URL, byte size and SHA-256. The latest release is the highest ingested version that is **not paused**; its staged-rollout percentage SHALL NOT affect this endpoint. The view SHALL be derived from the stored signed envelope after its signature has been re-verified against a trusted key, and SHALL NOT depend on the caller. The automatic updater SHALL NOT use this endpoint.

#### Scenario: A release is ingested but rolled out at zero

- **WHEN** the newest ingested release is not paused and has a rollout percentage of 0
- **THEN** the endpoint SHALL return `200` with that release's view

#### Scenario: The newest release is paused

- **WHEN** the newest ingested release is paused and an older one is not
- **THEN** the endpoint SHALL describe the highest non-paused release, not the paused one

#### Scenario: Nothing to show

- **WHEN** the product and channel have no ingested release, every release is paused, or the product or channel is unknown
- **THEN** the endpoint SHALL return `204 No Content` with no body

#### Scenario: A stored row no longer verifies

- **WHEN** the stored envelope of the selected release fails signature verification against every trusted key
- **THEN** the endpoint SHALL return `204 No Content` and SHALL NOT serve any field of that row

#### Scenario: Identical for every caller

- **WHEN** two different callers request the same product and channel at the same time
- **THEN** they SHALL receive byte-identical responses, with a `Cache-Control` header allowing shared caching, and the request SHALL carry no account, install identifier or client version

#### Scenario: Read from the site in a browser

- **WHEN** a page served from a configured web origin requests the endpoint
- **THEN** the response SHALL permit that origin to read it, and SHALL NOT permit any other origin
