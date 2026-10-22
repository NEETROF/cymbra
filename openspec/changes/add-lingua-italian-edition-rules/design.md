# Design — add-lingua-italian-edition-rules

## Context

- **The shared rules.** `reduce_common.Edition` holds what the shared rules need from an edition:
  - `form_of`: an untagged sense that only points;
  - `mwe_form_of`, `pointer_tags`, `pointer_fields`;
  - `notes`: what the wiki writes for its own readers;
  - `letter`, `dangling`;
  - `capitalised`: whether a table gloss opens on a capital;
  - `long_parenthesis`.

  `reduce_edition_es.py` adds a pre-pass (`read_as_meanings`, `typography`, its obsolete senses written
  last); `reduce_edition_en.py` adds its own (`_read_senses` lending a pointer's target,
  `english_typography`). A pair's rule digest counts the modules its reducer imports
  (`pack_sources.rule_files`), so a module no committed reducer imports moves no pin.
- **The source.** kaikki.org's Italian Wiktionary dump (`raw-wiktextract-data.jsonl.gz`, 43,849,715 B,
  2026-10-21, the dump it-fr's design cites), decompressed 566,646,259 B, derived by
  `pack_sources.derive`:

  | Section | File | Entries | Senses | Headwords | Senses tagged `form-of` |
  |---|---|---|---|---|---|
  | English (en-it) | `kaikki-it-Inglese.jsonl`, 13,670,867 B | 29,888 | 41,754 | 25,700 | 3,226 |
  | French (fr-it) | `kaikki-it-Francese.jsonl`, 13,314,042 B | 27,377 | 35,026 | 26,108 | 26,433 |
  | Spanish (es-it) | `kaikki-it-Spagnolo.jsonl`, 2,488,861 B | 7,181 | 8,660 | 6,861 | 1,611 |

  The programme's 41,749 / 35,027 / 8,665 senses hold within five.
- **How kaikki exposes a sense**: `glosses`, `tags`, `raw_tags`, `topics`, `categories`, `form_of`,
  `examples`. There is no qualifier field, no `alt_of`, and no links field. Every sense of a « Voce
  verbale » or « …, forma flessa » entry is tagged `form-of`, with `form_of` set to the gloss's last
  linked word, which is often Italian: « plurale di eye, occhi » points at `occhi`.
- **The French section** is 75.5 % French verb forms (18,385 « Voce verbale » entries): only 6,579 of
  its 26,108 headwords have a sense that is not a pointer.
- **Measured** with `reduce_common.reduce_gloss` over each section after `without_letter_senses`,
  against the committed `tables/{en,fr,es}/freq.tsv`, read as `reduce-en-es`, `reduce-fr-es` and
  `reduce-es-en` read them. A neutral `Edition` (the form-of tag and field only) is the base. Figures
  are the 5,000 / 10,000 / 20,000 commonest lemmas.

## Goals / Non-Goals

**Goals:**
- A gloss from the Italian Wiktionary holds its meanings and none of its pointers or notes, in the
  edition's own case and typography.
- Nothing committed moves.

**Non-Goals:**
- A pair's own rules: names on common words (en-es's precedent puts them in the pair), contractions'
  glosses, labels before a colon, examples inside a gloss — changes 36/37, 47/48 and 51/52.
- Registering the sections: each pair's change does (36, 47, 51).
- `dictionary_words`: English's, French's and Spanish's references (en-fr, fr-en, es-fr) do not read
  the Italian edition, and Italian's dictionary words are it-fr's.

## Decisions

### D1 — One module, read by the pairs glossed in Italian alone

`scripts/lingua-data/reduce_edition_it.py` holds `IT = common.Edition(code="it", form_of=…,
notes=…, letter=…, capitalised=False)`, `typography(text)` and `read_as_meanings(src, dst)`, the
Spanish edition's shape. It is imported by en-it's, fr-it's and es-it's reducers and their tests, and by
no committed reducer: with a stand-in module in place, all six committed pairs' `rule_files`,
`rules_sha256` and `pack_version` are as pinned, `check_reducer` passes, and `test_pack_sources`'
glob over `reduce[-_]*.py` passes. it-fr and it-en read the Italian Wiktionary's *translation tables*
only, whose words their own editions case, and do not import it.

### D2 — Pointers: the Italian wordings, ending on their word

