## ADDED Requirements

### Requirement: French is glossed in English from the English Wiktionary's French section
fr-en's glosses, sense runs and expressions SHALL be reduced from the English Wiktionary's French section alone — the derived file French's forms come from, at the same pinned snapshot — cleaned by the English Wiktionary's rules as es-en's glosses are, and no translation table SHALL gloss a French word or expression in fr-en.
Before the shared rules read the section, its letters and the entries written under a single capital
letter are left out, its senses are read as meanings and written in one English typography — the
rules es-en's glosses are read by — and the English edition's long-parenthesis bound and etymology
merging apply as they are set. A typographic apostrophe in a headword is read as `'`, as French's
forms are. Nothing is pivoted or machine-translated: a word or an expression the section does not
gloss has no gloss. fr-en being French's reference pair, its glossed lemmas are French's dictionary
words. The coverage of the 5,000, 10,000 and 20,000 commonest French lemmas SHALL be at least 91.9,
85.1 and 74.4 %, checked by the reduce job, and no site SHALL publish it before a package lists
fr-en. The English edition's rules SHALL be read by es-en and fr-en and by no other committed pair,
so that a change to them re-pins those two and no other.

#### Scenario: Coverage
- **WHEN** `gloss_coverage.py --pair fr-en` runs on the committed tables, against its floor of 91.9, 85.1 and 74.4 %
- **THEN** it passes, and the measured figures are shown in the pull request

#### Scenario: A sense-group label
- **WHEN** the section nests « a room », « a hotel room », « a bedroom » and « a house of a parliament » under « chambre »'s « a chamber in its various senses, including: »
- **THEN** « chambre »'s gloss opens on « a room; a hotel room », and does not hold « in its various senses »

#### Scenario: A pronoun's senses under its description
- **WHEN** the section nests « we » and « us, to us » under « nous »'s « the plural personal pronoun in the first person: »
- **THEN** « nous »'s gloss opens on « we; us, to us »

#### Scenario: The edition's description in lower case
- **WHEN** the section glosses the partitive article « du » « Forms the partitive article. »
- **THEN** « du »'s gloss is « forms the partitive article »

#### Scenario: A word only a translation table glosses
- **WHEN** the section has no entry for `end`, ranked among the 1,000 commonest French words, while the French Wiktionary's entry `END` lists « NDE, NDI, NDT » as its English translations
- **THEN** `end` has no gloss in fr-en, and is no dictionary word of French

#### Scenario: A bigram an English entry translates
- **WHEN** the English Wiktionary's English entry « he's » lists the French « il est » among its translations
- **THEN** fr-en has no expression « il est »

#### Scenario: French's dictionary words
- **WHEN** fr-en is reduced
- **THEN** `tables/fr/lexical.tsv` lists exactly the lemmas `tables/fr-en/gloss.tsv` glosses, and fr-en's pack carries no lexical table

#### Scenario: A rule of the English edition
- **WHEN** `reduce_edition_en.py` changes
- **THEN** es-en's and fr-en's rule digests move, and en-fr's, es-fr's and en-es's do not

### Requirement: fr-en is committed at its studied tables' snapshot, and the French baseline runs on it
fr-en's native side SHALL be reduced from the pin French's studied tables were reduced from, fetching no source, and committed beside them, every studied table but the dictionary words byte for byte; its pack SHALL be built and checked against its pin wherever the committed pairs' packs are, SHALL stay under the size budget, and no package SHALL list it; and from the pull request that commits fr-en's glosses on, the French invariance baseline SHALL run over the pack built from the committed French tables.
fr-en's folder holds its glosses, sense runs and expressions, its notice and manifest — which credit
the English Wiktionary's French section for the glosses too — its pin and its README; the pin keeps
its snapshot and its sources, and names the new rule digest and pack. The French golden SHALL be
re-blessed once, in that pull request, which changes no French rule and says, probe kind by probe
kind, what moved and why; the hand-written fr-en fixture stays for the tests that build it, its
manifest following French's analyser version. en-fr's, es-fr's, es-en's and en-es's tables, pins,
packs and goldens SHALL NOT move.

#### Scenario: The same snapshot
- **WHEN** fr-en is reduced from its pin with its native side
- **THEN** its pin keeps its snapshot and its sources, every table of `tables/fr/` but `lexical.tsv` is byte for byte as committed, and `tables/fr-en/` holds `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json` and `README.md`

#### Scenario: The pack is built, not shipped
- **WHEN** a pull request runs the extension's checks
- **THEN** fr-en's pack is built from `tables/fr/` and `tables/fr-en/`, has the sha256 its pin records and weighs less than 5 MiB, and the extension's list of shipped pairs does not name it

#### Scenario: The French baseline on the committed pack
- **WHEN** fr-en's glosses are committed
- **THEN** the French invariance baseline builds fr-en's pack from the committed tables, beside es-en's, its golden is re-blessed once in that pull request, its pack line names fr-en's `pack_version`, and its `beside es-en` line does not move

#### Scenario: The fixture stays
- **WHEN** a test builds the hand-written fr-en fixture after the hand-over
- **THEN** it builds, and loads at French's analyser version

#### Scenario: Nothing else moves
- **WHEN** fr-en's native side is committed
- **THEN** en-fr's, es-fr's, es-en's and en-es's tables, pins and packs are byte for byte as before, and their invariance baselines and the extension's snapshots pass without re-blessing
