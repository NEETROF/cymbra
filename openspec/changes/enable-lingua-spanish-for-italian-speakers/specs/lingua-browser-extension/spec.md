# Spec Delta

## ADDED Requirements

### Requirement: Spanish is studied by readers of Italian
The extension SHALL offer Spanish, in its settings and at onboarding, to a reader whose native language is Italian once a listed pair studies Spanish glossed in Italian, after English; a Spanish document SHALL be read, highlighted, glossed, recorded and read aloud for them through es-it; and every surface that names a studied language, asks or shows its level, or says that its levels are estimated SHALL say so of Spanish in the Italian interface. A Spanish word on the card SHALL be marked as Spanish inside the Italian lines, and the card SHALL name Spanish's forms in Italian.

#### Scenario: An Italian speaker adds Spanish
- **WHEN** es-it is listed and a reader whose native language is Italian, studying English, ticks Spanish in the settings
- **THEN** the profile holds English then Spanish, a Spanish page is highlighted through es-it, and the Spanish level block says, in the Italian interface, that Spanish's level is estimated

#### Scenario: A stored profile naming Spanish, Italian native
- **WHEN** the stored backup's profile studies English then Spanish with Italian as the native language
- **THEN** every surface that reads the profile without an engine reads English then Spanish, and the backup is written in schema version 4

#### Scenario: A Spanish word on an Italian card
- **WHEN** a reader whose native language is Italian opens the card of a Spanish verb in the pretérito indefinido
- **THEN** the card's lines are Italian, its tense is named « passato remoto », and the Spanish words in it are marked as Spanish

## MODIFIED Requirements

### Requirement: The store listings name each studied language
Each store listing of Cymbra Lingua SHALL name every language the packages teach, in one listing per store: the Chrome Web Store, addons.mozilla.org and the App Store listing of the Safari app; each listing SHALL carry a text in each native language a shipped pair is glossed in, naming the languages the packages teach a reader of that language and never offering that reader a language they cannot study. For each language, a text SHALL say whether extended translation is offered and what it downloads, and SHALL point to the published coverage of that language's glosses in the text's language. Where a text names the levels of a language whose levels are estimated, it SHALL say that they are estimated. The App Store listing SHALL NOT call the app a beta or a trial. The listings' text SHALL be kept in the repository's listing files, in the owner's wording; an Italian text in wording the Italian reviewer agent has reviewed.

#### Scenario: Spanish ships
- **WHEN** the packages ship en-fr and es-fr
- **THEN** each store's single listing names English and Spanish

#### Scenario: Translation offered for English only
- **WHEN** extended translation is offered for English and not yet for Spanish
- **THEN** the listings give the English model's download size and say that Spanish translation comes later

#### Scenario: The App Store listing
- **WHEN** the App Store listing is prepared
- **THEN** it calls the app neither a beta nor a trial, and says nothing of price

#### Scenario: A listing in English
- **WHEN** the packages ship es-en for English speakers learning Spanish
- **THEN** each store's listing carries its English texts, written for them, beside the French

#### Scenario: A listing in Spanish
- **WHEN** the packages ship en-es for Spanish speakers learning English
- **THEN** each store's listing carries its Spanish texts, written for them, beside the French and the English

#### Scenario: English speakers learning French
- **WHEN** the packages ship fr-en beside es-en
- **THEN** the English texts name Spanish and French, say that French's levels are estimated, and give the French model's download size, and no French text offers French

#### Scenario: fr-es held below its floor
- **WHEN** the packages ship fr-en and neither fr-es nor it-es
- **THEN** the Spanish texts name English alone as what their readers study, and say that English speakers study Spanish and French

#### Scenario: Spanish speakers learning French
- **WHEN** the packages ship fr-es beside en-es, and not it-es
- **THEN** the Spanish texts name English and French, say that French's levels are estimated, give the download of French through English, and say that their dictionary explains fewer French words than English ones, pointing to the published figures

#### Scenario: French speakers learning Italian
- **WHEN** it-fr is listed
- **THEN** the French texts name English, Spanish and Italian, say that Italian's levels are estimated wherever they name levels, say whether Italian is translated — through English — and, if so, what it downloads, and point to the published coverage; the English and Spanish texts say that French speakers study it; no Spanish text offers Italian to its readers while it-es is not listed, nor any English text while it-en is not listed

#### Scenario: English speakers learning Italian
- **WHEN** it-en is listed beside it-fr
- **THEN** the English texts name Spanish, French and Italian, say that Italian's levels are estimated wherever they name levels, say whether Italian is translated — straight into English — and, if so, what it downloads, and point to the published coverage; the French and Spanish texts that say what English speakers study name Italian; and no Spanish text offers Italian to its readers while it-es is not listed

#### Scenario: A listing in Italian
- **WHEN** the packages ship en-it for Italian speakers learning English, and neither fr-it nor es-it
- **THEN** each store's listing carries its Italian texts, written for them, beside the French, the English and the Spanish; they name English alone as what their readers study, and the App Store carries them as its Italian localisation

#### Scenario: Spanish speakers learning Italian
- **WHEN** it-es is listed beside en-es and fr-es
- **THEN** the Spanish texts name English, French and Italian, say that Italian's levels are estimated wherever they name levels, say whether Italian is translated — through English — and, if so, what it downloads, say that their dictionary explains fewer Italian words than English ones where it-es's committed coverage is under en-es's at each top, pointing to the published figures; and the French, English and Italian texts that say what Spanish speakers study name Italian

#### Scenario: Italian speakers learning French
- **WHEN** fr-it is listed beside en-it, and es-it is not
- **THEN** the Italian texts name English and French, say that French's levels are estimated wherever they name levels, say whether French is translated — through English — and, if so, what it downloads, say that their dictionary explains fewer French words than English ones where fr-it's committed coverage is under en-it's at each top, pointing to the published figures; and the English texts that say what Italian speakers study name French

#### Scenario: Italian speakers learning Spanish
- **WHEN** es-it is listed beside en-it
- **THEN** the Italian texts name English and Spanish, and French where fr-it is listed, say that Spanish's levels are estimated wherever they name levels, say whether Spanish is translated — through English — and, if so, what it downloads, say that their dictionary explains fewer Spanish words than English ones where es-it's committed coverage is under en-it's at each top, pointing to the published figures; and the English texts that say what Italian speakers study name Spanish
