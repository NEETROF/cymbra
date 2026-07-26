## MODIFIED Requirements

### Requirement: Output verification

The system SHALL verify each produced `.mxl` by re-opening it and confirming it
parses as MusicXML; a `.mxl` that fails re-parsing SHALL NOT be counted as a
successful conversion.

Verification SHALL additionally require that the produced score contains at least
one **playable note** (a pitched, non-rest note); a `.mxl` that parses but yields
zero playable notes SHALL be rejected (`RejectReason::NoNotes`) and SHALL NOT be
counted as a successful conversion nor ingested into the catalog. This gate SHALL
apply to **every** acceptance path — native MusicXML conversion AND the pass-through
acceptance of already-`.mxl`/MuseScore-origin inputs — so no path admits a
noteless score.

#### Scenario: Produced .mxl re-parses
- **WHEN** an `.mxl` is generated
- **THEN** the tool re-opens and parses it, and only marks the item converted if
  parsing succeeds

#### Scenario: Noteless score is rejected
- **WHEN** a produced or accepted `.mxl` parses successfully but contains no
  pitched, non-rest note
- **THEN** it is rejected as `NoNotes`, not counted as converted, and not ingested

#### Scenario: Gate applies to the .mxl/MuseScore acceptance path
- **WHEN** an already-`.mxl` or MuseScore-origin input is accepted by re-parse
  verification
- **THEN** the same playable-note gate is enforced, so a noteless `.mxl`/MuseScore
  input is rejected rather than admitted to the catalog
