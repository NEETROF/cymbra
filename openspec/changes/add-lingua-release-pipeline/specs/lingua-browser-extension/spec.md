## ADDED Requirements

### Requirement: Versioned extension releases

The extension SHALL carry one version, derived from the repository's Conventional Commits, held in exactly one file and stamped onto every built variant's manifest at build time, so that each published build names a commit range and a changelog entry.

The browser manifest SHALL NOT hold a copy of it: a second place to write the same value drifts, and the copy would win over the stamp. The version SHALL be refused unless it is a shape browsers accept for an extension manifest — dot-separated integers, with no pre-release suffix. A release SHALL be produced from a tag, not from a branch.

#### Scenario: One source reaches every variant
- **WHEN** a release raises the extension's version
- **THEN** one file changes, and each built variant's manifest reports that version

#### Scenario: A copy put back into the browser manifest
- **WHEN** the browser manifest is given a version of its own
- **THEN** the gate fails and says the stamp is the only source

#### Scenario: A version a browser would refuse
- **WHEN** the version is not dot-separated integers (for instance a release candidate suffix)
- **THEN** the release stops before building anything, rather than producing a package no store will take

### Requirement: Releasing and publishing are separate acts

Pushing an extension release tag SHALL produce the production build of the variants that are distributed on their own — the real language pack and the production endpoint, never a test pack or a local endpoint — and attach them to the release.

It SHALL NOT submit them to any store. Naming a version is not putting it in front of readers, and a release SHALL NOT fail for want of credentials that live outside the repository.

Submitting SHALL be a separate act that names the tag it deploys, and SHALL refuse to run without one. It SHALL submit the Chromium package to the Chrome Web Store and the Firefox package to addons.mozilla.org, the latter carrying the human-readable source of the generated code, which that store requires. The safari variant SHALL NOT be submitted at all: it is distributed inside the Apple host application, built from the same commit.

A submission SHALL stop with a message naming what is missing when a store's credentials are absent, rather than publishing part of a release. Success SHALL mean the store accepted the version for review; the run SHALL say so, and SHALL NOT claim readers have it.

#### Scenario: A tag builds and attaches, and stops there
- **WHEN** an extension release tag is pushed
- **THEN** both distributable packages are built from the production configuration and attached to the release, nothing is submitted, and the run succeeds even though no store credentials exist

#### Scenario: Submitting a tagged version
- **WHEN** publication is asked for, naming a release tag
- **THEN** that tag is built again and both packages are submitted to their stores

#### Scenario: Asked to publish without naming a version
- **WHEN** publication is asked for with no tag
- **THEN** the run stops and says so, rather than succeeding having published nothing

#### Scenario: A build that would ship the wrong pack or endpoint
- **WHEN** the build would bundle the test pack or a non-production endpoint
- **THEN** the run fails instead of producing a package

#### Scenario: The Firefox store asks for the source
- **WHEN** the Firefox package is submitted
- **THEN** an archive of the human-readable source, with the instructions to rebuild it, is submitted with it

#### Scenario: Credentials not yet created
- **WHEN** a store's credentials are absent and publication is asked for
- **THEN** the run stops and names them, and nothing is published

#### Scenario: What success means
- **WHEN** both submissions are accepted
- **THEN** the run reports the version as being in review at each store, not as delivered to readers
