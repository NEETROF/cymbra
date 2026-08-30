# music-batch-score-import — spec (add-private-score-catalog)

## ADDED Requirements

### Requirement: Multi-file selection in the contribution flow

The app's score contribution flow SHALL let a signed-in user select **multiple**
MusicXML files (plain or zipped, same accepted formats as single upload) in one
picker pass. Selecting a single file MUST keep today's single-file wizard
behavior; selecting more than one file MUST enter the batch import flow.

#### Scenario: Multiple files enter the batch flow

- **WHEN** an authenticated user picks several files in the contribution flow
- **THEN** the batch import flow opens with the selected files listed

#### Scenario: Single file keeps the existing wizard

- **WHEN** the user picks exactly one file
- **THEN** the existing single-file wizard runs unchanged

### Requirement: One attestation and one difficulty govern the batch

Before any upload starts, the batch flow SHALL collect exactly one rights
attestation (basis + confirmation checkbox, same options and rules as the
single-file attestation, including `private_use`) and one difficulty
(Beginner / Intermediate / Advanced), both applying to **every** file in the
batch and recorded per uploaded score. Starting the batch MUST be impossible
until both are provided.

#### Scenario: Batch blocked without attestation and difficulty

- **WHEN** the rights attestation is incomplete or no difficulty is selected
- **THEN** the batch upload cannot start

#### Scenario: Choices recorded on every file

- **WHEN** a batch completes with basis B and difficulty D
- **THEN** every successfully imported score persists basis B, an affirmative
  confirmation, and difficulty D

### Requirement: Sequential upload with isolated per-file outcomes

The batch flow SHALL upload the files one at a time through the same backend
upload operation as the single-file flow, and SHALL report a per-file outcome:
**imported**, **duplicate** (per-owner content dedup), **invalid** (failed
validation), or **quota exceeded**. A failing file MUST NOT abort the batch —
remaining files are still attempted — and outcomes MUST be visible in a result
board when the batch ends, with localized, non-technical failure descriptions.

#### Scenario: Failures do not abort the batch

- **WHEN** a file in the batch fails validation or is a duplicate
- **THEN** that file is marked with its outcome and the remaining files are
  still uploaded

#### Scenario: Result board summarises the batch

- **WHEN** the batch finishes
- **THEN** the user sees each file with its outcome (imported / duplicate /
  invalid / quota exceeded) in localized wording

### Requirement: Quota pre-check before starting the batch

Before starting uploads, the batch flow SHALL compare the selection size against
the caller's remaining upload quota and, when the selection exceeds it, SHALL
warn the user that only part of the selection can be imported, before any upload
starts. The user MAY proceed; files beyond the quota then surface as
quota-exceeded outcomes.

#### Scenario: Oversized selection warns upfront

- **WHEN** the user confirms a batch larger than their remaining quota
- **THEN** a warning states not all files can be imported before any upload runs
