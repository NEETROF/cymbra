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
(occurrences and distinct words), and the reader's setting — the declared CEFR level when
the studied language has CEFR data, otherwise the frequency calibration.

#### Scenario: Badge on an analysed page
- **WHEN** a page at 94% known tokens is active
- **THEN** the badge shows "94%"

#### Scenario: Popup shows the declared level
- **WHEN** English (which has CEFR data) is the studied language and the user has declared B2
- **THEN** the popup shows B2 as the reader's level rather than a frequency calibration value

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

### Requirement: Firefox variant
The extension SHALL ship on Firefox (desktop and Android) as a build variant produced from the same source as the chromium variant: an event page (`background.scripts` declared alongside `service_worker`), WASM analysis loaded in the event page and consumed by the content script through the `AnalyzerPort`, optional host permissions requested at install, the panel via `sidebar_action` (the same page as the Chromium side panel), and AMO publication for desktop and Android from the same zip.

#### Scenario: One build, two artefacts
- **WHEN** the release build runs
- **THEN** it produces the chromium and firefox variants from the same source, differing only in the manifest and the `AnalyzerPort` implementation (the safari variant joins this build with `add-lingua-apple`)

#### Scenario: Firefox permissions at install
- **WHEN** the user installs the extension on Firefox
- **THEN** the extension works in its per-page mode (« surligner cette page » — the UI ships in French) and offers the global grant through the optional-permissions prompt

### Requirement: CEFR level picker and progression ladder
The popup SHALL let the user declare their CEFR level (A1–C2) for a studied language that
has CEFR data, keeping the frequency slider only as the fallback for languages without it.
The learning-statistics screen SHALL present a CEFR ladder: one row per level A1→C2 showing
confirmed, presumed, and to-learn counts, plus an estimated overall position. Where the
studied language has no CEFR data the ladder SHALL fall back to frequency bands and SHALL
label the result as an estimate, never as a CEFR assessment.

#### Scenario: Declaring a level
- **WHEN** the user picks B2 in the popup
- **THEN** the declared level is stored and words below B2 stop being highlighted

#### Scenario: Ladder with real CEFR data
- **WHEN** the stats screen opens for English
- **THEN** it shows A1→C2 rows with confirmed/presumed/to-learn progress and an estimated position

#### Scenario: Ladder without CEFR data
- **WHEN** the studied language has no CEFR data
- **THEN** the screen shows frequency bands labelled as an estimate, not CEFR levels

### Requirement: Highlighting gated at the declared level
When a declared CEFR level is in effect, the extension SHALL highlight only words at that
level and above; words below it (presumed known) SHALL NOT be highlighted, so the reader is
not bothered by vocabulary they claim to know. A word the user acts on (marks, adds to a
deck, ignores) SHALL take an explicit status and be treated accordingly regardless of its
level.

#### Scenario: Below-level words are quiet
- **WHEN** the user has declared B2 and reads a page containing A1–B1 and B2–C1 words
- **THEN** only the B2 and above unknown words are highlighted

### Requirement: Level-targeted deck feeding control
The extension SHALL offer a control to feed a deck from a chosen level ("Renforcer un
niveau"): the user picks a level, a word count (bounded by the seeding cap), and an order,
and the extension seeds that many words via the deck's level-targeted seeding, reporting how
many were added.

#### Scenario: Feed a level
- **WHEN** the user picks level B2, a count of 20, commonest-first, and confirms
- **THEN** up to 20 B2 words are added to the deck and the control reports the number added

