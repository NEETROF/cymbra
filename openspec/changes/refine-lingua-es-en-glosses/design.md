# Design — refine-lingua-es-en-glosses

## Context

See proposal.md (Why). What exists:

| Where | What |
|---|---|
| `reduce-es-en.py` | `native_fields` (the extract cut to `word`, `pos`, `senses` and each sense's `glosses`, `tags`, `form_of`, `alt_of`) → `without_letters` → `english.without_letter_headwords` → `english.merge_same_pos_etymologies` (off) → `common.native_tables(…, edition=english.EN, fallbacks=[(direct, list)])` |
| `reduce_edition_en.py` | `EN`: `_FORM_OF` (untagged pointer wordings), `_LETTER`, pointer tags `form-of`/`alt-of` and fields `form_of`/`alt_of`, `capitalised=False`, `long_parenthesis=LONG_PARENTHESIS` (0); the two pre-passes above. Loaded by es-en alone: editing it re-pins es-en and no other pair (change 6 D2, change 21 D5) |
| `reduce_common.py` | `_read_entries` reads a sense by `glosses[0]` and skips it when `_is_form_of`; `clean_gloss` (notes, the long-parenthesis bound, whitespace, `.strip(" ;,").rstrip(".:")`, the cut); `_join_senses_by_pos` (round-robin across a word's entries in file order, up to eight senses, grouped by part of speech in the order they first appear); a word with no meaning sense borrows its pointer's target's senses in the same part of speech; an all-capitals headword does not gloss a lower-case word that has entries (`_acronym`). Every pair loads it: editing it re-pins en-fr, es-fr, es-en and en-es |
| kaikki's senses | A nested sense carries its parents' glosses first: `glosses: ["Figurative senses.", "to come from, originate"]`; its parent may also appear alone as a sense (`["now, right now, …"]` over « by now ») or not (`["to make"]`, `["Figurative senses."]`) |
| `tables/es-en/` | 31,876 glossed lemmas, 15,490 expressions; pinned at `2026.10.08` from `lingua-pack-sources-es-en-2026.10.08`; coverage 93.0 / 86.5 / 76.5 % against `FLOORS["es-en"]` 87.6 / 77.2 / 63.7 |
| Change 23 | `crates/lingua-wasm/tests/es_en_baseline.rs` + `baseline/es-en.golden` (the reference's probes and 40 lemmas, among them `a`, `su`, `pero`, `o`, `ya`, `qué`, `bien`, `ahora`, `nada`, `ni`, `otro`, `tanto`, and the card of « viaje », its shown gloss a constant); `test/word-card-es-en.spec.ts` + `test/baseline/word-card-es-en.txt`; `test/row-gloss-tables.spec.ts` (no row of any pair ends on an opening mark; the French rows pinned); both re-blessed by `lingua-pack-update` |
| Change 21 | Tasks 2.2 and 5.1 open: the owner's two settings (M20), sampled on today's tables |

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

Rows are the lemmas of `tables/es-en/gloss.tsv` (31,876); "top 10k" those ranked 1–10,000 in
`tables/es/freq.tsv` (8,646 glossed). Text classes are counted on the committed table; structural
ones (nested senses, pointers, tags) on the pinned extract's senses, as `reduce-es-en.py` cuts them;
each rule's effect on a replica of the shared reduction that reproduces the 30,918 rows the
edition's own senses gloss, byte for byte with their runs (the 958 borrowed or translation-table
rows are not replayed: the pull request measures on the real reduction). Classes overlap.

| # | Class | Rows: whole / top 10k | Examples | Decision |
|---|---|---|---|---|
| 1 | A sense-group label or a list's introduction read as a sense | 9 / 6 | « venir » (881) « Senses relating to literal movement; Figurative senses »; « lata » (4342) « …; unit of measurement for »; « cusco » (7161) « places in Peru » | Fix (D2) |
| 2 | A sense nested under a pointer read as the pointer | 4 / 2 | « su » (15) « apocopic form of suyo »; « sí » (96) « prepositional form of se »; « weón » (24897) « pronunciation spelling of huevón » | Fix (D2) |
| 3 | A shortened or respelled form: its pointer kept as a meaning, or the meaning it carries lost | wording kept 13 / 5; apocopic senses under 12 / 7 lemmas; the rule changes 27 / 9 | « tas » (7587) « apheretic form of estás »; « mi » (22) « mu, the Greek letter Μ, μ; mi » (« my » lost); « muy » (32) « much, a lot, far, way, many times; very; … » (borrowed from « mucho ») | Fix (D3) |
| 4 | An abbreviation tagged `alt-of` lending its target's gloss | 227 / 88 of the 851 / 175 borrowed rows; single letters 3 / 3 | « q » (373) « who, that; that, whom, which » (que); « k » (910) « that; than; … »; « t » (610) « time; …; weather (…) » (tiempo, the first of four targets). « qe » (6499) « Querétaro (…) » is the acronym `QE`'s own gloss (*An acronym with no common word*) | Leave; « q », « k », « t » for the owner (Q2) |
| 5 | A proper noun opening a lower-case lemma's gloss | 242 / 124 (es-fr: 114 / 49); under a function word 1 / 1 | « como » (17) « Como (a city…); Como (a province…); as … »; « amor » (198) « a surname; love »; « chile » (328) « Chile (…); a chili pepper » | Fix « como » (D4); the rest for the owner (Q3) |
| 5b | A common noun's sense in an English capital | 1,164 / 236 rows open on a capitalised sense that is no proper noun's | « tierra » (236) « Earth (planet…) »; « mayo » (324) « May »; « julio » (300) « July » | Leave: English capitalises the planet, months, languages and demonyms |
| 6 | The edition's description in a capital, mid-gloss or first | 170 / 58 (202 / 69 senses, 59 / 8 opening the row); 314 expressions | « a » (6) « …; Used before words referring to people… »; « se » (9) « A reflexive or reciprocal pronoun… »; « ni » (48) « Used when negating two or more elements… » | Fix (D5) |
| 7 | A reference to a numbered sense of the source | 2 / 2 | « ya » (26) « (difference from sense 4 depends on context) »; « jurado » (1925) « (member of a jury [sense 1]) » | Fix (D5) |
| 8 | IPA inside a sense | 2 / 1 | « bueno » (104) « …with the pronunciation /bweˈno/, rather than /ˈbweno/ »; « seseo » (53052) « …as /s/ rather than /θ/ » | Leave (D6) |
| 9 | Ellipses written three ways | `...` 12 / 10 (9 / 7 unspaced, 3 / 3 spaced); `…` inside a sense 2 / 2; 5 expressions; the rule changes 9 / 9 | « nada » (47) « not...anything »; « tanto » (76) « both ... and »; « o » (21) « either … or »; « ahora » (45) « whether...or... » loses its last one to the final-period strip | Fix (D5) |
| 10 | « etc » without its period at a sense's end | 35 / 16 (and « vs », « Mrs », « e.g »: 3 / 2); es-fr 28, en-fr 57, en-es 47 rows | « qué » (37) « …cómo, cuándo, etc »; « tanto » (76) « so much, long, hard, often, etc »; « actual » (356) « of the current month, year, etc » | Leave here: `reduce_common.clean_gloss`, every pair (D6) |
| 11 | Straight quotes | `"` 38 / 21; `'…'` 2 / 1; “ ” already 62 / 32 | « pero » (20) « but (instance of saying "but") »; « otro » (73) « "Not again!" or "What, again?" »; « tener » (79) « (e.g. to "hold the power to", …) » | Fix the double ones (D5); leave the single (D6) |
| 12 | A row opening on an unexpected part of speech or sense | 3,628 / 1,487 rows have two runs or more; 39 / 35 put an open-class run before a function word's, most reading right (« más », « ya », « ahora ») | « hasta » (43) « even » before « until »; « primero » (202) the noun « former (…) » before « first »; « estado » (68) « country, land » before « state »; « yo » (28) « first-person singular pronoun in the nominative case, I » | Leave (D6); the order for the owner (Q4) |
| 13 | A sense whose label is dropped | obsolete/archaic 252 / 99 (98 / 28 open the row; 77 / 19 rows hold nothing else); dated/historical/rare 474 / 155; regional 2,725 / 882; register 2,031 / 718 | « o » (21) « …; where » (obsolete); « ese » (62) « that; hello » (Mexico, informal); « buena » (139) « inheritance » (obsolete; its form-of pointer to « bueno » never lends) | For the owner (Q1) |
| 14 | Upstream text | — | « a » (6) « indiference »; « otro » (73) « Otra vez! » with no ¡; « pero » (20) « well well, so, well » | Leave (D6) |
| 15 | Found: an example sentence after a line break | 2 / 0 | « canino » (18202) « …hungry as a hog Marcos siempre estaba canino… »; « tembleque » (49440) | Fix (D5) |

The rules D2–D5 together, on the replica: **251 rows change, 101 of the top 10,000; the first sense
of 91 / 21**; one lemma gains a gloss (« er »), none loses one; about 325 expressions change, nearly
all for D5's lower case. Coverage cannot fall below today's 93.0 / 86.5 / 76.5 %.

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
  `alt-of`) or the edition's pointer wording (`_FORM_OF`, with D3's).

