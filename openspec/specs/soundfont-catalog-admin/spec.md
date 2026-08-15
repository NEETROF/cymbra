# soundfont-catalog-admin Specification

## Purpose
TBD - created by archiving change add-soundfont-back-office-management. Update Purpose after archive.
## Requirements
### Requirement: Administered SoundFont catalog

The system SHALL let a **music-scope moderator/admin** administer the SoundFont
catalog from the back office: list every catalog font (with its id, label, tier,
licence, attribution, and whether its object is stored), add a font, edit a font's
metadata, and remove a font. Every administrative operation SHALL require music-scope
moderator/admin authorization enforced by the server; a caller lacking it MUST be
refused and no change made. The catalog administered here is the same server-owned
catalog the delivery route and the app's listing read, so an administered change is
reflected by both without a separate publish step.

#### Scenario: Admin lists the catalog

- **WHEN** a music-scope admin opens the SoundFont management screen
- **THEN** the backend returns every catalog font with its metadata for administration

#### Scenario: Non-admin is refused

- **WHEN** a caller without music-scope moderator/admin attempts any catalog
  administration (list, add, edit, or remove)
- **THEN** the server refuses the operation and the catalog is unchanged

#### Scenario: Administered change is reflected everywhere

- **WHEN** an admin adds, edits, or removes a font
- **THEN** the app's font listing and the delivery route reflect the change with no
  additional deployment or publish

### Requirement: Adding a font stores its bytes and metadata together

Adding a font SHALL take both the `.sf2` bytes and its metadata (label, licence,
attribution, tier) and result in **both** a stored object and a catalog row, so the
catalog never lists a font whose bytes are missing. The system SHALL validate that
the uploaded file is a real SoundFont before accepting it and MUST reject an invalid
file without creating a row or storing an object. The stored object's key SHALL be
derived by the server (clients do not choose storage keys), and adding a font whose
id already exists MUST be refused rather than silently overwriting.

#### Scenario: A valid font is added

- **WHEN** an admin uploads a valid `.sf2` with its metadata
- **THEN** the bytes are stored in the private SoundFont store and a catalog row is
  created, and the font becomes listable and downloadable

#### Scenario: An invalid file is rejected

- **WHEN** an admin uploads a file that is not a valid SoundFont
- **THEN** the server rejects it and no catalog row is created and no object is stored

#### Scenario: Bytes are stored before the row is recorded

- **WHEN** adding a font
- **THEN** the catalog row is recorded only after its object is fully stored, so a
  failure never leaves a listed font without bytes (at worst an inert unreferenced
  object)

#### Scenario: Duplicate id is refused

- **WHEN** an admin adds a font whose id already exists in the catalog
- **THEN** the add is refused and the existing font is left unchanged

### Requirement: Removing a font deletes its row and object

Removing a font SHALL delete both its catalog row and its stored object, leaving no
orphaned row and no referenced-but-absent bytes. The row SHALL be removed even if
object deletion fails, so the font immediately stops being offered; a leftover object
is a non-fatal, sweepable remnant.

#### Scenario: Remove deletes both

- **WHEN** an admin removes a font
- **THEN** its catalog row and its stored object are both deleted and it is no longer
  listed or downloadable

#### Scenario: Row is removed even if object deletion fails

- **WHEN** an admin removes a font and deleting the stored object fails
- **THEN** the catalog row is still removed so the font is no longer offered, and the
  failure is logged rather than blocking removal

### Requirement: A font records its instrument family

Every SoundFont SHALL record the **instrument family** it is for (e.g. piano), so a
font can be correlated to the matching instrument scores. Adding a font SHALL require
choosing an instrument, defaulting to piano; for now piano is the only choice. The
instrument SHALL be set when the font is added and is not changed by a metadata edit.
(Access tiers/paid gating are **not** part of this capability — that is deferred to a
later change.)

#### Scenario: Instrument is chosen on add

- **WHEN** an admin adds a font
- **THEN** it is stored with the chosen instrument (piano by default), shown in the
  catalog listing

#### Scenario: Instrument is immutable on edit

- **WHEN** an admin edits a font's metadata
- **THEN** its instrument is unchanged (only label/licence/attribution are editable)

### Requirement: SoundFont management screen in the back office

The back office SHALL present the SoundFont catalog on a dedicated screen, reachable
from the navigation only by music-scope admins, showing the catalog as a table.
Creating and editing a font SHALL happen in a drawer opened from the screen: the
create drawer offers a `.sf2` file picker plus metadata fields, and the edit drawer
offers the metadata fields for an existing font. A remove affordance SHALL be
available per row. Before saving, the drawer SHALL let the moderator **audition** the
candidate font — the picked file when creating, or the stored font when editing — by
playing a **catalog piece of their choice** with that font. The screen SHALL reflect
the outcome of each operation — including authorization, validation, and preview
failures — as user-facing state, never a raw backend error string.

#### Scenario: Admin sees and manages the catalog

- **WHEN** a music-scope admin navigates to the SoundFont management screen
- **THEN** the current catalog is listed and add/edit/remove controls are available

#### Scenario: Non-admin cannot reach the screen

- **WHEN** a signed-in user who is not a music-scope admin tries to open the screen
- **THEN** it is not offered in navigation and access is denied

#### Scenario: Failures are shown as state

- **WHEN** an add or edit fails (invalid file, unauthorized, or a backend error)
- **THEN** the screen shows a user-facing message and the table stays consistent,
  without surfacing a raw status/exception string

#### Scenario: Create and edit happen in a drawer

- **WHEN** the admin chooses to add a font, or to edit a row
- **THEN** a drawer opens with the font's fields (a file picker on create; metadata on
  edit) and the save action lives in that drawer

#### Scenario: Auditioning a candidate font before saving

- **WHEN** the admin, in the create or edit drawer, selects a catalog piece and plays
  the preview
- **THEN** that piece is played rendered with the candidate font (the picked file on
  create, or the stored font on edit) so the sound can be judged before saving

#### Scenario: Preview failure does not block editing

- **WHEN** the preview cannot be produced (no audio, a missing stored font, or an
  unloadable piece)
- **THEN** a non-fatal message is shown and the create/edit/save controls remain usable

