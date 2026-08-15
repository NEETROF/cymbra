## ADDED Requirements

### Requirement: Signed Mac App Store package as the macOS release artifact

The macOS release artifact of Cymbra Music SHALL be a `.pkg` signed for Mac App
Store distribution — the application signed with an Apple Distribution identity
under a Mac App Store provisioning profile, and the installer package signed with a
Mac Installer Distribution identity. An unsigned or ad-hoc-signed `.app` SHALL NOT
be published as a release artifact.

#### Scenario: Release produces a store-signable package

- **WHEN** a `music-v*` tag triggers the release build
- **THEN** the macOS job SHALL archive and export with the App Store distribution
  method, producing a `.pkg` whose embedded `.app` is signed by Apple Distribution
  under the team that owns `com.cymbra.music`

#### Scenario: No unsigned artifact is published

- **WHEN** the release assets for a tag are listed
- **THEN** they SHALL NOT contain an unsigned or ad-hoc-signed macOS `.app` archive

#### Scenario: Embedded frameworks carry the same signature

- **WHEN** the exported package is inspected
- **THEN** every embedded framework and dynamic library — including the Flutter
  engine, the Rust engine library and all CocoaPods frameworks — SHALL be signed
  under the same team as the host application, with no unsigned nested code

#### Scenario: Missing signing material fails loudly

- **WHEN** the macOS release job runs without its distribution certificates or
  provisioning profile configured
- **THEN** the job SHALL fail with an explicit error naming the missing material,
  rather than silently falling back to an unsigned build

### Requirement: Store-validation metadata in the macOS bundle

The macOS `Info.plist` SHALL declare the metadata App Store Connect requires to
accept a delivery: an application category, and an export-compliance declaration
consistent with the app's actual use of encryption.

#### Scenario: Application category is declared

- **WHEN** the package is validated by App Store Connect
- **THEN** validation SHALL NOT fail for a missing `LSApplicationCategoryType`, and
  the declared category SHALL match the category chosen for the listing

#### Scenario: Export compliance is pre-declared

- **WHEN** a build is delivered to App Store Connect
- **THEN** the bundle SHALL declare that the app uses no non-exempt encryption
  (its only cryptography being TLS to the Cymbra backend and platform-provided
  storage encryption), so no per-build export-compliance question is raised

### Requirement: Entitlements sufficient for the sandboxed app's dependencies

The macOS application SHALL run under App Sandbox with exactly the entitlements its
shipped functionality requires, including the keychain access group that the
credential-storage dependency needs in order to read and write items under the
sandbox.

#### Scenario: Credentials survive under the sandbox

- **WHEN** a signed, sandboxed build signs a user in and is then relaunched
- **THEN** the stored session credential SHALL be readable and the user SHALL
  remain signed in, rather than the keychain access failing under the sandbox

#### Scenario: No keychain prompt on launch

- **WHEN** a store-signed build reads its stored credentials
- **THEN** the read SHALL be authorised by the app's own keychain access group and
  SHALL NOT ask the user for their login keychain password — credentials therefore
  SHALL be stored in the app-scoped keychain, not in a keychain whose items are
  guarded by an ACL bound to the binary's designated requirement (which changes
  with the signing identity, so every store build would prompt)

#### Scenario: Debug and release entitlements stay consistent

- **WHEN** the debug/profile and release entitlement sets are compared
- **THEN** every capability the shipped app depends on SHALL be present in both, so
  a behaviour cannot work in development and fail only in the released build

#### Scenario: No unnecessary entitlements are requested

- **WHEN** the release entitlements are reviewed
- **THEN** they SHALL NOT grant capabilities the app does not exercise — in
  particular the app SHALL NOT request an incoming-network-connection entitlement,
  since macOS sign-in uses the platform authentication session rather than the
  desktop loopback flow

### Requirement: Automated delivery to App Store Connect

