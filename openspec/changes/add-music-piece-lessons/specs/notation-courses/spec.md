# Spec Delta

## MODIFIED Requirements

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
- **piece** — the user is sent to the **full score player** on a bundled or catalogue piece with the
  settings the block imposes (hands, speed, Wait Mode, metronome, reading aid) and a success
  threshold; the player returns to the lesson with the run's result, and the step is passed or
  visited accordingly. A visited step still offers a way to continue.

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

#### Scenario: A piece block runs in the full player and comes back

- **WHEN** the user reaches a piece block and starts it
- **THEN** the full score player opens on that piece with the imposed settings, and ending the run returns to the lesson with the step marked passed or visited
