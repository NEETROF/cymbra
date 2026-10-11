## ADDED Requirements

### Requirement: French is translated into English directly, and into Spanish through English
The catalogue SHALL carry Mozilla's fr-en model, version 2.0, pinned like every model and verifiable against Mozilla's publications of it. fr-en's route SHALL be the fr-en model alone, and fr-es's route SHALL be the fr-en model then the en-es model, through English, since Mozilla publishes no model between French and Spanish. A route SHALL be needed only by a reader whose pairs include its pair. For each native language, the routes of all the pairs glossed in it SHALL need at most two models together, so that a reader who keeps their native language never makes the engine delete a model to translate.

#### Scenario: An English-native reader of French
- **WHEN** fr-en ships and a reader whose native language is English ticks « Traduction étendue »
- **THEN** the fr-en model alone is downloaded, and a French sentence is translated by it in one step

#### Scenario: A Spanish-native reader of French
- **WHEN** fr-es ships and a reader whose native language is Spanish ticks « Traduction étendue »
- **THEN** the fr-en and en-es models are downloaded, and a French sentence comes back in Spanish from fr-en then en-es, in one request to the engine

#### Scenario: A Spanish-native reader of English and French
- **WHEN** en-es and fr-es ship and a reader whose native language is Spanish translates on an English page, then on a French page, then on an English page again
- **THEN** the device and the engine hold the en-es and fr-en models, en-es serving both routes, and nothing is deleted

#### Scenario: Every native language's pairs
- **WHEN** the routes of the pairs glossed in each native language are taken together
- **THEN** they need two models each: en-fr and es-en for French, es-en and fr-en for English, en-es and fr-en for Spanish

#### Scenario: Pinned against Mozilla's publications
- **WHEN** the fr-en entry is written
- **THEN** each file's decompressed sha256 equals the one Mozilla publishes for fr-en 2.0, and the model host's assembly keeps every file

#### Scenario: A vocabulary fr-en shares with en-fr
- **WHEN** a device holds the en-fr model and its reader's pairs come to need the fr-en model
- **THEN** the vocabulary file, the same bytes once decompressed, is kept while either model is needed and is not downloaded again

## MODIFIED Requirements

### Requirement: Translation models are listed in one catalogue, with a route per pair
The package SHALL carry one catalogue of the translation models it may download, each file pinned by its address, its size as served, its decompressed size and the sha256 of its decompressed bytes, and SHALL give for each pair the route of models that translates its studied language into its native language, in order. A route SHALL be keyed `<studied>-<native>`, SHALL name only models of the catalogue, SHALL start from the pair's studied language, SHALL chain each model's target to the next model's source, and SHALL end in the pair's native language; a catalogue that breaks any of these SHALL be refused, and nothing SHALL be fetched from it. The assembly of the model host, its mirror releases and the check run before a package is submitted SHALL cover every model of the catalogue.

#### Scenario: Every reader today
- **WHEN** the package is built
- **THEN** its catalogue holds the en-fr, es-en, en-es and fr-en models, the en-fr route is the en-fr model alone, the es-fr route is es-en then en-fr, the es-en route is the es-en model alone, the en-es route is the en-es model alone, the fr-en route is the fr-en model alone, the fr-es route is fr-en then en-es, and a reader whose native language is French downloads, stores and loads as before

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

### Requirement: A pair's marks are measured before they are shown
A pair SHALL have its selection marked only once its marks, measured on the committed corpus of its studied language's selections with the extension's own marking through that pair's route, reach the programme's first tier: at least 90 % of the shown marks correct, and at most 25 % of the selections without a mark. A mark SHALL be judged against the rendering, in the pair's native language, of the selected words. A selection whose sentence the engine traps on twice, after being started again as the extension would, SHALL count as a selection without a mark; one whose fragment alone traps twice SHALL keep the sentence's own marks, unreconciled, as the extension shows them. The pairs that reached it SHALL be listed by name in the extension, and a pair outside the list SHALL be translated without a mark. The corpus, the rule that chose its selections, the harness and every judgment SHALL be committed, filed by pair, so that the measurement can be run again and checked. Every studied language's selections SHALL be chosen by one rule from the same sentences of a parallel treebank, and adding a studied language SHALL NOT move another language's selections, nor any committed result.

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

#### Scenario: A French selection, judged in English
- **WHEN** fr-en's marks are measured on the French selections through the fr-en model and judged against the English rendering
- **THEN** fr-en is marked only if its own figures reach the first tier

#### Scenario: French through English, judged in Spanish
- **WHEN** fr-es's marks are measured on the same French selections through fr-en then en-es and judged against the Spanish rendering
- **THEN** fr-es is marked only if its own figures reach the first tier, whatever fr-en's are

#### Scenario: A studied language added to the corpus
- **WHEN** French's selections are added to the corpus
- **THEN** each is taken from the sentence its step's English and Spanish selections were taken from, and every English and Spanish selection, and every committed result of en-fr, es-fr, es-en and en-es, stays as it was
