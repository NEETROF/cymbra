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
- **The French section** is 69.9 % French verb forms (24,489 senses in 18,385 « Voce verbale »
  entries), and 75.5 % senses tagged `form-of` with the noun, adjective and phrase inflections: only 6,579 of
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
- Registering the sections: change 36 registers the English section, change 47 the French; change 27a
  already registers the Spanish section, which it-es reads backwards.
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
only, whose words their own editions case, and do not import it. Every committed pair loads exactly one
edition's module, the one whose senses are its definitions; the MODIFIED requirement says so, and change
36, the first pair to import this module, declares `archiveAfter: add-lingua-italian-edition-rules`.

### D2 — Pointers: the Italian wordings, ending on their word

A sense kaikki does not tag as a form is a pointer when it is one of the edition's wordings, followed by
« di » or one of its articled forms (« del », « dell' », …), and a target of at most three words that ends
the sense:
- inflections: « plurale di », « femminile di », persons (« terza persona … di », « 3ª pers … di »),
  « participio … di », « passato … di »;
- « variante [ortografica | statunitense | grafica] di », « abbreviazione di », « comparativo di »,
  « superlativo di », « forma corta di »;
- « vedi X » alone;
- the English grammar wordings (« simple past tense e past participle di », « Present participle di »);
- the Spanish section's 41 English bot pointers, read whole (« The first-person singular of congelar in
  the future subjunctive. »).

In the English section, 852 untagged senses are pure pointers (39 / 62 / 81 at the top), « contrazione
di » left out. The same untagged wordings with text after the target stay meanings (about 96 in en,
17 / 26 / 34 at the top): « plurale di that: quelli, quelle », « comparativo di hard: più duro », « forma corta di goodbye,
ciao, addio ». So do the look-alikes: « forma possessiva di I …, il mio » (`my`), « forma particolare
di diamante », « scrittura di canzoni », a bare « femminile », and names.

A sense kaikki tags `form-of` stays a pointer under its tag, text after its target or not (112 in en:
`upper` « comparativo di up; più alto », `raindrops` « plurale di raindrop, gocce di pioggia »); D4 reads
the ones that are meanings, and the rest are change 37's (D10).

**« contrazione di » never makes a sense a pointer**, as en-es reads « Contracción de will y not »: as a
pointer it would leave 11 of the 5,000 commonest English lemmas unglossed (`doesn't`, `isn't`, `won't`),
with only `it's` recovered from the tables. en-fr leaves them unglossed; the reader of en-it gets
« contrazione di is not », which says what the word is. With D4, `didn't`, `gonna` and `gotta`, which
kaikki tags `form-of`, are read by that meaning too.

**« abbreviazione di » is a pointer only when the word it names lends a gloss**, for the same reason:
« abbreviazione di » an English expansion no entry glosses says what `fc`, `bs` and `ps` are, as en-es
keeps « Abreviatura de ». As a pointer it cost those three at the 5,000, and `usd`, `ce`, `qc`, `dns` and
`pw` at the 20,000.

### D3 — A pointer lends the word it names

kaikki sets `form_of` to the gloss's last link, often an Italian word. The pre-pass sets it to the word
the wording names, so a pointer lends that word's gloss: 7 / 15 / 21 English lemmas regain one
(`labor`, rank 1354, « lavoro » from `labour`; `e-mail`; `cafe`; `recognise`). With D2's wordings
alone, the English section loses 21 / 32 / 42 lemmas; with D3 and D2's contractions, 3 / 6 / 11 (with
every rule together; `auf` at the 20,000 to the placeholder rule). D2's abbreviation rule keeps the
abbreviations among them; what remains is re-measured at implementation.

### D4 — A `form-of` tag on a meaning

A sense kaikki tags `form-of` that is not worded as an inflection is read as a meaning only when its
word has no other meaning: +2 / 4 / 6 English lemmas (`hear`, rank 479, « sentire, udire; ascoltare…;
capire »; `mosque`; `dune`). Reading every such sense as a meaning was rejected: `go` would read
« vado; vai » and `eat` « mangio ».

### D5 — A nested gloss

A gloss nested under a pointer (33 senses in en have two glosses or more) is read by its own text:
`could` (rank 83) gains five usage notes, « utilizzato in contrapposizione alla situazione attuale; … »,
and no translation.

### D6 — Notes, and « vedi italiano »

