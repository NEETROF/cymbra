## ADDED Requirements

### Requirement: Run artefacts are written to the crawler work location

The derived manifest export and the rejection log SHALL be written to the crawler's work
location, never to the served corpus root: they are audit artefacts of a crawl run, not
servable corpus objects, so the corpus root holds only objects the application serves and the
off-box mirror never carries them.

#### Scenario: Manifest export lands outside the corpus

- **WHEN** a crawl run exports the manifest
- **THEN** `manifest.csv` and `manifest.json` are written under the crawler work location and
  the corpus root gains neither

#### Scenario: Rejection log lands outside the corpus

- **WHEN** a crawl run records excluded items
- **THEN** `rejected.log` is written under the crawler work location and the corpus root gains
  no such file

#### Scenario: Artefacts stay auditable after relocation

- **WHEN** an operator inspects a completed run's exclusions and exported manifest
- **THEN** both are found at the work location, with the content the existing manifest and
  rejection-log requirements define
