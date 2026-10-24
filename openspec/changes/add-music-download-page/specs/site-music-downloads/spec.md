# Spec Delta

## Purpose

Lets a visitor of cymbra.app, and a subscriber who has just paid on the web, download Cymbra Music for Windows and Linux without a store, with the steps each platform's installer needs.

## ADDED Requirements

### Requirement: The Music page offers the latest Windows and Linux builds

The site's Music page, in French, English and Spanish, SHALL present a download card for Windows and one for Linux, each showing the latest version, its release date, the file size, the file's SHA-256 and a download link. The cards SHALL be filled from the backend's public latest-release view when the page loads in the browser, so that a new release appears without redeploying the site. The visitor's own platform, when it can be told, SHALL be shown first or highlighted; both platforms SHALL remain visible.

#### Scenario: A release is available

- **WHEN** a visitor opens the Music page and the backend describes a latest release with a Windows and a Linux target
- **THEN** both cards show that version, date, size, SHA-256 and a download link, and the visitor's platform is highlighted

#### Scenario: A new release is ingested

- **WHEN** a release is ingested after the site was last deployed
- **THEN** the next page load shows the new version without any site deployment

#### Scenario: A platform has no target

- **WHEN** the latest release lists a target for one platform only
- **THEN** the other platform's card says its build is not available yet, and offers no download link

### Requirement: Downloads degrade to the releases page

The Music page SHALL always carry a link to the project's GitHub Releases page. When a latest release is shown, each card SHALL also link to that version's own release page, which is where its portable archives (Windows zip, Linux tarball) live. When the latest-release view is unreachable, answers "no content", is malformed, or the page's script does not run, the section SHALL still render that link and SHALL NOT show an error message, a raw status code or an empty card.

#### Scenario: The backend is down

- **WHEN** the latest-release request fails or times out
- **THEN** the section shows the releases-page link and a neutral "downloads are available on the releases page" message

#### Scenario: No release is servable

- **WHEN** the endpoint answers "no content" (nothing ingested, or the newest release is paused)
- **THEN** the section shows the same fallback, not a stale or empty card

#### Scenario: Scripts disabled

- **WHEN** the page is read without running its scripts
- **THEN** the releases-page link is present in the page's content

### Requirement: Download links point only to the project's release assets

A download link SHALL be shown only when its address is an HTTPS asset of the project's GitHub Releases. An address from the backend that is anything else SHALL NOT be rendered as a link.

#### Scenario: Unexpected address

- **WHEN** the backend's view carries a target whose address is not an HTTPS asset of the project's releases
- **THEN** that card shows no download link and the releases-page fallback is offered instead

### Requirement: Install guidance per platform

Each platform card SHALL state, in the page's language, what installing that build involves. For Windows: that the installer is published by NEETROF, that Windows may show a SmartScreen notice for a recently published app, and that it may be run only when that notice names NEETROF as the publisher. For Linux: that the package installs through the distribution's software installer or `apt`, and that distributions not derived from Debian or Ubuntu use the tarball, which does not update itself. No card SHALL suggest disabling a system security feature. The guidance SHALL be visible without any interaction beyond opening the card.

#### Scenario: Windows card

- **WHEN** a visitor reads the Windows card
- **THEN** it names NEETROF as the publisher, explains the SmartScreen notice a young app may get, and says to proceed only when that publisher is shown

#### Scenario: Linux card

- **WHEN** a visitor reads the Linux card
- **THEN** it explains installing the package with the software installer or `sudo apt install`, and links the tarball on that version's release page for other distributions

#### Scenario: Never a security bypass

- **WHEN** any card's guidance is read in any locale
- **THEN** it SHALL NOT instruct the visitor to turn off Smart App Control, SmartScreen or any other protection

### Requirement: Desktop downloads on the web-purchase landing

The page shown after a web purchase and the account page SHALL offer the Windows and Linux downloads described above, and no "coming soon" placeholder SHALL remain for them anywhere on the site. The store buttons for iOS, iPadOS, macOS and Android SHALL be unchanged.

#### Scenario: After a web purchase

- **WHEN** a subscriber lands on the post-checkout page
- **THEN** the Windows and Linux downloads are offered alongside the store links, with no "coming soon" label

#### Scenario: Release notes

- **WHEN** the latest release carries a notes link
- **THEN** the cards show a "release notes" link to it, and show nothing in its place when it does not
