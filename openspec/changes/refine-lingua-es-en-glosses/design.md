# Design — refine-lingua-es-en-glosses

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `reduce-es-en.py` | `native_fields` (the extract cut to `word`, `pos`, `senses` and each sense's `glosses`, `tags`, `form_of`, `alt_of`) → `without_letters` → `english.without_letter_headwords` → `english.merge_same_pos_etymologies` (off) → `common.native_tables(…, edition=english.EN, fallbacks=[(direct, list)])` |
| `reduce_edition_en.py` | `EN`: `_FORM_OF` (untagged pointer wordings), `_LETTER`, pointer tags `form-of`/`alt-of` and fields `form_of`/`alt_of`, `capitalised=False`, `long_parenthesis=LONG_PARENTHESIS` (0); the two pre-passes above. Loaded by es-en alone: editing it re-pins es-en and no other pair (change 6 D2, change 21 D5) |
| `reduce_common.py` | `_read_entries` reads a sense by `glosses[0]` and skips it when `_is_form_of`; `clean_gloss` (notes, the long-parenthesis bound, whitespace, `.strip(" ;,").rstrip(".:")`, the cut); `_join_senses_by_pos` (round-robin across a word's entries in file order, up to eight senses, grouped by part of speech in the order they first appear; a sense's own `;` written `,`); a word with no meaning sense borrows its pointer's target's senses in the same part of speech, from a target of three letters or more (`_MIN_BASE`); an all-capitals headword does not gloss a lower-case word that has entries (`_acronym`). Every pair loads it: editing it re-pins en-fr, es-fr, es-en and en-es |
| kaikki's senses | A nested sense carries its parents' glosses first: `glosses: ["Figurative senses.", "to come from, originate"]`; its parent may also appear alone as a sense (`["now, right now, …"]` over « by now ») or not (`["to make"]`, `["Figurative senses."]`). A pointer names its target in `alt_of`/`form_of` — `word`, and `extra` for a meaning it carries (« mucho », « very ») — as kaikki parsed the template: « malo bad » with `extra` « evil », « voy a », « cincuenta y uno », or « mío » and « my » as two targets |
| `tables/es-en/` | 31,876 glossed lemmas, 15,490 expressions; pinned at `2026.10.08` from `lingua-pack-sources-es-en-2026.10.08`; coverage 93.0 / 86.5 / 76.5 % against `FLOORS["es-en"]` 87.6 / 77.2 / 63.7 |
| Change 23 | `crates/lingua-wasm/tests/es_en_baseline.rs` + `baseline/es-en.golden` (the reference's probes and 40 lemmas, among them `a`, `su`, `pero`, `o`, `ya`, `qué`, `bien`, `ahora`, `nada`, `ni`, `otro`, `tanto`, and the card of « viaje », its shown gloss a constant); `test/word-card-es-en.spec.ts` + `test/baseline/word-card-es-en.txt`; `test/row-gloss-tables.spec.ts` (no row of any pair ends on an opening mark; the French rows pinned in `test/baseline/selection-rows-fr.txt`); `lingua-pack-update` re-blesses all of them on a dictionary update's branch |
| Change 21 | Tasks 2.2 and 5.1 open: the owner's two settings (M20), sampled on today's tables |
| Change 38 | `migrate-lingua-pack-sources-to-raw-dumps`, merged (#804): re-pins no pair, keeps a legacy `kaikki` record as recorded (its D5), and measured es-en's tables identical from the pinned extract and from the English dump of 2026-10-03, under today's rules (its D6; `SOURCES.md`, *Extract and dump are measured against each other*) |

## Goals / Non-Goals

**Goals:**
- es-en's glosses read as the English Wiktionary's meanings, in one typography, before change 34
  ships them — by rules of the English edition, measured on the whole table and the top 10,000.
- Every defect class of change 23's list decided — fixed, for the owner, or left — with its counts
  and where its fix belongs.

**Non-Goals:**
- M20 (change 21's two settings); any rule of `reduce_common.py`; any other pair's bytes.
- New data in the packs (labels as a field, a part-of-speech frequency); the studied side
  (`tables/es/`, es-fr's reduction); the card's wording (changes 23, 33).
- A new fetch: es-en keeps its snapshot.

## Measured

Rows are the lemmas of `tables/es-en/gloss.tsv` (31,876): 30,918 / 8,463 glossed by their own
senses, 829 / 167 borrowed from a pointer's target, 129 / 16 from the translation table (848 / 175
rows have a pointer and no sense of their own; 19 / 8 of them borrow nothing). "top 10k" are those
ranked 1–10,000 in `tables/es/freq.tsv` (8,646 glossed). Text classes are counted on the committed
table; structural ones (nested senses, pointers, tags) on the pinned extract's senses, as
`reduce-es-en.py` cuts them. Each rule's effect is measured on the real reduction: the shared rules
(`common.native_tables`, the borrowing and the translation-table fallback included) over the cut
file, with a scratch prototype of D2–D5 as the pre-pass — which, with no rule, reproduces the
committed `gloss.tsv`, `senses.tsv` and `mwe.tsv` byte for byte. Classes overlap.

| # | Class | Rows: whole / top 10k | Examples | Decision |
|---|---|---|---|---|
| 1 | A sense-group label or a list's introduction read as a sense | 9 / 6 | « venir » (881) « Senses relating to literal movement; Figurative senses »; « lata » (4342) « …; unit of measurement for »; « cusco » (7161) « places in Peru » | Fix (D2) |
| 2 | A sense nested under a pointer read as the pointer | 13 / 5, of which 5 / 1 have no other sense | « su » (15) « apocopic form of suyo »; « sí » (96) « prepositional form of se »; « casita » (6636) without « small house », nested under « diminutive of casa »; « cosita » (9065), « cajita », « chiquillo », « ramita » unglossed, every sense under « diminutive of … »; « québec » (34322) under « alternative form of Quebec: »; « weón » (24897) « pronunciation spelling of huevón » | Fix (D2) |
| 3 | A shortened or respelled form: its pointer kept as a meaning, or the meaning it carries lost | wording kept 13 / 5; 87 senses so worded in the lemmas and expressions (70 tagged `alt-of`, 17 untagged), and 3 tagged `apocopic` or `prepositional` that are meanings; D3 changes 35 / 15 rows and glosses 4 more lemmas and 25 expressions | « tas » (7587) « apheretic form of estás »; « mi » (22) « mu, the Greek letter Μ, μ; mi » (« my » lost); « muy » (32) « much, a lot, far, way, many times; very; … » (borrowed from « mucho »); « cincuenta y un » unglossed (« apocopic form of cincuenta y uno (“fifty-one”) ») | Fix (D3) |
| 3b | A pronoun's case form whose pointer carries its meaning, skipped | 3 / 3 (9 senses) | « lo » (14) loses « him, you (formal), it, that » and reads only its article; « nos » (35) reads only two archaic senses; « les » (71) « the (plural) », « to them, for them » skipped | Fix (D3) |
| 4 | An abbreviation tagged `alt-of` lending its target's gloss | 191 / 70 of the 829 / 167 borrowed rows lend through « abbreviation of », « initialism of », « acronym of » or « clipping of »; single letters 3 / 3 | « q » (373) « who, that; that, whom, which » (que); « k » (910) « that; than; … »; « t » (610) « time; …; weather (…) » (tiempo, the first of four targets). « qe » (6499) « Querétaro (…) » is the acronym `QE`'s own gloss (*An acronym with no common word*) | Leave; « q », « k », « t » for the owner (Q2) |
| 5 | A proper noun opening a lower-case lemma's gloss | 242 / 124 rows open on a proper noun's run before another (es-fr: 114 / 49); under a function word 1 / 1 | « como » (17) « Como (a city…); Como (a province…); as … »; « amor » (198) « a surname; love »; « chile » (328) « Chile (…); a chili pepper » | Fix « como » (D4); the rest for the owner (Q3) |
| 5b | A common noun's sense in an English capital | 1,164 / 236 rows open on a capitalised sense that is no proper noun's | « tierra » (236) « Earth (planet…) »; « mayo » (324) « May »; « julio » (300) « July » | Leave: English capitalises the planet, months, languages and demonyms |
| 6 | The edition's description in a capital, mid-gloss or first | 170 / 58 (202 / 69 senses, 59 / 8 opening the row); 314 expressions. « The » before a capital: 5 lemma senses (3 titles or names, « The Nutcracker (ballet) », « The Inca Empire »; 2 species) and 38 expression senses, 37 of them a species (« The Eurasian treecreeper », « The Puna teal, Anas puna »), the last « The Game (mind game) » | « a » (6) « …; Used before words referring to people… »; « se » (9) « A reflexive or reciprocal pronoun… »; « ni » (48) « Used when negating two or more elements… » | Fix (D5); « The » before a capital kept, the species with it (D5) |
| 7 | A reference to a numbered sense of the source | 2 / 2 | « ya » (26) « (difference from sense 4 depends on context) »; « jurado » (1925) « (member of a jury [sense 1]) » and « judge (member of a jury [sense 2], officiator of a competitive event) » | Fix (D5) |
| 8 | IPA inside a sense | 2 / 1 | « bueno » (104) « …with the pronunciation /bweˈno/, rather than /ˈbweno/ »; « seseo » (53052) « …as /s/ rather than /θ/ » | Leave (D6) |
| 9 | Ellipses written three ways | `...` 12 / 10 (9 / 7 unspaced, 3 / 3 spaced); `…` inside a sense 2 / 2; 5 expressions, and 5 more whose final `...` the shared cleaning drops; the rule changes 13 / 10 rows and 10 expressions | « nada » (47) « not...anything »; « tanto » (76) « both ... and »; « o » (21) « either … or »; « ahora » (45) « whether...or... » loses its last one to the final-period strip, « a la mierda » « to hell with » | Fix (D5) |
| 10 | « etc » without its period at a sense's end | 35 / 16 (and « vs », « Mrs », « e.g »: 3 / 2); es-fr 28, en-fr 57, en-es 47 rows | « qué » (37) « …cómo, cuándo, etc »; « tanto » (76) « so much, long, hard, often, etc »; « actual » (356) « of the current month, year, etc » | Leave here: `reduce_common.clean_gloss`, every pair (D6) |
| 11 | Straight quotes | `"` 38 / 21; `'…'` 2 / 1; “ ” already 62 / 32; the rule changes 38 / 21 rows and 6 expressions | « pero » (20) « but (instance of saying "but") »; « otro » (73) « "Not again!" or "What, again?" »; « tener » (79) « (e.g. to "hold the power to", …) » | Fix the double ones (D5); leave the single (D6) |
| 12 | A row opening on an unexpected part of speech or sense | 3,628 / 1,487 rows have two runs or more; 39 / 35 put an open-class run before a function word's, most reading right (« más », « ya », « ahora ») | « hasta » (43) « even » before « until »; « primero » (202) the noun « former (…) » before « first »; « estado » (68) « country, land » before « state »; « yo » (28) « first-person singular pronoun in the nominative case, I » | Leave (D6); the order for the owner (Q4) |
| 13 | A sense whose label is dropped | obsolete/archaic 252 / 99 (98 / 28 open the row; 77 / 19 rows hold nothing else); dated/historical/rare 474 / 155; regional 2,725 / 882; register 2,031 / 718 | « o » (21) « …; where » (obsolete); « ese » (62) « that; hello » (Mexico, informal); « buena » (139) « inheritance » (obsolete; its form-of pointer to « bueno » never lends) | For the owner (Q1) |
| 14 | Upstream text | — | « a » (6) « indiference »; « otro » (73) « Otra vez! » with no ¡; « pero » (20) « well well, so, well » | Leave (D6) |
| 15 | Found: an example sentence after a line break | 2 / 0 | « canino » (18202) « …hungry as a hog Marcos siempre estaba canino… »; « tembleque » (49440) | Fix (D5) |

The rules D2–D5 together, on the real reduction: **268 rows change, 111 of the top 10,000** —
250 / 104 glossed by their own senses and 18 / 7 borrowed from a pointer's target (« tu », « muy »,
« na », « val », « aver », « cártel », « weon »; « alv », « agora », « ná », « awa », « wawa »…); the
first sense of 102 / 27. **9 lemmas gain a gloss and none loses one**: « cosita » (9065), « cajita »,
« chiquillo », « ramita » and « québec » through D2, « mui », « vien », « kiero » and « pid » through D3.
**332 expressions change** (330 for D5, nearly all its lower case; 2 for D2) and **25 gain a gloss**
through D3 — the apocopic numerals and ordinals (« cincuenta y un » « fifty-one », « vigésimo primer »
« twenty-first »), « cuando quier » and « po favó » — none losing one; the runs of 46 rows move. Rule
by rule, alone: D2 22 / 11 rows (5 / 1 of them gaining a gloss); D3 38 / 18 rows and 4 lemmas gained
(35 / 15 shortened forms, 3 / 3 case forms); D4 1 / 1 (« como »); D5 219 / 87 rows and 330
expressions. Coverage cannot fall below today's 93.0 / 86.5 / 76.5 %: no lemma loses its gloss, one
of the top 10,000 gains one (« cosita ») and four of the top 20,000. These are a prototype's
figures: the pull request measures again on its own code, and the sample (D8) is drawn from it.
Measured again on the implementation (`read_as_meanings`, es-en reduced from its pinned release):
every figure above holds, rule by rule and together, the census of D5 included (202 / 69 senses
lowered, 43 « The » kept), and its `gloss.tsv`, `senses.tsv` and `mwe.tsv` are the prototype's byte
for byte; with every rule off, the pre-pass reproduces the committed tables before it byte for byte.

## Decisions

### D1 — es-en's rules, in the English edition, before the etymology merging

Every rule is the English edition's and runs on the entries before the shared rules read them: one
pre-pass, `english.read_as_meanings(src, dst)` in `reduce_edition_en.py`, which `reduce-es-en.py`
runs after `without_letter_headwords` and before `merge_same_pos_etymologies`, plus the pointer
wordings D3 adds to `_FORM_OF`. The pass rewrites a sense's `glosses`, moves a few entries and
reads the cut file at most four times (the targets D3 looks up, the function words D4 needs); a line
it cannot read is written as it is, as the other pre-passes do.

Why not `reduce_common.py`: it is every pair's, so editing it re-pins en-fr, es-fr and en-es — and
the French Wiktionary does not write these layouts (no `glosses` chain under a label, no « apocopic
form of »). Why not a field of `Edition`: the class is `reduce_common.py`'s, the same re-pin. Why a
pre-pass and not a pass over the reduced glosses: the shared cleaning would undo some of it (it
strips a sense's final periods, so « whether...or... » loses its last ellipsis unless the dots are
already « … »), and the senses must be meanings before the round-robin picks eight. The expressions
read the same entries, so they get the same rules; the translation-table fallback (the Spanish
Wiktionary's words) is not touched.

Where a shared fix is better, it is named and left out (D6): « etc »'s period, a card that reads a
token's capital.

### D2 — A nested sense under a label or a pointer is read by its own gloss

kaikki writes a nested sense with its parents' glosses first, and the shared rules read
`glosses[0]`, so a nested sense reads as its top parent. When the parent is a meaning — listed
alone or not (« now, right now, … » over « by now »; « to make » over « to create, to build »;
« half (…) » over its kinds) — that is right and stays. When it is not, the sense is read by its own gloss, `glosses[-1]`:

- **a label**: the parent ends on a colon (« unit of measurement for: », « places in Peru: », « a
  kind of bird: ») or names senses (« Figurative senses. », « Senses relating to literal
  movement. »);
- **a pointer**: the parent is a pointer sense of the same entry (« apocopic form of suyo », tagged
  `alt-of`) or the edition's pointer wording (`_FORM_OF`, with D3's) — a shortened or respelled
  form (« prepositional form of se », « eye dialect spelling of ahuevonado »), a diminutive
  (« diminutive of casa » over « small house » and « house »; « cosita », « cajita », « chiquillo »,
  « ramita », « campanita » « notification bell », « figurita » « shorty »), an alternative form
  (« alternative form of Quebec: » over « Quebec (a province in eastern Canada) »).

« venir » reads « to come (move closer to some location …); to arrive; to come from, originate; … »,
« cusco » « a region of Peru; a province of Cusco; … », « sí »'s fifth sense « himself, herself,
itself, … », « casita » « small house; house; mother-in-law apartment; … ». A parent that is
neither a label nor a pointer is never replaced, and a nested pointer (« ellipsis of goma de
mascar ») stays a pointer. Of the 22 / 11 rows D2 moves, 9 / 6 are under a label and 13 / 5 under a
pointer; 5 / 1 had no gloss (« cosita », « cajita », « chiquillo », « ramita », « québec »).

### D3 — A shortened or respelled form, or a pronoun's case form, reads as its meaning

The English Wiktionary writes an apocope, an apheresis, a syncope, a prepositional form and a
pronunciation or eye-dialect spelling as a pointer, tagged (`alt-of`, `apocopic`, with `alt_of`) or
not (« apheretic form of estás »), and often writes the meaning after its target: « apocopic form
of mío, my », « apocopic form of mucho; very », « apocopic form of valle: valley », « apocopic form
of uno (“one”) », « apheretic form of papá (“dad”) ». The shared rules skip a tagged pointer — so
« mi » loses « my » to its Greek letter and its note name — and lend the target's senses only to a
word with no other sense, so « muy » borrows all of « mucho ». An untagged one is read as a meaning
(« tas », « toy »).

**Which senses.** A sense is a shortened or respelled form when its gloss opens on that pointer
wording — « apocopic form of », « apheretic form of », « syncopic form of », « prepositional form
of », « pronunciation spelling of », « eye dialect spelling of », in any case — tagged as a pointer
(70 senses) or not (17). The wording is the key, never a tag alone: « mal »'s adjective « amiss,
awry, off, wrong » is tagged `apocopic` and « nos »'s « first person nominative, prepositional and
vocative plural pronoun » `prepositional`, and both are meanings; the 8 senses tagged `alt-of` beside
a shortened form's tag under another pointer's wording (« obsolete form of treinta y un »,
« alternative form of hueveo ») stay the shared rules' pointers.

A **pronoun's case form** is the same layout under another wording: « accusative of él and usted
(when referring to a man), and a variant of ello in many constructions; him, you (formal), it,
that », « dative of nosotros: to us, for us », « dative of ellos and ellas; to them, for them » —
tagged `form-of`, so skipped: « lo » (14) reads only its article, « nos » (35) only two archaic
senses, « les » (71) only « the (plural) ». A pointer sense of a pronoun entry whose gloss names a
case (« nominative », « accusative », « dative », « genitive », « reflexive », « prepositional »,
« disjunctive ») « of » a word, then carries its meaning after a colon or a semicolon, is read by
that meaning — rule 1 only, nothing is lent to it: 9 senses, of these three lemmas alone (« me »,
« te », « la », « los » are forms, not glossed lemmas).

**Its target and its meaning.** The target is the word kaikki names in the sense's `alt_of` or
`form_of` (the first), whole: « voy a », « cincuenta y uno », « por favor », « malo bad ». Its meaning
is kaikki's `extra` for that target when there is one (« very », « one », « fifty-one », « I'm going
to », « evil »), else the gloss's text after the target, past a comma, a semicolon or a colon, or
within “ ” (« apocopic form of mío, my » → « my », « tuyo, your » → « your »). For a pronoun's case
form it is the text after the gloss's first colon or semicolon (« him, you (formal), it, that »: lo's
`extra` opens on the note). A sense with no pointer field names its target in its gloss — the text
after the wording up to a comma, a semicolon, a colon, an opening parenthesis or the end (« estoy »,
« adelante ») — and its meaning after it (« apheretic form of mamá (“mom”) » → « mom »). The target is
never the gloss's first word after the wording: « cincuenta y un » would read « fifty », « vigésimo
primer » « twentieth ».

The pre-pass replaces such a sense, in its place, by:
1. the meaning it carries: « my », « very », « valley », « one », « evil », « fifty-one », « him,
   you (formal), it, that »;
2. else its target's meaning senses in the same part of speech (« su » → « suyo »'s determiner
   senses; « alante » → « adelante »'s; « po favó » → « por favor »'s « please; you're welcome »),
   following the target's own pointer once when the target is only a form (« toy » → « estoy » →
   « estar »), from a target of three letters or more;
