# lingua-browser-extension Specification

## Purpose
TBD - created by archiving change add-lingua-extension-reading. Update Purpose after archive.
## Requirements
### Requirement: In-place highlighting without DOM mutation
The extension SHALL highlight unknown words (and, distinctly, "learning" words) through the
CSS Custom Highlight API, without wrapping words in elements or otherwise modifying the
page DOM. Dynamic content (SPAs) SHALL be re-analysed per mutated subtree, debounced. The
content script SHALL consume analysis exclusively through a message port (`AnalyzerPort`);
within this Chromium scope the implementation is the WASM module instantiated in the
content script — the port is the seam planned for the later variants in the stack (WASM in
the event page on Firefox, nativeMessaging on Safari), with no change to the content
script.

#### Scenario: English page highlighted
- **WHEN** an English article page is loaded with the extension active
- **THEN** unknown words appear highlighted and the page DOM contains no node added by the extension (outside the extension's own UI hosts)

#### Scenario: Dynamically inserted content
- **WHEN** an SPA inserts a new English paragraph
- **THEN** that paragraph is analysed and highlighted without re-analysing the whole page

### Requirement: Page percentage always visible
The icon badge SHALL show the percentage of known words for the active tab, refreshed on
every state change; on a page that cannot be analysed it SHALL show a neutral state. The
icon popup SHALL break this down: percentage known, words analysed, unknown words
(occurrences and distinct words), and the calibration setting.

#### Scenario: Badge on an analysed page
- **WHEN** a page at 94% known tokens is active
- **THEN** the badge shows "94%"

### Requirement: Word popup on click
Clicking a highlighted word SHALL open a panel (closed shadow DOM) showing: the dictionary
form, the form as seen if it differs, the native-language gloss (from the pack, offline),
the frequency rank in plain language, and the actions "Je connais", "+ Deck" and "Ignorer"
(the shipping UI copy is French). Each action SHALL update the status, repaint the page
immediately and propagate to the other tabs.

#### Scenario: Adding to the deck
- **WHEN** the user clicks "+ Deck" on a highlighted word
- **THEN** the word switches to the "learning" highlight on this tab and on every other open tab, and a card is created with the source sentence

### Requirement: Selection capture on a keyboard shortcut
A keyboard shortcut SHALL capture the current selection (a word or a phrase, bounded in
length) and open the panel with the source sentence extracted automatically, allowing the
selection to be added to the deck as a phrase card.

#### Scenario: Capturing a phrase
- **WHEN** the user selects three words and presses the shortcut
- **THEN** the panel shows the phrase and its source sentence, and "+ Deck" creates the card

### Requirement: Cymbra visual identity
The UI surfaces the extension owns (icon popup, extension pages) SHALL apply the Cymbra
visual identity — the "Sonic Luminescence" palette from
`apps/music/lib/theme/cymbra_theme.dart`, already mirrored into CSS variables by the back
office (`apps/back-office/src/styles.css`) — through a single token sheet embedded in the
extension. Surfaces injected into third-party pages (the word popup) SHALL consume the same
tokens, with legibility on both light and dark pages taking precedence over fidelity to the
dark theme. Highlight tints SHALL derive from the palette's amber (`handLeft` —
"learning") and coral (`error` — "unknown"). No color SHALL be hard-coded outside the token
sheet.

#### Scenario: Extension surfaces stay consistent
- **WHEN** the user opens the icon popup and then an extension page
- **THEN** both surfaces render with the Cymbra tokens (Midnight Navy backgrounds, violet primary, the identity's radii) and no hex outside the token sheet exists in the styles (checked by lint)

#### Scenario: Highlighting legible on a light page
- **WHEN** a light-background page is highlighted
- **THEN** the amber and coral highlight tints leave the page text in its original color and stay distinguishable from each other

### Requirement: Minimal permission posture
The extension SHALL install with `activeTab` and declare `<all_urls>` as an optional
permission: "highlight this page" SHALL work without a global grant; "always highlight"
SHALL ask for the grant exactly once. No page text SHALL leave the device.

#### Scenario: First use without a global grant
- **WHEN** the user clicks the icon on a page without having granted `<all_urls>`
- **THEN** the current page is analysed and highlighted through `activeTab`

### Requirement: No network requests
In v1 the extension SHALL issue no network request at all: the pack and the glosses are
local assets, and neither page text nor user data leaves the device.

#### Scenario: Working offline
- **WHEN** the user reads an already-loaded page with no network connection
- **THEN** highlighting and the word popup (gloss included) work in full

### Requirement: Versioned local state
State (statuses, cards, calibration, preferences) SHALL live in `chrome.storage.local`
under a versioned schema with forward migration. A full reset SHALL be offered.

#### Scenario: Schema migration
- **WHEN** the extension starts on state from an earlier version
- **THEN** the state is migrated without loss and the stored version is updated

### Requirement: Two review surfaces
The extension SHALL offer review in the **browser's native panel** where one exists (Side
Panel on Chromium, sidebar on Firefox — the page is pushed, the panel survives navigation)
and in a collapsible **injected panel** (shadow DOM) for micro-reviews. On Safari, which
has no panel API, the injected panel SHALL carry in-browser review on its own. Every
surface SHALL operate on the same local state. Lossless backup/restore (defined by
`lingua-decks-review`) SHALL be reachable from the side panel (file download and
re-import).

#### Scenario: Side panel during navigation
- **WHEN** the user opens the side panel and then navigates to another page
- **THEN** the side panel stays open and its review session continues

#### Scenario: Backup from the side panel
- **WHEN** the user triggers a backup from the side panel
- **THEN** a versioned backup file is downloaded containing the complete state (cards field by field, statuses, calibration, FSRS parameters), and re-importing it restores the state identically

