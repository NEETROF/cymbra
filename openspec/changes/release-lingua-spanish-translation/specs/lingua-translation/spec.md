## ADDED Requirements

### Requirement: A language's marks are measured before they are shown
A language whose translation goes through another language SHALL have its selection marked only once its marks, measured on the committed corpus of selections with the extension's own marking, reach the programme's first tier: at least 90 % of the shown marks correct, and at most 25 % of the selections without a mark. The corpus, the rule that chose its selections, the harness and every judgment SHALL be committed, so that the measurement can be run again and checked.

#### Scenario: Measuring again
- **WHEN** the harness runs on the committed corpus with the catalogue's models
- **THEN** it produces each selection's translated sentence and mark, as the extension would, and the judged results give each language's share of correct and withheld marks

#### Scenario: Spanish below the first tier
- **WHEN** Spanish's measured marks fall short of 90 % correct or exceed 25 % withheld
- **THEN** its sentences stay translated without a mark

#### Scenario: Spanish on the first tier
- **WHEN** Spanish's measured marks reach 90 % correct with at most 25 % withheld
- **THEN** its selection is marked in the translated sentence, as English's is
