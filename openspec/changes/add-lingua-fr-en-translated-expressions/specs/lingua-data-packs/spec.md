## ADDED Requirements

### Requirement: fr-en's expressions are completed by the French Wiktionary's English translations
fr-en's expressions SHALL be completed by the English translations the French Wiktionary's French entries list — for a headword with a space, or a single word French's tokenisation splits, that no expression of the English Wiktionary's French section glosses or keys alike — and no translation table SHALL gloss a French word in fr-en.
A headword's typographic apostrophe is read as `'`, a proper noun's entry is left out, and an
expression's gloss is up to three of its English translations, in the table's order, as one sense; a
translation that only repeats the headword is no gloss, and a headword left with none is no
expression. The section's expression wins: a headword it glosses, a headword whose words the forms
table reads as one of its expressions, and a headword its rules leave out — by name, or because its
senses only point at another spelling, an inverted form or their pieces — are not taken from the
table. The table's expressions that a person read wrong in most of the contexts they are met in are
left out by name, each with its reason, beside the section's. fr-en's glosses, sense runs and
dictionary words, and the coverage its floor holds, are the section's alone.

#### Scenario: An expression only the table glosses
- **WHEN** the section glosses no « il existe » and the French Wiktionary's entry « il existe » lists « there exists »
- **THEN** fr-en glosses « il existe » « there exists », and « Il existe plusieurs solutions. » meets it

#### Scenario: A proper noun's entry in the section
- **WHEN** the section has « Moyen Âge » only as a proper noun's entry, and the French Wiktionary's common noun « Moyen Âge » lists « Middle Ages »
- **THEN** « au Moyen Âge » meets « moyen âge » « Middle Ages »

#### Scenario: A word the tokenisation splits
- **WHEN** the section has no « s'asseoir » and the French Wiktionary's lists « sit, sit down, sit up »
- **THEN** fr-en offers « s'asseoir » as an expression, and « Elle s'assit » meets it

#### Scenario: The section's headword wins
- **WHEN** the section glosses « par exemple » « for example, for instance; on the other hand, by the way » and the table lists « for example, for instance »
- **THEN** fr-en glosses « par exemple » with the section's gloss

#### Scenario: The section's key wins
- **WHEN** the table lists « crème brulée », whose words the forms table reads as the section's « crème brûlée »
- **THEN** fr-en has no « crème brulée », and « une crème brûlée » meets « crème brûlée » with the section's gloss

#### Scenario: What the section leaves out stays out
- **WHEN** the table lists « à la » « a la, à la » and « sur ce » « on that note », which fr-en leaves out by name, and « boite à gants », a post-1990 spelling whose section entry only points
- **THEN** fr-en has none of the three, and « Il va à la gare. » meets no expression

#### Scenario: A translation that repeats its headword
- **WHEN** the table glosses « à deux » « à deux » alone
- **THEN** fr-en has no expression « à deux », and « à deux heures » meets none

#### Scenario: An expression read wrong where it is met
- **WHEN** the table glosses « les autres » « other people » and « le grand » « the Great », both left out by name with their reasons
- **THEN** « Les autres enfants jouent dehors. » meets no expression, and « le grand pont de bois » no « le grand »

#### Scenario: The table glosses no word
- **WHEN** the French Wiktionary's entry `END` lists « NDE, NDI, NDT » and the section has no entry for `end`
- **THEN** `end` has no gloss in fr-en, and `tables/fr-en/gloss.tsv`, `senses.tsv` and `tables/fr/lexical.tsv` are byte for byte what the section's rules write

#### Scenario: Coverage
- **WHEN** `gloss_coverage.py --pair fr-en` runs on the committed tables
- **THEN** it passes against fr-en's floor with the figures the section alone gives

### Requirement: An update can keep an edition's recorded bytes
An update of a pair SHALL keep, for each edition it is asked to keep among those the pair reads, the bytes the pair's pin records — that edition's record carried byte for byte and its files fetched from the release the record names, checked by sha256, its dump not read — while every other source is read anew under the new snapshot, whose release SHALL hold only the files read anew.
An edition to keep that the pair does not read, or that its pin records without derived files, fails
the update before anything is fetched, naming the pair and the edition. A re-reduction reads each
record's files from the release it names, so a pin whose records name two releases reduces as one
naming one. Keeping an edition adds a source to a pair without a new snapshot of the others; the
next update that keeps nothing reads every edition anew.

#### Scenario: fr-en gains the French edition
- **WHEN** `lingua-pack-update` reads today's sources for fr-en, keeping the English edition
- **THEN** it fetches the French edition's dump alone, publishes `kaikki-fr-traductions-en.jsonl` under `lingua-pack-sources-fr-en-<snapshot>`, keeps the `kaikki-en` record — its release `lingua-pack-sources-fr-en-2026.10.09`, its dump and `kaikki-French.jsonl` — byte for byte, and keeps no dump

#### Scenario: An edition the pin cannot keep
- **WHEN** an update is asked to keep an edition the pair does not read, or one its pin records as a per-language extract
- **THEN** it fails before fetching anything, naming the pair and the edition

#### Scenario: A re-reduction reads both releases
- **WHEN** `build.sh --reduce fr-en` runs on a pin whose `kaikki-en` names one release and `kaikki-fr` another
- **THEN** it fetches each record's files from its own release, checked by sha256, reads no dump, and reproduces the committed tables

#### Scenario: An update that keeps nothing
- **WHEN** a later update of fr-en keeps no edition
- **THEN** it reads the English and the French editions' dumps and publishes both derived files under its own release

### Requirement: fr-en's pin records the French Wiktionary's dump its expressions read
fr-en's pin SHALL record, beside the English Wiktionary's French section, the English translations the French Wiktionary's French entries list, derived from that edition's dump and published under fr-en's own release; its notice and manifest SHALL credit the French Wiktionary for them; and reading them SHALL move no table of `tables/fr/` and no table of fr-en but its expressions.
The catalogue lists the derived file in the French edition and fr-en registers it there. fr-en's
pack is built and checked against its pin wherever the committed pairs' packs are, and stays under
the size budget. Once the expressions are committed, the French invariance baseline is re-blessed in
that pull request, which says, probe by probe, what moved and why; fr-es's tables, pin and pack, and
en-fr's, es-fr's, es-en's and en-es's tables, pins, packs and goldens SHALL NOT move.

#### Scenario: Two editions in fr-en's pin
- **WHEN** fr-en's expressions are committed
- **THEN** its pin records `kaikki-en`, with `kaikki-French.jsonl`, and `kaikki-fr`, with `kaikki-fr-traductions-en.jsonl`, each under the release that holds it, and `pack_sources.py` registers both files for fr-en

#### Scenario: The credit
- **WHEN** fr-en's pack is built
- **THEN** its notice names the French Wiktionary for the English translations of expressions, and its manifest's licences name « kaikki / enwiktionary, frwiktionary (CC BY-SA 4.0 + GFDL) »

#### Scenario: French's tables do not move
- **WHEN** fr-en is reduced with the French Wiktionary's translations
- **THEN** every table of `tables/fr/` and fr-en's `gloss.tsv` and `senses.tsv` are byte for byte as before, and fr-es's tables, pin and pack too

#### Scenario: The pack
- **WHEN** a pull request runs the extension's checks
- **THEN** fr-en's pack is built from `tables/fr/` and `tables/fr-en/`, has the sha256 its pin records, weighs less than 5 MiB, and no package lists it before the list of shipped pairs names fr-en

#### Scenario: The French goldens
- **WHEN** fr-en's expressions are committed
- **THEN** `fr-en.golden` is re-blessed — its pack, notice, licences and the phrase probes that meet a new expression move, and phrase probes that meet a table expression, one left out by name and two the section wins are added —, `fr-es.golden` gains the same probes and nothing else of it moves, and the English, Spanish, es-en and en-es baselines pass as committed
