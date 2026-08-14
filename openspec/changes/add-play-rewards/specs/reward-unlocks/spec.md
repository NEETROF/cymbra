## MODIFIED Requirements

### Requirement: Immediate and milestone reward feedback

When a user earns points for an action, the app SHALL give **immediate** feedback on that
action — a "+N" points cue where the work happened. This SHALL apply to **every** earning
action, not only rating: coverage points show their cue on the rating action, and points
earned by playing show theirs on the session summary at the end of the run. When a user
crosses a level, unlocks content, or earns a badge, the app SHALL show a **celebration**
moment for that event, whichever activity caused it.

#### Scenario: Coverage points shown immediately on rating

- **WHEN** a user rates a score and earns coverage points
- **THEN** a "+N" points cue is shown on the rating action

#### Scenario: Play points shown on the session summary

- **WHEN** a user finishes a run that earned points
- **THEN** a "+N" points cue is shown on the session summary

#### Scenario: A session that earned nothing shows no cue

- **WHEN** a user finishes a run that earned no points
- **THEN** no points cue is shown, and the session summary is otherwise unchanged

#### Scenario: Level-up / unlock / badge is celebrated

- **WHEN** the user crosses a level, unlocks a piano, or earns a badge
- **THEN** the app shows a celebration for that event

#### Scenario: A level crossed by playing is celebrated the same way

- **WHEN** a user crosses a level because of points earned by playing
- **THEN** the same celebration is shown as for a level crossed by rating
