## MODIFIED Requirements

### Requirement: Licence hygiene
The build pipeline SHALL accept only sources whose licence permits commercial use (ESDB, WordNet, wordfreq CC BY-SA, kaikki CC BY-SA) and SHALL reject any GPL, AGPL or non-commercial source (documented denylist). Every pack SHALL embed the complete stack of notices, and the user interface SHALL expose an attributions page.
Inflections SHALL come from ESDB, the maintained successor of AGID, completed by the Wiktionary form
links kaikki carries where the inflection is regular. A form a source marks archaic, rarer or
doubtful SHALL NOT be taken as an inflection.

#### Scenario: Notices embedded
- **WHEN** a pack is built
- **THEN** its NOTICE carries the ESDB copyright notice, wordfreq and kaikki, and the extension's "Attributions" page displays them

#### Scenario: A recent plural
- **WHEN** the analysis reads "smartphones" with a pack built from ESDB and kaikki
- **THEN** it is the lemma "smartphone", with that word's status and gloss

#### Scenario: An archaic variant is not an inflection
- **WHEN** the analysis reads "born" or "art"
- **THEN** neither is taken for a form of "bear" or of "be"
