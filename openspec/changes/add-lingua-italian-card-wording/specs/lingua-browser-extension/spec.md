## ADDED Requirements

### Requirement: An Italian interface names forms in Italian grammar's terms
When the interface language is Italian, the word card SHALL name a form's readings with an Italian renderer, typed after the French one. It SHALL name:
- Italian's forms by Treccani's names, the mood before the tense: « indicativo presente », « indicativo imperfetto », « passato remoto », « futuro semplice », « condizionale presente », « congiuntivo presente », « congiuntivo imperfetto », « imperativo », « gerundio », « participio presente », « participio passato », « infinito »;
- French's and Spanish's forms by the Italian name of the same tense, a Spanish future subjunctive as « congiuntivo futuro (futuro de subjuntivo) »;
- English's forms as the Italian Wiktionary names them — « presente semplice », « passato semplice », « participio passato » — and the -ing form as « forma in -ing ».

Persons SHALL read « terza persona singolare », several persons of one tense and number named once. Genders and numbers SHALL read « maschile », « femminile », « singolare » and « plurale », the genders of one number named once. A preposition SHALL contract with a tense's article (« del », « dell' »), and never with a word of the page. The order of tenses SHALL be the one the other renderers pin for each studied language. The renderer SHALL name what the other renderers name, and leave unnamed what they leave unnamed. No reader SHALL see it before the Italian interface is shown. The French, English and Spanish renderers SHALL NOT change.

#### Scenario: An Italian past
- **WHEN** the interface is Italian and the card names `parlò` as *parlare*
- **THEN** it reads « terza persona singolare del passato remoto di parlare »

#### Scenario: A French and a Spanish past
- **WHEN** the interface is Italian and the card names the French `parla` as *parler* and the Spanish `habló` as *hablar*
- **THEN** it reads « terza persona singolare del passato remoto di parler » and « terza persona singolare del passato remoto di hablar »

#### Scenario: English forms
- **WHEN** the interface is Italian and the card names `went` and `going` as *go*
- **THEN** it reads « passato semplice di go » and « forma in -ing di go »

#### Scenario: Two persons named once
- **WHEN** the interface is Italian and the card names `hablaba` as *hablar*
- **THEN** it reads « prima e terza persona singolare dell'indicativo imperfetto di hablar »

#### Scenario: An adjective's agreement
- **WHEN** the interface is Italian and the card names `rápidas` as *rápido*
- **THEN** it reads « femminile plurale di rápido »

#### Scenario: The other renderers do not move
- **WHEN** every probe of the existing card snapshots is rendered in French, English and Spanish after this change
- **THEN** each snapshot is byte for byte what it was

### Requirement: The Italian renderer is pinned on the real packs' readings
The extension's tests SHALL render, through the Italian renderer, the grammar lines and sense headings of every grammar probe of the en-es, es-en, fr-en and it-fr goldens into a committed snapshot, run where the other card snapshots are run and re-blessed where they are re-blessed, so that a change moving an Italian card's grammar moves the snapshot in its pull request.

#### Scenario: A reading moved
- **WHEN** a change moves the readings of a grammar probe of one of those goldens and re-blesses it
- **THEN** the Italian grammar snapshot moves in the same pull request

#### Scenario: A renderer's name moved
- **WHEN** a name of the Italian renderer changes and the snapshot is not re-blessed
- **THEN** the extension's tests fail
