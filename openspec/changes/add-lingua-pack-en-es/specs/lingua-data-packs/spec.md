## ADDED Requirements

### Requirement: English is glossed in Spanish from the Spanish Wiktionary's English section and the English Wiktionary's translation tables
en-es's glosses SHALL be reduced from the Spanish Wiktionary's English section with the Spanish edition's rules, with the English Wiktionary's Spanish translation tables as the direct fallback — words people wrote, in Spanish — and no inverted table, no pivot and no machine translation; the pair SHALL read the English studied tables through en-fr's reducer as a reader pair does. The measured coverage of the 5,000, 10,000 and 20,000 commonest English lemmas SHALL be at least 91.4, 83.2 and 69.9 %, checked by the reduce job, and the share of glosses that are translation-table words SHALL be measured and shown beside it.

#### Scenario: A definition first
- **WHEN** an English lemma has an entry in the Spanish Wiktionary
- **THEN** its gloss is that entry's definition, reduced by the Spanish edition's rules

#### Scenario: A translation-table gloss
- **WHEN** an English lemma has no Spanish Wiktionary entry and its English Wiktionary entry lists Spanish translations
- **THEN** its gloss is those translations, at most three, capitalised as the edition does

#### Scenario: The floor
- **WHEN** a committed en-es measures under 91.4, 83.2 or 69.9 %
- **THEN** the reduce job fails, naming the figure

#### Scenario: Not shipped
- **WHEN** this change is merged
- **THEN** `packs.json`, the package and the site's figures are unchanged

### Requirement: A source derived from a large dump is fetched as its derived files
A pair whose source is a dump or an extract too large for the reduce job SHALL record it in its pin with its address, size, sha256 and modification date, SHALL derive from it, at an update, only the files the reducer reads, SHALL publish those files as the snapshot's release assets, and SHALL fetch those files alone when reduced from its pinned sources.

#### Scenario: An update
- **WHEN** `lingua-pack-update` runs for en-es in update mode
- **THEN** it fetches the English Wiktionary's English extract once, derives the Spanish translation tables from it, publishes them with the snapshot, and keeps the extract nowhere

#### Scenario: A pinned reduction
- **WHEN** the reduce job reduces en-es from its pinned sources
- **THEN** it fetches the derived files and en-fr's assets, never the extract, and reproduces every committed byte

#### Scenario: The extract moves
- **WHEN** the extract's modification date or sha256 differs from the pin's at an update
- **THEN** the update records the new values, and the reduce job keeps reproducing the pinned bytes from the published assets
