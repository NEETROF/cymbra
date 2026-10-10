# Design — refine-lingua-fr-en-glosses

## Context

See proposal.md (Why). What exists, on `main` at `34fa74b2` (change 48's implementation, #855):

| Where | What |
|---|---|
| `reduce-fr-en.py` | French's reference reducer. Studied side: `forms.tsv`, `freq.tsv`, `grammar.tsv`, then `level.tsv` (change 46: `estimated_levels`, English's sizes in rank order, `no_level`'s five rules read from the section, « never from a pair's glosses »). Native side (change 48, `native_side`): `native_fields` → `common.without_letter_senses` → `english.without_letter_headwords` → `english.read_as_meanings` (23b) → `english.merge_same_pos_etymologies` (off) → `expression_senses` (pointer-only senses and `LEFT_OUT`'s `à la` out) → `common.native_tables(…, fallbacks=[])` and `split_words`. GSD's training and development sections are read for `(form, lemma)` counts (`read_gsd_counts`) |
| `reduce_common.py`, `reduce_edition_en.py` | The shared rules (every pair's digest) and the English edition's (es-en's and fr-en's digests): editing either re-pins es-en at least. Not edited here |
| `pack_sources.py split` | Files a reduction's tables; for a reference pair, writes `tables/<studied>/lexical.tsv` from its `gloss.tsv` (*A studied language's tables are kept once*). In no rule digest |
| `crates/lingua-pack/src/tables.rs` | `check_studied`: a studied folder's `lexical.tsv` must be exactly its reference's glossed lemmas, or the build of every pair of that language fails |
| `tables/fr-en/`, `tables/fr/` | 30,055 glossed lemmas — French's dictionary words — and 17,479 expressions; coverage 93.6 / 86.8 / 76.3 % (4,678 / 8,684 / 15,252) against `FLOORS["fr-en"]` 91.9 / 85.1 / 74.4; 8,302 levelled lemmas (A1 ranks 1–1,080 … C2 9,413–10,762); the pack 2,527,222 B, carrying no lexical table; pinned at snapshot `2026.10.09` from `lingua-pack-sources-fr-en-2026.10.09` |
| Change 41's names rule | `document_names` (`engine.rs`): a form never written in lowercase, capitalised in mid-sentence (or after an elided piece), whose lemma is no dictionary word of the pack, is set aside. On the committed tables only `Myriel` is (`french_baseline.rs`: `Paris`, `Lot`, `Aube`, `Jean-Pierre`, `Saint-Étienne` are Unknown words) |
| The vocabulary estimate | `Pack::dictionary_words`: the ranked lemmas that are dictionary words or carry a level — 30,095 for fr-en (30,055 glossed and 40 levelled unglossed) |
| `fr-en.golden` | 147 probes, 336,433 B, over the pack built from the committed tables since 48's hand-over; a change to `tables/fr/` or `tables/fr-en/` that moves it re-blesses it and says so |
| The owner's decisions (2026-10-10) | The floor stays; names are no French dictionary words; no level without a gloss; no translation table for expressions here (proposal, Why) |

## Goals / Non-Goals

**Goals:**
- fr-en's glosses read as the French word's meanings, in the order French uses them, without the
  page's notes, before change 52 ships them — by rules of fr-en's own reducer, each measured on the
  whole table and on the top 10,000, kept only where the measurement says it reads better.
- French's dictionary words are words: no lemma glossed as a name alone; no level without a gloss.
- Every defect of change 48's D7 decided — fixed, left with its reason, or put to the owner.

**Non-Goals:**
- Any byte of en-fr, es-fr, es-en or en-es: no rule of `reduce_common.py` or of the English edition,
  though some of these rules would suit es-en (D9).
- The French Wiktionary's English translations for expressions (the owner's decision 4).
- M20 (the English edition's two settings) and 23b's Q1–Q4, which answer for es-en and fr-en
  together.
- French's studied side beyond `lexical.tsv` and `level.tsv`: forms, ranks and readings
  (`ca`, read by wordfreq as an unaccented `ça`, is change 43's lemma).
- A new source or fetch: fr-en keeps its snapshot.

## Measured

A prototype in the scratchpad (never committed): `reduce-fr-en.py` of `main`, its inputs the pinned
release's `kaikki-French.jsonl` (sha256 `2d7bbe5f…`, the bytes the pin records), GSD's two sections at
the pinned commit (both sha256 the pin's), wordfreq 3.1.1, with each rule behind a switch. With every
switch off it reproduces the committed `forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`,
`lexical.tsv`, `gloss.tsv`, `senses.tsv` and `mwe.tsv` byte for byte. Rows are the lemmas of
`gloss.tsv`; « top 10k » those ranked 1–10,000 in `tables/fr/freq.tsv`. The golden is the French
baseline re-blessed in a scratch copy of `main` over the prototype's tables.

**The defects of change 48's D7**, counted on the committed rows, and what this design does:

| # | Class | Rows / top 10k | Examples | Decision |
|---|---|---|---|---|
| 1 | A name glossed and a dictionary word | 3,586 / 914 | `france`, `paris`, `lyon`, `durand` « a surname » | Out of the dictionary words (D2) |
| 2 | A level with no gloss | 40 levelled lemmas | `parce` (A1, 103), `quant`, `x`, `pme`, `stp`, `rsa` | No level (D3) |
| 3 | A borrowed gloss wrong for the word | `des` (6); `ca` (144) | « des » « of the; some, the feminine partitive article », borrowed from `de la`; « ca » « board of directors », from `CA` | `des`: D4. `ca`: left (D9) |
| 4 | A pointer's own meaning left out | 680 pointer senses with a meaning in 579 ranked words, 101 in expressions | `mieux` without « better », `moins` without « less », `ouais` without « yeah »; « il y a » « ago » without « there is » | D4, on a closed list of wordings for a word |
| 5 | The part of speech a row opens on | 2,897 / 1,264 rows hold two runs or more | `pas` « step », `son` « sound », `leur` « (to) them », `phare` « leading, flagship » | D5, where UD French-GSD says so |
| 6 | A proper noun's run in a common word's row | first 218 / 108; after another run 484 / 260 | « marche » « Marche (a department of France); march… », « réunion », « somme »; « le » ending on « a surname from Vietnamese », « on » on « a village in Luxembourg » | D5 and D6 for these; the rest left (D9) |
| 7 | An expression whose sense needs a context | 5 expressions | `et des` « or thereabouts », `que de` « how (modifier) », `sur ce` « thereupon », `et si` « what if », `un coup` « used to soften an order; once » | Left out (D7) |
| 8 | A post-1990 spelling keyed apart | 6 expressions | `à priori`, `à postériori`, `et cétéra`, `sur son trente-et-un` and two verbs on it | Lend the traditional spelling's gloss (D7) |
| 9 | The page's own notes | « see usage notes » 5 / 3; « (all senses) » 98 / 30; « in its various senses » 1 / 1; « (Folk etymology: …) » 2 / 2 | `en`, `dans`, `ne`; `contrôle` « control (all senses) »; `consul`; `mon`, `ma` | Out (D8) |
| 10 | A description in a capital outside 23b's list | 14 / 8 rows | `que` « Substitutes for… », `il` « Impersonal subject, it », `mon` « Followed by rank… », `soi` « Designating… » | Lower case, a closed list (D8) |
| 11 | A citation inside a sense | 2 / 2 | `liberté` « liberty, freedom. 1688, Guy Miège, … »; `office` | Cut (D8) |
| 12 | A source's numbered sense | 2 / 1 | `téléphonie` « telephony (2) »; `proscription` | Out (D8) |
| 13 | « etc » without its period | 117 / 42 rows, 7 expressions | `le` « the, my, your, etc », `pas`, `possible` | « etc. », fr-en alone (D8) |
| 14 | Labels left out | — | `or`, `monde`: an obsolete or archaic sense | 23b's Q1 (D9) |

**Each rule alone, then together** (the gloss rules; the dictionary words and levels below):

| Rule | Rows / top 10k | First sense | Lemmas gained | Expressions | Examples |
|---|---|---|---|---|---|
| D4, a pointer's meaning | 70 / 24 | 32 / 15 | 12 (4 of the top 10,000) | 15 changed, 168 gained | « des » « some; of the, from the, some », « du » « forms the partitive article; of the », « mieux » « better; best; … », « ouais »; « il y a » « there is, there are; ago » |
| D5, UD French-GSD's part of speech | 68 / 65 | 68 / 65 | — | — | « pas » opens on its negation, « son » « his, her… », « leur » « their », « bien » « well », « devoir » « must »; « marche », « réunion », « somme » on the common noun |
| D6, no name under a function word | 2 / 2 | — | — | — | « le », « on » |
| D7, expressions | — | — | — | 5 left out, 6 gained | `et des`…; `à priori` |
| D8, notes | 106 / 36 | 96 / 28 | — | 3 changed | « en », « dans », « ne », « contrôle », « consul », « mon » |
| D8, openers | 14 / 8 | 8 / 2 | — | 1 changed | « que », « il », « mon », « ma », « soi », « wesh » |
| D8, citations, numbers | 4 / 3 | 2 / 2 | — | — | « liberté », « office », « téléphonie », « proscription » |
| D8, « etc. » | 117 / 42 | 59 / 10 | — | 7 changed | « le », « pas », « possible » |
| **Together** | **375 / 174** | **264 / 121** | **12, none lost** | **26 changed, 174 gained, 5 left out** | coverage 93.6 / 86.9 / 76.3 % (4,679 / 8,688 / 15,260), the floor 91.9 / 85.1 / 74.4 |

The 12 lemmas gained are words whose only senses were pointers of D4's wordings: `french`
« French », `burger`, `dev`, `ès` « in the » (four of change 48's 40 levelled lemmas with no
gloss, which therefore keep their level), `because` « because; because of », `chui` « I'm »,
`blockchain`, `broyeuse`, `axis`, `ive`, `loix`, `sherry`. Expressions: 17,479 → 17,648.

**Names** (D2). Of the 30,067 lemmas glossed with these rules, 3,578 are glossed by a proper noun's
senses alone — every run of `senses.tsv` `PROPN` — 359 / 911 / 1,762 of the 5,000 / 10,000 / 20,000
commonest. Measured three ways:

| Where | Coverage 5k / 10k / 20k | Dictionary words | What else |
|---|---|---|---|
| fr-en leaves them unglossed | 86.4 / 77.8 / 67.5 % — under the floor at every cut | 26,489 | a reader who opens `Paris` at a sentence's head gets no gloss |
| **Out of `tables/fr/lexical.tsv`, still glossed (this design)** | **93.6 / 86.9 / 76.3 %** | **26,489** | fr-en's pack carries a lexical table |
| Change 41's names rule sets them aside by a name test | unchanged | 30,067 | the vocabulary estimate still counts them, and the pack would need to say which lemmas are names |

English's and Spanish's dictionary words keep theirs: 3,046 of en-fr's glossed lemmas (993 of the
top 10,000) and 1,693 of es-fr's (550) are glossed by a proper noun's senses alone (Q1).

**The `noms` page** then: `Durand`, `Lefèvre`, `Jean-Pierre`, `Saint-Étienne`, `Rhône`, `Garonne`,
`Paris`, `Renault`, `François`, `Lyon` and `Grenoble` are set aside with `Myriel` (the page counts
59 words instead of 70); `Lot`, `Aube`, `Orange`, `Vienne`, `Nice`, `Seine`, `Digne` and `Tours` stay
words, a common word's senses among theirs; `Mme` and `Marie-Claire` open their blocks. On the
`proust` page `François` and `Charles` are set aside too (119 → 117 words).

**Levels** (D3), with English's sizes kept: 45 lemmas lose their level — 36 fr-en does not gloss
(`parce`, `x`, `quant`, `to`, `mm`, `for`, `pp`, `tom`, `com`, `inter`, `rio`, `av`, `encontre`, `fur`,
`pme`, `po`, `am`, `ong`, `instar`, `ken`, `ep`, `bo`, `app`, `cie`, `pass`, `stp`, `caf`, `expliquez`,
`sp`, `rc`, `ht`, `rsa`, `tnt`, `rip`, `tpe`, `nc`) and 9 it glosses as names alone (`pq`, `jo`,
`coran`, `bcp`, `satan`, `vo`, `cb`, `cac`, `mao`) —, 45 gain one at C2's end (ranks 10,763–10,820,
`sous-titre`, `sèche-cheveux`, `tire-bouchon`…), and 122 move one band up (5 A2 → A1, 10 B1 → A2, 23
B2 → B1, 40 C1 → B2, 44 C2 → C1). The spans become A1 1–1,085, A2 1,086–2,385, B1 2,386–4,832, B2
4,833–8,139, C1 8,140–9,474, C2 9,475–10,820; 8,302 levels, as many at each level as before.

**The pack**: 2,527,222 → 2,538,159 B (+10,937: the glosses +3,425, the lexical table and the levels
+7,512), under the 5 MiB budget.

**The golden**: 47 of 147 probes move, 336,433 → 331,322 B (D11).

## Decisions

### D1 — Every rule is fr-en's, in its reducer

The rules run in `reduce-fr-en.py` and nowhere else:
- `read_as_french(src, dst, counts)`, a pre-pass after `english.read_as_meanings` and before
  `english.merge_same_pos_etymologies`, reads each entry once (D4, D6, D8's text rules) and moves a
  headword's lines (D5); it reuses 23b's `english._as_meaning` and `english.english_typography` for a
  carried meaning, and edits neither;
- `with_etc_period`, a post-pass over the reduced glosses and expressions (D8: the shared cleaning
  takes off a sense's final period, so a pre-pass cannot keep it);
- `LEFT_OUT` gains five expressions, and `traditional_spellings` lends six glosses (D7);
- `dictionary_words(glosses, runs)` writes `lexical.tsv` (D2), and `estimated_levels` reads it (D3),
  so the levels are computed after the native side.

*Why not the English edition.* Several rules would read es-en's rows right too (« (all senses) »
holds in 15 es-en rows), but a rule of `reduce_edition_en.py` re-pins es-en, which must not move here;
the rules are named for es-en's next refinement (D9). *Why not `reduce_common.py`.* Every pair's
digest. fr-en's rule digest is `reduce-fr-en.py`, `reduce_common.py` and `reduce_edition_en.py`: only
the first is edited, so only fr-en re-pins.

### D2 — Names are no French dictionary words

**Which lemmas.** A lemma fr-en glosses by a proper noun's senses alone: every sense run of its gloss
(`senses.tsv`) is `PROPN`. It is read off the committed tables, so a check can hold it: 3,578 lemmas
with D4–D8 (3,586 before them), among them 74 of change 48's borrowed from a name through a pointer
(`etats-unis`, `egypte`, `ecosse`: a name spelled without its accent). A word with a common sense
beside a name's stays a word (`lot`, `aube`, `nice`, `marche`). A lemma whose common entry repeats
the name's gloss word for word is read by whichever entry the round-robin meets first: `islam`
« Islam » was a name's before D5 and is a noun's after it, as the treebank counts it.

**Where** (measured above). Left unglossed, the names would take fr-en under its floor at every cut,
which the owner's decision 1 forbids. Read by change 41's names rule as names, they would still count
in the vocabulary estimate, and the core would need data the pack does not carry. Left out of
`tables/fr/lexical.tsv` and still glossed, they are no dictionary words — the vocabulary estimate's
universe falls from 30,095 to 26,489, the names rule sets them aside as written (its scenario *A
city* is this case: « `paris` is a lemma of the pack but not one of its dictionary words ») — and a
reader who opens `Paris` at a sentence's head, or a lowercase `lyon`, still reads its gloss.

**How.** `reduce-fr-en.py` writes `lexical.tsv` into its work folder: its glossed lemmas less those
glossed by a proper noun's senses alone, byte-sorted. `pack_sources.py split` files a reference's own
`lexical.tsv` when its reducer wrote one, else writes the reference's glossed lemmas, as today: en-fr
and es-fr write none, so English's and Spanish's dictionary words are byte for byte what they were.
The rule lives in a file of fr-en's digest; `split` is plumbing. `crates/lingua-pack/src/tables.rs`'s
`check_studied` accepts a studied folder's `lexical.tsv` when it is the reference's glossed lemmas, or
those less every one glossed by a proper noun's senses alone, and fails on anything else, naming the
lemma (*Names left out by halves*). The builder is unchanged: fr-en's dictionary words now differ
from its glossed lemmas, so it writes fr-en's lexical table, as it writes es-en's.

*Rejected — a field in `studied.json` saying which set a language takes.* A switch committed by hand,
outside any digest, beside a reducer that must compute the same set for the levels (D3): two places
to keep in step. The reducer's own file is one.

### D3 — A French level only for a French dictionary word

`no_level` gains a sixth rule, read last: a lemma `lexical.tsv` (D2) does not list takes no level.
The levels are then computed after the native side, in the same reduction. Leaving a lemma out gives
its slot to the next one in rank order, as change 46's other rules do, so the six levels keep
English's sizes and *French's estimated levels* holds (*Six levels of English's sizes*: 8,302).
Measured: 45 lose their level, 45 gain one, 122 move one band up (*Measured*). Every levelled lemma
is a dictionary word, so the vocabulary estimate's universe is French's dictionary words alone
(26,489), and a card seeded from a level carries a gloss.

The requirement is ADDED beside change 46's, which is implemented but not archived: its words « never
from a pair's glosses » and its scenario *The table does not wait for the glosses* were true when
fr-en glossed nothing; from this change a change to fr-en's glosses that adds or removes a dictionary
word can move `level.tsv` (Q6). Which lemmas a CEFR list would hold is still read from the section.
`committed_tables.rs` checks that every lemma of `tables/fr/level.tsv` is listed by
`tables/fr/lexical.tsv`.

*Rejected — dropping the 45 levels without giving their slots away.* 8,257 levels, A1 1,014 lemmas:
the sizes *French's estimated levels* states would no longer hold.

### D4 — A pointer that carries its meaning is read as that meaning

The shared rules skip a pointer sense; a word with no other sense borrows its target's senses (from a
target of three letters or more). The section writes many pointers with their meaning: « comparative
degree of bien; better », « synonym of oui; yeah, yep… », « plural of un (“some”, the plural
indefinite article) », « contraction of de + les, literally “of the, from the, some” »,
« impersonal singular present indicative of y avoir: there is, there are ». `read_as_french` reads
such a sense as its meaning, in its place, as 23b's D3 reads a shortened form (its rule 1; nothing is
lent here):
- **the meaning**: the section's `extra` for the pointer's target — its first “quoted” text when it
  has one (« some »; « of the, from the, some »), an initialism after a colon dropped, a pointer's
  text read past its last colon or semicolon — else the gloss's quoted text after the target, else
  its text after a colon or a semicolon;
- **a word's pointers**: those whose wording opens on « comparative degree of », « superlative
  degree of », « synonym of », « female equivalent of », « plural of » or « contraction of »;
- **an expression's pointers**: any wording (« il y a » « there is, there are »; « sais pas »
  « dunno »; « la vôtre » « yours (the one belonging to you) »; « chou rave » « kohlrabi »);
