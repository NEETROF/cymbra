# Spec Delta

## MODIFIED Requirements

### Requirement: The setting's memory follows the models the reader's pairs need
The translation setting SHALL state the memory of two models when the reader's pairs need two models or more — one route through English, two direct routes, or three models the engine holds two at a time — and the memory of one model when they need one, as « environ 340 Mo » and « environ 200 Mo » say it in the French interface. It SHALL count the models the reader's pairs need, never whether a route goes through another language.

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

#### Scenario: A French speaker reading Italian
- **WHEN** it-fr is listed and a reader whose native language is French accepts Italian alone
- **THEN** the setting says, in the French interface, that it downloads it-en's and en-fr's files once, their sum read from the catalogue, and uses « environ 340 Mo », the two models it-fr's route goes through

#### Scenario: A French speaker reading English, Spanish and Italian
- **WHEN** it-fr is listed and a reader whose native language is French accepts English, Spanish and Italian
- **THEN** the setting says, in the French interface, that it downloads en-fr's, es-en's and it-en's files once, their sum read from the catalogue, and uses « environ 340 Mo », the engine holding two of the three at a time

#### Scenario: An English speaker reading Italian alone
- **WHEN** it-en is listed and a reader whose native language is English accepts Italian alone
- **THEN** the setting says, in the English interface, that it downloads it-en's files once, their size read from the catalogue, and uses "about 200 MB"

#### Scenario: An English speaker reading Spanish, French and Italian
- **WHEN** it-en is listed and a reader whose native language is English accepts Spanish, French and Italian
- **THEN** the setting says, in the English interface, that it downloads es-en's, fr-en's and it-en's files once, their sum read from the catalogue, and uses "about 340 MB", the engine holding two of the three at a time

#### Scenario: An Italian speaker reading English
- **WHEN** en-it is listed, extended translation serves it, and a reader whose native language is Italian accepts English alone
- **THEN** the setting says, in the Italian interface, that it downloads en-it's files once, their size read from the catalogue, and uses about 200 MB, as the Italian interface writes it

#### Scenario: A Spanish speaker reading Italian alone
- **WHEN** it-es is listed, extended translation serves it, and a reader whose native language is Spanish accepts Italian alone
- **THEN** the setting says, in the Spanish interface, that it downloads it-en's and en-es's files once, their sum read from the catalogue, and uses unos 340 MB, the two models it-es's route goes through

#### Scenario: A Spanish speaker reading English, French and Italian
- **WHEN** fr-es and it-es are listed, extended translation serves both, and a reader whose native language is Spanish accepts English, French and Italian
- **THEN** the setting says, in the Spanish interface, that it downloads en-es's, fr-en's and it-en's files once, their sum read from the catalogue, and uses unos 340 MB, the engine holding two of the three at a time

#### Scenario: An Italian speaker reading French alone
- **WHEN** fr-it is listed, extended translation serves it, and a reader whose native language is Italian accepts French alone
- **THEN** the setting says, in the Italian interface, that it downloads fr-en's and en-it's files once, their sum read from the catalogue, and uses about 340 MB, as the Italian interface writes it, the two models fr-it's route goes through

#### Scenario: An Italian speaker reading English and French
- **WHEN** fr-it is listed, extended translation serves en-it and fr-it, and a reader whose native language is Italian accepts English and French
- **THEN** the setting says, in the Italian interface, that it downloads en-it's and fr-en's files once, their sum read from the catalogue, and uses about 340 MB, as the Italian interface writes it, en-it serving both routes