3. else nothing: `_FORM_OF` gains the six wordings above, so the sense is a pointer, as a tagged
   one already is (« seó », « apocopic form of seor »; « er », whose target has two letters).

**Three letters.** Rule 2 lends from a target of three letters or more, the bound every lending of
the shared rules keeps (`_MIN_BASE`; *A word that is only a form of another takes that word's
gloss*: a word of fewer than three letters gives none). Lifting it here would move two rows and no
other: « er » (5656), « pronunciation spelling of el », would gain « el »'s article senses, and
« sho » (21886) would add « yo »'s « I » after its two interjections. The price would be a second,
es-en-only exception to a requirement every pair holds — the reader cannot tell which rule lent a
gloss — for two respellings of the commonest words. Kept: that requirement holds as written,
« er » stays unglossed and « sho » reads « shush!, hush!; wow!, whoa! », as today. Rule 1 lends
nothing, so the bound is not its: a carried meaning is read whatever its target.

The new sense keeps the original's other tags and loses its pointer tags and fields. It holds for a
word that has other senses too: « mi » reads « my; mu, the Greek letter Μ, μ; mi », « tu » « your »,
« un » « an, a; one », « lo » « him, you (formal), it, that; neuter definite article… », « nos » « to
us, for us; us; ourselves, each other; first person … », « les » « to them, for them; to you all,
for you all (formal); you all (formal); them; the (plural) ». A carried or lent sense takes a place
in the round-robin: « mal » (123) gains its adjective's « evil » and loses « used as an intensifier,
very » from its eight; « güey » (26642) gains « buey »'s « ox, bullock, steer » and three more after
« dude, guy, buddy »; « callao » (6243) opens on « callado »'s « quiet, silent », its adjective's
entry coming first. The studied side is untouched — es-fr's reduction already reads `buen` as a
form of `bueno` (*A Spanish apocope reads as its full word*); `mi`, `tu`, `su`, `muy`, `lo`, `nos`
and `les` are lemmas of their own, glossed here.