A sense is a pointer when it is one of the edition's wordings, followed by « di » (or « del », « dell' »,
…, or « of » in the English bot wordings), and a target of at most three words that ends the sense:
- inflections: « plurale di », « femminile di », persons (« terza persona … di », « 3ª pers … di »),
  « participio … di », « passato … di »;
- « variante [ortografica | statunitense | grafica] di », « abbreviazione di », « comparativo di »,
  « superlativo di », « forma corta di »;
- « vedi X » alone;
- the English grammar wordings (« simple past tense e past participle di », « Present participle di »)
  and the Spanish section's 41 English bot pointers (« The first-person singular of congelar in the
  future subjunctive. »).

In the English section, 868 untagged senses are pure pointers (51 / 74 / 93 at the top). The same
wordings with text after the target stay meanings (about 96 untagged in en, 17 / 26 / 34 at the
top): « plurale di that: quelli, quelle », « comparativo di hard: più duro », « forma corta di goodbye,
ciao, addio ». So do the look-alikes: « forma possessiva di I …, il mio » (`my`), « forma particolare
di diamante », « scrittura di canzoni », a bare « femminile », and names.

**« contrazione di » stays a meaning**, as en-es reads « Contracción de will y not »: as a pointer it
would leave 11 of the 5,000 commonest English lemmas unglossed (`doesn't`, `isn't`, `won't`), with
only `it's` recovered from the tables. en-fr leaves them unglossed; the reader of en-it gets
« contrazione di is not », which says what the word is. With D4, `didn't`, `gonna` and `gotta`, which
kaikki tags `form-of`, are read by that meaning too.

### D3 — A pointer lends the word it names

kaikki sets `form_of` to the gloss's last link, often an Italian word. The pre-pass sets it to the word
the wording names, so a pointer lends that word's gloss: 7 / 15 / 21 English lemmas regain one
(`labor`, rank 1354, « lavoro » from `labour`; `e-mail`; `cafe`; `recognise`). With D2's wordings
alone, the English section loses 21 / 32 / 42 lemmas; with D3 and D2's contractions, 3 / 6 / 11 —
at the 5,000 `fc`, `bs` and `ps`, each « abbreviazione di » an English expansion no entry glosses.

### D4 — A `form-of` tag on a meaning

A sense kaikki tags `form-of` that is not worded as an inflection is read as a meaning only when its
word has no other meaning: +2 / 4 / 6 English lemmas (`hear`, rank 479, « sentire, udire; ascoltare…;
capire »; `mosque`; `dune`). Reading every such sense as a meaning was rejected: `go` would read
« vado; vai » and `eat` « mangio ».

### D5 — A nested gloss

A sense with two glosses or more (33 in en) is read by its own last gloss under a pointer:
`could` (rank 83) gains « utilizzato in contrapposizione alla situazione attuale; … ».

### D6 — Notes, and « vedi italiano »

`notes` takes out, wherever it sits:
- the placeholder « definizione mancante; se vuoi, aggiungila tu » (11 en, 16 fr, 3 es);
- « vedi italiano » after a meaning (7 en, 3 fr, 17 es);
- « (inv) » (20 en, 17 of them in the top 5,000: `first`, `second`, `mail`, `honey`);
- template residue: « ( approfondimento) », « () », « [[]] » (9 en, 1 fr, 1 es).

15 / 22 / 32 English rows change (`paper`, rank 735; `no`, rank 46, « no »).

« vedi italiano » alone, or after a qualifier, means the Italian word is the headword itself: the
pre-pass reads it as that word, restoring 1 / 6 / 11 English and 7 / 12 / 13 Spanish lemmas (`panda`,
`guerra`, `grande`), where dropping it as a note would lose them. A qualifier before the note
(« (relazione extraconiugale) vedi italiano », es `amante`) is missed by this reading and listed for
change 52.

Labels the edition leaves in the text (« (figurato) », « (colloquiale) »: 43 in en, 15 in fr, 1 in es)
and context in parentheses (« (di persona) ») stay, as in the other editions.

### D7 — A letter's name

`letter` matches « nome della lettera X » and « La lettera Zeta »: `ex` and `zeta` lose their letter
senses.

### D8 — Order: obsolete and heraldic senses last

