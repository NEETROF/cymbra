## ADDED Requirements

### Requirement: Courses are stored on the server and delivered to the app

Course content SHALL be stored **server-side in the database** as a manifest (a JSON blob) with
metadata, and delivered to the app over gRPC (a list endpoint and a fetch-by-id endpoint). The app
SHALL NOT ship course content bundled in the binary; it SHALL fetch courses from the server and MAY
cache them locally for offline use. First-party courses SHALL be provisioned by seeding the
database.

#### Scenario: The app lists courses from the server

- **WHEN** the home screen loads
- **THEN** the app obtains the available courses from the server (or its local cache), not from bundled assets

#### Scenario: A course's content is fetched on demand

- **WHEN** the user opens a course
- **THEN** its manifest is fetched from the server (or cache) and run by the lesson player

#### Scenario: Cached courses work offline

- **WHEN** the device is offline and a course was previously fetched
- **THEN** the cached course can still be opened and taken

### Requirement: Courses are a generic, versioned, forward-compatible block format

A course manifest SHALL declare a `schemaVersion`, an **`instrument`** it targets (e.g. `piano`), and
a **`track`** and **`level`** for grouping (e.g. solfège / app-usage / technique × beginner /
intermediate / advanced), plus inline-localized metadata and an **ordered list of typed blocks**,
where each block carries a `type`, its localized content, an optional **media reference (URL)**, and
an **advance gate** describing how the user proceeds. Interactive block types are **instrument-scoped**
(e.g. `playKey` targets a keyboard instrument), so the format can extend to other instruments later
(a drum track swapping the interactive blocks) without a breaking change. All
human-readable copy SHALL be **localized inline within the manifest** (per supported language), so a
manifest is self-contained and independent of the app's localization resources. The format SHALL be
**forward-compatible**: when the app encounters a block whose `type` (or required capability) it does
not support, it SHALL **skip that block gracefully** and continue the course, rather than failing.
Media SHALL be referenced by URL, never embedded in the manifest.

#### Scenario: A manifest parses into ordered, localized blocks

- **WHEN** a course manifest is loaded
- **THEN** it yields an ordered list of typed blocks with copy in the app's language taken from the manifest

#### Scenario: An unsupported block type is skipped, not fatal

- **WHEN** the app runs a course containing a block type it does not understand
- **THEN** it skips that block and continues the rest of the course without error

#### Scenario: An unknown schema version is handled safely

- **WHEN** a manifest declares a `schemaVersion` the app cannot handle at all
- **THEN** the app declines that course gracefully and still lists/opens the others

### Requirement: The lesson player runs the v1 block types

The lesson player SHALL run a course's blocks at the user's pace and support, at minimum, these
block types:

- **text** — a localized explanation.
- **diagram** — a built-in notation diagram (from a closed set of ids) rendered by the app's existing
  notation renderer.
- **image** — an image shown from a URL, with a localized caption.
- **video** — a video shown from a URL.
- **question** — a multiple-choice or true/false question that gives **immediate feedback** and MUST
  NOT hard-block the user from continuing whatever the answer.
- **playKey** — the user is asked to play a specific note or chord; the app validates the input from
  the **on-screen keyboard or a connected MIDI instrument**, and offers a way to continue (skip/next)
  so a stuck user is never trapped.
- **score** — an embedded notation excerpt rendered by the app's existing notation renderer,
  optionally **playable** by reusing the player/scoring so the user can perform it.

Advancing past a block SHALL follow the block's gate (e.g. a Next control, a correct answer, correct
input, or media finished), and every interactive block SHALL always offer a non-blocking way to
continue.

#### Scenario: A question gives feedback without blocking

- **WHEN** the user answers a question block
- **THEN** immediate feedback is shown and the user can continue whatever the answer

#### Scenario: A playKey block validates real input

- **WHEN** a playKey block asks for a note and the user plays it on the keyboard or a MIDI device
- **THEN** the block recognises the correct input and lets the user advance; a stuck user can still skip

#### Scenario: A score block renders (and can be played)

- **WHEN** the user reaches a score block
- **THEN** the embedded excerpt is engraved by the notation renderer, and where marked playable it can be performed

#### Scenario: A media block shows from a URL

- **WHEN** the user reaches an image or video block
- **THEN** the media is shown from its URL (no media is embedded in the manifest)

### Requirement: Courses are on the home screen, above the favorites

The app SHALL present a **Courses** section on the home (library) screen, positioned **above** the
favorited scores, with **one tile per available course** and a **completion indicator** on each,
**grouped by track and level** (from the manifest). The section SHALL be reachable without opening a
menu and MUST NOT displace or block the favorites below it; with no courses available it SHALL omit
gracefully.

#### Scenario: Courses appear above favorites, grouped by track/level

- **WHEN** the user opens the home screen
- **THEN** a Courses section is shown above the favorites, one tile per course grouped by track and level, each with a completion indicator

#### Scenario: Opening a course from its tile

- **WHEN** the user taps a course tile
- **THEN** the lesson player opens on that course

### Requirement: A course is skippable and infinitely replayable

Taking a course SHALL be **skippable** and MUST NOT be a prerequisite for playing a piece, and a
course SHALL be **replayable an unlimited number of times**; replaying a completed course keeps it
marked completed.

#### Scenario: Courses are optional

- **WHEN** the user chooses not to take a course
- **THEN** they can use and play the app fully

#### Scenario: A completed course can be replayed without limit

- **WHEN** the user reopens a course they already completed
- **THEN** they can take it again any number of times and it stays completed

### Requirement: Course completion is persisted across devices

The app SHALL persist a course's **completed** state so it is reflected on **any of the user's
devices**. When signed in, completion SHALL be recorded **server-side** and read back on other
devices; a guest MAY record completion locally until they sign in. Recording or reading completion
MUST NOT block taking or replaying a course.

#### Scenario: Completion shows across devices

- **WHEN** a signed-in user completes a course on one device
- **THEN** it shows completed when they open the app on another device

#### Scenario: A guest's completion is local until sign-in

- **WHEN** a guest completes a course
- **THEN** it shows completed locally without requiring an account

#### Scenario: Completion loading never blocks the course

- **WHEN** completion state is still being fetched
- **THEN** the user can still open and take the course

### Requirement: Finishing a course awards a badge, once

The app SHALL award a **badge** when the user first completes a course, surfaced through the existing
rewards/badges feedback, decided server-side so it is awarded **once per course** and never
double-awarded across devices or replays.

#### Scenario: First completion awards a badge

- **WHEN** the user completes a course for the first time
- **THEN** a completion badge is awarded and shown

#### Scenario: Replays and other devices do not re-award

- **WHEN** the user replays a completed course, or completes it again on another device
- **THEN** no additional badge is granted for that course

### Requirement: Course chrome is localized and accessible

The Courses section, tiles, and lesson-player controls SHALL be localized through the app's
localization system and be accessible (operable without relying on a single gesture, adequate
contrast, screen-reader friendly), consistent with the responsive layout. (Course *content* is
localized by the manifest.)

#### Scenario: The course UI follows the app language

- **WHEN** the app is used in a supported language
- **THEN** the Courses section and lesson-player controls appear in that language

#### Scenario: The lesson player is accessible

- **WHEN** a user relies on assistive input
- **THEN** they can navigate blocks and answer questions without depending on a single specific gesture
