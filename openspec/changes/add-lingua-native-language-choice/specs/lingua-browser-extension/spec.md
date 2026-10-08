## ADDED Requirements

### Requirement: The reader chooses their native language
The reader SHALL be able to choose their native language among those a shipped pair is glossed in, from the onboarding's first question, from the popup's first-run call to action, and from Réglages; the choice SHALL be shown only when two or more native languages have a shipped pair, and an extension updated from an earlier release SHALL keep its reader's native language without asking, new install and update told apart by the browser's installation event. On a new install the choice SHALL be preset from the browser's language when a shipped pair is glossed in it, from English when English ships, from French otherwise, and applied before the onboarding paints. Choosing SHALL rewrite the stored backup's profile — the native language, and the studied languages without it and without those no shipped pair glosses in it, the native language's first shipped pair's studied language standing in when none is left — and every engine SHALL be rebuilt for that native language with the state restored; every open surface SHALL show the new interface language, its pages reloading and the reading session rebuilt; the sync's accepted languages, the translation pairs and the review's language SHALL follow. A full reset SHALL keep the native language. The profile SHALL NOT be synced.

#### Scenario: Every reader today
- **WHEN** only French-native pairs ship
- **THEN** no surface shows the choice, and every reader stays French-native

#### Scenario: A new install when two native languages ship
- **WHEN** es-en ships beside en-fr and es-fr, and a reader whose browser is in English installs the extension
- **THEN** the onboarding paints in English with English preset, offers « Français » and "English", and the reader studies Spanish once confirmed

#### Scenario: A browser in another language
- **WHEN** the same install has its browser in German
- **THEN** the choice is preset to English, and the reader may pick French

#### Scenario: An installed extension is not asked
- **WHEN** an extension installed before this change updates to it, es-en shipping
- **THEN** the update marks the choice as made, no surface asks for the native language, and the reader stays French-native unless they choose otherwise in Réglages

#### Scenario: Safari without the onboarding
- **WHEN** the onboarding tab did not open on a new install, and a page was read before the popup opened
- **THEN** the popup's first run asks for the native language, preset from the browser's language, before the level, and asks no more once answered

#### Scenario: Changing the native language
- **WHEN** a French-native reader of English and Spanish chooses English in Réglages, es-en shipping
- **THEN** the backup's profile becomes English-native studying Spanish alone, every engine is rebuilt for English, the open pages reload and the reading session is rebuilt, the word card glosses Spanish in English, the interface is in English, the review opens in Spanish, and the sync's accepted languages are Spanish's

#### Scenario: A native language that was the only studied one
- **WHEN** a French-native reader of English alone chooses English, es-en shipping
- **THEN** the reader studies Spanish, and the choice said so before confirming

#### Scenario: A full reset keeps the native language
- **WHEN** an English-native reader erases their data
- **THEN** the fresh state is English-native, studying the first shipped pair's language

#### Scenario: A native language with no pair
- **WHEN** no shipped pair is glossed in Spanish
- **THEN** Spanish is not offered, and a request for it is refused
