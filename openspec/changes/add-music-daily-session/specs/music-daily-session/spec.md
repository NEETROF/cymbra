# Spec Delta

## Purpose

The Learn door opens on a short named session for today, built from the learner's position
on the path, with a goal they size and a ring that fills as the items get done.

## ADDED Requirements

### Requirement: Today's session is composed from the learner's position
The Learn door SHALL show a session for today of up to three items: the next lesson in
path order that is neither completed nor passed, the next unpassed piece step of that
lesson or, when it has none, a passed piece to replay, and a short review drawn from the
most recently completed unit. Each item SHALL open exactly what it names. When the path is
finished, the session SHALL offer a piece, a review and a replay suggestion.

#### Scenario: A learner mid-path gets lesson, piece and review
- **WHEN** a learner has completed units 1 and 2 and is on unit 3's second lesson
- **THEN** the session lists that lesson, its piece step if any or a passed piece to replay, and a review from unit 2

#### Scenario: A finished path still has a session
- **WHEN** every course on the path is completed or passed
- **THEN** the session lists a piece to replay, a review and a replay suggestion, with no lesson item

#### Scenario: An item opens its target
- **WHEN** the learner taps the lesson item
- **THEN** the lesson player opens on that lesson

### Requirement: The learner sizes the daily goal
The app SHALL let the learner choose a daily goal of one, two or three items, defaulting
to two, stored on the device. The session ring SHALL show items done today over the goal.
All three items SHALL stay available whatever the goal.

#### Scenario: The ring follows the goal
- **WHEN** the goal is two and one item was done today
- **THEN** the ring shows one of two

#### Scenario: Doing more than the goal is possible
- **WHEN** the goal is one and the learner has done it
- **THEN** the remaining items are still offered and the ring stays full

### Requirement: Done is detected on the player's local day
An item SHALL count as done when its target was completed or run on the current local day:
a lesson completed, a piece step run, a review finished. The local day SHALL be the same
local day the practice streak uses. Items SHALL reset when the local day changes. A lesson
or piece already recorded as done on the server for today SHALL also read as done on
another device.

#### Scenario: Completing the lesson marks the item
- **WHEN** the learner completes the session's lesson
- **THEN** the lesson item is shown done and the ring advances

#### Scenario: Midnight resets the session
- **WHEN** the local day changes
- **THEN** the items reset to not done and a new session is composed

#### Scenario: Yesterday's work does not count
- **WHEN** the learner completed the lesson yesterday and opens the app today
- **THEN** that lesson is no longer the session's lesson item and nothing is shown done

### Requirement: Finishing the goal is marked once a day
When the number of items done today reaches the goal, the session SHALL be marked complete
for the day and the app SHALL raise a completion cue once. Further items done that day
SHALL NOT raise it again.

#### Scenario: Reaching the goal raises the cue once
- **WHEN** the second of two items is done with a goal of two
- **THEN** the session is marked complete and the completion cue is raised

#### Scenario: A third item does not repeat the cue
- **WHEN** the learner does a third item after the goal was met
- **THEN** no second completion cue is raised

### Requirement: The session shows the streak it feeds
The session block SHALL show the learner's current streak as reported by the server and
whether today is already covered. It SHALL NOT compute or advance a streak itself. Signed
out, it SHALL show the session without a streak.

#### Scenario: The streak is shown from the server read
- **WHEN** the server reports a seven-day streak with today not yet covered
- **THEN** the block shows seven days and that today is not covered yet

#### Scenario: A guest sees the session alone
- **WHEN** the learner is signed out
- **THEN** the session is shown with its items and ring and no streak
