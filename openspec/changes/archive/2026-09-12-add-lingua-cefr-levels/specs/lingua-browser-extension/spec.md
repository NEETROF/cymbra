# lingua-browser-extension — CEFR level picker, ladder, level-gated highlighting

## MODIFIED Requirements

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

## ADDED Requirements

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
