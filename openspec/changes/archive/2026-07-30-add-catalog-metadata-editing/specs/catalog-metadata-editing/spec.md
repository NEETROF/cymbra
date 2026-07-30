## ADDED Requirements

### Requirement: Moderators and admins may edit curatorial metadata

The backend SHALL expose an authenticated operation, restricted to `moderator`/`admin`
identities, that edits a catalog score's **curatorial** fields — title, composer, arranger,
and difficulty level. A non-`moderator`/non-`admin` caller MUST be rejected with
`PERMISSION_DENIED`. Editing a non-existent score MUST be rejected. The difficulty level
MUST be validated against the allowed set; an invalid value MUST be rejected without any
change.

#### Scenario: Moderator corrects the composer

- **WHEN** a moderator sets a catalog score's composer to a corrected value
- **THEN** the score's stored composer becomes that value and the change is recorded

#### Scenario: Non-moderator cannot edit

- **WHEN** a caller without `moderator`/`admin` invokes the edit operation
- **THEN** it is rejected with `PERMISSION_DENIED` and no field changes

#### Scenario: Invalid level is rejected

- **WHEN** a moderator submits a difficulty level outside the allowed set
- **THEN** the edit is rejected and no field changes

### Requirement: Only curatorial fields are editable; derived facts are read-only

The edit operation SHALL accept **only** the curatorial fields (title, composer, arranger,
level). The fields **derived from the score's MusicXML** — time signature, note count,
measure count, staff count, tempo, and the musical facets (piano/chords/tuplets, etc.) —
MUST NOT be editable through this operation and remain authoritative from the score bytes.

#### Scenario: Derived fields cannot be changed

- **WHEN** a moderator edits a score
- **THEN** only the curatorial fields can change; the MusicXML-derived facts are unchanged
  and continue to reflect the score's content

### Requirement: An edit keeps the search keys consistent

When an edit changes the title or composer, the operation SHALL recompute the derived
normalized search keys — the normalized title, the normalized composer, and the same-work
grouping key — using the **same normalization the ingest pipeline uses**, in the same
update, so full-text/trigram search and grouping reflect the edited values. After an edit,
the score MUST be findable by its **new** title/composer and MUST NOT be found only by its
old value.

#### Scenario: Edited title is searchable by its new value

- **WHEN** a moderator corrects a score's title and then a search runs for the corrected
  title
- **THEN** the score is returned (its normalized search key was recomputed)

#### Scenario: Old value no longer matches after a rename

- **WHEN** a title is corrected from a misspelling and a search runs for the old misspelling
- **THEN** the score is no longer matched by the stale value

### Requirement: Every metadata edit is audited

Each metadata edit SHALL be recorded in a durable, append-only audit trail capturing at
least the target score, the editing identity, which field changed, its previous and new
values, and when. The trail SHALL be queryable to answer "who changed which field of which
score, and when", independent of the score's current state. The audit write SHALL be atomic
with the edit (an edit that is not recorded MUST NOT take effect).

#### Scenario: An edit records an audit entry per changed field

- **WHEN** a moderator changes a score's title and level in one edit
- **THEN** the audit trail gains an entry for the title change and one for the level change,
  each with the editor, the old and new values, and the time

#### Scenario: Audit is queryable

- **WHEN** an administrator reviews the edit history of a score
- **THEN** the audit trail answers who changed what and when, independent of the row's
  current values

### Requirement: Edited scores carry an edit provenance marker

A catalog score that has been manually edited SHALL carry a provenance marker recording
that it was hand-edited (the editing identity and time). Any future automated
metadata-refresh path SHALL skip rows that carry the marker, so manual corrections are never
silently overwritten by an ingest/refresh.

#### Scenario: A manual edit marks the row

- **WHEN** a moderator edits a score's metadata
- **THEN** the row is marked as manually edited, with the editor and time

#### Scenario: Refresh preserves manual edits

- **WHEN** an automated metadata refresh runs over a score that carries the manual-edit
  marker
- **THEN** it does not overwrite that score's curated metadata

### Requirement: Editing is limited to the crawled corpus in v1

The edit operation SHALL apply to crawled public-corpus catalog scores. A user-uploaded
score (one owned by a user account) MUST NOT be editable through this operation in v1; such
a request MUST be refused.

#### Scenario: Corpus score is editable

- **WHEN** a moderator edits a crawled corpus score
- **THEN** the edit is allowed

#### Scenario: User-uploaded score is refused

- **WHEN** a moderator attempts to edit a user-uploaded (owned) score
- **THEN** the request is refused (out of scope for v1)

### Requirement: Console edit form gated to moderators and admins

The moderation console SHALL provide an edit form for a catalog score's curatorial fields,
available only to `moderator`/`admin` identities, that submits through the edit operation.
The derived MusicXML facts SHALL be shown **read-only** in the form. On a successful edit
the view SHALL reflect the new values. A signed-in non-moderator MUST NOT see or be able to
use the form.

#### Scenario: Moderator edits from the console

- **WHEN** a moderator opens a score and edits its title/composer/arranger/level in the form
- **THEN** the change is submitted and, on success, the view shows the updated values

#### Scenario: Derived facts are not editable in the form

- **WHEN** a moderator uses the edit form
- **THEN** the time signature, counts, tempo and facets are shown read-only and cannot be
  changed

#### Scenario: Non-moderator has no edit affordance

- **WHEN** a signed-in user without `moderator`/`admin` views a score
- **THEN** no edit form is available and the edit operation would reject their call
