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

### Requirement: Translation models are listed in one catalogue, with a route per pair
The package SHALL carry one catalogue of the translation models it may download, each file pinned by its address, its size as served, its decompressed size and the sha256 of its decompressed bytes, and SHALL give for each pair the route of models that translates its studied language into its native language, in order. A route SHALL be keyed `<studied>-<native>`, SHALL name only models of the catalogue, SHALL start from the pair's studied language, SHALL chain each model's target to the next model's source, and SHALL end in the pair's native language; a catalogue that breaks any of these SHALL be refused, and nothing SHALL be fetched from it. The assembly of the model host, its mirror releases and the check run before a package is submitted SHALL cover every model of the catalogue.

#### Scenario: Every reader today
- **WHEN** the package is built
- **THEN** its catalogue holds the en-fr, es-en, en-es, fr-en, it-en and en-it models; the en-fr route is the en-fr model alone, the es-fr route is es-en then en-fr, the es-en route is the es-en model alone, the en-es route is the en-es model alone, the fr-en route is the fr-en model alone, the fr-es route is fr-en then en-es, the it-en route is the it-en model alone, the en-it route is the en-it model alone, the it-fr route is it-en then en-fr, the it-es route is it-en then en-es, the fr-it route is fr-en then en-it, the es-it route is es-en then en-it; and a reader whose native language is French downloads, stores and loads as before, and the it-en model too once they accept Italian, it-fr listed

#### Scenario: A route that does not reach French
- **WHEN** a catalogue's es-fr route ends in English
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A route that does not reach its native language
- **WHEN** a catalogue's en-es route ends in French
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A route that does not start from its studied language
- **WHEN** a catalogue's en-es route starts with a model that translates from Spanish
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A route whose key is no pair
- **WHEN** a catalogue keys a route `en` or `en-`
- **THEN** the catalogue is refused, and no model is fetched

#### Scenario: A model stored before the catalogue
- **WHEN** the extension updates on a device that holds the en-fr model
- **THEN** the model is still complete, and nothing is downloaded again

#### Scenario: The host misses a model
- **WHEN** the model host does not serve a file of one of the catalogue's models
- **THEN** the check run before a submission fails, naming that model's file

#### Scenario: A route of a pair not shipped
- **WHEN** the catalogue routes es-en and en-es and the shipped pairs are en-fr and es-fr
- **THEN** no reader's pairs need those routes: nothing is downloaded, kept or loaded for them, and the es-en model is held only as es-fr's first model, as before

#### Scenario: A route of a pair studying French
- **WHEN** the catalogue routes fr-en and fr-es and no shipped pair studies French
- **THEN** no reader's pairs need those routes: nothing is downloaded, kept or loaded for them, and no reader fetches the fr-en model

#### Scenario: A route of an Italian pair
- **WHEN** the catalogue routes it-en, en-it, it-fr, it-es, fr-it and es-it and no shipped pair studies Italian or is glossed in it
- **THEN** no reader's pairs need those routes: nothing is downloaded, kept or loaded for them, and no reader fetches the it-en or the en-it model
