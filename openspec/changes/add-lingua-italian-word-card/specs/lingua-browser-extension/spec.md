# Spec Delta

## ADDED Requirements

### Requirement: An Italian card names its forms in the interface language's grammar
The word card SHALL name the readings of a form of a pack studying Italian in the reader's interface language — French, English or Spanish — each renderer naming Italian's moods and tenses by its own grammar's terms for Italian: in French, French school terms (the passato remoto as « passé simple », the imperfetto as « imparfait de l’indicatif »); in English, the English Wiktionary's Italian form-of wording, the passato remoto as "past historic (passato remoto)"; in Spanish, the RAE's terms (the passato remoto as « pretérito perfecto simple de indicativo »). Tenses SHALL come in the order indicative (present, imperfect, past historic, future), conditional, subjunctive (present, imperfect), imperative. Where a form reads the indicative and the subjunctive of one tense with the same persons in one number, the card SHALL name them once, as it does for French, and SHALL name them apart where the persons differ. The gerund and the present participle SHALL be named. A plural spelled like the card's own dictionary form, a numeral's form, and a plural determiner or pronoun without a gender SHALL stay unnamed, as on every card. No Italian SHALL reach a card before Italian is a studied language the extension offers; English's, Spanish's and French's cards SHALL NOT change.

#### Scenario: The passato remoto, in each interface
- **WHEN** the card names `parlò` as *parlare*
- **THEN** it reads « 3e personne du singulier du passé simple de parlare » in French, "third-person singular past historic (passato remoto) of parlare" in English, and « tercera persona del singular del pretérito perfecto simple de indicativo de parlare » in Spanish

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

### Requirement: The Italian card is pinned on the real packs
The Italian invariance baseline SHALL hold the card's probes — one form for each name the Italian card says, and lemmas with two sense runs or more — and the extension's tests SHALL render every grammar and phrase probe of the it-fr golden into a committed snapshot of the card's lines in French, run and re-blessed where the other card snapshots are, so that a change moving an Italian card's wording moves the snapshot in its pull request.

#### Scenario: The snapshot
- **WHEN** the extension's tests run after this change
- **THEN** `test/baseline/word-card-it-fr.txt` holds the French card's lines for every grammar and phrase probe of `it-fr.golden`, and a run without re-blessing passes
