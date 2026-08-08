## ADDED Requirements

### Requirement: Courses are on the home screen, above the favorites

The app SHALL present a **Courses** section on the home (library) screen, positioned **above** the
favorited scores, with **one tile per available course**. Each tile SHALL show the course and a
**completion indicator** reflecting whether the user has finished it. The section SHALL be reachable
without opening a menu, and MUST NOT displace or block the favorites below it.

#### Scenario: Courses appear above favorites on the home screen

- **WHEN** the user opens the home screen
- **THEN** a Courses section is shown above the favorited scores, with one tile per course

#### Scenario: A tile shows completion

- **WHEN** a course has been completed by the user
- **THEN** its tile shows a completed indicator; an unfinished course's tile does not

#### Scenario: Opening a course from its tile

- **WHEN** the user taps a course tile
- **THEN** the lesson player opens on that course

### Requirement: Courses are declarative, versioned, inline-localized manifests

A course SHALL be defined by a **self-describing manifest** carrying a `schemaVersion`, a stable
course id, and an ordered list of **steps**, where a step is an **explanation**, a **diagram**, or a
**quiz**. All human-readable copy in a manifest (titles, explanations, quiz prompts, options,
feedback) SHALL be **localized inline within the manifest** (per supported language), so a course is
fully self-contained and independent of the app's own localization resources. First-party courses
SHALL ship **bundled with the app**. The format SHALL be defined so that a course authored by a
third party and delivered over the network would use the **same schema** (enabling later community
courses without a format change).

#### Scenario: A bundled course loads from its manifest

- **WHEN** the app starts
- **THEN** each bundled course manifest is parsed into an ordered set of steps with localized copy

#### Scenario: Course copy follows the app language from the manifest

- **WHEN** the app is used in a supported language
- **THEN** a course's titles, explanations and quiz copy are shown in that language, taken from the manifest (not the app's ARB)

#### Scenario: An unknown or unsupported schema version is handled safely

- **WHEN** a manifest declares a `schemaVersion` the app does not support
- **THEN** the app skips that course rather than crashing, and still shows the others

### Requirement: Self-paced lesson player

The app SHALL let the user step through a course's steps **at their own pace**: reading an
explanation, viewing a diagram, and answering **quiz** steps. A quiz step SHALL give **immediate
feedback** on the answer but MUST NOT block the user from continuing regardless of the answer.
Taking a course SHALL be **skippable** and MUST NOT be a prerequisite for playing a piece, and a
course SHALL be **replayable an unlimited number of times**.

#### Scenario: Stepping through a course

- **WHEN** the user starts a course
- **THEN** they advance through its steps at their own pace and can leave at any time

#### Scenario: A quiz gives feedback without blocking

- **WHEN** the user answers a quiz step
- **THEN** the app shows immediate feedback and lets the user continue whatever the answer

#### Scenario: A completed course can be replayed

- **WHEN** the user reopens a course they already completed
- **THEN** they can play it again, without limit, and completion stays recorded

### Requirement: Course completion is persisted across devices

The app SHALL persist a course's **completed** state so it is reflected on **any of the user's
devices**, not only the one where it was finished. When signed in, completion SHALL be recorded
**server-side** and read back on other devices; a guest (no account) MAY record completion locally
until they sign in. Recording or reading completion MUST NOT block taking or replaying a course.

#### Scenario: Completing a course marks it done everywhere

- **WHEN** a signed-in user completes a course on one device
- **THEN** the course shows as completed when they open the app on another device

#### Scenario: A guest's completion is local until sign-in

- **WHEN** a guest completes a course
- **THEN** it is shown completed locally, without requiring an account

#### Scenario: Completion state loads without blocking the course

- **WHEN** the completion state is still being fetched
- **THEN** the user can still open and take the course

### Requirement: Finishing a course awards a badge

The app SHALL award the user a **badge** when they first complete a course, surfaced through the
existing rewards/badges feedback. Replaying an already-completed course SHALL NOT award it again.

#### Scenario: First completion awards a badge

- **WHEN** the user completes a course for the first time
- **THEN** a completion badge is awarded and shown to them

#### Scenario: Replays do not re-award

- **WHEN** the user replays a course they already completed
- **THEN** no additional badge is awarded

### Requirement: Course chrome is localized and accessible

The Courses section, tiles, and lesson-player controls (navigation, quiz controls, feedback) SHALL
be localized through the app's localization system and be accessible (operable without relying on a
single gesture, adequate contrast, screen-reader friendly), consistent with the app's responsive
layout. (Course *content* is localized by the manifest, per the manifest requirement.)

#### Scenario: The course UI follows the app language

- **WHEN** the app is used in a supported language
- **THEN** the Courses section and lesson-player controls appear in that language

#### Scenario: The lesson player is accessible

- **WHEN** a user relies on assistive input
- **THEN** they can navigate steps and answer quizzes without depending on a single specific gesture
