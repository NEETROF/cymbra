## ADDED Requirements

### Requirement: English is studied by readers of Italian
Once a listed pair studies English glossed in Italian, the extension SHALL show its interface in Italian to a reader whose native language is Italian, English their default studied language. An English document SHALL be read, highlighted, glossed in Italian, recorded and read aloud for them through en-it; every surface that names a studied language, asks or shows its level, or links a Cymbra page SHALL do so in Italian, the links opening the Italian privacy policy, leaving page and account deletion page. An English word on the card SHALL be marked as English inside the Italian lines, and the card SHALL name English's forms in Italian.

#### Scenario: An English word on an Italian card
- **WHEN** a reader whose native language is Italian opens the card of « went »
- **THEN** the card's lines are Italian, it reads « passato semplice di go », and the English words in it are marked as English

#### Scenario: The Italian pages
- **WHEN** a reader whose interface is Italian opens the privacy policy from Settings, or, on Chrome or Firefox with counting on, removes the extension
- **THEN** `https://cymbra.app/it/privacy/` opens, or the browser opens `https://cymbra.app/it/lingua/arrivederci/` with its browser parameters

#### Scenario: Nothing moves for the other readers
- **WHEN** a reader whose native language is French, English or Spanish updates to a package that lists en-it
- **THEN** their pairs, studied languages, interface and links are what they were, but the account deletion link of a Spanish interface, or of a reader who has not chosen their language and whose browser is in Spanish or Italian, now opens the Spanish or Italian page; and a reader who chose their language, with a browser in Italian, sends their interface language to Cymbra ID (I10)

## MODIFIED Requirements

### Requirement: Store and site captures come from the real extension, by one command
The repository SHALL hold one command, run on a developer's machine and never in CI, that builds the Chromium package, loads it unpacked in a Chromium that honours unpacked extensions with the browser's field trials left on, opens committed fixture pages — one article per studied language, and a second English one for a reader who studies English alone, written for the purpose or in the public domain, their licence stated beside them — and a public-domain EPUB, restores a committed reader state through the extension's own restore, and writes, for each interface language: the site's images, a share image, a silent demo video of about thirty seconds with its poster, and 1280×800 PNG captures without alpha for the Chrome Web Store and addons.mozilla.org. The captures SHALL show a highlighted page with its percentage pill and a word card, a page in the reader's second studied language, or a second page in the one they study when they study one, a review card, in the review panel the extension draws in the page, with the answers the interface shows, the statistics ladder and the EPUB reader. The build it loads SHALL differ from a Chromium build made with the same settings only by declaring the reader's content script on every page, and SHALL be written apart from the published packages' folders. Safari and App Store captures SHALL stay manual.

#### Scenario: A set per interface language
- **WHEN** the owner runs the capture command
- **THEN** it writes one set per interface language — French on an English page, English on a Spanish page, Spanish on an English page, Italian on an English page — each set's interface in its language

#### Scenario: No alpha
- **WHEN** the command writes a store capture whose image has an alpha channel
- **THEN** it fails, naming the file

#### Scenario: The published package is untouched
- **WHEN** the capture build is made beside a Chromium build
- **THEN** every file but the manifest is identical, the manifests differ by the reader's content script on every page alone, and no published package's folder is written

#### Scenario: The same state each run
- **WHEN** the command is run twice on one commit
- **THEN** both runs restore the same deck, statuses and levels, with the same cards due on the run's day, and the statistics show the thirty days ending on each run's day