`notes` takes out, wherever it sits:
- the placeholder « definizione mancante; se vuoi, aggiungila tu » (11 en, 16 fr, 3 es);
- « vedi italiano » after a meaning (7 en, 3 fr, 17 es);
- « (inv) » (20 en, 17 of them in the top 5,000: `first`, `second`, `mail`, `honey`);
- template residue: « ( approfondimento) », « () », « [[]] », and stray brackets (`cream` « [crema]], fior
  fiore », `landowner` « proprietario terriero,]] possidente »).

The template residue is taken out by the pre-pass, before a sense is tested as a pointer, so that
« () plurale di neighborhood » is read as one; `notes` alone runs after that test. Where « vedi
italiano » follows a meaning, the note goes and a qualifier after it stays (`pernicioso` « pernicioso,
vedi italiano (dannoso) » → « pernicioso (dannoso) »). 15 / 22 / 32 English rows change (`paper`, rank
735; `no`, rank 46, « no »).

« vedi italiano » alone, or followed by a qualifier (`carpa` « vedi italiano (pesce) »), means the
Italian word is the headword itself: the
pre-pass reads it as that word, restoring 1 / 6 / 11 English and 7 / 12 / 13 Spanish lemmas (`panda`,
`guerra`, `grande`), where dropping it as a note would lose them. A qualifier before the note
(« (relazione extraconiugale) vedi italiano », es `amante`) is missed by this reading and listed for
change 52.

Labels the edition leaves in the text (« (figurato) », « (colloquiale) »: 43 in en, 15 in fr, 1 in es)
and context in parentheses (« (di persona) ») stay, as in the other editions.

### D7 — A letter's name

`letter` matches « nome della lettera X » and « La lettera Zeta »: `ex` and the Spanish `zeta` lose their
letter senses, the latter its only gloss. The English `zeta` (« Il nome della sesta lettera… ») is not
matched and is listed for change 37.

### D8 — Obsolete and heraldic senses: last, or no gloss

The pre-pass writes after the others, in their entry, the senses tagged obsolete or archaic, or
raw-tagged « arcaico » or « datato », as the Spanish edition writes its obsolete senses last. That
changes 11 / 16 / 20 English rows, the first sense of 5 / 6 / 8; `close` (rank 350) now opens on
« vicino ».