### D4 — A function word does not open on a place's name

« como » has three entries — adverb, conjunction, preposition — and the round-robin meets `Como`'s
proper-noun entry (the Italian city) first, in file order: the row opens on the city, twice. The
pre-pass writes the `name` entries of a headword with an initial capital — not all capitals, which
is an acronym and `_acronym`'s — after every other line, when the lower-case headword has an entry
whose kaikki part of speech is `prep`, `conj`, `pron`, `det` or `article`: the city's senses come
after « as (…) », if the eight still hold them. A headword's moved lines stay consecutive and in
their order, so `merge_same_pos_etymologies`, which reads a headword's etymologies as one run of
lines, still finds them together. Nothing else moves: in the whole table this rule changes « como »
alone.

Why not all capitals: « pr » (3980) has only pointers — `PR`'s « initialism of Puerto Rico », then
its own « abbreviation of por », « para », « pero » — and borrows from the first in file order:
« Puerto Rico (…) » today. Moving `PR`'s line after every other would put « por » first and open
« pr » on « by; for (…); through, … », a row no rule here sets out to change.

Why so narrow: a rule for every proper noun was measured both ways. Writing the names last everywhere
changes 243 / 124 rows — the 242 / 124 that open on a proper noun, and « tejas », borrowed, then
« roof tile; … » instead of « Texas (…) » — and puts « Brazil », « China », « María » after
« brazilwood », « pebble », « magpie »; writing personal names last helps « amor », « máximo »,
« norma » and opens « jorge » on « cockchafer », « fernando » on « fernet with Coca-Cola », « carmen »
on « a type of house in Granada ». Which reading a reader meets depends on the token's capital, which
the engine knows and the pack does not: that is the better fix, shared (D6, Q3).