The pre-pass writes after the others, in their entry, the senses tagged obsolete or archaic, or
raw-tagged « arcaico » or « datato », as the Spanish edition writes its obsolete senses last. That
changes 11 / 16 / 20 English rows, the first sense of 5 / 6 / 8; `close` (rank 350) now opens on
« vicino ».

Heraldic senses (`topics` heraldry: 1,803 en, 1,731 fr, 679 es) are written last the same way: 12 / 13
/ 16 English rows change, the first sense of 3 / 4 / 4, and the French `le`, `un`, `tout` and `mort` no
longer open on their heraldic sense. The two rules change no row in common. Two stronger rules were
measured and not taken:
- **out when the word has another meaning**: 99 / 137 / 177 rows change, but about 12 general meanings
  at the 5,000 go with them (`anchor` « ancora », `coach` « carrozza », `cock` « gallo », `trunk`
  « proboscide », `standard` « stendardo »), which kaikki files under the topic;
- **out but keeping senses of two words or fewer**: 66 / 92 / 117 rows, a length threshold no edition
  uses.

Ordering loses no meaning, and it does not hide a heraldic sense: the shared rules keep up to eight
senses (800 characters) per word, so 109 rows at the 5,000 still show one after the others (`a` « un,
uno, una…; sigla che nella blasonatura in trick indica il metallo argento », `rest` « riposare; resto;
riposo; clarion »). 18 open on one, and 13 are glossed by heraldry alone (the letters `s`, `p`, `o`,
`v`, `g`, and `plate`, `vol`, `barry`, `er`, `embrace`, `alternate`, `bout`, `hart`). Taking them out
needs a reviewed list that keeps `anchor` « ancora »: change 37's.

### D9 — Typography and case

`typography` takes out gender and number marks (« merda f » → « merda », « avvenimento m., evento m.,
fatto m » → « avvenimento, evento, fatto »: 21 en), a space before a comma (45) or inside a parenthesis,
writes « ... » as « … » (12), and pairs straight quotes as « » (71 en), the quotes the Italian
interface writes (change 33). 46 / 64 / 81 English rows change.

`capitalised=False`: 93.5–95.3 % of the edition's senses open in lower case, so a gloss of
translation-table words keeps its words' case (`up` « su », not « Su »). That moves 321 / 1,021 / 3,374
en-it table rows, none committed. A section gloss keeps the case its editor wrote; the 261 English
senses at the top that open on a capital are change 37's.

There are no sense-number links, unlike the Spanish edition's « ₉ ». The ordinal « 3ª » appears only
inside pointer wordings, and subscripts only in chemical formulas (H₂O), which stay.

### D10 — Not here

- `dangling` (2 senses open on a coordinator), `long_parenthesis` (107 parentheses of 40 characters or
  more in the English top 20,000; M20 keeps them), `mwe_form_of` (D2 covers multi-word headwords:
  1,890 → 1,801 English expressions, « cell phones », « good bye »).
