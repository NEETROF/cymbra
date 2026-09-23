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

The reader's state — word statuses, cards and their review history, declared level and calibration, daily statistics, and the cursors that describe what has been synchronised — SHALL live in a browser store whose capacity grows with the reader (IndexedDB), under a versioned schema with forward migration. Preferences, session tokens and transient marks MAY stay in the extension's settings storage.

One context SHALL own that store; every surface SHALL read and write it through the same seam, and SHALL be told which of its keys changed, so all surfaces show the same state. A store that cannot be opened SHALL NOT take the extension down: it SHALL fall back to the settings storage and keep working. Moving state from a previous store SHALL lose nothing. Once the new store holds a piece of state, the previous copy of it SHALL be released, so that nothing of the reader's data keeps space in the store it moved out of. A full reset SHALL be offered.

#### Scenario: Schema migration
- **WHEN** the extension starts on state from an earlier version
- **THEN** the state is migrated without loss and the stored version is updated

#### Scenario: Moving to the store that grows
- **WHEN** the extension starts for the first time after the state's home changes
- **THEN** every piece of the reader's state is readable from the new store, and no copy of it is left behind in the store it came from

#### Scenario: A copy an earlier build left behind
- **WHEN** a device moved its state while the previous copy was still being kept, and starts again
- **THEN** the copy is released, and anything the new store does not hold is left alone rather than lost

#### Scenario: A reader who reads a great deal
- **WHEN** the reader's state grows past what the settings storage would have accepted
- **THEN** marking a word, reviewing, synchronising and erasing all keep working

#### Scenario: Surfaces follow the same state
- **WHEN** a word is marked in the page while the panel is open
- **THEN** the panel shows the change without being reopened

#### Scenario: The store cannot be opened
- **WHEN** the browser refuses to open the store
- **THEN** the extension keeps reading, marking and reviewing on the settings storage

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

The report SHALL also name **Safari**, which this act never reaches: that variant ships inside the Apple host application, on its own tag. A report that names only the destinations an act can reach lets the one it cannot fall out of sight — and nothing fails when Safari is forgotten, so nothing else will raise it.

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

#### Scenario: The destination this act cannot reach
- **WHEN** a submission is reported, whatever the stores answered
- **THEN** the report also states that Safari does not have this version and how it is sent, rather than naming only the destinations this act reaches

#### Scenario: A store holds every credential and still refuses
- **WHEN** a store rejects the submission for a reason of its own, such as an unfinished listing
- **THEN** the run reports that store as having refused the version rather than as having received it, still submits to the other store, and fails

### Requirement: Selection capture on any pointer
Selecting text SHALL capture that selection on every pointer — mouse, touch and keyboard shortcut alike — and open the panel with the source sentence extracted automatically.
The capture SHALL fire once the selection has settled, not on a particular input event, so
that a touch selection adjusted with the platform's own handles is captured like a mouse
drag. A selection of one word SHALL open that word's popup with the actions matching its
current status; a selection of several words, bounded in length, SHALL open the
whole-selection card, which "+ Deck" adds as a phrase card. A selection longer than the
bound SHALL capture nothing.

#### Scenario: Capturing a phrase on a phone
- **WHEN** the reader selects three words on a touch device and lifts the finger
- **THEN** the panel shows the phrase and its source sentence, and "+ Deck" creates the card

#### Scenario: Adjusting a touch selection with the native handles
- **WHEN** the reader drags a selection handle to extend a one-word selection to three words
- **THEN** the panel follows the selection and ends on the three-word phrase, without any further gesture

#### Scenario: Capturing a phrase with the keyboard shortcut
- **WHEN** the user selects three words and presses the shortcut
- **THEN** the panel shows the phrase and its source sentence, and "+ Deck" creates the card

#### Scenario: Selecting a single already-known word
- **WHEN** the reader selects one word whose status is "known"
- **THEN** the popup opens for that word and does not offer "Je connais" again

#### Scenario: An over-long selection captures nothing
- **WHEN** the reader selects a passage longer than the phrase bound
- **THEN** no panel opens and the page keeps its selection

### Requirement: The reader never fights the platform's text selection
The reader SHALL NOT clear, suppress or pre-empt the host platform's native text selection.
It SHALL NOT cancel `selectstart`, SHALL NOT suppress the platform's selection callout or
context menu, and SHALL NOT remove the document's ranges on its own. Where the platform's
selection gesture and a reader gesture are the same physical gesture, the platform's
selection SHALL win, and the reader SHALL derive its behaviour from the resulting selection.
A platform-drawn selection menu appearing alongside the panel is accepted; the panel SHALL
be positioned so it does not sit under that menu.

#### Scenario: Press-and-hold on a phone
- **WHEN** the reader presses and holds a word on a touch device
- **THEN** the platform selects that word and its own menu appears, and the panel opens for that word with the actions matching its status

#### Scenario: Reclassifying a non-highlighted word on a phone
- **WHEN** the reader presses and holds a word marked "known" or "ignored"
- **THEN** the popup opens offering to reclassify it, reached through the platform selection rather than a competing long-press gesture

### Requirement: Stats view renders without dynamic innerHTML

The stats/review view (shared by the Chromium side panel and the Firefox/Safari in-page drawer) SHALL build its markup with DOM construction APIs (`createElement`, `createElementNS`, `append`, `textContent`) rather than assigning a dynamically-built string to `innerHTML`. This applies whether or not the interpolated data is user-controlled: the rule is about the rendering mechanism, not about proving any particular value is safe to interpolate.

#### Scenario: Rendering the vocabulary estimate, ladder, seed control, and daily cards

- **WHEN** the stats view mounts or re-renders any of its sections (vocabulary estimate, CEFR ladder, "Renforcer un niveau" control, per-metric daily-count cards)
- **THEN** the section's markup is constructed via DOM APIs, with no assignment of a dynamically-built string to an element's `innerHTML`

#### Scenario: A store review adds new stats markup

- **WHEN** a future change adds a new section or control to the stats view
- **THEN** its markup is built the same way — DOM construction, not `innerHTML` — consistent with the rest of the view

