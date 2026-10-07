## ADDED Requirements

### Requirement: English is glossed in Spanish from the Spanish Wiktionary's English section and the English Wiktionary's translation tables
en-es's glosses SHALL be reduced from the Spanish Wiktionary's English section with the Spanish edition's rules, with the English Wiktionary's Spanish translation tables as the direct fallback and the Spanish Wiktionary's English translation tables read backwards as the inverted one — words people wrote, in Spanish — and no pivot and no machine translation; the pair SHALL read the committed English studied tables as a reader pair does. The measured coverage of the 5,000, 10,000 and 20,000 commonest English lemmas SHALL be at least the floor the owner sets before the measurement is committed, checked by the reduce job, and the share of glossed lemmas among the 10,000 commonest whose gloss came from a translation table SHALL be measured and shown.

#### Scenario: A definition first
- **WHEN** an English lemma has an entry in the Spanish Wiktionary
- **THEN** its gloss is that entry's definition, reduced by the Spanish edition's rules

#### Scenario: A direct-table gloss
- **WHEN** an English lemma has no Spanish Wiktionary entry and its English Wiktionary entry lists Spanish translations
- **THEN** its gloss is those translations, at most three per part of speech, as the Spanish edition's rules write them

#### Scenario: An inverted-table gloss
- **WHEN** an English lemma has neither, and Spanish Wiktionary entries list it as their English translation
- **THEN** its gloss is those Spanish lemmas, ordered by Spanish frequency

#### Scenario: The floor
- **WHEN** a committed en-es measures under the floor the owner set
- **THEN** the reduce job fails, naming the figure

## MODIFIED Requirements

### Requirement: Sources derived from whole Wiktionary dumps are pinned
A pair MAY read kaikki's dump of a whole Wiktionary edition, or kaikki's extract of one language's entries, served gzipped or plain. The pipeline SHALL never keep such a dump or extract whole: it SHALL keep each file the pair derives from it (a language's entries, or the translations its entries list into another language) as an asset of the snapshot's release, recorded in `pin.json` by the sha256 of its decompressed bytes, and SHALL refuse a fetched file whose bytes differ.

#### Scenario: An update keeps the derived files
- **WHEN** `lingua-pack-update` reads today's sources for es-fr
- **THEN** the files derived from the French and Spanish Wiktionaries' dumps are recorded in `pin.json` and published with the snapshot's release, and the dumps are not kept

#### Scenario: A re-reduction refuses other bytes
- **WHEN** `build.sh --reduce es-fr` fetches a derived file whose decompressed sha256 differs from `pin.json`
- **THEN** it fails, naming the source and the file

#### Scenario: An extract served plain
- **WHEN** `lingua-pack-update` reads the English Wiktionary's English extract for en-es
- **THEN** the Spanish translation tables are derived from it in one pass, published with the snapshot, and the extract is not kept