Heraldic senses (`topics` heraldry: 1,803 en, 1,731 fr, 679 es) take two rules:
- **A word whose every sense with a gloss is heraldic takes no gloss from its definitions**, so the translation
  tables gloss it. At the 5,000, the French section glosses 95 lemmas by heraldry alone (`le`, `vue`
  « affibbiatura », `vivre` « biscia… », `faux` « falce… », `défense` « testa di leopardo »), of which a
  table glosses 94; the Spanish 32 (`en` « posto, ordinato »), of which 31; the English 13 (the letters
  `s`, `p`, `o`, `v`, `g`, and `plate`, `vol`, `barry`, `er`, `embrace`, `alternate`, `bout`, `hart`), of
  which 8. This rule never touches a word with another meaning, so `anchor` keeps « ancora ». It holds
  for a headword of several words as for one (amended by change 36): it takes 424 of the English
  section's 1,804 expressions out of the definitions (1,804 → 1,380), and the tables gloss 312 of them
  again (« arms of alliance » « arma d'alleanza »); a phrase glossed only by its heraldic sense is the
  heraldic term, which a reader rarely means.
- **A word with other senses has its heraldic ones written last**, in their entry: 12 / 13 / 16 English
  rows change, the first sense of 3 / 4 / 4. The obsolete and heraldic rules change no row in common.

So the French `le` and `tout`, heraldic only, take the tables' gloss, and `mort` opens on its meaning.
`un`, whose article entry's only sense is heraldic while its other entries are not, still opens on it,
as do the words whose heraldic sense sits in an entry of its own: change 48's.

Two stronger rules were measured and not taken:
- **out when the word has another meaning**: 99 / 137 / 177 rows change, but about 12 general meanings
  at the 5,000 go with them (`anchor` « ancora », `coach` « carrozza », `cock` « gallo », `trunk`
  « proboscide », `standard` « stendardo »), which kaikki files under the topic;
- **out but keeping senses of two words or fewer**: 66 / 92 / 117 rows, a length threshold no edition
  uses.

Ordering does not hide a heraldic sense: the shared rules keep up to eight senses (800 characters) per
word, so of the 109 English rows at the 5,000 that showed one, about 96 still do after the others (`a`
« un, uno, una…; sigla che nella blasonatura in trick indica il metallo argento », `rest` « riposare;
resto; riposo; clarion »). Taking those out needs a reviewed list that keeps `anchor` « ancora ». It is
an edition rule, so it lives in `reduce_edition_it.py`: change 37 writes it, and re-pins the pairs
glossed from the Italian Wiktionary that are committed by then; changes 48 and 52 extend it for French
and Spanish.

### D9 — Typography and case

`typography` takes out gender and number marks (« merda f » → « merda », « avvenimento m., evento m.,
fatto m » → « avvenimento, evento, fatto »: about 15 en), never after « di » or « of », so that the word
a pointer names keeps its letters (« participio presente di sing » stays), a space before a comma (45) or
inside a parenthesis,
writes « ... » as « … » (12), and pairs straight quotes as « » (71 en), the quotes the Italian
interface writes (change 33). 46 / 64 / 81 English rows change.

`capitalised=False`: 93.5–95.3 % of the edition's senses open in lower case, so a gloss of
translation-table words keeps its words' case (`up` « su », not « Su »). That moves 320 / 1,020 / 3,373
en-it table rows, and nearly every fr-it and es-it table row (2,524 / 4,744 / 8,499 and 1,488 / 2,592 /
4,207), none committed. A section gloss keeps the case its editor wrote; the senses at the top that open
on a capital (261 English, 151 French, 114 Spanish at the 5,000) are changes 37, 48 and 52's.

There are no sense-number links, unlike the Spanish edition's « ₉ ». The ordinal « 3ª » appears only
inside pointer wordings, and subscripts only in chemical formulas (H₂O), which stay.

### D10 — Not here

- `dangling` (2 senses open on a coordinator), `long_parenthesis` (107 parentheses of 40 characters or
  more in the English top 20,000; M20 keeps them), `mwe_form_of` (D2 covers multi-word headwords:
  1,890 → 1,804 English expressions, « cell phones », « good bye »).
- The defects that are a pair's, for changes 37, 48 and 52:
  - tagged pointers carrying a meaning after the target (112 in en: `upper`, `raindrops`, `mr`), and the
    untagged ones' wording before the meaning (96: `those` « plurale di that: quelli, quelle »);
  - a label before a colon (« in funzione di soggetto: egli, lui »: 75 / 107 / 146);
  - names on common words (93 / 181 / 227 en rows: `john`, `summer`, `mark`);
  - capital openings;
  - examples and English text in a gloss (`time` « what's the time?: che ore sono? »);
  - a missing main sense (`can`, `do`);
  - duplicates across entries (`well`);
  - other languages' words filed in the English section (`reclamo`, `modulate`);
  - kaikki's no-gloss senses at the top (16 / 47 / 46 in en / fr / es).

### D11 — The measured effect

English section, each rule alone on the base (the first run, with « contrazione di » and « abbreviazione
di » pointers, except the two rows of D8 measured in the second):

| Rule | Loose | Rows changed | Lemmas lost / gained |
|---|---|---|---|
| base | 84.9 / 72.4 / 53.3 | — | — |
| pointers (D2) | 84.5 / 72.1 / 53.1 | 38 / 56 / 70 | lost 21 / 32 / 42 |
| pointers lending their word (D2, D3) | 84.7 / 72.2 / 53.2 | 38 / 56 / 70 | lost 14 / 17 / 21 |
| `form-of` on a meaning (D4) | 85.0 / 72.5 / 53.3 | 2 / 4 / 6 | gained 2 / 4 / 6 |
| a gloss nested under a pointer (D5) | — | 2 / 2 / 2 | gained 1 |
| notes and « vedi italiano » (D6) | — | 15 / 22 / 32 | lost 0 / 0 / 1 |
| a letter's name (D7) | — | 1 / 1 / 1 | — |
| obsolete last (D8) | — | 11 / 16 / 20 | — |
| heraldic last (D8) | — | 12 / 13 / 16 | — |
| typography (D9) | — | 46 / 64 / 81 | — |

**The rules together** (the second run: « contrazione di » a meaning, heraldry last; D2's abbreviation
rule and D8's heraldry-only rule came from the review and are re-measured at implementation):

| | 5,000 | 10,000 | 20,000 |
|---|---|---|---|
| loose (base) | 85.0 (84.9) | 72.4 (72.4) | 53.3 (53.3) |
| strict (base) | 84.8 (84.9) | 72.2 (72.4) | 53.1 (53.2) |
| rows changed | 115 | 166 | 213 |
| first sense changed | 47 | 79 | 110 |
| lemmas lost / gained | 3 / 5 | 6 / 7 | 11 / 9 |

Lost at the 5,000: `fc`, `bs`, `ps` (D3), which D2's abbreviation rule keeps. Gained: `could` (D5), `didn't`, `gonna`, `gotta` (D2, D4),
`hear` (D4).

- **fr-it** from definitions: 30.7 / 22.5 / 15.1, unchanged by the second run's rules; 22 / 31 / 43
  rows change.
- **es-it**: 34.3 / 23.9 / 15.4 loose, unchanged by them; strict 34.2 / 23.9 / 15.3, `zeta` (D7) and
  three pointer-only rows losing their gloss; 23 / 36 / 40 rows change.
- **With the review's two rules** (D2's abbreviations, D8's heraldry-only words), estimated on the
  second run's outputs, not yet measured through the pre-pass:

  | Pair | Definitions alone | With both tables |
  |---|---|---|
  | en-it | 84.8 / 72.2 / 53.0 | 91.5 / 82.8 / 70.3 |
  | fr-it | 28.8 / 20.9 / 13.8 | 81.4 / 70.2 / 57.8 |
  | es-it | 33.6 / 23.4 / 14.9 | 64.2 / 50.0 / 36.6 |

  The heraldry-only rule takes about 95 French and 32 Spanish lemmas at the 5,000 out of the
  definitions, and the tables gloss them again: with both tables, the three pairs barely move.
- **A sample** of 50 random en-it rows at the 5,000: 43 right, 5 partly, 2 wrong in the source itself
  (`prove` « prove », its verb missing; `anytime` « affatto, in nessun modo »). The partly right are
  `guy` « cavo; uomo, ragazzo » (the rope first), `treatment`, `layer`, and `belt` and `bloody`, which
  end on a heraldic sense before the rules as after (D8). `event` reads « avvenimento, evento, fatto »
  (D9).
- **For the pairs' changes** (definitions, then the direct table, then the other edition's table read
  backwards; the second run's rules):

  | Pair | Definitions | + direct table | + table read backwards | Precedent |
  |---|---|---|---|---|
  | en-it | 85.0 / 72.4 / 53.3 | 90.7 / 81.2 / 67.0 | 91.5 / 82.8 / 70.3 | en-es 93.4 / 85.2 / 71.9 |
  | fr-it | 30.7 / 22.5 / 15.1 | 80.0 / 68.9 / 56.9 | 81.5 / 70.2 / 57.9 | fr-es 83.4 / 70.8 / 56.5 |
  | es-it | 34.3 / 23.9 / 15.4 | 55.9 / 42.6 / 30.0 | 64.2 / 50.0 / 36.6 | thin (I4) |

  741 of English's 5,000 commonest lemmas have no English entry in the dump (`from`, `up`, `there`,
  `them`, `between`), kaikki's extraction gaps rather than rules.

### D12 — What moves elsewhere

- **No committed pair** (D1).
- **it-es** (change 27) reads the Italian Wiktionary's Spanish section backwards through its own
  pre-pass. 36 « vedi italiano » and 3 placeholder senses pass its two-word filter and would become
  Italian expression keys. Its design, spec and tasks are amended here: `reduce_italian_es.py` takes the
  notes out itself and drops a sense left empty (11 of the 36 carry a word before the note: « analogia,
  vedi italiano »), so that it-es never imports `reduce_edition_it.py`, whose later edits (changes 37, 48,
  52) would otherwise re-pin it.
- **The programme's *Measured figures***: en-it's, fr-it's and es-it's rows give these sections'
  measured sizes and D11's figures; fr-it's « as fr-es was, or better » holds only with both tables.

## Risks / Trade-offs

- **A heraldic sense still shown** (about 96 English rows at the 5,000, after the others) → D8, a
  measured trade against losing general meanings; change 37's reviewed list takes the rest.
- **« vedi italiano » read as the headword when the Italian word differs** → the note says they are
  the same; the sample found no counter-example, and change 37's review reads the rows.
- **A Wiktionary wording added later** → the pointer rule is a list; a new wording shows as an
  unglossed or pointer-glossed row in the pairs' samples, and is added there.

## Migration Plan

Nothing deploys, and no table or pin moves. Rollback is a revert.

## Effort

2–3 ideal days, against the programme's 2–4: the module and its pre-pass 1–1.5, the tests 0.5–1,
change 27's amendment, `SOURCES.md` and the programme's figures 0.25, review 0.25.
