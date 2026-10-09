## MODIFIED Requirements

### Requirement: The store listings name each studied language
Each store listing of Cymbra Lingua SHALL name every language the packages teach, in one listing per store: the Chrome Web Store, addons.mozilla.org and the App Store listing of the Safari app; each listing SHALL carry a text in each native language a shipped pair is glossed in, naming the languages the packages teach a reader of that language and never offering that reader a language they cannot study. For each language, a text SHALL say whether extended translation is offered and what it downloads, and SHALL point to the published coverage of that language's glosses in the text's language. Where a text names the levels of a language whose levels are estimated, it SHALL say that they are estimated. The App Store listing SHALL NOT call the app a beta or a trial. The listings' text SHALL be kept in the repository's listing files, in the owner's wording.

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
- **WHEN** the packages ship fr-en and not fr-es
- **THEN** the Spanish texts name English alone as what their readers study, and say that English speakers study Spanish and French

#### Scenario: Spanish speakers learning French
- **WHEN** the packages ship fr-es beside en-es
- **THEN** the Spanish texts name English and French, say that French's levels are estimated, give the download of French through English, and say that their dictionary explains fewer French words than English ones, pointing to the published figures

## ADDED Requirements

### Requirement: The stores' summary names what its readers can study
The description each package carries for a native language a shipped pair is glossed in — read by the browser from `_locales`, and by both stores as the listing's summary — SHALL name, in that language, every language the shipped pairs glossed in it study, and no other language. The extension's version check SHALL fail when a shipped native's description does not, naming that language, the languages it names and the languages its pairs study.

#### Scenario: Today's pairs
- **WHEN** the packages list en-fr and es-fr
- **THEN** the version check passes on the French description, which names English and Spanish

#### Scenario: French listed with an English summary that names Spanish alone
- **WHEN** `packs.json` lists es-en and fr-en and `_locales/en` reads "Read Spanish on the web…"
- **THEN** the version check fails for English, saying that the description names Spanish while its pairs study Spanish and French

#### Scenario: A summary that names French before French ships
- **WHEN** `packs.json` lists es-en and not fr-en and `_locales/en` names Spanish and French
- **THEN** the version check fails for English, saying that the description names French, which no shipped English-glossed pair studies
