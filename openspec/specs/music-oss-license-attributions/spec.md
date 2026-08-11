# music-oss-license-attributions Specification

## Purpose
TBD - created by archiving change add-oss-license-attributions. Update Purpose after archive.
## Requirements
### Requirement: Open source licenses entry near legal links

The app SHALL expose an "Open Source Licenses" entry in the same legal/account
area as the Terms of Service and Privacy Policy entries (see `legal-links`).
Tapping it SHALL open an in-app license disclosure page; it SHALL NOT open an
external browser.

#### Scenario: Entry is visible next to legal links
- **WHEN** a user opens the account/legal menu
- **THEN** an "Open Source Licenses" entry is shown alongside the Terms of
  Service and Privacy Policy entries

#### Scenario: Tapping the entry opens the in-app license page
- **WHEN** the user taps the "Open Source Licenses" entry
- **THEN** the app navigates to an in-app page listing third-party licenses,
  without launching an external browser

### Requirement: Dart dependency licenses are disclosed automatically

The license page SHALL list every Dart/Flutter pub package's bundled license
by reading Flutter's `LicenseRegistry`, requiring no manual per-package
maintenance when dependencies change.

#### Scenario: A newly added pub dependency appears without code changes
- **WHEN** a new third-party pub package with a bundled `LICENSE` file is
  added to `apps/music/pubspec.yaml` and `pub get` runs
- **THEN** that package's license appears on the license page without any
  change to the license-page code

### Requirement: Statically-linked Rust crate licenses are disclosed

The license page SHALL also list the license of every third-party Rust crate
statically linked into the shipped `apps/music/rust` binary (including
transitive dependencies), sourced from a build-time-generated notices file
derived from `Cargo.lock`. Crates that are not part of that dependency tree
(e.g. backend-only or dev-only crates) SHALL NOT appear.

#### Scenario: A shipped Rust dependency's license is listed
- **WHEN** the license page is opened
- **THEN** it includes an entry for each third-party crate reachable from
  `apps/music/rust`'s dependency tree (e.g. `midir`, `rustysynth`, `cpal`),
  with that crate's license name and full text

#### Scenario: Backend-only crates are excluded
- **WHEN** the license page is opened
- **THEN** crates used only by `backend/*` (not linked into the app binary)
  do not appear

### Requirement: First-party code is excluded

The license page SHALL exclude Cymbra's own crates and packages (e.g.
`cymbra-musicxml-core`, the app's local FFI crate) from the disclosed list.

#### Scenario: First-party crate does not self-list
- **WHEN** the license page is opened
- **THEN** no entry corresponding to a Cymbra-owned crate or local path
  package is shown

### Requirement: Rust notices asset is generated, not hand-maintained

The Rust third-party notices consumed by the license page SHALL be produced
by an automated build-time step from `Cargo.lock`, not authored or edited by
hand, so the disclosed list cannot drift from what is actually shipped.

#### Scenario: Notices regenerate after a dependency bump
- **WHEN** a third-party Rust dependency version changes in `Cargo.lock`
- **THEN** re-running the generation step produces an updated notices asset
  reflecting the new version's license, without manual editing

#### Scenario: Missing generated asset degrades gracefully
- **WHEN** the generated Rust notices asset is absent at app startup (e.g. a
  dev build that skipped the generation step)
- **THEN** the license page still shows the Dart/Flutter license list rather
  than failing to open

