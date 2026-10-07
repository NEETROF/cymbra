## ADDED Requirements

### Requirement: The store listings name each studied language
Each store listing of Cymbra Lingua SHALL name every language the packages teach, in one listing per store: the Chrome Web Store, addons.mozilla.org and the App Store listing of the Safari app. For each language, a listing SHALL say whether extended translation is offered and what it downloads, and SHALL point to the published coverage of that language's French glosses. The App Store listing SHALL NOT call the app a beta or a trial. The listings' text SHALL be kept in the repository's listing files, in the owner's wording.

#### Scenario: Spanish ships
- **WHEN** the packages ship en-fr and es-fr
- **THEN** each store's single listing names English and Spanish

#### Scenario: Translation offered for English only
- **WHEN** extended translation is offered for English and not yet for Spanish
- **THEN** the listings give the English model's download size and say that Spanish translation comes later

#### Scenario: The App Store listing
- **WHEN** the App Store listing is prepared
- **THEN** it calls the app neither a beta nor a trial, and says nothing of price
