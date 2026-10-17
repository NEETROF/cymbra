# Spec Delta

## Purpose

A lesson can hand the learner to the real score player on a chosen piece with imposed
settings and a bar to clear, then take them back with the result, so the piano learning
path leads to playing pieces and not only to reading them.

## ADDED Requirements

### Requirement: A piece block opens the player with imposed settings
A course block of type `piece` SHALL open the score player on the referenced bundled or
catalogue piece with the settings the block imposes: hands to play (right, left or both),
playback speed, Wait Mode, metronome and reading aid. The pre-play setup SHALL NOT be shown
before such a run. The player's own saved preferences SHALL NOT be modified by a lesson
run, and the next free play SHALL start from those preferences unchanged.

#### Scenario: The run starts with the block's settings
- **WHEN** the learner starts a piece step imposing the right hand at 60 % speed with Wait Mode on
- **THEN** the player opens on that piece with only the right hand awaited, at 60 % speed, Wait Mode on, with no setup modal

#### Scenario: Free play is untouched afterwards
- **WHEN** the learner leaves the lesson run and opens a score from the library
- **THEN** that run starts from the player's own saved preferences, not the lesson's

#### Scenario: The imposed settings are visible but locked
- **WHEN** the learner opens the play settings during a lesson run
- **THEN** the imposed settings are shown as set by the lesson and cannot be changed for that run

### Requirement: A piece step is passed or visited against its threshold
Each `piece` block SHALL carry a success threshold as an overall synchronisation
percentage, defaulting to 80 when absent. When the run ends, the step SHALL be **passed** if
the run's overall synchronisation percentage meets the threshold and **visited** otherwise.
A visited step SHALL offer *Try again* and SHALL still let the learner continue to the next
block; a step SHALL never block the lesson.

#### Scenario: Meeting the threshold passes the step
- **WHEN** a run on a piece step with threshold 80 ends at 84 % overall synchronisation
- **THEN** the step is marked passed and the lesson offers to continue

#### Scenario: Missing the threshold leaves the step visited
- **WHEN** a run ends at 71 % on a step with threshold 80
- **THEN** the step is marked visited, *Try again* is offered, and *Next* stays available

#### Scenario: Leaving the player without finishing
- **WHEN** the learner quits the player before the end of the piece
- **THEN** the step stays in its previous state and the lesson is shown again where it was

### Requirement: The player returns to the lesson it came from
During a lesson run the player SHALL show the lesson's title and the step's target, and the
end-of-run summary's primary action SHALL return to the lesson. The other summary actions
(see mistakes, retry) SHALL remain available.

#### Scenario: Back to the lesson from the summary
- **WHEN** the end-of-run summary is shown during a lesson run
- **THEN** its primary action returns to the lesson with the run's result applied to the step

#### Scenario: Retry stays in the lesson run
- **WHEN** the learner chooses retry on the summary during a lesson run
- **THEN** the piece restarts with the same imposed settings and the lesson context is kept

### Requirement: Passed state and best result are persisted across devices
The app SHALL persist, per course, whether its piece steps have been passed and the best
overall synchronisation percentage reached on them. When signed in these SHALL be recorded
server-side and read back on other devices; a guest MAY keep them locally until sign-in.
Passed SHALL be set on the first pass and kept; the best percentage SHALL only ever rise.
Recording SHALL NOT block the lesson.

#### Scenario: A pass shows on another device
- **WHEN** a signed-in learner passes a course's piece step on one device
- **THEN** the path on another device shows that course as passed

#### Scenario: The best never regresses
- **WHEN** a learner who reached 90 % on a piece step later ends a run at 75 %
- **THEN** the recorded best stays 90 and the course stays passed

#### Scenario: A course without piece steps records nothing new
- **WHEN** a course with no piece step is completed
- **THEN** only its completion is recorded, as before

### Requirement: The path distinguishes passed, visited and not started
On the learning path, a course containing piece steps SHALL show one of three states: not
started, visited (completed without passing) and passed. The continue card SHALL point to
the first course that is neither completed nor passed, in path order.

#### Scenario: A visited course is shown as such
- **WHEN** a learner completed a course whose piece step was not passed
- **THEN** the path shows that course as visited, distinct from passed and from not started

### Requirement: Each unit closes with a piece lesson it has prepared
The piano path content SHALL end each unit with a piece lesson whose piece uses only
notes, clefs, rhythmic figures and accidentals already taught in that unit or earlier.
Pieces SHALL come from the bundled scores or from catalogue pieces whose licence allows
redistribution. A chain of steps on one piece SHALL end with a free-run step at full tempo
with Wait Mode off.

#### Scenario: A unit's closing piece fits its content
- **WHEN** the content corpus is validated
- **THEN** every unit's closing piece lesson references a piece whose notes and rhythms are within that unit's taught set, and the validation fails otherwise

#### Scenario: Older apps skip piece blocks
- **WHEN** an app that predates this change opens a lesson containing a piece block
- **THEN** the block is skipped and the rest of the lesson runs

#### Scenario: A piece unavailable offline does not trap the learner
- **WHEN** the device is offline and the step's piece is not cached
- **THEN** the step shows that the piece is not available offline and still offers to continue
