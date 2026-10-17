# Spec Delta

## Purpose

A short, skippable placement puts the learning path's starting point on the right unit for
a player who already knows some of it, without marking anything as completed.

## ADDED Requirements

### Requirement: A placement is offered at the first Learn visit and stays reachable
The Learn door SHALL offer the placement while none was taken and no lesson was completed
on the device; the offer SHALL disappear once a lesson is completed or the placement taken.
The placement SHALL remain reachable from the path at any time and SHALL be skippable at
any step.

#### Scenario: First visit shows the offer
- **WHEN** a learner opens the Learn door for the first time on a device
- **THEN** a placement offer is shown above the path

#### Scenario: Completing a lesson removes the offer
- **WHEN** a learner completes a lesson without taking the placement
- **THEN** the offer no longer shows, and the placement is still reachable from the path header

### Requirement: The placement is a server-authored course probing each unit
The placement SHALL be delivered as a course whose track is `placement`, excluded from the
learning path listing. Its exercise blocks SHALL each carry the unit they probe, covering
the path's units in order, and MAY end with a piece step. The placement SHALL take no
longer than a short set of five to seven exercises plus one piece.

#### Scenario: The placement is not a path lesson
- **WHEN** the learning path is listed
- **THEN** the placement course does not appear among its units

#### Scenario: Every block probes a unit
- **WHEN** the placement content is validated
- **THEN** every exercise block carries a unit tag and the tags cover the units in path order, and validation fails otherwise

### Requirement: The result is a starting unit, chosen by the first miss
On finishing the placement the app SHALL compute the starting unit as the unit of the first
exercise not solved at first try, a skipped exercise counting as a miss, and the placement
SHALL end at that first miss; when every exercise
is solved the starting unit SHALL be the unit after the last one probed. The learner SHALL
be shown the result and MAY adjust it one unit up or down before confirming.

#### Scenario: A miss on unit 3 starts the path there
- **WHEN** the learner solves the unit 1 and 2 exercises first try and misses the unit 3 one
- **THEN** the result proposes unit 3 as the starting unit

#### Scenario: Solving everything starts after the last probed unit
- **WHEN** the learner solves every exercise first try
- **THEN** the result proposes the unit after the last probed one

#### Scenario: The placement stops at the first miss
- **WHEN** the learner misses the unit 2 exercise, or says they do not know it yet
- **THEN** the result is shown at once, without the exercises of the later units

#### Scenario: The learner adjusts the result
- **WHEN** the result proposes unit 3 and the learner picks unit 2 before confirming
- **THEN** the starting unit is unit 2

### Requirement: Units before the start are skipped, never completed
Units before the starting unit SHALL be shown on the path as skipped by placement, distinct
from completed, and SHALL remain open. The continue card and today's session SHALL start
from the starting unit. Skipping by placement SHALL NOT record any completion nor award any
badge.

#### Scenario: Earlier units are collapsed but open
- **WHEN** the starting unit is unit 3
- **THEN** units 1 and 2 are shown as skipped by placement and any of their lessons can still be opened

#### Scenario: No badge for skipped units
- **WHEN** a placement sets the starting unit to unit 3
- **THEN** no course completion is recorded and no badge is awarded for units 1 and 2

#### Scenario: The continue card starts at the starting unit
- **WHEN** the starting unit is unit 3 and nothing is completed
- **THEN** the continue card points to unit 3's first lesson

### Requirement: The placement can be retaken or cleared, and lives on the device
Taking the placement again SHALL replace the starting unit; "Start from the beginning"
SHALL clear it. The starting unit SHALL be stored on the device; a device without it SHALL
start at the first unit and offer the placement.

#### Scenario: Retaking replaces the result
- **WHEN** a learner with starting unit 3 retakes the placement and misses unit 2
- **THEN** the starting unit becomes unit 2

#### Scenario: Clearing restores the full path
- **WHEN** the learner chooses to start from the beginning
- **THEN** no unit is shown as skipped and the continue card points to the first lesson

#### Scenario: Another device starts fresh
- **WHEN** the learner opens the Learn door on a device where no placement was taken
- **THEN** the path starts at unit 1 and the placement is offered
