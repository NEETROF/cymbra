## ADDED Requirements

### Requirement: Download a catalog score's MusicXML from the back office

The back-office catalog table SHALL provide, per score row, a control that downloads the
score's canonical MusicXML file to the operator's local machine. Activating the control
SHALL fetch the score's decoded MusicXML bytes via the existing byte-serving operation and
save them as a file whose name is derived from the score (its title when available, else
its identifier) with a `.musicxml` extension. The download SHALL NOT require navigating to
the score detail view.

#### Scenario: Moderator downloads a score's MusicXML from the catalog table

- **WHEN** a `music` moderator/admin activates the download control on a catalog row
- **THEN** the score's canonical MusicXML bytes are fetched and the browser saves a
  `<title-or-id>.musicxml` file to the operator's machine

#### Scenario: File name falls back to the identifier

- **WHEN** the score has no usable title
- **THEN** the downloaded file is named from the score's identifier with a `.musicxml`
  extension

#### Scenario: Downloaded bytes are the decoded MusicXML

- **WHEN** the stored object is a compressed `.mxl`
- **THEN** the served bytes are the decompressed canonical MusicXML and the file is saved
  with the `.musicxml` extension (not `.mxl`)

### Requirement: Download is authorized to back-office moderators/admins only

The download SHALL be served only to an authenticated back-office operator holding a
`music` `moderator` or `admin` role (or `global/admin`) — the same authorization that
already permits fetching the bytes of a score in any moderation status. The download
control SHALL be rendered only for such operators, and the byte-serving path SHALL reject
an unauthorized caller. This provenance check reuses the existing byte-serving guard; no
new unauthenticated or public download path is introduced.

#### Scenario: Moderator may download any-status score

- **WHEN** a `music` moderator/admin downloads a `pending`, `rejected`, or `accepted`
  score
- **THEN** the bytes are served and the file is saved

#### Scenario: Download control hidden from non-moderators

- **WHEN** an operator without `moderator`/`admin` views the catalog table
- **THEN** the download control is not rendered

#### Scenario: Unauthorized byte request refused

- **WHEN** a caller without `moderator`/`admin` requests a non-`accepted` score's bytes
- **THEN** the request is refused (permission denied / not found), as for the existing
  byte-serving operation

### Requirement: Per-row download feedback is localized and non-blocking

Each row's download SHALL surface its own loading and error state without blocking the
rest of the table or the catalog browse experience. On failure — including a score whose
underlying object is not yet available — the operator SHALL see a localized error message;
a raw gRPC/exception string SHALL NOT be shown. A download in progress on one row SHALL
NOT prevent viewing, sorting, or downloading other rows.

#### Scenario: Download in progress shows per-row loading

- **WHEN** the operator activates the download control on a row
- **THEN** that row indicates the download is in progress while other rows remain
  interactive

#### Scenario: Missing object reports a localized error

- **WHEN** the score's underlying MusicXML object is not available
- **THEN** a localized error message is shown and no file is saved, with the raw
  technical error only logged, not displayed

#### Scenario: One row's failure does not break the table

- **WHEN** a download fails on one row
- **THEN** the rest of the catalog table stays usable and other rows can still be
  downloaded
