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
The pipeline SHALL read a pin recorded before this rule — one naming kaikki's per-language extract as an asset of the pair's release — as it is recorded: a re-reduction SHALL fetch the extract from that release under the asset name the record gives, check it by sha256, reproduce the committed tables byte for byte and keep the record in the pin, pruning nothing. A pair's move to the dumps SHALL be an update of that pair, reviewed as any update is; a pin written before the rule stays readable until then. Before that update, the pair's tables SHALL be reduced from both readings of one regeneration — the extract, and the section derived from the edition's dump — and every table that differs SHALL be recorded in `SOURCES.md` with the difference between kaikki's two outputs that causes it; that update's report shows it beside the upstream drift, and a difference that breaks another requirement of this capability SHALL be fixed by a change of its own before that update is merged.

#### Scenario: The committed pairs reduce as before
- **WHEN** the reduce job runs with en-fr's, es-fr's and es-en's pins recorded against their extracts
- **THEN** en-fr, es-fr and es-en are reduced from their pinned extracts and derived files, every committed table, manifest and pin is reproduced byte for byte, and no pin changed

#### Scenario: A legacy record is kept in the pin
- **WHEN** `fetch-pinned` reads es-en's pin, whose `kaikki` record names the Spanish extract as `kaikki-Spanish.jsonl.zst` and whose `kaikki-es` record names derived files
- **THEN** it fetches the extract from the release the record names into `kaikki-Spanish.jsonl`, keeps both records, prunes nothing, and the pin's bytes are what they were

#### Scenario: The next update moves a pair
- **WHEN** en-fr is next updated
- **THEN** its pin records the French edition's dump and the English entries derived from it, names no extract, and the update's report lists the upstream drift since the pinned regeneration and nothing else

#### Scenario: Extract and dump are measured against each other
- **WHEN**, on one day and from one regeneration — the extract's and the dump's `Last-Modified` checked —, es-fr and es-en are reduced from their pinned sources with the Spanish section read once as kaikki's extract and once derived from the English edition's dump, en-fr likewise with the French edition's English section, and en-es with its Spanish translations derived once from the English extract and once from the English edition's dump
- **THEN** `pack_report.py --identical` compares every folder both reductions write, `tables/en/` and `tables/es/` included, and `SOURCES.md` records, pair by pair, the tables that are byte for byte the same, the rows that are not and the cause of each; no pin, table, pack or baseline moves

#### Scenario: A pair whose two readings differ
- **WHEN** the two readings give different tables — on 2026-10-08 es-fr's `es/grammar.tsv`, 4 readings, the dump leaving the page's categories on the entry, and en-es's `gloss.tsv` and `mwe.tsv`, 186 and 55 entries, the dump keeping translation tables in page order
- **THEN** the pair keeps its pin, the difference is carried by the pair's next update and named in its report, and es-fr's — which reads `betas` as the plural of the letter *beta*, against *A letter's name gives no reading of its plural* — is fixed by a change of its own before es-fr's next update is merged

#### Scenario: kaikki retires its per-language files
- **WHEN** kaikki no longer serves a per-language extract
- **THEN** a re-reduction of a pair pinned against one still reads it from the pair's release, and an update reads the editions' dumps and needs no extract

### Requirement: A dump is read once per run
The pipeline SHALL fetch, in a run that updates or checks several pairs, each edition's dump at most once, derive the edition's whole catalogue at that first read and serve the later pairs of the run from it, deleting the dump after its pass; the monthly check SHALL run every pair in one job for that reason; and an update SHALL fetch only the dumps of the editions its pair reads, the pairs it brings along reading their own pins.

#### Scenario: The monthly check
- **WHEN** the monthly dry run checks en-fr, es-fr and es-en
- **THEN** the English, French and Spanish editions' dumps are fetched once each, about 3.6 GiB in all, each pair reduces into a dry root of its own, every pair's report and every studied language's — its reference's drift, not the committed copy a later pair laid down — is written, a failing pair stops no other pair and fails the job at the end, and nothing is committed or published

#### Scenario: One pair's update
- **WHEN** es-en alone is updated
- **THEN** the English and Spanish editions' dumps are fetched, the French edition's is not, and es-en's release holds the two files it reads

#### Scenario: A file no pair of the run reads
- **WHEN** the French edition's dump is read for en-fr's update
- **THEN** its Spanish entries and its Spanish and English translations are derived in the same pass and discarded with the run, and en-fr's release holds its English entries alone

## MODIFIED Requirements

### Requirement: Sources derived from whole Wiktionary dumps are pinned
The pipeline SHALL read kaikki's dump of a whole Wiktionary edition served gzipped or plain, told apart by the gzip magic, and SHALL never keep such a dump whole: it SHALL keep each file the pair derives from it (a language's entries, or the translations its entries list into another language) as an asset of the snapshot's release, recorded in `pin.json` by the sha256 of its decompressed bytes, and SHALL refuse a fetched file whose bytes differ.

#### Scenario: An update keeps the derived files
- **WHEN** `lingua-pack-update` reads today's sources for es-fr
- **THEN** the files derived from the English, French and Spanish Wiktionaries' dumps are recorded in `pin.json` and published with the snapshot's release, and the dumps are not kept

#### Scenario: A re-reduction refuses other bytes
- **WHEN** `build.sh --reduce es-fr` fetches a derived file whose decompressed sha256 differs from `pin.json`
- **THEN** it fails, naming the source and the file

#### Scenario: An extract served plain
- **WHEN** kaikki serves an edition's dump plain rather than gzipped
- **THEN** `derive` tells it from a gzipped one by the gzip magic and reads it the same way, in one pass, deriving the same files, recorded by the same decompressed sha256

### Requirement: A translation table is read wherever the edition writes it
A file derived from a dump's translations SHALL hold every translation into its target language that an entry lists, whether the edition writes the table for the whole entry or under one of its senses, with the sense the table names when it names one.

#### Scenario: The English Wiktionary's tables under senses
- **WHEN** the Spanish translations of the English Wiktionary's English entries are derived from kaikki's per-language extract, which moves each table under the sense it translates
- **THEN** the entries whose tables sit under a sense are kept with them, about 65,752 entries with a table under a sense against 5,074 for the whole entry

#### Scenario: An edition that writes tables per entry
- **WHEN** an entry lists its translations only for the whole entry, as every French and Spanish Wiktionary entry es-fr reads does
- **THEN** its derived line is byte for byte the one derived before this change

#### Scenario: The English edition's dump writes its tables on the entry
- **WHEN** the Spanish translations of the English Wiktionary's English entries are derived from the English edition's dump
- **THEN** every table is kept, all of them on the entry in the page's order — 68,582 entries, none with a table under a sense — and a translation the page lists twice is kept twice
