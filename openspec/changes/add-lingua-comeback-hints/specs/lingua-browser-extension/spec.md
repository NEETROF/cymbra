## MODIFIED Requirements

### Requirement: Page percentage always visible
The icon badge SHALL show the percentage of known words for the active tab when its page has been analysed, refreshed on every state change. Where the active tab shows no percentage — a page not analysed, a page with nothing to count, a tab where the reader is switched off — the badge SHALL show the count of cards waiting for review that *The toolbar icon counts the cards waiting for review* describes, or nothing; it SHALL NOT show a percentage carried over from another page, except in a browser that keeps one badge for every tab (Safari before 18), where the badge SHALL keep showing the last percentage set, as before. The icon popup SHALL break the percentage down: percentage known, words analysed, unknown words (occurrences and distinct words), and the reader's setting — the declared CEFR level when the studied language has CEFR data, otherwise the frequency calibration.

#### Scenario: Badge on an analysed page
- **WHEN** a page at 94% known tokens is active
- **THEN** the badge shows "94%"

#### Scenario: A page with nothing to count
- **WHEN** the active page has no text in a studied language and 12 cards wait for review
- **THEN** the badge shows "12" in the count's colour, not a dash

#### Scenario: Leaving an analysed page
- **WHEN** the reader follows a link from a page at 94% to a page the reader does not analyse
- **THEN** the badge no longer shows "94%"

#### Scenario: Popup shows the declared level
- **WHEN** English (which has CEFR data) is the studied language and the user has declared B2
- **THEN** the popup shows B2 as the reader's level rather than a frequency calibration value

## ADDED Requirements

### Requirement: The toolbar icon counts the cards waiting for review
Where the toolbar icon shows no page percentage, its badge SHALL show how many cards a review would hold today across the reader's studied languages — the reviewed cards due and, in each language, the never-reviewed cards within that language's daily allowance left today — as the number, as « 99+ » above 99, and as nothing at 0. The count SHALL carry no « % »; where the browser lets the extension colour the badge, it SHALL have a colour of its own, distinct from the percentage's and from the colours of word statuses, and black on the e-ink colour presets. The icon's title SHALL say the count in words, naming each language when several hold cards (French UI copy: « Cymbra Lingua — 12 carte(s) à revoir »). The count SHALL be worked out on the device from the reader's own data, only when an event happens — the browser starting, the extension installed or updated, a tab activated, a browser window focused, the reader's data changing (an answer, a capture, a synchronisation applied, a restore, a review made in the Cymbra Lingua app) or a setting it depends on changing — and never from a timer or an alarm; between two events the badge SHALL keep the count worked out at the last one. Working out the count SHALL need no new permission and SHALL make no network request; the synchronisation SHALL keep its own triggers, this count adding none and removing none. A setting (French UI copy: « Afficher les cartes à réviser sur l'icône »), on by default and kept on the device only, SHALL turn the count off; off, the badge SHALL show page percentages only. In a browser that keeps one badge for every tab (Safari before 18), the count SHALL NOT be shown and the setting SHALL NOT be offered.

#### Scenario: Coming back to the browser
- **WHEN** a reader with 12 English cards due and no new word waiting switches to a tab they did not analyse
- **THEN** the badge shows "12" and the icon's title says 12 cards to review

#### Scenario: New words beyond the day's allowance
- **WHEN** 3 reviewed cards are due and 40 never-reviewed cards wait, the daily allowance is 10 and none was introduced today
- **THEN** the badge shows "13"

#### Scenario: The allowance spent
- **WHEN** the reader has let 10 new words into review today and only never-reviewed cards remain
- **THEN** the badge shows nothing until the next local day, and shows up to 10 then, at the next event

#### Scenario: Two languages
- **WHEN** a reader holds 8 English and 4 Spanish cards a review would show today
- **THEN** the badge shows "12" and the title names English 8 and Spanish 4

#### Scenario: Many cards
- **WHEN** 140 cards wait
- **THEN** the badge shows "99+"

#### Scenario: Nothing waiting
- **WHEN** no card would be shown by a review today
- **THEN** the badge is empty on every tab without a percentage

#### Scenario: A card comes due while the reader is away
- **WHEN** a card comes due at 14:00 and the reader does nothing in the browser until 15:00, then activates a tab
- **THEN** nothing in the extension runs at 14:00, and the badge counts that card from the tab's activation

#### Scenario: A review finished in the browser
- **WHEN** the reader answers the last due card in the side panel or the drawer
- **THEN** the badge's count goes down without any page being reloaded

#### Scenario: A review made in the Cymbra Lingua app
- **WHEN** the reader reviews in the app, then comes back to Safari on the same device
- **THEN** the badge counts the app's answers from the moment the extension takes them in from the app, as it already does for the deck

#### Scenario: A review made on another device
- **WHEN** a signed-in reader reviews on their phone, then switches tabs in a browser that has not synced since
- **THEN** the badge may still count those cards, and stops counting them once that browser's next synchronisation applies the answers

#### Scenario: E-ink colours
- **WHEN** the reader chose an e-ink colour preset
- **THEN** both the percentage and the count are drawn black with white text, the « % » telling them apart

#### Scenario: Turned off
- **WHEN** the reader turns the setting off
- **THEN** no count is shown on any tab, the icon's title is the extension's name, and analysed pages still show their percentage

#### Scenario: No new permission
- **WHEN** a reader's browser updates the extension to the version that counts the cards
- **THEN** the browser asks for no new permission

#### Scenario: Safari 17
- **WHEN** a reader uses the extension in Safari 17 with 12 cards waiting
- **THEN** the badge shows no count, Réglages do not offer the setting, and the popup still shows « Réviser (N) »

### Requirement: Réglages carry a link to rate the extension
Réglages › Données SHALL offer a link to rate the extension (French UI copy: « Noter Cymbra Lingua ») in every browser, opening in a new tab the page of the store the build is distributed by — the Chrome Web Store listing's reviews for Chromium, the Firefox Add-ons listing for Firefox, the App Store's write-review page for Safari — through the extension's background, so that no address of a page being read is sent with it. Nothing SHALL be sent before the reader follows the link.

#### Scenario: On Firefox
- **WHEN** a reader of the Firefox build follows the link
- **THEN** the Cymbra Lingua listing on Firefox Add-ons opens in a new tab

#### Scenario: In Safari
- **WHEN** a reader of the Safari extension follows the link
- **THEN** the App Store opens on Cymbra Lingua's write-review page

#### Scenario: From the drawer on a page
- **WHEN** the reader follows the link from Réglages in the drawer on a news article
- **THEN** the store page opens without the article's address being sent to the store
