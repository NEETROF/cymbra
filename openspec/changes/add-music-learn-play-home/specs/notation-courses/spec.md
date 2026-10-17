# Spec Delta

## MODIFIED Requirements

### Requirement: Courses are on the home screen, above the favorites

The app SHALL present the courses on the home screen as the body of the **Learn** door: a
**continue card** carrying the learner's next lesson (its title, unit and unit progress) and
the full **learning path** grouped by unit, with one node per course and a **completion
indicator** on each. The Learn door SHALL be reachable without opening a menu. Under the
**Play** door the favorites SHALL NOT be preceded by a courses card. When the two-door home
is switched off by its runtime flag, the app SHALL fall back to a single home with the
continue card and the entry to the path pinned **above** the favorites, which MUST NOT be
displaced or blocked by it. With no courses available the courses content SHALL omit itself
gracefully in either layout.

#### Scenario: The Learn door shows the path

- **WHEN** the user opens the home and selects the Learn door
- **THEN** the continue card and the learning path are shown, grouped by unit, each node with a completion indicator

#### Scenario: The Play door shows no courses card

- **WHEN** the user opens the home on the Play door
- **THEN** the favorites start at the top of the body and no courses card precedes them

#### Scenario: Courses appear above favorites, grouped by track/level

- **WHEN** the two-door home is switched off by its runtime flag
- **THEN** the continue card and the entry to the path, grouped by track and level, are shown above the favorites without displacing them

#### Scenario: Opening a course from its tile

- **WHEN** the user taps a lesson node or the continue card
- **THEN** the lesson player opens on that course
