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

**Each store SHALL be answered on its own credentials.** A store whose credentials are absent SHALL be skipped, named as not submitted, and SHALL NOT prevent the other store from receiving the version; only a run that can reach no store at all SHALL fail. A store that refuses the submission SHALL NOT prevent the other from receiving it either, and the run SHALL fail.

A submission SHALL be able to name which stores it is for. Credentials answer whether a run *can* reach a store, never whether it *should*: a store that already holds the version would refuse a second submission of it, and that refusal is indistinguishable, in the report, from a store that never received it — while a submission that did not refuse would replace a package under review. A store left out SHALL be reported as deliberately omitted, distinctly from one whose credentials do not exist.

The run SHALL report each store separately, on **what the submission did** rather than on whether its credentials existed — accepted, refused, or never attempted. It SHALL NOT claim readers have the version, nor that a store received anything it refused or never saw.

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

#### Scenario: One store's credentials are not yet created
- **WHEN** publication is asked for and only one store's credentials exist
- **THEN** that store receives the version, the other is named as not submitted, and the run succeeds

#### Scenario: No store can be reached
- **WHEN** publication is asked for and none of the stores it names can be reached
- **THEN** the run stops and says why for each, and nothing is published

#### Scenario: The skipped store, later
- **WHEN** the same tag is submitted again once the missing credentials exist
- **THEN** the store that was skipped receives the same version, rebuilt from that tag

#### Scenario: Catching up one store while the other is still in review
- **WHEN** that later submission names only the store that was skipped
- **THEN** only that store is submitted to, and the store that already holds the version is reported as deliberately left out rather than as lacking it

#### Scenario: What success means
- **WHEN** a submission is accepted
- **THEN** the run reports that store as holding the version in review, not as having delivered it to readers

#### Scenario: A store holds every credential and still refuses
- **WHEN** a store rejects the submission for a reason of its own, such as an unfinished listing
- **THEN** the run reports that store as having refused the version rather than as having received it, still submits to the other store, and fails