### D5 — One English typography

The pre-pass writes each sense's own gloss, in this order:

- **No text after a line break**: a gloss stops at its first line (« canino »'s example sentence).
- **No numbered sense**: a bracketed « [sense N] » goes first, then a parenthesis still naming
  « sense N » goes whole. So jurado's « judge (member of a jury [sense 2]; officiator of a
  competitive event) » keeps its parenthesis (« judge (member of a jury, officiator of a competitive
  event) », the sense's own semicolon written as a comma by the shared rules), « juror, juryman,
  juryperson (member of a jury [sense 1]) » reads « juror, juryman, juryperson (member of a jury) »,
  and ya's « (difference from sense 4 depends on context) » goes.
- **The edition's description in lower case**, as 98.3 % of its senses are written: a sense opening
  on one of a closed list — « A », « An », « The », « Any », « One », « Some », « Certain »,
  « Various », « Either », « Used », « Said », « Indicate », « Indicates », « Expresses »,
  « Denotes », « Forms », « Replaces », « Introduces », « Refers », « Related », « Relating »,
  « Pertaining », « Of », « Having », « In », « To », « Someone », « Something », « Term »,
  « Expression », « Interjection » — then a space and a letter, opens in lower case: the openers of
  the 202 senses measured. « The » before a capital stays, whatever follows it: a title or a name
  (« The Nutcracker (ballet) », « The Inca Empire »), and the species the edition writes the same
  way (« The Eurasian treecreeper »: 37 expression senses and 2 lemma senses, which the rule cannot
  tell from a title). A word followed by no letter stays (« A (highest grade in testing) »). Any
  other capital stays: proper adjectives, months, « Earth », and the odd common word the edition
  capitalised (« Civility » under « policía »). Whether the card capitalises a sense for display
  stays a renderer option (change 23, M9): the data is now one case to decide it on.
- **One ellipsis**: `...` becomes `…`; between two words it is spaced on both sides (« not …
  anything », « both … and », « either … or », « sometimes … other times »); anywhere else the
  spacing written stays (« whether … or… », « let's see… », « my name is …, I am … »). A `…` is no
  period, so the shared cleaning keeps a sense's last one: « ahora »'s « whether … or… » keeps it, and
  « a la mierda » reads « to hell with… » where it read « to hell with ».
- **Curly double quotes**: a sense with an even number of `"` has them paired “ ” in order (« “Not
  again!” or “What, again?” »); an odd number stays as written. Single quotes stay: ’ is also the
  apostrophe, and 2 rows hold quoting ones.

None of them moves a word otherwise. Measured on the changed rows, no row of the card (`rowGloss`)
ends on an opening mark; `row-gloss-tables.spec.ts` stays the gate.

### D6 — Left, for the owner or for a shared fix

- **« etc » and abbreviations at a sense's end** lose their period in every pair:
  `clean_gloss`'s `.rstrip(".:")` (es-en 35 rows, es-fr 28, en-fr 57, en-es 47). Keeping the period of a
  closing abbreviation is one rule for every pair — a follow-up that re-pins all four, named in the
  pull request. A mid-sense « etc) » is the source's (« tiempo »).
- **A proper noun before a common word** (242 / 124 rows; es-fr 114 / 49): a case-aware card — the
  engine knows a token's capital in mid-sentence (`mid_sentence`, *A Spanish document's names are set
  aside*) and could open on the proper-noun run for « China » and on the common word for « china » —
  engine and every pair, a follow-up (Q3).
