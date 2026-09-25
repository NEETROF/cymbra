## ADDED Requirements

### Requirement: A dictionary update keeps what the reader marked
When a pack merges an old lemma into a new one, a status the reader explicitly gave the old lemma SHALL carry over to the new lemma if the new lemma has no explicit status, once per pack version.
The carry-over SHALL be an ordinary status change at the time it is applied, so it synchronises to
the reader's other devices. It SHALL never overwrite an explicit status, never remove the old one,
and never move a deck card.

#### Scenario: A plural the reader had marked
- **WHEN** the reader had marked "smartphones" known, has no status for "smartphone", and a pack merging "smartphones" into "smartphone" is loaded
- **THEN** "smartphone" is known, and "smartphones" is read as known from then on

#### Scenario: The new lemma already has a status
- **WHEN** the reader had marked "smartphones" known and "smartphone" learning, and the merging pack is loaded
- **THEN** "smartphone" stays learning

#### Scenario: Loaded twice
- **WHEN** the same pack version is loaded again, on this device or after a restart
- **THEN** no status changes a second time
