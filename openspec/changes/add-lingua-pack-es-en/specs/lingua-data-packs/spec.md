## ADDED Requirements

### Requirement: A pair of a studied language's second native language
A pair whose studied language already has a reference pair SHALL be reduced from the committed studied tables — its lemmas and their ranks read as committed, nothing of the studied side computed — and from its own native sources, SHALL commit its native side only, and SHALL load no other pair's reducer, so that the reference's rule digest, pins and pack bytes do not move. The pair SHALL be reduced after its reference in every loop, and SHALL be measured for gloss coverage against a floor without being published before it ships.

#### Scenario: The first reader pair
- **WHEN** es-en is reduced from its pinned sources
- **THEN** `tables/es-en/` holds its glosses, senses, expressions, notice, manifest, pin and README, `tables/es/` and `tables/es-fr/` are byte for byte unchanged, and es-fr's pin is unchanged

#### Scenario: The reduce job
- **WHEN** the reduce job runs
- **THEN** es-fr is reduced before es-en, both from their pinned sources, and every committed byte is reproduced

#### Scenario: A rules-only change of the reference
- **WHEN** `reduce-es-fr.py` changes and es-fr is reduced again without a studied table moving
- **THEN** es-en's pin and pack are unchanged, and its checks pass

### Requirement: A reader pair's pin records the studied tables it read
The pin of a pair that is not its studied language's reference SHALL record the reference pair and the sha256 of each of the six studied tables the build read; the checks SHALL fail when a committed studied table differs from what the pair's pin recorded, naming the pair and the table; the pair's `pack_version` SHALL name its own snapshot and rule digest and a digest of the studied tables it recorded, in an update as after a re-reduction, so that it moves when one of them moves; the pair's notice and manifest SHALL credit both sides' sources. Each pair SHALL pin its own extract, and an update SHALL publish a pair's own assets only.

#### Scenario: The studied side moves
- **WHEN** es-fr is re-reduced and `tables/es/level.tsv` changes, and es-en is not re-pinned
- **THEN** the checks fail, naming es-en and `es/level.tsv`

#### Scenario: An updated dictionary for a reader pair
- **WHEN** es-fr is re-reduced and `tables/es/level.tsv` changes, and es-en is reduced again from its pinned sources under the same rules
- **THEN** es-en's `pack_version` names the same snapshot and rule digest and differs from the one built before

#### Scenario: Each pair's own extract
- **WHEN** es-fr's update brings es-en along
- **THEN** es-en is reduced from its own pin, nothing of es-en is published, and es-en's pin still names its own release

#### Scenario: The credits
- **WHEN** es-en's pack is built
- **THEN** its notice names the English Wiktionary's Spanish section, the Spanish Wiktionary's translations, the French Wiktionary (whose glosses decide the dictionary words and which lemmas take a level), wordfreq and UD Spanish-GSD, and its manifest says the levels are estimated

### Requirement: Spanish is glossed in English from the English Wiktionary's Spanish section
es-en's glosses SHALL be reduced from the English Wiktionary's Spanish section with the English edition's rules, with the Spanish Wiktionary's English translations as the direct fallback and no inverted table, no pivot and no machine translation; the English edition's long-parenthesis bound and its etymology merging SHALL be settings of the English edition that touch no French-native pair's rules; the measured coverage of the 5,000, 10,000 and 20,000 commonest lemmas SHALL be at least 87.6, 77.2 and 63.7 %, es-fr's published figures, checked by the reduce job. A letter of the alphabet SHALL gloss no Spanish word: a sense that only names a letter, an entry written under a single capital letter, and a letter the Spanish Wiktionary translates as itself are left out.

#### Scenario: Coverage
- **WHEN** `gloss_coverage.py --pair es-en` runs on the committed tables, against its floor of 87.6, 77.2 and 63.7 %
- **THEN** it passes, and the measured figures are shown in the pull request

#### Scenario: The commonest preposition, in English
- **WHEN** the reader of es-en opens the card of `a`, `r` or `b`
- **THEN** `a`'s gloss opens on « to », with no « bishop » and no sense naming the letter A, and `r` and `b` have no gloss

#### Scenario: A gloss from a translation table
- **WHEN** a Spanish lemma has no English entry but the Spanish Wiktionary lists its English translations
- **THEN** its gloss is those translations, at most three per part of speech, written as the English edition's rules write them

#### Scenario: A setting of the English edition
- **WHEN** the English edition's long-parenthesis bound or its etymology merging changes
- **THEN** es-en is re-pinned, and en-fr's and es-fr's pins are unchanged
