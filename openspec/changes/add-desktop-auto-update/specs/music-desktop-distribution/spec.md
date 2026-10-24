## ADDED Requirements

### Requirement: Windows releases ship a per-user installer

The Windows release artifact SHALL include an installer that installs for the
current user only, into the user's local application data directory, so that
subsequent updates can be applied without administrator elevation. The portable
archive SHALL continue to be published alongside it.

#### Scenario: Release produces an installer

- **WHEN** a `music-v*` tag triggers the release build
- **THEN** the Windows job SHALL produce an installer executable and attach it
  to the GitHub Release, in addition to the portable archive

#### Scenario: Installation requires no elevation

- **WHEN** the installer is run by a standard user
- **THEN** it SHALL install without requesting administrator elevation, into a
  per-user location

#### Scenario: The application identity is stable across versions

- **WHEN** an installer for a later version runs over an existing installation
- **THEN** it SHALL upgrade that installation in place rather than creating a
  second parallel one

#### Scenario: The install method is discoverable at runtime

- **WHEN** the installer completes
- **THEN** it SHALL leave a marker in the installation directory identifying the
  installation as installer-managed, so the running application can tell it
  apart from a portable extraction

#### Scenario: Silent installation relaunches the application

- **WHEN** the installer is run in silent mode
- **THEN** it SHALL close a running instance, replace the files, and relaunch the
  application without user interaction

### Requirement: Linux releases ship a Debian package

The Linux release artifact SHALL include a `.deb` package for Debian and Ubuntu
derivatives that installs the application system-wide with a desktop entry and
icon, and that carries the privileged update helper and its system
authorization policy. The tarball SHALL continue to be published alongside it.

#### Scenario: Release produces a package

- **WHEN** a `music-v*` tag triggers the release build
- **THEN** the Linux job SHALL produce a `.deb` package and attach it to the
  GitHub Release, in addition to the tarball

#### Scenario: Installation through the package manager

- **WHEN** the package is installed with the system package manager on a
  supported distribution
- **THEN** the package manager SHALL install its declared runtime dependencies,
  and the application SHALL appear in the desktop's application list with its
  name and icon

#### Scenario: The install method is discoverable at runtime

- **WHEN** the package is installed
- **THEN** it SHALL leave a marker beside the application identifying the
  installation as package-managed, so the running application can tell it apart
  from a tarball extraction

#### Scenario: Upgrade in place

- **WHEN** a package for a later version is installed over an existing one
- **THEN** it SHALL upgrade that installation rather than install a second copy

### Requirement: Linux builds run on the oldest supported distribution

The Linux binaries SHALL be built against the oldest supported Ubuntu release,
so that they start on that release, on later ones, and on Debian releases of the
same generation.

#### Scenario: Older system

- **WHEN** a released Linux build is started on Ubuntu 22.04 or Debian 12
- **THEN** it SHALL start, rather than fail on a missing system library version

### Requirement: Windows binaries are code-signed

The Windows application executable, its bundled libraries and the installer SHALL
carry an Authenticode signature in the name of the publishing company, applied
before the installer is hashed into the release manifest.

#### Scenario: Signed release

- **WHEN** a `music-v*` tag builds the Windows artifacts
- **THEN** the installer and the application executable SHALL carry a valid
  signature naming the publisher, and the installer's size and SHA-256 in the
  manifest SHALL be those of the signed file

#### Scenario: A dry run without signing material

- **WHEN** the release workflow runs without the signing secrets
- **THEN** it SHALL build unsigned artifacts, state that they are unsigned, and
  SHALL NOT publish them

### Requirement: Every release publishes a signed update manifest

The release pipeline SHALL produce a manifest describing the release — version,
minimum supported version, release notes location, and for each target its
download URL, byte size and SHA-256 — and SHALL sign it with an Ed25519 key held
only as a continuous-integration secret.

#### Scenario: Manifest is produced and signed at release time

- **WHEN** a `music-v*` tag build completes its Windows and Linux jobs
- **THEN** the pipeline SHALL emit a manifest covering both targets, sign it, and
  publish the signed envelope

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
