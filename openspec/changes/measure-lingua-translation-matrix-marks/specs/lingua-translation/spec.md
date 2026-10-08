## MODIFIED Requirements

### Requirement: A pair's marks are measured before they are shown
A pair SHALL have its selection marked only once its marks, measured on the committed corpus of its studied language's selections with the extension's own marking through that pair's route, reach the programme's first tier: at least 90 % of the shown marks correct, and at most 25 % of the selections without a mark. A mark SHALL be judged against the rendering, in the pair's native language, of the selected words. A selection whose sentence the engine traps on twice, after being started again as the extension would, SHALL count as a selection without a mark; one whose fragment alone traps twice SHALL keep the sentence's own marks, unreconciled, as the extension shows them. The pairs that reached it SHALL be listed by name in the extension, and a pair outside the list SHALL be translated without a mark. The corpus, the rule that chose its selections, the harness and every judgment SHALL be committed, filed by pair, so that the measurement can be run again and checked.

#### Scenario: Measuring again
- **WHEN** the harness runs for a pair on the committed corpus of its studied language with the catalogue's models
- **THEN** it produces each selection's translated sentence and mark, as the extension would through that pair's route, and the judged results give that pair's share of correct and withheld marks

#### Scenario: The shipped pairs today
- **WHEN** a reader whose native language is French selects words on an English or a Spanish page
- **THEN** the selection is marked in the translated sentence, as en-fr's and es-fr's measurements allow

#### Scenario: Spanish below the first tier
- **WHEN** es-fr's measured marks fall short of 90 % correct or exceed 25 % withheld
- **THEN** its sentences stay translated without a mark

#### Scenario: Spanish on the first tier
- **WHEN** es-fr's measured marks reach 90 % correct with at most 25 % withheld
- **THEN** its selection is marked in the translated sentence, as en-fr's is

#### Scenario: A pair measured in another native language
- **WHEN** es-en is listed in the catalogue and its marks are measured on the Spanish selections, judged against the English rendering
- **THEN** es-en is marked only if its own figures reach the first tier, whatever es-fr's are

#### Scenario: An English-native reader of Spanish
- **WHEN** es-en ships and its measured marks reach the first tier
- **THEN** the reader's Spanish selection is marked in the English sentence

#### Scenario: A selection the engine traps on
- **WHEN** the engine traps twice on a selection's sentence while en-es is measured
- **THEN** the selection is recorded as trapped and counted without a mark, and the measurement goes on
