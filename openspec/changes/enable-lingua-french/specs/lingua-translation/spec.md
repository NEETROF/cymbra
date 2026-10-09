## ADDED Requirements

### Requirement: The setting's memory follows the models the reader's pairs need
The translation setting SHALL state the memory of two models when the reader's pairs need two models together — one route through English, or two direct routes — and the memory of one model when they need one, as « environ 340 Mo » and « environ 200 Mo » say it in the French interface. It SHALL count the models the reader's pairs need, never whether a route goes through another language.

#### Scenario: An English speaker reading Spanish and French
- **WHEN** a reader whose native language is English accepts Spanish and French, and opens « Traduction étendue » before ticking it
- **THEN** it says, in the English interface, that it downloads 52.5 MB once and uses about 340 MB of memory while translating

#### Scenario: An English speaker reading French alone
- **WHEN** a reader whose native language is English accepts French alone
- **THEN** the setting says, in the English interface, that it downloads 26.2 MB once and uses about 200 MB

#### Scenario: A Spanish speaker reading French
- **WHEN** fr-es is listed and a reader whose native language is Spanish accepts French
- **THEN** the setting says, in the Spanish interface, that it downloads 51,6 MB once and uses unos 340 MB, the two models fr-es's route goes through

#### Scenario: A French speaker as before
- **WHEN** a reader whose native language is French opens the setting with English, Spanish or both accepted
- **THEN** it says what it said before: 25,8 Mo and « environ 200 Mo » for English alone, 52,0 Mo and « environ 340 Mo » with Spanish