- **The part of speech a row opens on** (« hasta », « primero ») is the page's entry order. No
  committed table says which part of speech a lemma is most often; UD Spanish-GSD's counts would, as a
  studied-side table es-fr's reduction writes for every Spanish pair — a follow-up (Q4). The order of
  senses within a part of speech (« estado ») and the edition's wording (« yo », « pero ») are
  upstream.
- **Labels** (obsolete, regional, register): a format and a product call (Q1); the tags are in the
  extract, so any answer is a rule of this edition.
- **Upstream text** — « indiference », « Otra vez! » without ¡, IPA in « bueno »'s usage note (in
  « seseo », the IPA is the meaning): corrected on the English Wiktionary, they arrive with es-en's
  next update. A table of corrections would be glosses written here, not by the source.
- **« q », « k », « t »** borrow « que »'s and « tiempo »'s senses through an abbreviation (Q2); the
  other 188 rows that borrow through an abbreviation, an initialism, an acronym or a clipping read
  right (« ue » « European Union », « adn » « DNA », « tmb » « also, too »).

### D7 — es-en re-pinned alone, at its snapshot

`build.sh --reduce es-en` from `lingua-pack-sources-es-en-2026.10.08`: the pinned extract and the
derived translations, nothing fetched beyond the release's assets. The pin keeps its `snapshot`, its
`studied` record and its `sources` byte for byte — the legacy `kaikki` record of the extract (asset,
sha256, size, dates, address), `kaikki-es` and its derived file, `wordfreq`; its `reducer` digest
moves (`reduce-es-en.py`, `reduce_edition_en.py`), and with it `pack_version`
(`2026.10.08+<digest[:7]>.08034dc`, the studied digest unchanged), the pack's sha256 and size.
`gloss_coverage.py --pair es-en` holds `FLOORS["es-en"]`; the figures in `tables/es-en/README.md`
and `SOURCES.md` follow. en-fr's, es-fr's and en-es's tables and pins are byte for byte unchanged —
their rule digests do not name `reduce_edition_en.py` — and the reduce job reproduces every
committed byte.