- The defects that are a pair's, for changes 37, 48 and 52:
  - pointers carrying a meaning after the target (96 untagged and 112 tagged in en: `those`, `mr`);
  - a label before a colon (« in funzione di soggetto: egli, lui »: 75 / 107 / 146);
  - names on common words (93 / 181 / 227 en rows: `john`, `summer`, `mark`);
  - capital openings;
  - examples and English text in a gloss (`time` « what's the time?: che ore sono? »);
  - a missing main sense (`can`, `do`);
  - duplicates across entries (`well`);
  - other languages' words filed in the English section (`reclamo`, `modulate`);
  - kaikki's no-gloss senses at the top (16 / 47 / 46 in en / fr / es).

### D11 — The measured effect

English section, each rule alone on the base (the first run, with « contrazione di » a pointer):

| Rule | Loose | Rows changed | Lemmas lost / gained |
|---|---|---|---|
| base | 84.9 / 72.4 / 53.3 | — | — |
| pointers (D2) | 84.5 / 72.1 / 53.1 | 38 / 56 / 70 | lost 21 / 32 / 42 |
| pointers lending their word (D2, D3) | 84.7 / 72.2 / 53.2 | 38 / 56 / 70 | lost 14 / 17 / 21 |
| `form-of` on a meaning (D4) | 85.0 / 72.5 / 53.3 | 2 / 4 / 6 | gained 2 / 4 / 6 |
| notes and « vedi italiano » (D6) | — | 15 / 22 / 32 | lost 0 / 0 / 1 |
| obsolete last (D8) | — | 11 / 16 / 20 | — |
| heraldic last (D8) | — | 12 / 13 / 16 | — |
| typography (D9) | — | 46 / 64 / 81 | — |

**Every rule of this design together**, « contrazione di » a meaning and heraldry last:

| | 5,000 | 10,000 | 20,000 |
|---|---|---|---|
| loose (base) | 85.0 (84.9) | 72.4 (72.4) | 53.3 (53.3) |
| strict (base) | 84.8 (84.9) | 72.2 (72.4) | 53.1 (53.2) |
| rows changed | 115 | 166 | 213 |
| first sense changed | 47 | 79 | 110 |
| lemmas lost / gained | 3 / 5 | 6 / 7 | 11 / 9 |

Lost at the 5,000: `fc`, `bs`, `ps` (D3). Gained: `could` (D5), `didn't`, `gonna`, `gotta` (D2, D4),
`hear` (D4).

- **fr-it** from definitions: 30.7 / 22.5 / 15.1, unchanged; 22 / 31 / 43 rows change.
- **es-it**: 34.3 / 23.9 / 15.4 loose, unchanged; strict 34.2 / 23.9 / 15.3, `zeta` (D7) and three
  pointer-only rows losing their gloss; 23 / 36 / 40 rows change.
- **A sample** of 50 random en-it rows at the 5,000: 43 right, 5 partly, 2 wrong in the source itself
  (`prove` « prove », its verb missing; `anytime` « affatto, in nessun modo »). The partly right are
  `guy` « cavo; uomo, ragazzo » (the rope first), `treatment`, `layer`, and `belt` and `bloody`, which
  now end on a heraldic sense (D8). `event` reads « avvenimento, evento, fatto » (D9).
- **For the pairs' changes** (definitions, then the direct table, then the other edition's table read
  backwards; the first run's rules):

  | Pair | Definitions | + direct table | + table read backwards | Precedent |
  |---|---|---|---|---|
  | en-it | 84.7 / 72.3 / 53.2 | 90.5 / 81.0 / 67.0 | 91.3 / 82.7 / 70.3 | en-es 93.4 / 85.2 / 71.9 |
  | fr-it | 30.7 / 22.5 / 15.1 | 80.0 / 68.9 / 56.9 | 81.5 / 70.2 / 57.9 | fr-es 83.4 / 70.8 / 56.5 |
  | es-it | 34.3 / 23.9 / 15.4 | 55.9 / 42.6 / 30.0 | 64.2 / 50.0 / 36.6 | thin (I4) |

  741 of English's 5,000 commonest lemmas have no entry in the whole dump (`from`, `up`, `there`,
  `them`, `between`), kaikki's extraction gaps rather than rules.

### D12 — What moves elsewhere

- **No committed pair** (D1).
- **it-es** (change 27) reads the Italian Wiktionary's Spanish section backwards through its own
  pre-pass. 36 « vedi italiano » and 3 placeholder senses pass its two-word filter and would become
  Italian expression keys. Its design is amended here: `reduce_italian_es.py` drops them itself, so
  that it-es never imports `reduce_edition_it.py`, whose later edits (changes 37, 48, 52) would
  otherwise re-pin it.
- **The programme's *Measured figures***: en-it's, fr-it's and es-it's rows give these sections'
  measured sizes and D11's figures; fr-it's « as fr-es was, or better » holds only with both tables.

## Risks / Trade-offs

- **A heraldic sense still shown** (109 rows at the 5,000, after the others) → D8's ordering, a
  measured trade against losing general meanings; change 37's reviewed list takes the rest.
- **« vedi italiano » read as the headword when the Italian word differs** → the note says they are
  the same; the sample found no counter-example, and change 37's review reads the rows.
- **A Wiktionary wording added later** → the pointer rule is a list; a new wording shows as an
  unglossed or pointer-glossed row in the pairs' samples, and is added there.

## Migration Plan

Nothing deploys, and no table or pin moves. Rollback is a revert.

## Effort

2–3 ideal days, against the programme's 2–4: the module and its pre-pass 1–1.5, the tests 0.5–1,
change 27's amendment and the programme's figures 0.25, review 0.25.