« venir » reads « to come (move closer to some location …); to arrive; to come from, originate; … »,
« cusco » « a region of Peru; a province of Cusco; … », « sí »'s fifth sense « himself, herself,
itself, … ». A parent that is neither a label nor a pointer is never replaced, and a nested pointer
(« ellipsis of goma de mascar ») stays a pointer.

### D3 — A shortened or respelled form reads as its meaning

The English Wiktionary writes an apocope, an apheresis, a syncope, a prepositional form and a
pronunciation or eye-dialect spelling as a pointer, tagged (`alt-of`, `apocopic`, with `alt_of`) or
not (« apheretic form of estás »), and often writes the meaning after its target: « apocopic form
of mío, my », « apocopic form of mucho; very », « apocopic form of valle: valley », « apocopic form
of uno (“one”) », « apheretic form of papá (“dad”) ». The shared rules skip a tagged pointer — so
« mi » loses « my » to its Greek letter and its note name — and lend the target's senses only to a
word with no other sense, so « muy » borrows all of « mucho ». An untagged one is read as a meaning
(« tas », « toy »).

The pre-pass replaces such a sense, in its place, by:
1. the meaning it carries — the text after its target, after a comma, a semicolon or a colon, or
   in quotation marks: « my », « very », « valley », « one », « dad »;
