## ADDED Requirements

### Requirement: A French voice from France by default
For French, the automatic voice choice SHALL prefer a voice of France to the voices of other regions within the same quality, on every browser, Firefox for Android's three-letter regions included, the other regions keeping the browser's order. It SHALL treat Apple's French Eloquence voice « Jacques » as it treats the other Eloquence voices. The extension SHALL offer no accent setting: the reader SHALL choose another voice in Réglages, and that choice SHALL be kept for French. English's and Spanish's automatic choices and the order of their voices SHALL NOT change.

#### Scenario: A voice of France listed after a Canadian one
- **WHEN** the browser lists `Amélie` (fr-CA) before `Thomas` (fr-FR), both of the same quality, as Chrome, Firefox and Safari do on macOS
- **THEN** the automatic choice for French is `Thomas`

#### Scenario: Chrome on macOS and its Eloquence voices
- **WHEN** Chrome lists `Amélie`, `Jacques` and `Thomas` among its French voices on the device, with Apple's other Eloquence voices
- **THEN** the automatic choice for French is `Thomas`, and Réglages lists `Thomas` and `Amélie` first and `Jacques` with the other Eloquence voices in the group of other voices

#### Scenario: Firefox for Android
- **WHEN** the reader has allowed Android's voices and Firefox for Android lists `fra-CAN-default` before `fra-FRA-default`
- **THEN** the automatic choice for French is the `fra-FRA-default` voice

#### Scenario: A voice the reader downloaded
- **WHEN** the only enhanced French voice is Canadian, and the voice of France is an ordinary one
- **THEN** the automatic choice for French is the enhanced Canadian voice

#### Scenario: English and Spanish as before
- **WHEN** a speaker reads English or Spanish, on any of the voice lists captured on real browsers, with Android's voices and the online voices allowed or not
- **THEN** its automatic choice and the order of its voices are what they were before French preferred France

### Requirement: French voices are named in the reader's interface language
Réglages SHALL name a French voice by the name the platform gives it, followed by its region in the reader's interface language, Firefox for Android's three-letter regions included, and SHALL NOT show a region's code where the browser can name that region.

#### Scenario: An English-speaking reader
- **WHEN** a reader whose interface is in English opens Réglages for French on Safari for macOS
- **THEN** the voices read "Thomas — France" and "Amélie — Canada", and the automatic choice reads "Automatic (Thomas)"

#### Scenario: A Spanish-speaking reader
- **WHEN** a reader whose interface is in Spanish opens Réglages for French on Safari for macOS
- **THEN** the voices read « Thomas — Francia » and « Amélie — Canadá », and the automatic choice reads « Automática (Thomas) »

#### Scenario: Firefox for Android
- **WHEN** the reader has allowed Android's voices and Firefox for Android lists `français (FRA,DEFAULT)` as `fra-FRA-default`
- **THEN** Réglages names it « français (FRA,DEFAULT) — France » in English and « français (FRA,DEFAULT) — Francia » in Spanish, never « — FRA »

#### Scenario: Belgium and Switzerland
- **WHEN** the browser lists a French voice of Belgium (`fr-BE`) and one of Switzerland (`fr-CH`)
- **THEN** Réglages names their regions "Belgium" and "Switzerland" in English, « Bélgica » and « Suiza » in Spanish

### Requirement: Réglages speaks of French voices in the reader's interface language
Where the speaker reads French, Réglages' read-aloud block SHALL say in the reader's interface language that no French voice is installed when none is on the device, its install help SHALL name the language Windows lists for a French voice of France and the language macOS lists, both in the interface language, and its preview SHALL speak one French sentence, the same whatever the interface language. The French interface SHALL read exactly as before.

#### Scenario: No French voice, in English
- **WHEN** a French page is read, the browser lists Google's remote French voice and no French voice on the device, and the interface is in English
- **THEN** the card shows no listen row, and Réglages says "No French voice is installed on this device.", its tooltip names "French (France)" for Windows and "French" for macOS, and it offers "Use the browser's online voices", off

#### Scenario: No French voice, in Spanish
- **WHEN** the same browser shows Réglages with the interface in Spanish
- **THEN** Réglages says « No hay ninguna voz francesa instalada en este dispositivo. », its tooltip names « Francés (Francia) » for Windows and « Francés » for macOS, and it offers « Usar las voces en línea del navegador », off

#### Scenario: The preview
- **WHEN** the reader selects a French voice in Réglages and presses the preview button
- **THEN** that voice speaks « Voici comment sonneront tes pages quand Lingua les lira à voix haute. », in English and Spanish interfaces alike

#### Scenario: The French interface
- **WHEN** a reader whose interface is in French opens Réglages, reading English or Spanish
- **THEN** the read-aloud block reads exactly as before

### Requirement: An elided French word is heard with the word it leans on
When the card's speaker reads French, the word button of a card opened on an elided piece — a selection the page writes glued to the next word, as `l'` in « l'homme » — SHALL read the piece together with the word it leans on as the page writes them, up to the next space, hyphen, digit or punctuation — an apostrophe between two letters carrying on, as in « jusqu'aujourd'hui » — and SHALL be labelled with that text where the card labels the form seen. A card opened on a piece of a word the analysis split, which the page writes as one word (`à` of `au`), SHALL read that word as the page writes it. The dictionary form's button and the sentence button SHALL read as before. A card read in English or Spanish SHALL read exactly as before.

#### Scenario: An elided article
- **WHEN** the reader, with the interface in English, opens the card of `l'` in « L'homme est venu. » on a French page
- **THEN** the card offers « ▶ L'homme », which speaks « L'homme », then « ▶ le », which speaks `le`, then "▶ Sentence"

#### Scenario: Before a hyphen
- **WHEN** the reader opens the card of `qu'` in « Qu'est-ce que c'est ? »
- **THEN** the form seen's button speaks « Qu'est »

#### Scenario: A typographic apostrophe
- **WHEN** the reader opens the card of `jusqu’` in « jusqu’à demain »
- **THEN** the form seen's button speaks « jusqu’à »

#### Scenario: A contraction
- **WHEN** the reader opens the card of `au` in « Il va au marché. », whose piece is `à`
- **THEN** the word button speaks « au »

#### Scenario: English and Spanish
- **WHEN** the reader opens a card on an English or a Spanish page, a split word's included
- **THEN** its buttons are and speak what they were before this requirement
