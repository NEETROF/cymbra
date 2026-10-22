## MODIFIED Requirements

### Requirement: The Lingua page says what each studied language's card and levels are
The site's Lingua page SHALL say, in each site language, which studied languages of the shipped pairs have a word card that names the tense and the gender, and which have levels estimated from word frequency, naming them together in one sentence each, the page's readers' pairs first; it SHALL name the level scale as the extension's interface in that language names it: CEFR in French and in English, MCER in Spanish, QCER in Italian. What each studied language's card and levels are SHALL be read from one table, and the build SHALL fail on a shipped pair whose studied language the table does not describe.

#### Scenario: Today's pairs
- **WHEN** Cymbra Lingua ships en-fr and es-fr
- **THEN** the French page says that in Spanish the card also names the tense and the gender, and that Spanish's levels are estimated for want of a freely licensed CEFR list, the scale named « CEFR »

#### Scenario: French studied
- **WHEN** fr-en ships beside es-en
- **THEN** the English page says that in Spanish and French the card also names the tense and the gender, and that Spanish's and French's levels are estimated

#### Scenario: English is never said to be estimated
- **WHEN** en-fr and en-es ship
- **THEN** no page says that English's levels are estimated or that its card names a gender

#### Scenario: A studied language the table does not describe
- **WHEN** a shipped pair studies a language the table has no entry for
- **THEN** the site's build fails, naming that language

#### Scenario: Italian studied
- **WHEN** it-fr ships beside the six pairs
- **THEN** the French page says that in Spanish, Italian and French the card also names the tense and the gender and that Spanish's, Italian's and French's levels are estimated, the scale named « CEFR », and the English and Spanish pages say the same of Italian in their languages

#### Scenario: The Italian page
- **WHEN** en-it ships
- **THEN** the Italian page names the level scale « QCER », and says neither that English's levels are estimated nor that its card names a gender

### Requirement: The coverage table reads at a phone's width
While every shipped pair is glossed in one language, the Lingua page's coverage table SHALL keep one column per pair. Once pairs of two or more native languages are listed, it SHALL have one row per pair, named "<studied> → <native>", the page's readers' pairs first, with one column per size of the commonest-words list; and the page SHALL NOT scroll sideways at a phone's width, the table scrolling within its own box where it is wider than the text.

#### Scenario: Today's pairs
- **WHEN** Cymbra Lingua ships en-fr and es-fr
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
- **WHEN** a visitor opens `/it/lingua/`
- **THEN** the hero shows the Italian interface on an English page with a word card open, a second English page stands where a second studied language would, and every image's alt text is in Italian
