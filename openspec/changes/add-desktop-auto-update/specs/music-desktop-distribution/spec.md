## ADDED Requirements

### Requirement: Windows releases ship through the Microsoft Store

The Windows release SHALL be an MSIX package built from the release's own Windows
build and submitted to the Microsoft Store, which signs it and delivers updates.
No unsigned Windows executable SHALL be offered to the public.

#### Scenario: A tagged release is submitted

- **WHEN** a `music-v*` tag triggers the release build and the Store credentials
  are configured
- **THEN** the Windows job SHALL package the build as an MSIX whose version is the
  release's `major.minor.patch` followed by `.0`, and SHALL submit it to the
  Store under the configured product

#### Scenario: The package carries the release build

- **WHEN** the MSIX is produced
- **THEN** it SHALL wrap the Windows build made by the same job with the
  production configuration, rather than a separate build without it

#### Scenario: A dry run submits nothing

- **WHEN** the release workflow runs without a `music-v*` tag
- **THEN** it SHALL still produce the MSIX, so a packaging regression fails the
  job, and SHALL NOT submit anything to the Store

#### Scenario: Missing Store credentials on a tag

- **WHEN** a `music-v*` tag runs without the Store credentials
- **THEN** the Windows job SHALL fail rather than skip the Store submission
  silently

#### Scenario: No public unsigned download

- **WHEN** a release is published
- **THEN** no unsigned Windows executable or archive SHALL be attached to the
  GitHub Release

### Requirement: Linux releases ship an AppImage

The Linux release artifact SHALL include a self-contained AppImage that runs
without installation or root, so that the application can replace its own file
when updating. The tarball SHALL continue to be published alongside it.

#### Scenario: Release produces an AppImage

- **WHEN** a `music-v*` tag triggers the release build
- **THEN** the Linux job SHALL produce an executable AppImage and attach it to
  the GitHub Release, in addition to the tarball

#### Scenario: The AppImage is self-contained

- **WHEN** the AppImage is run on a supported distribution without the
  application's build dependencies installed
- **THEN** it SHALL launch, bundling the Flutter and native runtime libraries it
  needs

#### Scenario: Desktop metadata is embedded

- **WHEN** the AppImage is inspected or integrated by a desktop environment
- **THEN** it SHALL carry the application name, icon and desktop entry

### Requirement: Linux builds run on the oldest supported distribution

The Linux binaries SHALL be built against the oldest supported Ubuntu release,
so that they start on that release, on later ones, and on Debian releases of the
same generation.

#### Scenario: Older system

- **WHEN** a released Linux build is started on Ubuntu 22.04 or Debian 12
- **THEN** it SHALL start, rather than fail on a missing system library version

### Requirement: Every release publishes a signed update manifest

The release pipeline SHALL produce a manifest describing the release — version,
minimum supported version, release notes location, and for each target its
download URL, byte size and SHA-256 — and SHALL sign it with an Ed25519 key held
only as a continuous-integration secret.

#### Scenario: Manifest is produced and signed at release time

- **WHEN** a `music-v*` tag build completes its Linux job
- **THEN** the pipeline SHALL emit a manifest covering the Linux target, sign
  it, and publish the signed envelope

#### Scenario: Checksums match the published artifacts

- **WHEN** an artifact referenced by a published manifest is downloaded from its
  URL
- **THEN** its byte size and SHA-256 SHALL equal the values in the manifest

#### Scenario: The signing key never leaves continuous integration

- **WHEN** the repository and the backend configuration are inspected
- **THEN** neither SHALL contain the private signing key; the repository SHALL
  contain only the public key used for verification

#### Scenario: Key rotation is possible without a redesign

- **WHEN** a manifest is signed
- **THEN** it SHALL be published with a key identifier, so that verifiers can
  hold several trusted keys and a new key can be introduced before the old one
  is retired

#### Scenario: A dry-run build publishes nothing

- **WHEN** the release workflow is dispatched without a tag
- **THEN** it SHALL build and package the artifacts but SHALL NOT sign, upload or
  ingest a manifest

### Requirement: Signature verification is proven compatible across implementations

The repository SHALL contain a checked-in signed manifest fixture, and both the
Rust and the Dart verification implementations SHALL be tested against it — the
signature is produced in Rust and verified on both sides, so a divergence between
them must fail a test rather than an installation.

#### Scenario: Both implementations accept the valid fixture

- **WHEN** the Rust and the Dart verification paths are run against the
  checked-in signed manifest fixture and its public key
- **THEN** both SHALL accept it

#### Scenario: Both implementations reject a tampered fixture

- **WHEN** the same implementations are run against fixtures whose manifest
  bytes, signature or key identifier have been altered
- **THEN** both SHALL reject them
