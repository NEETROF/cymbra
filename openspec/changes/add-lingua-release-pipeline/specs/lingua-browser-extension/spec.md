## ADDED Requirements

### Requirement: Versioned extension releases

The extension SHALL carry one version, derived from the repository's Conventional Commits and written by the release tooling into both the package manifest and the browser manifest, so that every published build names a commit range and a changelog entry.

The two files SHALL be checked to agree, and the version SHALL be refused unless it is a shape browsers accept for an extension manifest — dot-separated integers, with no pre-release suffix. A release SHALL be produced from a tag, not from a branch.

#### Scenario: The release tooling bumps both files
- **WHEN** a release raises the extension's version
- **THEN** the package manifest and the browser manifest carry the same version, and each built variant reports it

#### Scenario: The two files disagree
- **WHEN** a change leaves the package manifest and the browser manifest on different versions
- **THEN** the gate fails and names both versions

#### Scenario: A version a browser would refuse
- **WHEN** the version is not dot-separated integers (for instance a release candidate suffix)
- **THEN** the release stops before building anything, rather than producing a package no store will take

### Requirement: Store publication from a tag

Pushing an extension release tag SHALL produce the production build of the variants that are distributed on their own — the real language pack and the production endpoint, never a test pack or a local endpoint — attach them to the release, and submit each one to its store: the Chromium package to the Chrome Web Store, the Firefox package to addons.mozilla.org.

The Firefox submission SHALL carry the human-readable source of the generated code, which that store requires. The safari variant SHALL NOT be submitted separately: it is distributed inside the Apple host application, built from the same commit.

A run SHALL stop with a message naming what is missing when the credentials for a store are absent, rather than publishing part of a release. Success SHALL mean the store accepted the version for review; the run SHALL say so, and SHALL NOT claim readers have it.

#### Scenario: A tag publishes both packages
- **WHEN** an extension release tag is pushed
- **THEN** the Chromium and Firefox packages are built from the production configuration, attached to the release, and submitted to their stores

#### Scenario: A build that would ship the wrong pack or endpoint
- **WHEN** the release build would bundle the test pack or a non-production endpoint
- **THEN** the run fails instead of publishing

#### Scenario: The Firefox store asks for the source
- **WHEN** the Firefox package is submitted
- **THEN** an archive of the human-readable source, with the instructions to rebuild it, is submitted with it

#### Scenario: Credentials not yet created
- **WHEN** a store's credentials are absent
- **THEN** the run stops and names them, and nothing is published

#### Scenario: What success means
- **WHEN** both submissions are accepted
- **THEN** the run reports the version as being in review at each store, not as delivered to readers