2. else its target's meaning senses in the same part of speech (« su » → « suyo »'s determiner
   senses; « alante » → « adelante »'s), following the target's own pointer once when the target is
   only a form (« toy » → « estoy » → « estar »);
3. else nothing: `_FORM_OF` gains these wordings (« apocopic », « apheretic », « syncopic »,
   « prepositional form of », « pronunciation spelling of », « eye dialect spelling of »), so the sense
   is a pointer, as a tagged one already is.

The new sense keeps the original's other tags and loses its pointer tags and fields. It holds for a
word that has other senses too: « mi » reads « my; mu, the Greek letter Μ, μ; mi », « tu » « your »,
« un » « an, a; one ». The studied side is untouched — es-fr's reduction already reads `buen` as a
form of `bueno` (*A Spanish apocope reads as its full word*); `mi`, `tu`, `su` and `muy` are lemmas of
their own, glossed here.

### D4 — A function word does not open on a place's name

« como » has three entries — adverb, conjunction, preposition — and the round-robin meets `Como`'s
proper-noun entry (the Italian city) first, in file order: the row opens on the city, twice. The
pre-pass writes the `name` entries of a capitalised headword after every other line when the
lower-case headword has an entry that is a preposition, a conjunction, a pronoun, a determiner or an
article: the city's senses come after « as (…) », if the eight still hold them. Nothing else moves: on
the top 10,000 this rule changes « como » alone.

Why so narrow: a rule for every proper noun was measured both ways. Writing the names last everywhere
changes 242 / 123 rows and puts « Brazil », « China », « María » after « brazilwood », « pebble »,
« magpie »; writing personal names last helps « amor », « máximo », « norma » and opens « jorge » on
« cockchafer », « fernando » on « fernet with Coca-Cola », « carmen » on « a type of house in
Granada ». Which reading a reader meets depends on the token's capital, which the engine knows and
the pack does not: that is the better fix, shared (D6, Q3).

### D5 — One English typography

The pre-pass writes each sense's own gloss:

- **The edition's description in lower case**, as 98.3 % of its senses are written: a sense opening
  on « A », « An », « The », « Any », « One », « Some », « Certain », « Various », « Either »,
  « Used », « Said », « Indicate(s) », « Expresses », « Denotes », « Forms », « Replaces »,
  « Introduces », « Refers », « Related », « Relating », « Pertaining », « Of », « Having », « In »,
  « To », « Someone », « Something », « Term », « Expression » or « Interjection », then a space and
  a letter, opens in lower case — the openers of the 202 senses measured. « The » before a capital is
  a title or a name and stays (« The Nutcracker (ballet) », « The Inca Empire »), and a word followed
  by no letter stays (« A (highest grade in testing) »). Any other capital stays: proper adjectives,
  months, « Earth », and the odd common word the edition capitalised (« Civility » under
  « policía »). Whether the card capitalises a sense for display stays a renderer option (change 23,
  M9): the data is now one case to decide it on.
