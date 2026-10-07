## ADDED Requirements

### Requirement: The reader chooses their native language
The reader SHALL be able to choose their native language among those a shipped pair is glossed in, from the onboarding's first question, from the popup's first-run call to action, and from Réglages; the choice SHALL be shown only when two or more native languages have a shipped pair, and an installed extension SHALL keep its reader's native language without asking. The onboarding SHALL preset the choice from the browser's language when it is one of them, and English otherwise. Choosing SHALL rebuild every engine for the native language with the reader's state restored, SHALL drop the native language from the studied languages — the native language's first shipped pair's studied language standing in when none is left — and SHALL be followed by the interface language, the sync's accepted languages, the translation pairs and the review's language. A full reset SHALL keep the native language. The profile SHALL NOT be synced.

#### Scenario: Every reader today
- **WHEN** only French-native pairs ship
- **THEN** no surface shows the choice, and every reader stays French-native

#### Scenario: A new install when two native languages ship
- **WHEN** es-en ships beside en-fr and es-fr, and a reader whose browser is in English installs the extension
- **THEN** the onboarding's first question offers « Français » and "English", preset to English, and the reader studies Spanish once confirmed

#### Scenario: A browser in another language
- **WHEN** the same install has its browser in German
- **THEN** the choice is preset to English, and the reader may pick French

#### Scenario: Safari without the onboarding
- **WHEN** the onboarding tab did not open and the native language was never chosen on this device
- **THEN** the popup's first run asks for it before the level

#### Scenario: Changing the native language
- **WHEN** a French-native reader of English and Spanish chooses English in Réglages, es-en shipping
- **THEN** every engine is rebuilt for English, the reader studies Spanish alone, the word card glosses Spanish in English, the interface is in English, the review opens in Spanish, and the sync pulls the cards of es-en

#### Scenario: A native language that was the only studied one
- **WHEN** a French-native reader of English alone chooses English, es-en shipping
- **THEN** the reader studies Spanish, and the choice said so before confirming

#### Scenario: A full reset keeps the native language
- **WHEN** an English-native reader erases their data
- **THEN** the fresh state is English-native, studying the first shipped pair's language

#### Scenario: A native language with no pair
- **WHEN** no shipped pair is glossed in Spanish
- **THEN** Spanish is not offered, and a request for it is refused