A tagged release SHALL deliver the signed macOS package to App Store Connect
automatically, using the same App Store Connect API credentials as the iOS
delivery. A manual run SHALL NOT deliver unless delivery is explicitly requested,
and SHALL make its signed package retrievable either way.

#### Scenario: Tagged release delivers

- **WHEN** a `music-v*` tag push completes the macOS build
- **THEN** the signed `.pkg` SHALL be uploaded to App Store Connect for the macOS
  platform of the `com.cymbra.music` record

#### Scenario: Manual dispatch validates without delivering

- **WHEN** the release workflow is dispatched manually rather than by a tag push,
  without explicitly requesting delivery
- **THEN** the macOS job SHALL build and sign to prove the chain works, but SHALL
  NOT upload, so an already-delivered build number is never re-sent

#### Scenario: Manual dispatch can deliver on request

- **WHEN** the workflow is dispatched with delivery explicitly requested
- **THEN** the signed package SHALL be uploaded, so a build can reach TestFlight
  without cutting a release tag — and requesting delivery SHALL affect the macOS
  package only, never another platform's store upload

#### Scenario: A dry-run package is retrievable

- **WHEN** the workflow is dispatched without a tag
- **THEN** the signed package SHALL be retrievable as a private, short-lived build
  artifact, so it can be delivered by hand — and it SHALL carry the production
  configuration, since the build-time endpoint cannot be verified by inspecting
  the package afterwards

#### Scenario: Absent delivery credentials skip cleanly

- **WHEN** the App Store Connect API credentials are not configured
- **THEN** the upload step SHALL be skipped with a warning and the job SHALL still
  succeed, leaving the signed package available

### Requirement: Sandboxed runtime conformance of core features

Every user-facing capability of the macOS app SHALL be verified to work on a
signed, sandboxed build before submission, because the previously published
unsigned artifact could not exercise sandbox restrictions.

#### Scenario: MIDI input works under the sandbox

- **WHEN** a MIDI keyboard is connected to a machine running the signed sandboxed
  build, including while the app is already running
- **THEN** the device SHALL be detected and its note events SHALL drive playback and
  scoring, with hot-plug still observed

#### Scenario: Audio output works under the sandbox

- **WHEN** a score is played on the signed sandboxed build
- **THEN** the synthesized audio SHALL be audible through the system output device

#### Scenario: SoundFont import works under the sandbox

- **WHEN** the user picks a `.sf2` file through the system file picker
- **THEN** the file SHALL be readable, importable into the app's own storage, and
  selectable as the playback instrument

#### Scenario: Sign-in works under the sandbox

- **WHEN** the user signs in with Google or with Apple on the signed sandboxed build
- **THEN** the authentication session SHALL complete and the account SHALL be usable

#### Scenario: Local storage works under the sandbox

- **WHEN** the app writes its local database, preferences and encrypted offline
  score cache
- **THEN** all of them SHALL persist inside the app's sandbox container and be
  readable after relaunch

### Requirement: macOS listing assets

The repository SHALL contain the version-controlled assets and metadata required to
create the Mac App Store listing, alongside the existing mobile listing material.

#### Scenario: macOS screenshots are available

- **WHEN** preparing the Mac App Store listing
- **THEN** at least one screenshot of the app's real UI SHALL be available at a
  display size the Mac App Store accepts

#### Scenario: macOS category and copy are recorded

- **WHEN** submitting the macOS listing
- **THEN** the chosen macOS category SHALL be recorded in the repository, and the
  existing localized description and promotional copy SHALL be reused rather than
  duplicated per platform

### Requirement: Documented macOS signing setup

The repository SHALL document the macOS signing and distribution setup so a
maintainer can reproduce a store build without reverse-engineering the workflow.

#### Scenario: Setup is reproducible from the docs

- **WHEN** a maintainer prepares macOS distribution for the first time
- **THEN** the documentation SHALL name the required Apple-side prerequisites (App
  ID platform, the two distribution certificates, the Mac App Store provisioning
  profile, the App Store Connect platform record), the repository secrets the
  workflow reads, and the command to produce a store package locally
