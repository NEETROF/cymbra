## ADDED Requirements

<!-- Phase 2, deferred. These requirements are specified now for a coherent single spec but are
     delivered after the notation-help capability ships. No virtual/AI tutor: lessons are scripted. -->

### Requirement: Self-paced scripted staff-reading lessons

The app SHALL offer a set of **self-paced, scripted** beginner lessons that teach reading the
staff (for example: the staff and its lines/spaces, note names, the treble and bass clefs,
accidentals, the key signature, and note/rest durations). Each lesson SHALL be composed of ordered
steps built from **explanation text, a diagram, and an optional light quiz step**, and SHALL run
without a network connection, a backend, or any AI/LLM/text-to-speech component. Lessons SHALL be
**skippable** and MUST NOT be a prerequisite for playing a piece.

#### Scenario: Start and step through a lesson

- **WHEN** the user starts a lesson from the lessons list
- **THEN** they progress through its steps at their own pace and can leave at any time

#### Scenario: A quiz step checks understanding without blocking

- **WHEN** a lesson includes a quiz step and the user answers
- **THEN** the app gives immediate feedback and lets the user continue regardless of the answer

#### Scenario: Lessons are optional

- **WHEN** the user chooses not to take any lesson
- **THEN** they can still use and play the app fully

### Requirement: Lesson progress is persisted and lessons are replayable

The app SHALL persist, **locally**, which lessons the user has completed, so completed lessons are
marked as such across app launches, and SHALL let the user **replay** any lesson from the help/tips
surface.

#### Scenario: Completed lessons are remembered

- **WHEN** the user completes a lesson and later reopens the lessons list
- **THEN** that lesson is shown as completed

#### Scenario: Replay a lesson from help

- **WHEN** the user wants to take a completed lesson again
- **THEN** they can replay it from the help/tips surface

### Requirement: Lessons are reachable, localized, and accessible

The lessons SHALL be reachable from a stable entry point in the help/tips surface, SHALL have all
copy authored through the app's localization system in every supported language, and SHALL be
accessible and consistent with the app's responsive/landscape layout.

#### Scenario: Lessons entry point is discoverable

- **WHEN** the user opens the help/tips surface
- **THEN** the staff-reading lessons are reachable from there

#### Scenario: Lesson copy follows the app language

- **WHEN** the app is used in a supported language
- **THEN** the lesson explanations and quiz copy appear in that language
