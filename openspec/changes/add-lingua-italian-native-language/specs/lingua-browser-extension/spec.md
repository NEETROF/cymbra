# Spec Delta

## ADDED Requirements

### Requirement: Italian is a native language the extension knows before it is offered
The extension SHALL know Italian as a native language — read from a stored profile, named « Italiano » in every interface language, accepted as the native language of a day's counts — while its interface languages stay French, English and Spanish until an Italian interface exists. Italian SHALL be offered as a native language, and a browser in Italian preset to it, only once a shipped pair is glossed in Italian; until then the choice, the preset, the interface and every request SHALL be what they were.

#### Scenario: No pair glossed in Italian
- **WHEN** no shipped pair is glossed in Italian and a reader installs the extension in a browser set to Italian
- **THEN** the native languages offered are French, English and Spanish, the preset is English, and nothing a reader sees moves

#### Scenario: An Italian label from the app
- **WHEN** the Cymbra Lingua app reports a day's counts labelled with the native language `it`
- **THEN** the extension keeps them under `it` instead of dropping them

#### Scenario: The own name
- **WHEN** Italian is named as a native language in the French, English or Spanish interface
- **THEN** it is « Italiano » in each

#### Scenario: A backup naming an Italian native
- **WHEN** a backup whose profile names Italian as the native language is restored while no pair is glossed in Italian
- **THEN** it restores, the reader is served in French as any profile naming a native language with no pair is, and the backup keeps Italian
