# lingua-apple-app Specification

## Purpose
TBD - created by archiving change add-lingua-release-pipeline. Update Purpose after archive.
## Requirements
### Requirement: Versioned App Store releases

The host application's shipped version SHALL come from the release tooling — one file the tooling maintains, stamped onto the build at archive time — and SHALL NOT be a value written by hand in the Xcode project, which CI already rewrites for distribution signing.

Pushing an application release tag SHALL build that tag, stamp its version, and deliver both platforms to App Store Connect. A tag whose version disagrees with the maintained file SHALL stop the run. Building from a branch SHALL remain possible and SHALL keep its explicit opt-in before anything is delivered, so a dispatch cannot deliver branch code by surprise.

Its version SHALL be independent of the extension's: the two are released separately and are not read as a pair.

#### Scenario: A tag ships a versioned build
- **WHEN** an application release tag is pushed
- **THEN** that tag is built, both packages carry its version, and both are delivered to App Store Connect

#### Scenario: A tag that disagrees with the maintained version
- **WHEN** the tag's version and the maintained file differ
- **THEN** the run stops before building, rather than shipping a version the changelog does not describe

#### Scenario: A dispatch from a branch
- **WHEN** the lane is dispatched without a tag
- **THEN** it builds and signs from that branch and delivers nothing unless delivery was asked for explicitly

