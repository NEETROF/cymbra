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

## ADDED Requirements

### Requirement: A pack lists the lemmas it merges
A pack built after an update that moves forms to a different lemma SHALL carry the list of those merges, each an old lemma and the lemma it now belongs to.
The list SHALL be computed from the previous and the new tables, SHALL be part of the reviewed
update, and SHALL be empty when nothing moved. A lemma of one or two letters SHALL never be the
target of a merge.

#### Scenario: A plural merged into its singular
- **WHEN** the new tables send "smartphones" to "smartphone", where the previous ones kept it as a lemma
- **THEN** the pack lists the merge "smartphones" → "smartphone"

#### Scenario: Nothing moved
- **WHEN** an update changes glosses only
- **THEN** the pack lists no merge