- **never**: a name's entry, an acronym's, a meaning written only in capitals (« NE », « SE »).

Measured: 70 rows (24 of the top 10,000); 12 lemmas gained; 15 expressions changed and 168 gained,
none lost. `des` borrows nothing any more and reads « some; of the, from the, some »; `du` « forms the
partitive article; of the » (the contraction's run is `X`, as kaikki's `contraction` is everywhere).
Some read oddly and are in the owner's sample: « directrice » « directress » where it borrowed
« director; school principal », « fiancée » « fiancé », « matelas » gains « French tacos » (a synonym
the page gives), « j't'à » « the 't' is epenthetic ».

*Rejected — every pointer wording for a word* (109 rows, 35 of the top 10,000): `y` (30) gains
« he; they (male) » from its dialectal « alternative form of il », `fol` reads « used only when the
following noun starts with a vowel… » instead of « mad, crazy », `click` « especially of a computer
mouse », `electro` « music genre; électroménager ». An alternative form, a spelling, a feminine
singular, an ellipsis or a clipping of a word stays a pointer.

### D5 — A row opens on the part of speech UD French-GSD reads the word as

The round-robin groups a gloss's senses by part of speech in the order the page's entries come, so
`pas` opens on « step » and `son` on « sound ». fr-en's pin already holds GSD's training and
development sections (390,368 words), read today for which lemma a form takes. `read_as_french`
counts each word under each part of speech — the auxiliary as a verb; a word inside a fixed expression
for none (UD tags « conséquent » in « par conséquent » a noun); a noun, verb, adjective or proper noun
under its lemma, any other part of speech under its own form (UD lemmatises « ton », « leur »,
« mon » as « son ») — and writes the entries of a headword's commonest part of speech first when the
treebank reads it at least **10** times and at least **twice** as often as the part of speech the page
opens on. A proper noun is never moved first; the other entries keep their order.