- **One ellipsis**: `...` becomes `…`; between two words it is spaced on both sides (« not …
  anything », « both … and », « either … or », « sometimes … other times »), elsewhere the spacing
  written stays. A `…` is no period, so the shared cleaning keeps a sense's last one (« whether …
  or … »).
- **Curly double quotes**: a sense with an even number of `"` has them paired “ ” in order (« “Not
  again!” or “What, again?” »); an odd number stays as written. Single quotes stay: ’ is also the
  apostrophe, and 2 rows hold quoting ones.
- **No numbered sense**: « [sense N] » goes, and a parenthesis naming « sense N » goes whole
  (« indicates completion of an action »; « juror, juryman, juryperson (member of a jury) »).
- **No text after a line break**: a gloss stops at its first line (« canino »'s example sentence).

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
  other 224 abbreviation borrowings read right (« ue » « European Union », « adn » « DNA », « tmb »
  « also, too »).

### D7 — es-en re-pinned alone, at its snapshot

`build.sh --reduce es-en` from `lingua-pack-sources-es-en-2026.10.08`: the pinned extract and the
derived translations, nothing fetched beyond the release's assets. The pin keeps its snapshot,
sources and `studied` record; its rule digest moves (`reduce-es-en.py`, `reduce_edition_en.py`), and
with it `pack_version` (`2026.10.08+<digest[:7]>.08034dc`, the studied digest unchanged), the pack's
sha256 and size. `gloss_coverage.py --pair es-en` holds `FLOORS["es-en"]`; the figures in
`tables/es-en/README.md` and `SOURCES.md` follow. en-fr's, es-fr's and en-es's tables and pins are
byte for byte unchanged — their rule digests do not name `reduce_edition_en.py` — and the reduce job
reproduces every committed byte.

### D8 — Every moved line read, and a sample for the owner

`LINGUA_BLESS=1 cargo test -p lingua-wasm --test es_en_baseline` and `yarn vitest run
test/word-card-es-en.spec.ts test/row-gloss-tables.spec.ts -u` re-bless the golden and the snapshot;
`es_en_baseline.rs`'s shown first page of « viaje » follows D5 (« a state of hallucination… »). The
French snapshot of rows does not move. The pull request lists every changed line of both, with the
rule that moved it, and a before/after sample: every changed row of the top 10,000 (about 101) with
its first differing sense, the changed expressions by rule, and 30 rows drawn from the rest — what
the owner reviews (M9).

### D9 — With M20, whichever is settled first

The pipeline's order is fixed: D2–D5 run before `merge_same_pos_etymologies`, and D5's text before
`clean_gloss` and its long-parenthesis bound. es-en's tables are therefore a function of these rules
and the two settings, whatever order the pull requests land in; whichever lands second re-pins es-en
on both.

- **The bound** (`LONG_PARENTHESIS`, 0 or 40): at 40 it removes some parentheses D5 would edit
  (« (difference from sense 4 depends on context) » goes either way; « [sense 1] » is a bracket, only
  D5 removes it). D5 shortens a parenthesis by at most two characters per ellipsis, so one at the
  threshold may cross it: counted in whichever pull request lands second.
- **The merging** (`MERGE_SAME_POS_ETYMOLOGIES`): D2 and D3 act per sense, D4 on another headword's
  lines, so a merged entry keeps them; merging reorders senses 2–8, these rules what a sense says.
- **The samples** of change 21 (tasks 2.2 and 5.1) were drawn on today's tables. If this change lands
  first, they are drawn again on its tables before the owner picks (the 1,057 and 247 rows each
  setting changes move by at most the 251 this change touches); if M20 is settled first, this change's
  sample is drawn with the chosen values.

Change 38 (`migrate-lingua-pack-sources-to-raw-dumps`) re-pins no pair; the rules read the same
entries from a dump at es-en's next update.

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
  senses' own glosses, so no meaning is lost; every changed row of the top 10,000 is in the sample.
- **A lent sense reads oddly** (« su » ← « suyo »: « his, hers, its, to her; … ») → it is the target's
  own gloss, and the sample shows it; a wording the owner wants is change 33's or Q-listed.
- **The lower case on a proper word** → the closed list and the « The » exception; the census of the
  202 senses is in the pull request.
- **A golden that moves** → re-blessed in the pull request, each line read (D8).

## Migration Plan

No release: es-en's tables, its pin and the tests that pin it, all inert until change 34 lists es-en.
