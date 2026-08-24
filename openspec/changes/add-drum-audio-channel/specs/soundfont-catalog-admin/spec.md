## MODIFIED Requirements

### Requirement: A font records its instrument family

Every SoundFont SHALL record the **instrument family** it is for, spelled with the
**score vocabulary** — `keyboard` or `percussion` — so a font correlates to the
matching instrument scores by equality, with no per-site translation (the column's
former `piano` spelling is migrated to `keyboard`, and the upload boundary
normalises legacy `piano` input permanently so older clients keep working).
Adding a font SHALL require choosing a family, defaulting to `keyboard`, and both
families SHALL be offered now that percussion scores can sound. The declared
family SHALL be **verified against the file's preset banks** before it is
recorded — `percussion` requires at least one bank-128 preset, `keyboard` at
least one melodic-bank preset, and a mismatch is refused rather than stored — per
the verification rule in `music-drum-audio`. The instrument SHALL be set when the
font is added and is not changed by a metadata edit. (Access tiers/paid gating
are **not** part of this capability — that is deferred to a later change.)

#### Scenario: Instrument is chosen on add

- **WHEN** an admin adds a font
- **THEN** it is stored with the chosen family (`keyboard` by default), shown in
  the catalog listing

#### Scenario: A drum kit is added as percussion

- **WHEN** an admin uploads a font holding bank-128 presets and declares it
  `percussion`
- **THEN** it is verified against its banks, stored with the `percussion`
  family, and offered wherever percussion-family fonts are listed

#### Scenario: A mismatched declaration is refused

- **WHEN** an admin declares `percussion` for a font with no bank-128 preset, or
  `keyboard` for a kit-only font
- **THEN** the add is refused with a typed reason and no row or object is stored

#### Scenario: Legacy spelling is normalised at the boundary

- **WHEN** an upload arrives declaring the legacy family `piano`
- **THEN** it is recorded as `keyboard`, subject to the same verification

#### Scenario: Instrument is immutable on edit

- **WHEN** an admin edits a font's metadata
- **THEN** its instrument is unchanged (only label/licence/attribution are editable)
