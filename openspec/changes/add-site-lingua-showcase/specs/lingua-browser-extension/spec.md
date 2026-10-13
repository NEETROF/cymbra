## ADDED Requirements

### Requirement: Store and site captures come from the real extension, by one command
The repository SHALL hold one command, run on a developer's machine and never in CI, that builds the Chromium package, loads it unpacked in a Chromium that honours unpacked extensions with the browser's field trials left on, opens committed fixture pages — one article per studied language, written for the purpose or in the public domain, their licence stated beside them — and a public-domain EPUB, restores a committed reader state through the extension's own restore, and writes, for each interface language: the site's images, a share image, a silent demo video of about thirty seconds with its poster, and 1280×800 PNG captures without alpha for the Chrome Web Store and addons.mozilla.org. The captures SHALL show a highlighted page with its percentage pill and a word card, a page in the reader's second studied language, a review card with the answers the interface shows, the statistics ladder and the EPUB reader. The build it loads SHALL differ from the published Chromium package only by holding the permission to read every page, and SHALL be written apart from the published packages' folders. Safari and App Store captures SHALL stay manual.

#### Scenario: A set per interface language
- **WHEN** the owner runs the capture command
- **THEN** it writes one set per interface language — French on an English page, English on a Spanish page, Spanish on an English page — each set's interface in its language

#### Scenario: No alpha
- **WHEN** the command writes a store capture whose image has an alpha channel
- **THEN** it fails, naming the file

#### Scenario: The published package is untouched
- **WHEN** the capture build is made beside a Chromium build
- **THEN** every file but the manifest is identical, the manifests differ by the permission to read every page alone, and no published package's folder is written

#### Scenario: The same state each run
- **WHEN** the command is run twice on one commit
- **THEN** both runs restore the same deck, statuses and levels, and the statistics show the fortnight before each run

### Requirement: The store listings describe the extension as it is
The listing texts kept in the repository SHALL describe the review with the answers each interface shows — « Pas su » and « Su », "Didn't know" and "Knew it", « No la sabía » and « La sabía » — SHALL name Firefox on computers and on Android wherever they name the platforms, extended translation's included, and SHALL say that the browser listings' screenshots come from the capture command, listing the shots in their order per listing language. The texts SHALL be pasted by the owner.

#### Scenario: The review in a screenshot plan
- **WHEN** the owner reads a listing's screenshot plan
- **THEN** the review shot shows a revealed card with the interface's two answers, and no plan mentions four answers

#### Scenario: Firefox for Android
- **WHEN** a description names where Cymbra Lingua is available
- **THEN** it names Firefox on computers and on Android, in the description's language
