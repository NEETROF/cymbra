# Spec Delta

## ADDED Requirements

### Requirement: Catalan, Galician and Italy's regional languages are not read as Italian
The core SHALL refuse as Italian a block the detector reads as Italian whose function words of the neighbour languages the guard knows — Catalan and Galician, and the regional languages of Italy and Corsica, Latin, Occitan and Romanian as the design's measurement keeps them —, counted together, outnumber its Italian ones, a neighbour's function word counting only where it is written in lowercase, a word opened by an apostrophe counting with it. A refused block SHALL be excluded from an Italian analysis and SHALL give Italian no vote in a document's language; a block with as many Italian markers as neighbour ones, or with none, SHALL stay Italian. No marker SHALL be a word Italian writes in any register or region. The rule SHALL bump Italian's analyser version and no other, and no part of it SHALL run on text read as English, Spanish or French, whose output SHALL NOT move.

#### Scenario: A Catalan sentence
- **WHEN** « El far s'alçava a la vora del penya-segat, i ningú no recordava qui l'havia construït. », which the detector reads as Italian, is offered as Italian
- **THEN** it is not Italian, and an Italian analysis excludes it

#### Scenario: A Galician sentence
- **WHEN** « Non hai unha solución sinxela, pero o concello xa traballa nela. », read as Italian by the detector, is offered as Italian
- **THEN** it is not Italian

#### Scenario: A Venetian sentence
- **WHEN** « Ghe xe tanta zente in piassa e tuti i parla. », read as Italian by the detector, is offered as Italian
- **THEN** it is not Italian

#### Scenario: A Sicilian sentence
- **WHEN** « Lu patruni dici ca nun ci su' sordi, ma iddu campa bonu. », read as Italian by the detector, is offered as Italian
- **THEN** it is not Italian

#### Scenario: A Neapolitan sentence
- **WHEN** « Tengo 'na casa piccerella vicino 'o mare, e 'a sera guardo 'e stelle. », read as Italian by the detector, is offered as Italian
- **THEN** it is not Italian, its elided articles `'na`, `'o` and `'a` counting with their apostrophe

#### Scenario: A Sardinian, a Corsican and a Friulian sentence
- **WHEN** « Sa domo est manna e sos pitzinnos giogant in sa pratza. », « A casa mea hè vicinu à a chjesa, è u paese hè chjucu. » and « Cheste e je la cjase dai miei nonos, e jo o soi nassût culì. », each read as Italian by the detector, are offered as Italian
- **THEN** none is Italian

#### Scenario: A Latin, an Occitan and a Romanian sentence
- **WHEN** « Ubi concordia, ibi victoria, sed discordia omnia perdit. », « La lenga occitana foguèt parlada dins tota la region. » and « Copiii se jucau în curte toată ziua, dar mama îi chema acasă. », each read as Italian by the detector, are offered as Italian
- **THEN** none is Italian

#### Scenario: Italian naming Catalan, Sicilian and Latin places and works is kept
- **WHEN** « Abbiamo visitato il Castell dels Tres Dragons a Barcellona, poi il Lu Capu a Siracusa, e letto anche il De bello Gallico. » is offered as Italian
- **THEN** it is Italian: its one neighbour marker, `dels` inside a name, is outnumbered by its Italian ones, `abbiamo` and `anche`

#### Scenario: Regional Italian is kept
- **WHEN** « Ma va', non ci credo mica che sei andato giù in piazza con quel caldo. » and « Dai, vieni su che ti faccio vedere una cosa. » are offered as Italian
- **THEN** both are Italian: `su`, `ci`, `ma` and `va'` are markers of neither table

#### Scenario: A capital makes a name
- **WHEN** « Durante il viaggio abbiamo letto che El Niño e Os Lusíadas sono nomi, non parole. » is offered as Italian
- **THEN** it is Italian: `El` and `Os`, written with a capital, do not count against it

#### Scenario: A Catalan page for a reader of French and Italian
- **WHEN** a page whose blocks are Catalan and Galician, read as Italian by the detector, is offered with French and Italian as candidates
- **THEN** its blocks give Italian no vote

#### Scenario: English, Spanish and French do not move
- **WHEN** the six invariance baselines run after the guard is added
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, and English's, Spanish's and French's analyser versions are unchanged

#### Scenario: The leak the guard leaves
- **WHEN** « Bedda matri, chi caluri oggi! », a Sicilian line of more than twelve bytes read as Italian by the detector that holds no marker of either table, is offered as Italian
- **THEN** the guard does not refuse it, as this requirement states

#### Scenario: A pack built before is refused
- **WHEN** an Italian pack built for the analyser version before this rule is loaded
- **THEN** it is refused as built for another analyser generation, and a French, Spanish or English pack loads as before

## MODIFIED Requirements

### Requirement: An Italian invariance baseline runs beside the English, Spanish and French ones
The core's tests SHALL hold an Italian invariance baseline: the engine's output over a fixed thirteen-page raw-text Italian corpus — pages authored around one phenomenon each (news, fiction, homographs, elisions, the articulated prepositions in both readings, enclitic pronouns, names and titles, informal Italian with the apostrophe for an accent, a recipe, technical prose with one block in NFD, a mixed page with English, Spanish, French, Catalan, Galician and Latin blocks, verb forms) and two public-domain excerpts, set in real Italian punctuation — through the same probes the other baselines use, over a fixture pack studying Italian glossed in French until the Italian tables are committed, with the engine started on the real en-fr pack. The baseline SHALL run wherever the others run; a change that moves its golden SHALL re-bless it and say why, and a change of an Italian rule SHALL bump Italian's analyser version.

#### Scenario: The corpus
- **WHEN** the baseline reads its corpus
- **THEN** it holds the thirteen pages in order, each with at least one block, the reader's pages are among them, and the technical page's NFD block is still decomposed

#### Scenario: Nothing of Italian changed
- **WHEN** the baseline runs on a pull request that changes no Italian rule, no fixture table and no beside pack
- **THEN** every probe is byte for byte the golden

#### Scenario: What the baseline shows today
- **WHEN** the baseline analyses its `elisioni`, `preposizioni`, `clitici`, `tecnico` and `misto` pages
- **THEN** « l'uomo » is `lo` and `uomo`, « del » is whole, « al » is `a` and `il`, « po' » is `poco`, « dirmelo » resolves to `dire`, the NFD block's words resolve to the pack's, `non` and `di` are function words, the Catalan and Galician blocks are excluded as not Italian, and the analysis reports Italian's own version

#### Scenario: An Italian rule lands
- **WHEN** a later change gives Italian a tokenisation, lemmatisation or detection rule
- **THEN** the golden moves, and that pull request re-blesses it, bumps Italian's analyser version and says what moved

#### Scenario: The committed tables replace the fixture
- **WHEN** the Italian tables and the it-fr pair's tables are committed
- **THEN** the baseline's pack is built from them, and the golden is re-blessed once, in that pull request

#### Scenario: The reader's backup
- **WHEN** the baseline backs up its reader, whose statuses, exposures and cards are Italian
- **THEN** the backup has schema version 4
