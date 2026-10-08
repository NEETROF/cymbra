## ADDED Requirements

### Requirement: Every kaikki source is derived from one dump per Wiktionary edition
The update SHALL read, of kaikki, only its dump of a whole Wiktionary edition — the English, the French and the Spanish Wiktionaries' — and SHALL derive from it, in one pass, every file a pair reads of that edition: a language's entries as the dump writes them, or the translations its entries list into one language. The dump SHALL be recorded in the pair's pin by its address, the date kaikki regenerated it, and the sha256 and size of its decompressed bytes, computed in the same pass, and SHALL be neither kept nor published. A pair's derived files SHALL be published as assets of the pair's own release and pinned by the sha256 of their decompressed bytes, as derived files are today, so that a re-reduction fetches derived files and never a dump. The files every pair of the programme reads SHALL be listed once, per edition, in a catalogue a pair registers its reads against.

#### Scenario: An update of es-fr reads three dumps
- **WHEN** `lingua-pack-update` reads today's sources for es-fr
- **THEN** it fetches the English, French and Spanish editions' dumps once each, derives the Spanish entries of the English edition, the Spanish entries and the Spanish translations of the French edition and the French translations of the Spanish edition, publishes those four files under `lingua-pack-sources-es-fr-<snapshot>`, records each dump's address, regeneration date, sha256 and size in es-fr's pin, and keeps no dump

#### Scenario: No dump is an asset
- **WHEN** an update publishes a snapshot's release
- **THEN** every asset is a derived file, the largest the English Wiktionary's Spanish section at about 52 MB compressed, and no asset is a dump — the English edition's weighs 2.78 GiB gzipped, above the 2 GiB a release asset may weigh

#### Scenario: Two pairs, one regeneration
- **WHEN** es-fr and es-en are updated from the same regeneration of the English edition's dump
- **THEN** their pins record the same dump sha256 and the same sha256 for the Spanish entries, and the reduce job fetches that asset once

#### Scenario: A re-reduction reads no dump
- **WHEN** `build.sh --reduce es-fr` runs after es-fr's first update under this rule
- **THEN** it fetches es-fr's four derived files from its release, checked by sha256, and no dump

#### Scenario: A pair of stage 3 registers what it reads
- **WHEN** fr-en is added with the English Wiktionary's French section, the English entries' French translations and the French Wiktionary's English translations as its sources
- **THEN** it names those three files of the catalogue, by edition, and adds no derivation

### Requirement: A pin recorded against kaikki's per-language extract stays readable
The pipeline SHALL read a pin recorded before this change — one naming kaikki's per-language extract as an asset of the pair's release — as it is recorded: a re-reduction SHALL fetch the extract from that release, check it by sha256 and reproduce the committed tables byte for byte. This change SHALL re-pin no pair and SHALL move no committed byte, and a pair's move to the dumps SHALL be its next update, reviewed as any update is. Before the first update under the new rule, the tables a pair reduces from an edition's derived entries SHALL be shown to be byte for byte what it reduces from kaikki's per-language extract of the same regeneration, and the measurement SHALL be recorded with the sources.

#### Scenario: The committed pairs reduce as before
- **WHEN** the reduce job runs on this change
- **THEN** en-fr, es-fr and es-en are reduced from their pinned extracts and derived files, every committed table, manifest and pin is reproduced byte for byte, and no pin changed

#### Scenario: The next update moves a pair
- **WHEN** en-fr is next updated
- **THEN** its pin records the French edition's dump and the English entries derived from it, names no extract, and the update's report lists the upstream drift since the pinned regeneration and nothing else

#### Scenario: Extract and dump agree
- **WHEN**, on one day, es-fr and es-en are reduced from their pinned sources with the Spanish section read once as kaikki's extract and once derived from the English edition's dump of the same regeneration
- **THEN** both reductions give the same tables byte for byte, in the pairs' folders and in `tables/es/`, and `SOURCES.md` records the measurement

#### Scenario: kaikki retires its per-language files
- **WHEN** kaikki no longer serves a per-language extract
- **THEN** a re-reduction of a pair pinned against one still reads it from the pair's release, and an update reads the editions' dumps and needs no extract

### Requirement: A dump is read once per run
The pipeline SHALL fetch, in a run that updates or checks several pairs, each edition's dump at most once, derive the edition's whole catalogue at that first read and serve the later pairs of the run from it, deleting the dump after its pass; the monthly check SHALL run every pair in one job for that reason; and an update SHALL fetch only the dumps of the editions its pair reads, the pairs it brings along reading their own pins.

#### Scenario: The monthly check
- **WHEN** the monthly dry run checks en-fr, es-fr and es-en
- **THEN** the English, French and Spanish editions' dumps are fetched once each, about 3.6 GiB in all, every pair's report is written, and nothing is committed or published

#### Scenario: One pair's update
- **WHEN** es-en alone is updated
- **THEN** the English and Spanish editions' dumps are fetched, the French edition's is not, and es-en's release holds the two files it reads

#### Scenario: A file no pair of the run reads
- **WHEN** the French edition's dump is read for en-fr's update
- **THEN** its Spanish entries and its Spanish and English translations are derived in the same pass and discarded with the run, and en-fr's release holds its English entries alone

## MODIFIED Requirements

### Requirement: Sources derived from whole Wiktionary dumps are pinned
The pipeline SHALL derive every kaikki source a pair reads from kaikki's dump of a whole Wiktionary edition, served gzipped or plain, and SHALL never keep such a dump whole nor publish it: it SHALL keep each file the pair derives from it (a language's entries, or the translations its entries list into another language) as an asset of the snapshot's release, recorded in `pin.json` by the sha256 of its decompressed bytes beside the dump's own address, regeneration date, sha256 and size, and SHALL refuse a fetched file whose bytes differ.

#### Scenario: An update keeps the derived files
- **WHEN** `lingua-pack-update` reads today's sources for es-fr
- **THEN** the files derived from the English, French and Spanish Wiktionaries' dumps are recorded in `pin.json` and published with the snapshot's release, and the dumps are not kept

#### Scenario: A re-reduction refuses other bytes
- **WHEN** `build.sh --reduce es-fr` fetches a derived file whose decompressed sha256 differs from `pin.json`
- **THEN** it fails, naming the source and the file

#### Scenario: An extract served plain
- **WHEN** kaikki serves a file plain rather than gzipped — as it serves its per-language extracts, which a pin recorded before this change may still name
- **THEN** `derive` and the fetch read it as they read a gzipped dump, told apart by the gzip magic, so that en-es's Spanish translation tables derive from the English edition's dump exactly as they derived from the English extract
