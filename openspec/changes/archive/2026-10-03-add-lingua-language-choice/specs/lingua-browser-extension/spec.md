## ADDED Requirements

### Requirement: The reader chooses the languages they study
The settings SHALL offer the reader the languages the package ships, each ticked when the reader studies it. Ticking SHALL add a language after the others and unticking SHALL remove it, and the last language SHALL NOT be removable. The choice SHALL be saved in the reader's profile, and every surface SHALL follow it. The choice SHALL be hidden when the package ships a single language. When the package ships several, onboarding SHALL offer the same choice before the level.

#### Scenario: Adding Spanish
- **WHEN** the package ships en-fr and es-fr, and a reader who studies English ticks Spanish
- **THEN** the profile holds English then Spanish, and the settings show a level block for each

#### Scenario: The last language
- **WHEN** a reader studies a single language
- **THEN** its box cannot be unticked

#### Scenario: Every reader today
- **WHEN** the package ships en-fr alone
- **THEN** no choice of languages is shown, in the settings or at onboarding

### Requirement: A level per studied language
The settings SHALL show a level block for each language the reader accepts, titled with that language, each with its own chips, hint and calibration. Whether the reader still has to choose a level SHALL depend on the decisions made in that language only.

#### Scenario: A Spanish level does not answer for English
- **WHEN** a reader who studies English and Spanish has chosen a Spanish level and no English one
- **THEN** the English block still asks for a level

#### Scenario: Every reader today
- **WHEN** a reader studies English alone
- **THEN** a single block, « Niveau d'anglais », behaves as before

### Requirement: A voice per studied language
The voice the reader chooses SHALL be kept for the language it reads, and SHALL NOT be used for another language. The settings' voice block SHALL edit the voice of the language its speaker reads. A voice kept before this requirement SHALL be kept as the English voice.

#### Scenario: Two languages, two voices
- **WHEN** the reader chooses a voice while reading Spanish, and another while reading English
- **THEN** Spanish is read with the first and English with the second

#### Scenario: A voice chosen before
- **WHEN** the extension updates on a device whose reader had chosen a voice
- **THEN** English is still read with it

### Requirement: Languages are named in one place
Every label that names a studied language SHALL come from one module, so that it names the language it speaks of, and no surface SHALL hard-code a language's name.

#### Scenario: A Spanish page without enough text
- **WHEN** the popup reports a page without enough text in the reader's language, and the page is read in Spanish
- **THEN** it says no Spanish text was detected

#### Scenario: Checked by lint
- **WHEN** a source file or page outside the labels module names « anglais »
- **THEN** the lint spec fails
