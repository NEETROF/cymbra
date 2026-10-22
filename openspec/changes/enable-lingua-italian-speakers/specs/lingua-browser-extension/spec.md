## ADDED Requirements

### Requirement: English is studied by readers of Italian
Once a listed pair studies English glossed in Italian, the extension SHALL offer « Italiano » in the native-language choice and preset it for a browser in Italian, SHALL show its interface in Italian to a reader whose native language is Italian, and SHALL offer them English as their default studied language. An English document SHALL be read, highlighted, glossed in Italian, recorded and read aloud for them through en-it; every surface that names a studied language, asks or shows its level, or links a Cymbra page SHALL do so in Italian, the links opening the Italian privacy policy, leaving page and account deletion page. An English word on the card SHALL be marked as English inside the Italian lines, and the card SHALL name English's forms in Italian.

#### Scenario: An Italian speaker installs Cymbra Lingua
- **WHEN** a reader whose browser is in Italian installs a package that lists en-it
- **THEN** the native-language choice is offered, preset to « Italiano », the interface is Italian, and English is their studied language

#### Scenario: An English word on an Italian card
- **WHEN** a reader whose native language is Italian opens the card of « went »
- **THEN** the card's lines are Italian, it reads « passato semplice di go », and the English words in it are marked as English

#### Scenario: The Italian pages
- **WHEN** a reader whose interface is Italian opens the privacy policy from Settings, or removes the extension while counting is on
- **THEN** `https://cymbra.app/it/privacy/` opens, or the browser opens `https://cymbra.app/it/lingua/arrivederci/`

#### Scenario: Nothing moves for the other readers
- **WHEN** a reader whose native language is French, English or Spanish updates to a package that lists en-it
- **THEN** their pairs, studied languages, interface and links are what they were, but a Spanish reader's account deletion link, which now opens `https://cymbra.app/es/eliminar-cuenta/`
