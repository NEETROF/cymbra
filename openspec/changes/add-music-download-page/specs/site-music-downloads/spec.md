# Spec Delta

## Purpose

Lets a visitor of cymbra.app, and a subscriber who has just paid on the web, get Cymbra Music for Windows (from the Microsoft Store) and for Linux (the AppImage), with the steps each platform needs.

## ADDED Requirements

### Requirement: The Music page offers Windows through the Microsoft Store

The site's Music page, in French, English and Spanish, SHALL present a Windows card whose action installs Cymbra Music from the Microsoft Store through the Store's web installer, so that the visitor does not have to browse the Store. The card SHALL NOT offer any other Windows download.

#### Scenario: Windows card

- **WHEN** a visitor opens the Music page
- **THEN** the Windows card offers the Microsoft Store install, and no executable or archive hosted elsewhere

#### Scenario: Store not configured yet

- **WHEN** the site is built without the Store product identifier
- **THEN** the Windows card says the Windows version is coming soon and offers no link

### Requirement: The Music page offers the latest Linux AppImage

The Music page SHALL present a Linux card showing the latest version, its release date, the AppImage's size, its SHA-256 and a download link. The card SHALL be filled from the backend's public latest-release view when the page loads in the browser, so that a new release appears without redeploying the site. When the visitor's platform can be told, it SHALL be shown first or highlighted; both cards SHALL remain visible.

#### Scenario: A release is available

- **WHEN** a visitor opens the Music page and the backend describes a latest release with a Linux target
- **THEN** the Linux card shows that version, date, size, SHA-256 and a download link

#### Scenario: A new release is ingested

- **WHEN** a release is ingested after the site was last deployed
- **THEN** the next page load shows the new version without any site deployment

#### Scenario: No Linux target

- **WHEN** the latest release carries no Linux target
- **THEN** the Linux card says its build is not available yet, and offers no download link

### Requirement: Linux downloads degrade to the releases page

The Music page SHALL always carry a link to the project's GitHub Releases page. When a latest release is shown, the Linux card SHALL also link to that version's own release page, where its tarball lives. When the latest-release view is unreachable, answers "no content", is malformed, or the page's script does not run, the section SHALL still render that link and SHALL NOT show an error message, a raw status code or an empty card.

#### Scenario: The backend is down

- **WHEN** the latest-release request fails or times out
- **THEN** the Linux card shows the releases-page link and a neutral "downloads are available on the releases page" message

#### Scenario: No release is servable

- **WHEN** the endpoint answers "no content" (nothing ingested, or the newest release is paused)
- **THEN** the Linux card shows the same fallback, not a stale or empty card

#### Scenario: Scripts disabled

- **WHEN** the page is read without running its scripts
- **THEN** the releases-page link and the Windows Store action are present in the page's content

### Requirement: Download links point only to the project's release assets

A Linux download link SHALL be shown only when its address is an HTTPS asset of the project's GitHub Releases. An address from the backend that is anything else SHALL NOT be rendered as a link.

#### Scenario: Unexpected address

- **WHEN** the backend's view carries a Linux target whose address is not an HTTPS asset of the project's releases
- **THEN** the card shows no download link and the releases-page fallback is offered instead

### Requirement: Install guidance per platform

Each card SHALL state, in the page's language, what installing that build involves. For Windows: that it installs and updates through the Microsoft Store. For Linux: that the AppImage is one file for every distribution, that it must be allowed to run as a program (file properties, or `chmod +x`), that it updates itself from the app when kept in a folder the user can write to (for example `~/Applications`), that it needs FUSE support, which current desktops include, and that an integration tool adds it to the application menu. No card SHALL suggest disabling a system security feature. The guidance SHALL be visible without any interaction beyond opening the card.

#### Scenario: Windows card

- **WHEN** a visitor reads the Windows card
- **THEN** it says the app installs and updates through the Microsoft Store

#### Scenario: Linux card

- **WHEN** a visitor reads the Linux card
- **THEN** it explains making the AppImage executable, keeping it in a writable folder for self-updates, the FUSE requirement and menu integration, and links the tarball on that version's release page as an alternative

#### Scenario: Never a security bypass

- **WHEN** any card's guidance is read in any locale
- **THEN** it SHALL NOT instruct the visitor to turn off Smart App Control, SmartScreen or any other protection

### Requirement: Desktop downloads on the web-purchase landing

The page shown after a web purchase and the account page SHALL offer the Windows and Linux downloads described above, and no "coming soon" placeholder SHALL remain for them anywhere on the site once each is available. The store buttons for iOS, iPadOS, macOS and Android SHALL be unchanged.

#### Scenario: After a web purchase

- **WHEN** a subscriber lands on the post-checkout page
- **THEN** the Windows and Linux downloads are offered alongside the store links

#### Scenario: Release notes

- **WHEN** the latest Linux release carries a notes link
- **THEN** the Linux card shows a "release notes" link to it, and shows nothing in its place when it does not
