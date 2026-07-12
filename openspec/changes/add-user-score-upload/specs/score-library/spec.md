## ADDED Requirements

### Requirement: User's Contributed Scores In Library

When a user is signed in, the library SHALL additionally present the scores that
user has contributed, fetched from the backend, as a section distinct from the
bundled catalog. Each contributed entry SHALL show its title and difficulty
level, and SHALL be selectable to open in the player like a catalog entry. When no
user is signed in, no contributed-scores section is shown. The bundled-catalog
behavior is unchanged.

#### Scenario: Contributed scores listed when signed in

- **WHEN** a signed-in user opens the library and has contributed scores
- **THEN** the library shows those scores in a section distinct from the bundled
  catalog, each with its title and difficulty level

#### Scenario: No contributed section when signed out

- **WHEN** no user is signed in
- **THEN** the library shows only the bundled catalog and no contributed-scores
  section

#### Scenario: Contributed scores isolated per user

- **WHEN** a signed-in user's contributed scores are listed
- **THEN** only that user's contributions appear

### Requirement: Open A Contributed Score In The Player

Selecting a contributed score in the library SHALL record it as the selected
score and navigate to the player, loading the score's bytes from the backend
(object store) rather than the asset bundle, and displaying it through the same
notation and playback path as bundled scores.

#### Scenario: Selecting a contributed score opens the player

- **WHEN** the user selects a contributed score
- **THEN** it becomes the selected score and the player opens, loading and
  rendering the score from its backend-provided bytes

### Requirement: Delete A Contributed Score From The Library

The library SHALL offer a delete action only for scores the signed-in user owns
(their contributed scores), and MUST NOT offer deletion for bundled-catalog
scores. Confirming deletion SHALL request removal from the backend and, on
success, remove the entry from the contributed section.

#### Scenario: Owner deletes a contributed score

- **WHEN** the owner triggers delete on one of their contributed scores and
  confirms
- **THEN** the app requests deletion from the backend and, on success, removes the
  entry from the library

#### Scenario: Bundled scores have no delete action

- **WHEN** a bundled-catalog entry is shown
- **THEN** no delete action is offered for it
