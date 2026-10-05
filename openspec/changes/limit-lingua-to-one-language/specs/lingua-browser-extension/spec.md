## MODIFIED Requirements

### Requirement: The reader chooses the languages they study
The settings SHALL offer the languages the package ships as a single choice: the reader studies one language at a time, the one chosen. The choice SHALL be saved in the reader's profile as that language alone, and every surface SHALL follow it. A change made in the settings SHALL be confirmed first, with the date from which the reader may change again, and SHALL then be possible again 30 days later. While the reader waits, the other languages SHALL be shown unavailable, with that date. The wait SHALL be kept on the device, outside the backup. When the package ships several languages, onboarding SHALL offer the same choice before the level, and a choice made there SHALL NOT start a wait. A change SHALL delete nothing: the statuses, cards, levels and daily statistics of a language no longer studied SHALL stay, and SHALL be read again when the reader studies it again. A profile that holds several languages, written by an earlier build, SHALL be shown with its first language chosen. The choice SHALL be hidden when the package ships a single language.

#### Scenario: Adding Spanish
- **WHEN** the package ships en-fr and es-fr, and a reader who studies English, and has not changed language in the last 30 days, chooses Spanish in the settings and confirms
- **THEN** the profile holds Spanish alone, which replaces English instead of joining it, and the settings give the date, 30 days later, from which the reader may change again

#### Scenario: The last language
- **WHEN** a reader studies a single language
- **THEN** its option stays chosen: the reader cannot study no language

#### Scenario: Choosing Spanish at onboarding
- **WHEN** the package ships en-fr and es-fr, and a new reader chooses Spanish at onboarding
- **THEN** the profile holds Spanish alone, and the reader may change it in the settings at once

#### Scenario: A change not confirmed
- **WHEN** the reader chooses another language in the settings and cancels
- **THEN** the profile and the choice shown are unchanged

#### Scenario: Within 30 days of a change
- **WHEN** a reader changed language 10 days ago and opens the settings
- **THEN** the other languages are shown unavailable, with the date from which a change is possible

#### Scenario: Back to a language studied before
- **WHEN** a reader who studied English, then Spanish, chooses English again
- **THEN** their English statuses, cards, levels and statistics are as they left them

#### Scenario: A profile from an earlier build
- **WHEN** the profile holds English then Spanish
- **THEN** the settings show English chosen, and choosing Spanish leaves Spanish alone

#### Scenario: Every reader today
- **WHEN** the package ships en-fr alone
- **THEN** no choice of languages is shown, in the settings or at onboarding
