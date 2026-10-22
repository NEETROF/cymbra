# Spec Delta

## MODIFIED Requirements

### Requirement: The coverage table reads at a phone's width
While every shipped pair is glossed in one language, the Lingua page's coverage table SHALL keep one column per pair. Once pairs of two or more native languages are listed, it SHALL have one row per pair, named "<studied> → <native>", the page's readers' pairs first, with one column per size of the commonest-words list; and the page SHALL NOT scroll sideways at a phone's width, the table scrolling within its own box where it is wider than the text.

#### Scenario: Today's pairs
- **WHEN** Cymbra Lingua ships en-fr and es-fr, and no other pair
- **THEN** the table has one column per pair, as before

#### Scenario: Six pairs on a phone
- **WHEN** the six pairs ship, and no other pair, and the English page is shown 375 pixels wide
- **THEN** the table has six rows, Spanish → English and French → English first, and the page does not scroll sideways

#### Scenario: Seven pairs on a phone
- **WHEN** it-fr ships beside the six pairs, and no other pair, and the French page is shown 375 pixels wide
- **THEN** the table has seven rows, English → French, Spanish → French and Italian → French first, and the page does not scroll sideways

#### Scenario: Eight pairs on a phone
- **WHEN** it-en ships beside the seven pairs, and no other pair, and the English page is shown 375 pixels wide
- **THEN** the table has eight rows, Spanish → English, French → English and Italian → English first, and the page does not scroll sideways

#### Scenario: Nine pairs on a phone
- **WHEN** en-it ships beside the eight pairs, and no other pair, and the Italian page is shown 375 pixels wide
- **THEN** the table has nine rows, English → Italian first, and the page does not scroll sideways

#### Scenario: Nine pairs on a phone, it-es the ninth
- **WHEN** it-es ships beside the eight pairs, and no other pair, and the Spanish page is shown 375 pixels wide
- **THEN** the table has nine rows, English → Spanish, French → Spanish and Italian → Spanish first, and the page does not scroll sideways

#### Scenario: Ten pairs on a phone, en-it and it-es
- **WHEN** en-it and it-es ship beside the eight pairs, and no other pair, and the Spanish page is shown 375 pixels wide
- **THEN** the table has ten rows, English → Spanish, French → Spanish and Italian → Spanish first, and the page does not scroll sideways

#### Scenario: Ten pairs on a phone, en-it and fr-it
- **WHEN** fr-it ships beside the eight pairs and en-it, and no other pair, and the Italian page is shown 375 pixels wide
- **THEN** the table has ten rows, English → Italian and French → Italian first, and the page does not scroll sideways

#### Scenario: Eleven pairs on a phone
- **WHEN** en-it, it-es and fr-it ship beside the eight pairs, and no other pair, and the Italian page is shown 375 pixels wide
- **THEN** the table has eleven rows, English → Italian and French → Italian first, and the page does not scroll sideways

### Requirement: The Lingua page shows the extension at work
The site's Lingua page SHALL show, in each site language it is built in, captures of the real extension made by the capture command of `lingua-browser-extension`, whose interface is in the page's language and which read a language that the page's readers study: in the hero, a highlighted page with its percentage pill and a word card open; further down, a demo video, a page in the readers' second studied language, or a second page in the one they study when they study one, a review card, the statistics ladder and the EPUB reader. Every image SHALL declare its width and height and carry an alt text in the page's language that quotes no figure; every image below the first screen SHALL load lazily. The video SHALL have a poster, SHALL carry no audio track, SHALL show its controls, SHALL download nothing before it plays, and SHALL NOT start on its own for a visitor who prefers reduced motion, who saves data, or whose browser runs no script. The page SHALL load nothing from a third party for these media.

#### Scenario: The Spanish page
- **WHEN** a visitor opens `/es/lingua/`
- **THEN** the hero shows the Spanish interface on an English page with a word card open, and every image's alt text is in Spanish

#### Scenario: Reduced motion
- **WHEN** a visitor whose system asks for reduced motion scrolls the video into view
- **THEN** the video shows its poster and does not start until they press play

#### Scenario: Motion welcome
- **WHEN** a visitor who does not ask for reduced motion and does not save data scrolls the video into view
- **THEN** it plays without sound, and pauses when it leaves the screen

#### Scenario: No script
- **WHEN** a visitor's browser runs no script
- **THEN** the video shows its poster and controls, and nothing of it is downloaded until they press play

#### Scenario: A phone
- **WHEN** the page is shown 375 pixels wide
- **THEN** the hero shows a close-up of the word card that can be read at that width, and the page does not scroll sideways

#### Scenario: The Italian page
- **WHEN** en-it ships, and no other pair glossed in Italian, and a visitor opens `/it/lingua/`
- **THEN** the hero shows the Italian interface on an English page with a word card open, a second English page stands where a second studied language would, and every image's alt text is in Italian

#### Scenario: The Italian page with French
- **WHEN** fr-it ships beside en-it and a visitor opens `/it/lingua/`
- **THEN** the hero shows the Italian interface on an English page with a word card open, a French page stands as the readers' second studied language, and every image's alt text is in Italian
