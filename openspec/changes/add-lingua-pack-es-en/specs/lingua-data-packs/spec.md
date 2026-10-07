## ADDED Requirements

### Requirement: A pair of a studied language's second native language
A pair whose studied language already has a reference pair SHALL be reduced from the committed studied tables and its own native sources, SHALL commit its native side only, and SHALL compute its studied side through the reference pair's reducer loaded without editing it, so that the reference's rule digest, pins and pack bytes do not move. The pair SHALL be reduced after its reference in every loop, and SHALL be measured for gloss coverage without being published before it ships.

#### Scenario: The first reader pair
- **WHEN** es-en is reduced from its pinned sources
- **THEN** `tables/es-en/` holds its glosses, senses, expressions, notice, manifest, pin and README, `tables/es/` and `tables/es-fr/` are byte for byte unchanged, and es-fr's pin is unchanged

#### Scenario: The reduce job
- **WHEN** the reduce job runs
- **THEN** es-fr is reduced before es-en, both from their pinned sources, and every committed byte is reproduced

#### Scenario: Not shipped
- **WHEN** this change is merged
- **THEN** `packs.json` lists en-fr and es-fr only, the package carries two packs, and the site's coverage figures are unchanged

### Requirement: A non-reference pair's pin records what it read
The pin of a pair that is not its studied language's reference SHALL record the reference pair, the reference's snapshot and the sha256 of each studied table the build read; the pair's `pack_version` SHALL be its own snapshot and rule digest, the digest covering the reference's rules it loads; the committed-tables check SHALL fail when a studied table differs from what the pair's pin recorded, naming the pair; the pair's notice and manifest SHALL credit both sides' sources.

#### Scenario: The studied side moves
- **WHEN** es-fr is re-reduced and `tables/es/level.tsv` changes, and es-en is not re-pinned
- **THEN** the check fails, naming es-en and the table

#### Scenario: The reference's rules change
- **WHEN** `reduce-es-fr.py` changes and both pairs are re-reduced
- **THEN** es-fr's and es-en's `pack_version` both move, and es-en's pin records es-fr's new snapshot

#### Scenario: The credits
- **WHEN** es-en's pack is built
- **THEN** its notice names the English Wiktionary's Spanish section, the Spanish Wiktionary's translations, wordfreq and UD Spanish-GSD, and its manifest says the levels are estimated

### Requirement: Spanish is glossed in English from the English Wiktionary's Spanish section
es-en's glosses SHALL be reduced from the English Wiktionary's Spanish section with the English edition's rules, with the Spanish Wiktionary's English translations as the direct fallback and no inverted table, no pivot and no machine translation; the English edition's long-parenthesis bound and its etymology merging SHALL be settings decided by the owner on samples shown in the pull request; the measured coverage of the 5,000, 10,000 and 20,000 commonest lemmas SHALL be at least es-fr's published figures.

#### Scenario: Coverage
- **WHEN** `gloss_coverage.py --pair es-en` runs on the committed tables
- **THEN** the three figures are at least 87.6, 77.2 and 63.7

#### Scenario: A gloss from a translation table
- **WHEN** a Spanish lemma has no English entry but the Spanish Wiktionary lists its English translations
- **THEN** its gloss is those translations, at most three, capitalised as the edition does

#### Scenario: The owner's settings
- **WHEN** the pull request shows the samples for the long-parenthesis bound and the etymology merging
- **THEN** the values the owner picks are committed in the English edition's rules before the pin is recorded, and no French-native pair is re-pinned by it