### D8 — Every moved line read, and a sample for the owner

`LINGUA_BLESS=1 cargo test -p lingua-wasm --test es_en_baseline` re-blesses the golden, and `yarn
vitest run test/word-card-es-en.spec.ts -u` the English card's snapshot; `es_en_baseline.rs`'s shown
first page of « viaje » follows D5 (« a state of hallucination… »). `yarn vitest run
test/row-gloss-tables.spec.ts` runs without `-u`: it is the gate, its French snapshot
(`test/baseline/selection-rows-fr.txt`) must pass as committed — re-blessing it would hide a French
row that moved — and no row of any pair may end on an opening mark. (`lingua-pack-update` re-blesses
both specs on a dictionary update's branch, where the French tables may move; here they may not.)
The pull request lists every changed line of the golden and of the card's snapshot, with the rule
that moved it, and a before/after sample: every changed row of the top 10,000 (about 111) with its
first differing sense, the changed expressions by rule, and 30 rows drawn from the rest — what the
owner reviews (M9). The sample names the rows where a carried or lent sense crowds another out of
the eight or changes the opening (« mal », « güey », « callao »), the D2 parents ending on a colon
that were meanings (« audición »'s « public entertainment, show: » over « concert », « reading »,
« recital »), and the 18 / 7 borrowed rows that move.

### D9 — With M20, whichever is settled first; after change 38

The pipeline's order is fixed: D2–D5 run before `merge_same_pos_etymologies`, and D5's text before
`clean_gloss` and its long-parenthesis bound. es-en's tables are therefore a function of these rules
and the two settings, whatever order the pull requests land in; whichever lands second re-pins es-en
on both.

- **The bound** (`LONG_PARENTHESIS`, 0 or 40): at 40 it removes some parentheses D5 would edit
  (« (difference from sense 4 depends on context) » goes either way; « [sense 1] » is a bracket, only
  D5 removes it). D5 shortens a parenthesis by at most two characters per ellipsis, so one at the
  threshold may cross it: counted in whichever pull request lands second.
- **The merging** (`MERGE_SAME_POS_ETYMOLOGIES`): D2 and D3 act per sense, D4 on another headword's
  lines, kept consecutive, so a merged entry keeps them; merging reorders senses 2–8, these rules
  what a sense says.
- **The samples** of change 21 (tasks 2.2 and 5.1) were drawn on today's tables. If this change lands
  first, they are drawn again on its tables before the owner picks (the 1,057 and 247 rows each
  setting changes move by at most the 268 this change touches); if M20 is settled first, this change's
  sample is drawn with the chosen values.

**Change 38** (`migrate-lingua-pack-sources-to-raw-dumps`) landed first (#804), and this change
composes with it through its legacy-record path: 38 re-pins no pair and keeps es-en's `kaikki`
record as recorded (its D5), and this change re-pins es-en from that same pinned extract, its
`sources` byte for byte (D7, task 2.1) — neither moves the other's pin. 38's equivalence for es-en
(identical tables from the extract and from the English dump of 2026-10-03) was measured under
today's rules. These rules read the same fields of the same entries, but D4, D3's lending and the
round-robin read file order, and 37 entries or runs of entries stand elsewhere in the dump: the
implementation pull request reduces es-en from that dump as well while kaikki still serves it
(`pack_report.py --identical`, recorded in `SOURCES.md` beside 38's measurement); otherwise es-en's
next update, its first from the dump, names any difference in its report. The implementation builds
on 38's `SOURCES.md`, `tables/es-en/README.md` and `test_reduce_editions.py` as merged.

## For the owner

- **Q1 — Labels.** (a) Prefix a sense with its labels as the edition prints them — « (obsolete)
  where », « (Mexico, informal) hello »: 3,965 / 1,314 rows, 6,210 senses, 19.4 characters each on
  average; the tags are unscoped (« vosotros » is tagged obsolete for Latin America). (b) Leave out an
  obsolete or archaic sense where the word has a pointer to lend from: 32 / 12 rows, « buena » then
  reads « good (of good quality); … », « echo » « to throw, toss, cast… ». (c) Neither, as today. (a) and (b) combine.
- **Q2 — Single letters.** « q » and « k » (que, in chat) and « t » (tiempo, first of four targets)
  keep their borrowed gloss, or a single letter borrows none (*A letter of the alphabet SHALL gloss no
  Spanish word*)?
- **Q3 — Proper nouns before common words**: accepted until the case-aware card (a shared follow-up),
  or a data rule now despite D4's measured losses?
- **Q4 — The part of speech a row opens on**: accepted as the page's order until a studied-side
  frequency exists?

An answer that is a rule of the English edition joins this change before it merges (es-en re-pinned
again); any other is named as a follow-up in the pull request.

## Risks / Trade-offs

- **A rule reads a meaning as a layout** (a parent ending on a colon that was a meaning) → D2
  replaces a parent only when it ends on a colon, names senses or is a pointer, and keeps the nested
  senses' own glosses: « audición »'s « public entertainment, show: » gives way to its kinds,
  « concert », « reading », « recital »; every changed row of the top 10,000 is in the sample.
- **A tag read as a pointer** (« mal »'s « amiss, awry, off, wrong », tagged `apocopic`) → D3 keys on
  the pointer's wording alone, tested on « mal » and « nos ».
- **A target parsed from garbled text** (« malo bad », « es que in the Madrid dialect ») → the meaning
  kaikki records wins, the target is looked up whole, and a target that names no word with a meaning
  in that part of speech leaves the sense a pointer.
- **A lent sense reads oddly** (« su » ← « suyo »: « his, hers, its, to her; … ») → it is the target's
  own gloss, and the sample shows it; a wording the owner wants is change 33's or Q-listed.
- **The lower case on a proper word** → the closed list and the « The » exception; the census of the
  202 senses, and of the 43 senses the exception keeps, is in the pull request.
- **A golden that moves** → re-blessed in the pull request, each line read (D8); the French rows'
  snapshot is run, never re-blessed.

## Migration Plan

No release: es-en's tables, its pin and the tests that pin it, all inert until change 34 lists es-en.
