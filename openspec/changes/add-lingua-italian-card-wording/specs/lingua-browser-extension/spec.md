## ADDED Requirements

### Requirement: An Italian interface names forms in Italian grammar's terms
When the interface language is Italian, the word card SHALL name a form's readings with an Italian renderer, typed after the French one. It SHALL name:
- Italian's forms by Treccani's names, the mood before the tense: « indicativo presente », « indicativo imperfetto », « passato remoto », « futuro semplice », « condizionale presente », « congiuntivo presente », « congiuntivo imperfetto », « imperativo », « gerundio », « participio presente », « participio passato », « infinito »;
- French's and Spanish's forms by the Italian name of the same tense, a Spanish future subjunctive as « congiuntivo futuro (futuro de subjuntivo) »;
- English's forms as the Italian Wiktionary names them — « presente semplice », « passato semplice », « participio passato » — and the -ing form as « forma in -ing ».

Persons SHALL read « terza persona singolare », several persons of one tense and number named once. Genders and numbers SHALL read « maschile », « femminile », « singolare » and « plurale », the genders of one number named once. A preposition SHALL contract with a tense's article (« del », « dell' »), and never with a word of the page. The order of tenses SHALL be the one the other renderers pin for each studied language. The renderer SHALL name what the other renderers name, and leave unnamed what they leave unnamed. No reader SHALL see it before the Italian interface is shown. The French, English and Spanish renderers SHALL NOT change.

#### Scenario: A French and a Spanish past
- **WHEN** the interface is Italian and the card names the French `parla` as *parler* and the Spanish `habló` as *hablar*
- **THEN** it reads « terza persona singolare del passato remoto di parler » and « terza persona singolare del passato remoto di hablar »

#### Scenario: English forms
- **WHEN** the interface is Italian and the card names `went` as *go* and `running` as *run*
- **THEN** it reads « passato semplice di go » and « forma in -ing di run »

#### Scenario: Two persons named once
- **WHEN** the interface is Italian and the card names `hablaba` as *hablar*
- **THEN** it reads « prima e terza persona singolare dell'indicativo imperfetto di hablar »

#### Scenario: An adjective's agreement
- **WHEN** the interface is Italian and the card names `rápidas` as *rápido*
- **THEN** it reads « femminile plurale di rápido »

#### Scenario: The other renderers do not move
- **WHEN** every probe of the existing card snapshots is rendered in French, English and Spanish after this change
- **THEN** each snapshot is byte for byte what it was

### Requirement: The Italian renderer's grammar lines are pinned on the real packs' readings
The extension's tests SHALL render, through the Italian renderer, the grammar lines of every grammar probe of the en-es, es-en, fr-en and it-fr goldens into a committed snapshot, holding no gloss and no sense heading, run where the other card snapshots are run and re-blessed where they are re-blessed, so that a change moving an Italian card's grammar moves the snapshot in its pull request.

#### Scenario: A reading moved
- **WHEN** a change moves the readings of a grammar probe of one of those goldens and re-blesses it
- **THEN** the Italian grammar snapshot moves in the same pull request

#### Scenario: A renderer's name moved
- **WHEN** a name of the Italian renderer changes and the snapshot is not re-blessed
- **THEN** the extension's tests fail

## MODIFIED Requirements

### Requirement: An Italian card names its forms in the interface language's grammar
The word card SHALL name the readings of a form of a pack studying Italian in the reader's interface language — French, English, Spanish or Italian — each renderer naming Italian's moods and tenses by its own grammar's terms for Italian: in French, French school terms (the passato remoto as « passé simple », the imperfetto as « imparfait de l’indicatif »); in English, the English Wiktionary's Italian form-of wording, the passato remoto as "past historic (passato remoto)"; in Spanish, the RAE's terms (the passato remoto as « pretérito perfecto simple de indicativo »); in Italian, Treccani's names, the mood before the tense (« passato remoto », « indicativo imperfetto »). Tenses SHALL come in the order indicative (present, imperfect, past historic, future), conditional, subjunctive (present, imperfect), imperative. Where a form reads the indicative and the subjunctive of one tense with the same persons in one number, the card SHALL name them once, as it does for French, and SHALL name them apart where the persons differ. The gerund and the present participle SHALL be named. A plural spelled like the card's own dictionary form, a numeral's form, and a plural determiner or pronoun without a gender SHALL stay unnamed, as on every card. No Italian SHALL reach a card before Italian is a studied language the extension offers; English's, Spanish's and French's cards SHALL NOT change.

#### Scenario: The passato remoto, in each interface
- **WHEN** the card names `parlò` as *parlare*
- **THEN** it reads « 3e personne du singulier du passé simple de parlare » in French, "third-person singular past historic (passato remoto) of parlare" in English, « tercera persona del singular del pretérito perfecto simple de indicativo de parlare » in Spanish, and « terza persona singolare del passato remoto di parlare » in Italian

#### Scenario: The indicative and the subjunctive said once
- **WHEN** the card names `parliamo` as *parlare*, in French
- **THEN** it reads « 1re personne du pluriel du présent de l’indicatif ou du subjonctif et 1re personne du pluriel de l’impératif de parlare »

#### Scenario: A present of four readings
- **WHEN** the card names `parli` as *parlare*, in English
- **THEN** it reads "second-person singular present indicative and first-, second- and third-person singular present subjunctive of parlare", the moods named apart since their persons differ

#### Scenario: The gerund and the participles
- **WHEN** the card names `parlando` and `creduta` in Spanish, and a present participle its pack files under *parlare*
- **THEN** it reads « gerundio de parlare », « participio pasado femenino de credere » and « participio presente de parlare »

#### Scenario: The other studied languages do not move
- **WHEN** every form of the en-fr, es-fr, es-en, en-es, fr-en and fr-es packs is rendered after this change
- **THEN** each renders byte for byte as before