Measured: 68 rows, 65 of the top 10,000. Function words open on their function (`pas` 981 adverbs
against 8 nouns, `son` 1,506 determiners against 19 nouns, `leur`, `bien`, `quand`, `pendant`,
`juste`, `outre`, `envers`); four common nouns no longer open on a place (`marche`, `réunion`,
`somme`, `restauration`); verbs open on the verb (`devoir`, `devenir`, `toucher`, `tendre`); 47 are an
adjective and a noun swapped (`allemand`, `chinois`, `critique`, `objectif`, `ferme` « roof truss;
farm » before « firm », `mort` « dead person; death » before « dead »), where both are the word's and
the treebank says which is commoner in text.

*Rejected — no threshold, every word by lemma* (553 rows, 313 of the top 10,000): `le`'s surname
before its pronoun (30 « Le » in names), `ton` « tone » before « your » (UD's lemma « son »), `salut`
« wave » before « hi » (3 against 1), `bonjour`, `sinon`, `dessus`. *Rejected — a minimum of 5*
(115 rows): it adds `phare` « lighthouse » and `midi` « noon » first, and opens `sinon` on « also,
additionally » (7 adverbs against 5 conjunctions, UD's adverb being the page's « otherwise ») — the
owner's call (Q2). *Rejected — names last wherever no count decides*: 23b measured it on es-en
(« Brazil », « China » after « brazilwood », « pebble »); `jean` (91 « Jean » in GSD) keeps « a pair
of jeans » first, `pierre` « stone ».

### D6 — A function word's row holds no name

A `name` entry of a capitalised headword — not an acronym — is left out when every entry of the
lower-case headword is a preposition's, a conjunction's, a pronoun's, a determiner's, an article's or
a particle's: `Le` « a surname from Vietnamese », `On` « a village in Luxembourg, Belgium », `Cette`
(a form, no row). 2 rows, both of the top 10,000. 23b's D4 writes such a name last; here it adds
nothing a reader of `le` or `on` needs, and the card does not read a token's capital (23b's Q3).

