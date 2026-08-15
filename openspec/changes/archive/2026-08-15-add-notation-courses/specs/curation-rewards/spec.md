## ADDED Requirements

### Requirement: Course-completion badge

The rewards system SHALL include a **course-completion badge** that is awarded when a user first
completes a notation course. It SHALL be awarded **once per course** (a replay does not re-award),
and it SHALL be surfaced through the existing badge feedback. This adds to the badge set only; the
points, shop, and other reward rules are unchanged.

#### Scenario: Awarded on first course completion

- **WHEN** a user completes a notation course for the first time
- **THEN** the course-completion badge is awarded and surfaced through the normal badge feedback

#### Scenario: Not re-awarded on replay

- **WHEN** a user replays a course they had already completed
- **THEN** no additional course-completion badge is granted for that course
