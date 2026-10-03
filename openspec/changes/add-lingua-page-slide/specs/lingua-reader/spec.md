## MODIFIED Requirements

### Requirement: A page turn paints once

The reader page SHALL reveal a section only once it has been analysed and its highlights painted, bounded by a cap after which the section is revealed regardless, and SHALL paint a section whole rather than by viewport window, so that turning a page inside a section paints nothing new. The flow SHALL be paginated by default with tap zones to turn and no transition; a scrolled flow SHALL be available as a setting. A sliding page turn SHALL be available as a setting, off by default, offered with the text size and the theme, and SHALL NOT slide when the system asks for reduced motion.

#### Scenario: Turning a page within a section

- **WHEN** the reader turns to the next page of the current section
- **THEN** the page appears with its highlights already painted, in one paint

#### Scenario: Moving to the next section

- **WHEN** the reader moves to the next section
- **THEN** it appears with its highlights painted, or after the cap without them if the analysis has not answered, and never twice

#### Scenario: Scrolled flow chosen

- **WHEN** the reader chooses the scrolled flow in the settings
- **THEN** the book scrolls continuously and highlights still paint per section

#### Scenario: Sliding page turn chosen

- **WHEN** the reader chooses the sliding page turn and turns to the next page of the current section
- **THEN** the page slides aside to the next one, its highlights already painted, and the choice holds for every book

#### Scenario: Reduced motion

- **WHEN** the sliding page turn is chosen on a system that asks for reduced motion
- **THEN** the page turns in one jump
