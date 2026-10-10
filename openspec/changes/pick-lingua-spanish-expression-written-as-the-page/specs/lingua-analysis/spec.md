## ADDED Requirements

### Requirement: A Spanish match is named by the headword written as the page
The phrase gloss SHALL, where several headwords of a Spanish pack reach the key a run of a selection matches (*A Spanish pack keeps every headword of a key several reach*), report as the match's name and gloss those of the headword written most like the run, as *A French match is named by the headword written as the page* chooses for French: each headword read by Spanish's pre-pass as *A Spanish pack keys its expressions as Spanish is read* reads it, the headword whose pieces' text is the text of the run's tokens at the same places — compared in lowercase — at the most places, and between headwords written so at as many places, the first in the order the pack lists them. Every headword of the key SHALL be weighed, except that a headword one of whose words carries a clitic pronoun, as Spanish's enclitic rule reads a word (`tenerlos`, `hacerse`, `ayudarme`), SHALL be weighed only where the page writes a clitic pronoun for the run — attached to one of the run's words, as that rule reads it, or as the page's word just before the run (`me`, `te`, `se`, `nos`, `os`, `lo`, `la`, `los`, `las`, `le`, `les`) —; where no headword of the key is weighed so, every headword SHALL be. A key one headword reaches SHALL be matched as before, and no run SHALL match, or stop matching, for these rules. The reader's status SHALL be read on the name reported, and *A Spanish expression settled under its lemmas keeps that key* SHALL apply to that name, except that a lemma chain that is the name of a headword of the run's key SHALL NOT be reported as a chain settled before: a record on it is that headword's. For a Spanish selection, these rules SHALL take the place of the sentence of *A French match is named by the headword written as the page* by which Spanish selections do not change, of the sentence of *Spanish expressions are found on Spanish's reading of a selection* that reports the key's name, and of the reading of *A Spanish expression settled under its lemmas keeps that key* on every lemma chain, every other rule of those requirements and of *Expression lookup in a phrase gloss* still applying. The tokens SHALL be what they are without the table, and a page's analysis, Spanish's analyser version, and English and French selections SHALL NOT change.

#### Scenario: The headword written as the page
- **WHEN** « hace tiempo » is glossed with a Spanish pack whose key `hacer tiempo` holds `hacer tiempo`, which wins it, and `hace tiempo`
- **THEN** one match covers `hace` and `tiempo`, its key is `hace tiempo` and its gloss is `hace tiempo`'s

#### Scenario: The headword that wins the key, written as the page
- **WHEN** « hacer tiempo » is glossed with the same pack
- **THEN** one match covers the two tokens, and its key is `hacer tiempo`

#### Scenario: No headword written as the page, one closer
- **WHEN** « dando vueltas » is glossed with a Spanish pack whose key `dar vuelta` holds `dar vuelta`, which wins it, and `dar vueltas`
- **THEN** one match covers `dando` and `vueltas`, and its key is `dar vueltas`

#### Scenario: A clitic pronoun the page does not write
- **WHEN** « El guardaespaldas tenía puestos unos audífonos. » is glossed with a Spanish pack whose key `tener puesto` holds `tener puesto`, which wins it, and `tenerlos puestos`
- **THEN** one match covers `tenía` and `puestos`, and its key is `tener puesto`

#### Scenario: A clitic pronoun before the verb
- **WHEN** « No te hagas ilusiones. » is glossed with a Spanish pack whose key `hacer ilusión` holds `hacer ilusión`, which wins it, and `hacerse ilusiones`
- **THEN** one match covers `hagas` and `ilusiones`, and its key is `hacerse ilusiones`

#### Scenario: A clitic pronoun attached to the verb
- **WHEN** « Me encanta hacerme ilusiones. » is glossed with the same pack
- **THEN** one match covers `hacerme` and `ilusiones`, and its key is `hacerse ilusiones`

#### Scenario: Every headword writes a clitic pronoun, the page none
- **WHEN** « Ahora no puedes ayudar a Tom. » is glossed with a Spanish pack whose key `poder ayudar` holds `puede ayudarme`, which wins it, and `puedes ayudarme`
- **THEN** one match covers `puedes` and `ayudar`, as before, and its key is `puedes ayudarme`

#### Scenario: A key one headword reaches
- **WHEN** « un poquito » is glossed with a Spanish pack whose key `un poco` holds `un poco` alone
- **THEN** one match covers the two tokens, and its key is `un poco`

#### Scenario: The status follows the headword
- **WHEN** the reader has marked `hacer tiempo` known and « hace tiempo » and « hacer tiempo » are glossed with a Spanish pack whose key `hacer tiempo` holds `hacer tiempo` and `hace tiempo`
- **THEN** « hacer tiempo »'s match is `hacer tiempo`, classified known, and « hace tiempo »'s is `hace tiempo`, not classified known

#### Scenario: A lemma chain settled before
- **WHEN** the reader holds a status on `bueno día`, which is no headword's name, and « buenos días » is glossed with a Spanish pack whose key `bueno día` holds `buen día` and `buenos días`
- **THEN** the match's key is `bueno día`, its status read there, and its gloss is `buenos días`'s

#### Scenario: What the Spanish baselines show
- **WHEN** the es-fr and es-en invariance baselines are re-blessed with the phrase probes « hace tiempo », « buenos días », « un poquito », « No te hagas ilusiones. » and « El guardaespaldas tenía puestos unos audífonos. »
- **THEN** over es-fr « darse cuenta » answers `darse cuenta`, « hace tiempo » `hace tiempo`, « buenos días » `buenos días`, « un poquito » `un poquito`, and the last two none; over es-en « hace tiempo » none, « buenos días » `buenos días`, « un poquito » `un poco`, « No te hagas ilusiones. » `hacerse ilusiones` and « El guardaespaldas tenía puestos unos audífonos. » `tener puesto`; and every other probe recorded before is byte for byte but the pack's size

#### Scenario: English and French do not move
- **WHEN** the English, en-es, French and fr-es invariance baselines run after the rule is added
- **THEN** every probe is byte for byte the output recorded before, without re-blessing, but the French baseline's line describing the es-en pack beside it