### D7 — French's expressions: five left out, six lent

**Left out** (`LEFT_OUT`, each with its reason, as `à la`): `et des`, `que de`, `sur ce`, `et si`,
`un coup`. Their one sense needs a context the key does not hold, and the treebank's text never gives
it: of 204 « et des » none means « or thereabouts » (two follow a year, none a number); of 43
« que de » none is the exclamative « how much! »; of 25 « sur ce » none is « thereupon » (all « sur
ce » before a noun or « qui »/« que »); of 13 « et si » one at most reads « what if » (« Et si en
plus y'a personne »), the others « and if »; of 11 « un coup » none means « once » or softens an
order (7 « un coup de »). « du pain et des œufs » then
meets no expression, and « un coup d'œil » meets `coup d'œil` alone.

**Lent**: a headword whose every sense is « post-1990 spelling of X », and that the forms table keys
apart from X (a word of it reads as another lemma, or splits otherwise), takes X's gloss: `à priori`,
`à postériori`, `et cétéra`, `sur son trente-et-un`, `être sur son trente-et-un`, `se mettre sur son
trente-et-un`. The 11 others change 48 left out are keyed alike and meet their traditional spelling
already (`crème fraiche`, `boite à gants`, `s'il vous plait`). Change 48's D7 also named `crème
brulée`: the committed forms table reads `brulée` and `brûlée` as *brûler*, so it meets `crème
brûlée` already.

### D8 — The page's notes and typography

In `read_as_french`, on every sense of words and expressions, in this order:
- **notes out**: « see usage notes » (with « also » and the comma or semicolon before it, or its own
  parenthesis), « (all senses) », « in its various senses » (with « all » and « , including »),
  « (Folk etymology: …) »; an emptied parenthesis goes. 106 rows / 36, 3 expressions.
- **fr-en's description openers in lower case**: « Substitutes », « Impersonal », « Followed »,
  « Adverbial », « Designating », « Stresses », « Representing », « Indicating », « Names »,
  « Describing », « Exclamation », « Found », then a space and a lower-case letter or a parenthesis:
  the openers measured in fr-en's capitalised senses that describe a use rather than translate.
  14 rows / 8.
- **a citation cut**: a sense ends where a period is followed by a year and a comma (« 1688, Guy
  Miège, … »). 2 rows / 2.
- **a sense number out**: one digit in parentheses (« telephony (2) »). 2 rows / 1.

After the shared rules, `with_etc_period` writes « etc » not followed by a period « etc. » — in the
glosses (117 rows / 42) and the expressions (7). An expression's gloss may then run one character over
the shared rules' 80 (one does).

*Rejected — every capitalised opener whose lower-case form opens a sense elsewhere* (192 senses, 162
rows): it lowers « German », « Basque », « Ottoman », « May », « March », « Soviet ». 4.0 % of fr-en's
common senses open on a capital, most of them proper adjectives, languages and demonyms; the
definitions the page writes in sentence case (« Military rank equivalent to corporal », « Action of
loading ») stay (D9).

### D9 — Measured and left

- **An acronym's pointers lending to a word with an entry of its own**: dropping them changes `ca`
  to « approximately, about » but leaves `svp` and `jsp` with no gloss (« please », from `SVP`) and
  makes `cv` read « how are you? » — worse. `ca` keeps « board of directors »: wordfreq's `ca` is
  mostly an unaccented `ça`, change 43's lemma to decide (Q4).
- **Proper nouns still opening a row**: 219 rows (105 of the top 10,000: `louis`, `midi`,
  `belgique`, `jacques`, `noël`, `bordeaux`, `japon`), and 488 / 263 ending on one — 23b's Q3, a
  card that reads the token's capital (Q5).
- **Parts of speech the treebank does not decide**: 2,905 rows hold two runs, 68 move; `phare` and
  `midi` are met fewer than ten times (Q2).
- **Sentence-case definitions** (« Military rank… »), **labels** (23b's Q1), « (In various senses,
  such as …) » under `stand`.
- **es-en**: its own « (all senses) » (15 rows), its « etc » (35), its descriptions: a later
  refinement of the English edition, which re-pins es-en and fr-en together. **« etc »** in en-fr,
  es-fr and en-es: the shared fix 23b named (its D6).
- **Translation tables for expressions**: the owner's decision 4; a later update of fr-en if wanted.

### D10 — fr-en re-reduced at its pin; what moves and what cannot

`build.sh --reduce fr-en` from `lingua-pack-sources-fr-en-2026.10.09`, nothing fetched beyond the
release's asset and GSD's two pinned files. The pin keeps its `snapshot` and `sources` byte for byte;
its `reducer` digest moves (`reduce-fr-en.py`), with `pack_version` (`2026.10.09+<digest[:7]>`), the
pack's sha256 and size. `gloss_coverage.py --pair fr-en` passes. `tables/fr/` moves in `lexical.tsv`
(30,055 → 26,489) and `level.tsv` (212 rows) alone: forms, ranks, readings, tag pool and record byte
for byte.

**en-fr, es-fr, es-en, en-es cannot move**: no file of their rule digests is edited
(`reduce_common.py`, `reduce_edition_en.py`, `reduce_edition_es.py`, `reduce_edition_fr.py`, their
reducers); `pack_sources.py` and `build.sh` are in no digest, and `split` writes English's and
Spanish's `lexical.tsv` from their references' glossed lemmas as before, since `reduce-en-fr.py` and
`reduce-es-fr.py` write none; `check_studied` accepts their folders as before (their dictionary words
are their references' glossed lemmas); the builder is unchanged. Their tables, pins and packs, their
goldens, `word-card-es-en.txt`, `word-card-en-es.txt` and `selection-rows-fr.txt` pass as committed;
`check-reducer` passes for the five pairs. `fr-en.golden`'s `beside es-en` line does not move. The
French interface: no extension source changes, `packs.json` does not list fr-en.

### D11 — The golden, read probe by probe

`LINGUA_BLESS=1 cargo test -p lingua-wasm --test french_baseline` once. Measured over the
prototype's tables, 47 of 147 probes move, by cause:
- **the gloss rules alone** (42): `pack`; the 13 `analyse new-reader` pages, in their tokens' glosses
  only (`le`, `que`, `ne`, `pas`, `des`, `du`, `en`…); the `gloss` probes of `pas`, `son`, `du`, `des`
  and `le`; 19 `phrase-gloss` probes, among them « du pain et des œufs » and « un coup d'œil »
  (no `et des`, no `un coup`), « il y a » (« there is, there are; ago ») and every one holding `il`,
  `le` or `du`; `word-grammar du du` (a run `X` « of the ») and `l’ le`; the vocabulary estimate's
  universe (30,095 → 30,103, the gloss rules' 12 lemmas less the 4 levelled ones they gloss);
- **the dictionary words and levels alone** (10): `pack`; `analyse new-reader noms` (11 names set
  aside, 70 → 59 counted words) and `proust` (`François`, `Charles`, 119 → 117); the two vocabulary
  estimates (universe 26,489; the reader's estimate 2,304 → 2,184); `review-current` ×2, the deck
  seeded from the levels drawing `croissance` and `célèbre` instead of `cavité` and `certainement`
  (the C1 band moved); `export-status-ops`, `export-card-ops`, `backup`.
The 100 others are byte for byte: `about`, `beside es-en`, `notice`, `licences`, `has-levels`,
`level-ladder`, both `seed-level`, `start-review`, `review-remaining`, `calibration`,
`declared-level`, `export-declared-levels`, `promote-by-exposure`, the three counts, the four
`analyse reader` pages, 40 `gloss`,
29 `word-grammar` and 10 `phrase-gloss` probes. The pull request runs change 48's comparison script —
the same probe names in order, the unmoved lines, every `gloss <word>` probe equal to its lemma's row
— and lists each moved probe with its cause.

`french_baseline.rs`'s `french_has_its_pre_pass_and_its_analysis` follows D2: on the committed
tables `Paris`, `Jean-Pierre` and `Saint-Étienne` are set aside with `Myriel`, `Lot`, `Aube`,
`Orange`, `Vienne` and `Mme` stay Unknown words; `the_names_rule_reads_french_s_evidence` (the
fixture) is unchanged. `support/french.rs`'s doc follows D2 and D7.

### D12 — Order

| Change | Relation |
|---|---|
| 48 fr-en | before (required, on `main`): the tables, the reducer, the golden on the committed pack |
| 41, 43, 45, 46 | before, on `main`: the names rule, the studied tables, the readings, the levels this change refines |
| 49 fr-es | either side. After this change, fr-es reads French's dictionary words and levels as committed. Before it, this change re-reduces fr-es too — its pin's `studied` record (`lexical.tsv`, `level.tsv`) and its pack — and re-blesses what fr-es pins, as a change to a studied language's tables does |
| 51 word card | either side. After this change, its snapshots are drawn on these tables; before it, they are re-blessed here (`word-card-fr-en.txt`, the French golden's card probes) |
| 52 enable | after: fr-en ships with these glosses; 52's `archiveAfter` gains this change, as its design says |
| M20, 23b's Q1–Q4 | either side: the English edition's, re-pinning es-en and fr-en; whichever lands second re-reduces fr-en on both |

## For the owner

- **Q1 — Names in English and Spanish.** French's dictionary words now leave out the 3,578 lemmas
  glossed only as names (`paris`, `lyon`, `durand`); English's still count 3,046 (`london` read in an
  English text is a word to learn) and Spanish's 1,693. The same rule for them moves en-fr's and
  es-fr's dictionary words, packs and goldens: a later change of its own, or keep the difference?
- **Q2 — How much the treebank must say.** A row's part of speech moves when GSD reads the word at
  least 10 times and twice as often (68 rows). At 5 it would also open `phare` on « lighthouse » and
  `midi` on « noon », but `sinon` on « also, additionally ». And 47 of the 68 swap an adjective and a
  noun (`ferme` « roof truss; farm » before « firm », `mort` « dead person; death » before « dead »),
  and two put a content word first (`ensemble` « set » before « together », `nul` « of poor quality »
  before « no, none »): keep them, or move only a function word's or a name's row?
- **Q3 — A pointer's meaning, as the page writes it.** `directrice` reads « directress » instead of
  `directeur`'s « director; school principal », `fiancée` « fiancé », `matelas` gains « French tacos »:
  keep D4's wordings as they are, or leave « female equivalent of » out (30 rows)?
- **Q4 — `ca`.** « board of directors » stays (an initialism's meaning, lent); wordfreq's `ca` is
  mostly an unaccented `ça`. Read `ca` as a spelling of `ça` (change 43's forms), or leave it?
- **Q5 — Names opening a common word's row** (219 rows: `jean` « John » after « a pair of jeans » is
  right, `midi` « the Midi » before « noon » is not): accepted until a card that reads the token's
  capital (23b's Q3)?
- **Q6 — Sentences true of their time**, to amend when each change is archived, so that no two specs
  say opposite things: 48's « fr-en being French's reference pair, its glossed lemmas are French's
  dictionary words » and its scenario *French's dictionary words* (« lists exactly the lemmas
  `gloss.tsv` glosses, and fr-en's pack carries no lexical table ») → « less those it glosses by a
  proper noun's senses alone », « carries a lexical table »; 46's « never from a pair's glosses » and
  *The table does not wait for the glosses* → « and French's dictionary words », « when fr-en's
  glosses add or remove no dictionary word »; 49's and 52's « French's dictionary words, fr-en's
  glossed lemmas » → « French's dictionary words ». Not edited here.

## Risks / Trade-offs

- **[A rule reads a meaning wrong]** → each rule is measured alone on the whole table; every changed
  row of the top 10,000 (174) is in the owner's sample, with the odd ones named (D4, D5).
- **[A name a reader wants as a word]** → the lemma keeps its gloss; only the vocabulary estimate and
  the names rule read it differently, and a name read at a sentence's head or in lowercase is a word
  as before. `islam`, `coran`: names or words by the section's entries (D2), listed.
- **[Levels move]** → 212 rows, measured; English's sizes kept; the seeded deck in the golden moves,
  said in the pull request (D11).
- **[The treebank's register]** → GSD is news and encyclopaedic text: `salut`, `bonjour` and `sinon`
  were worse at no threshold, and stay at 10 and twice; the owner settles the threshold (Q2).
- **[es-en keeps defects fr-en loses]** → named for es-en's next refinement (D9); es-en does not move
  here by the owner's rule.
- **[A golden that moves]** → re-blessed once, each moved probe with its cause; the owner approves the
  re-bless.

## Migration Plan

No reader holds a French pack before change 52: nothing to migrate. A rollback is a revert of the
tables, the pin, `split`, `check_studied` and the golden.

## Effort

2–3.5 ideal days: `read_as_french`, the post-pass and the expression rules with their tests 1–1.5;
the dictionary words and levels — the reducer, `split`, `check_studied`, `committed_tables.rs` —
0.5–0.75; the re-reduction, pin, README and `SOURCES.md` 0.25–0.5; the golden, its review and
`french_baseline.rs` 0.25–0.5; the owner's sample 0.25.
