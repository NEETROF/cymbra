# Design — refine-lingua-it-en-glosses

## Context

See proposal.md (Why). What exists, on `main` at `2a5a5e285` (change 22, #1063; its tables committed
by the owner's update of 2026.10.25, run 40183685031, and reduced again on change 17's studied tables
by run 40187939732):

| Where | What |
|---|---|
| `reduce-it-en.py` | it-en's reducer: the native side alone, against `tables/it/` as committed — `native_fields` → the English edition's letters and `read_as_meanings` → `reduce_italian_en.read_as_italian` (change 22's D4) → `native_side` (the shared `reduce_gloss`, `reduce_expressions`, `fallback_glosses` over the Italian Wiktionary's translations) → `without_repeated_senses` → the split words → `acronym_glosses` |
| `reduce_italian_en.py` | it-en's own pre-pass, in it-en's rule digest alone: carried meanings by a reviewed list of wordings (D4.1, narrowed: a contraction's, a plural's or a degree's only for a word whose lowercase entries hold nothing else), acronyms off common words and names off function words (D4.2), Greek letters (D4.3), the page's notes (D4.4), split words (D4.5) |
| `reduce_common.py`, `reduce_edition_en.py` | The shared rules and the English edition's (es-en's, fr-en's and it-en's digests): not edited here |
| `reduce_french_treebank.py`, `reduce_italian_treebank.py` | fr-en's order rule (`commonest_first`: at least 10 and twice, the function parts of speech) and MarkIT's counts (`markit_pos_counts`), which it-fr imports for its D2 (change 17): imported here unchanged |
| `tables/it-en/` | 33,199 glossed lemmas (33,004 from the section, 194 from the Italian Wiktionary, `USA`); 9,831 expressions; coverage **91.3 / 84.2 / 75.8 %** against `FLOORS["it-en"]` 89.3 / 82.2 / 73.7; the pack 2,467,552 B, its layer 916,596 B |
| `tables/it/` | Italian's studied side, it-fr's, read as committed; no rule here writes it |
| The requirement this change touches | Change 22's *Italian is glossed in English from the English Wiktionary's Italian section and the Italian Wiktionary's English translations*, open, held by `add-lingua-pack-it-en` alone |
| The lists for this change | Change 22's *Known data defects* and *Left for change 23* (its design), its README's *Known defects*; the 1,000 commonest rows its review read (about 29 whose first sense is wrong); the it-en counterparts of change 17's D2, D5 and D6 |

## Goals / Non-Goals

**Goals:**
- it-en's glosses read as an Italian word's meanings, in the order Italian uses them, before change 25
  lists it-en — by rules of it-en's own modules, each measured alone on the committed tables and kept
  only where it reads better (the criteria below).
- Every item of the lists above decided: a rule, a reviewed row, left with its reason, or put to the
  owner in plain words with examples (*For the owner*).

**Non-Goals:**
- Any byte of `tables/it/` or of the seven other pairs (en-fr, es-fr, es-en, en-es, fr-en, fr-es,
  it-fr): no rule of `reduce_common.py`, `reduce_edition_en.py`, `reduce_italian_acronyms.py` or of the
  treebank modules is edited, though some of these rules would suit es-en and fr-en (D9).
- Italian's forms (it-fr's): a feminine ranked as a noun of its own (`brutta`, `amica`) reads as that
  noun wherever it is written, a matter of the studied tables (D9).
- A new source beyond MarkIT's two sections, which it-fr already pins (D1).
- The card's labels (offensive, regional, archaic): the card shows none, as es-en's refinement left them
  (D9).

## How this was measured

**Criteria, written on 2026-10-25 before any figure of this change was read** (change 22's own figures
were read as this change's list): the base is the committed tables on `main` at `2a5a5e285`; a class is
counted in ranked lemmas by band of `tables/it/freq.tsv` (1–1,000, 1,001–5,000, 5,001–20,000, past
20,000); a rule is kept when every row it changes among the 5,000 commonest reads right or no worse and
the rows it changes among 5,001–20,000 read right in a majority, else narrowed or dropped, the narrowing
recorded; a rule never takes the only gloss of a ranked lemma of the 20,000 unless that gloss is pure
noise (a name's note on a word that is no name, a pointer's wording); only it-en's own modules change,
a shared module imported unchanged is named, and a fix needing an edited shared module is reported, not
made; a reviewed row is written only for a clear error no rule reaches, with its reason, and a matter of
taste goes to the owner with examples; the precedents already settled apply (fr-en's owner: the
treebank reorders a function word's row, never a content word ahead of another — its Q2; « female
equivalent of » is no word's meaning — its Q3; names opening a row accepted — its Q5); coverage stays at
or above the floor. UD Italian-PUD and MarkIT's test section are never read; the gloss rules move no
form, lemma or reading, so the PUD gates cannot move (D10).

**The data.** it-en's pinned bytes, each checked by its sha256 before use: the English Wiktionary's
Italian section (`kaikki-Italian.jsonl`, `ae7de484…`, 665,207,473 B) and the Italian Wiktionary's English
translations (`kaikki-it-traductions-en.jsonl`, `e192643a…`, 7,216,558 B), local copies of the files
change 22's review derived; MarkIT's training and development sections at `e210079d` (`4f6fc342…`,
`633b6186…`, it-fr's pin). Read as data, by scripts outside the repository, with Python 3.12 and no
package beyond the standard library (the reducer's own).

**The prototype.** it-en's reducer as committed, run over those bytes, gives `gloss.tsv`, `senses.tsv`,
`mwe.tsv` and the `NOTICE` byte for byte. Each rule is that reducer with one pass added — after
`read_as_italian`, before the etymology merging, as change 22's pre-pass runs, and the « etc. » pass
after `without_repeated_senses` — and is compared row by row with the committed rows. Every figure below
is re-measured by the real reduction at implementation (task 3.2).

## Measured

**The classes, on the committed tables**, and what this design does:

| # | Class | Ranked rows, by band (≤1k / ≤5k / ≤20k / beyond) | Examples | Decision |
|---|---|---|---|---|
| 1 | The wrong sense first, by the page's entries or etymologies | About 29 of the 1,000 (the review's reading); 173 rows open on a sense the page marks archaic, obsolete, dated or historical (9 / 17 / 55 / 92) | `auto` « auto (genre of dramatic literature) », `testo` « an earthenware lid », `polizia` « polity », `bene` « upper-class, posh », `gente` « noble », `forte`, `presto`, `morire` « death », `prego` « prayer », `più` « anymore » | D2 |
| 2 | The eight-sense cap | — | `piano` (213) loses « plan », its row opening on « plane » and a pope's « Pian » | D2, D6 |
| 3 | A pointer's wording shown as the gloss | 21 senses of ranked lemmas (5 of the 1,000): 9 labelled forms and spellings, 12 clitic compounds; 31 rows nesting their meanings under a degree's wording (1 / 2 / 4 / 24) | `che` « proscribed spelling of ché », `de` (94) « Marche, Milan, Rome, and Venice form of di », `el` « archaic, regional, and Rome form of il », `amo` « Rome form of abbiamo… », `aprile` « compound of apri…, with le », `minimo` « superlative degree of piccolo », `minore` « comparative degree of piccolo » | D3, D4 |
| 4 | A usage note as the gloss | 1 row | `lo` (21): « the form of il that is used before the so-called impure consonants… » ending in « the » | D7 |
| 5 | A pointer that lends nothing | 106 rows hold a « female equivalent of » beside another sense (7 / 23 / 41 / 35); 7 rows a clipping carrying its meaning; 11 a pointer to a word of one or two letters, which the shared rules never follow; `guai` a plural beside an interjection | `amica` « girlfriend », `bambina` « babe », `compagna` « wife, spouse », `auto`, `moto`, `ce` « reinforcing particle, “do” », `guai` « woe! »; `vado` « ford » — settled by change 17, `vado` now *andare*'s | D3, D4, D5 |
| 6 | Names after a common word's senses | Rows holding a surname's note 1,143 of the 20,000 (72 / 276 / 795; 3,395 in all); a given name's 567 (22 / 146 / 399); a town's, a comune's or a village's beside a common word in 124 ranked rows, capitals and cities aside | `volta`, `guerra`, `musica` « …; a surname », `delfino` « …; a male given name », `noto` « …; a town in Syracuse, Sicily », `obbiettivo` « a surname » alone, `ventimiglia` « a town…; a surname transferred from the place name » | D6 |
| 7 | The page's typography | « etc » without its period in 71 rows (5 / 6 / 20 / 40) and 13 expressions; 1 list marker; 1 carried meaning left in its quotation marks | `prego` « …don't mention it etc », `itria` « # alternative form of lontra », `mar` « (“sea”) (used in poetry…) » | D7 |
| 8 | A source's slip | 1 expression | `all'antica` « ancient, anshent, antique » (the Italian Wiktionary's jocular spelling) | D8 |
| 9 | Notes the page writes for a verb | 154 rows with a bracketed construction (7 / 25 / 50 / 72), 12 with an auxiliary | `chiedere` « to ask [with di ‘about (someone)’] », `morire` « to kill [auxiliary avere] » | Left (D9) |
| 10 | Elsewhere | — | `USA` opening on the supercontinent; `negro` « …nigger » unlabelled; `dovere`'s interjection a description | Left (D9) |

**Each rule alone**, against the committed rows:

| Rule | Rows changed (bands) | First sense | Gained / lost | Examples |
|---|---|---|---|---|
| D2a, MarkIT for function words (it-fr's rule, fr-en's constants) | 16 (16 / 0 / 0 / 0) | 15 | — | `su` « on, upon… » before « up, upstairs », `solo` « only, just », `spesso` « often », `verso` « toward », `dopo`, `proprio` « really », `uno` « an, a »; `come` « as soon as »: worse, held (D2c) |
| *D2a without the boundary on content words (rejected)* | 33 (31 / 2 / 0 / 0) | 32 | — | adds `fine` « end », `forte`, `valore`, `cittadino` — and `fatto` « fact », `piacere` « pleasure », `diritto` « right side », `gente` « gens »: worse |
| D2b, a row opens on its first sense the page does not mark archaic, obsolete, dated or historical | 173 (9 / 17 / 55 / 92) | 173 | — | `polizia` « police », `università` « university », `ufficio` « office », `prodotto` « product », `parlamento`, `avanti` « forward », `dieta` « diet », `sincero`, `tortura`, `esca` « bait »; `contrastare` « to quarrel » and `dì` « daytime », right but less common |
| *D2b, every outdated sense last (rejected)* | 296 (20 / 42 / 98 / 136) | 173 | — | reshuffles whole rows: `tratta` « seine fishing », `signore`, `costo` |
| *D2b with rare, uncommon, literary and poetic (rejected)* | 318 first senses (17 / 32 / 101 / 168) | 318 | — | `grande` « really (intensifier) », `sito` « region of a protein », `testo` « an earthenware baking tray »: worse |
| D2c, `FIRST_RUN` (19 rows, task 0.1) | 17 (16 / 1 / 0 / 0) | 17 | — | `bene` « well », `gente` « people », `piano` « floor, storey; plan… », `testo` « text (writing) », `morire` « to die »; `come` and `loro` already open so alone |
| D3, pointer wordings, and a short word's pointer lending | 29 (10 / 3 / 9 / 7) | 14 | — | `che` without « proscribed spelling of ché », `de` *di*'s « of, ’s; from… », `el` « the; he », `ce` « us; ourselves… », `aprile` « April », `amo`, `eo` « I », `istoria` « history; story » |
| D4a, a meaning nested under a degree's wording | 31 (1 / 2 / 4 / 24) | 31 | — | `minimo` « minimum, (the) least… », `minore` « less; shorter; lesser… », `pochissimo` « very little », `gravissimo` |
| D4b, a degree's carried meaning wherever it stands | 5 (1 / 0 / 2 / 2) | 5 | — | `più` « more; anymore… », `bravissimo` « very good », `gentilissimo` « very kind » |
| D4c, a clipping's carried meaning | 7 (2 / 3 / 2 / 0) | 5 | — | `auto` « car; bus; auto (genre…) », `moto` « motorcycle », `bici`, `ser` « sir », `primo` « …first course » |
| D4d, a plural's carried meaning beside another part of speech | 2 (0 / 1 / 1 / 0) | 1 | — | `guai` « woe!; trouble », `vegetali` |
| *D5, a female equivalent lending the masculine's senses (rejected)* | 106 (7 / 23 / 41 / 35) | 50 | — | `amica` « friend… », and `signora` « lord (ruler) », `bambina` « boy, young boy », `compagna` « husband », `stronza` « turd », `fisica` « physicist (male) » |
| *D6 as first worded: a surname's and a given name's note off any word with a lowercase entry (rejected)* | 1,290 (80 / 282 / 532 / 396) | 119 | 17 lost | `francesco` « French; Frenchman », `luigi` « louis (coin) », `alice` « anchovy », `gennaro` « January »; `diego`, `roberta` lost |
| D6, names' notes as decided | 1,852 (82 / 304 / 669 / 797) | 186 (0 / 6 / 57 / 123) | `pinuccio` lost (37,110) | `marzo` « March », `musica`, `delfino` « dolphin; dauphin… », `noto`, `milanese` « Milanese », `ferrari` « an Italian car manufacturer », `obbiettivo` « objective » |
| D7, `lo`'s note | 1 (1 / 0 / 0 / 0) | 1 | — | `lo` « the; him; it, this or that thing » |
| D7 and D8, « etc. », the list marker, the quoted opener, `all'antica` | 72 (5 / 7 / 20 / 40) | 31 | — | `prego`, `scena` « stage (of a theatre etc.) », `mar` « sea (used in poetry…) »; 14 expressions |

**Together**: **2,175 rows change** (134 / 335 / 754 / 952), **476 in their first sense** (43 / 32 /
132 / 269); no lemma gains a gloss (`de`, `el`, `eo` read a meaning in place of a pointer's wording) and
one loses its only one, `pinuccio` (37,110), whose borrowed note was `Pino`'s given name; 19 expressions
change (13 « etc. », `all'antica`, 5 reordered by D2b: `a vanvera`, `stare a bada`). Coverage **91.3 /
84.2 / 75.8 %**, as now, against the floor 89.3 / 82.2 / 73.7. The rows the review read wrong among the
1,000 all open on their meaning but `amica`, whose row the owner decides (task 0.3).

## Decisions

### D1 — Every rule is it-en's, in its two modules; MarkIT joins it-en's pin

The gloss rules extend `read_as_italian` (`reduce_italian_en.py`, change 22's D4) with a pass that reads
a headword's entries together, and `reduce-it-en.py` with two post-passes (« etc. », `CORRECTIONS`).
The order rule calls `reduce_french_treebank.commonest_first` over
`reduce_italian_treebank.markit_pos_counts`, imported as written, as it-fr imports them (change 17's
D1): both join it-en's rule digest, and editing either would re-pin fr-en, fr-es, it-fr and it-en
together. `reduce_common.py`, `reduce_edition_en.py` and `reduce_italian_acronyms.py` are not edited, so
es-en, fr-en and it-fr keep their digests, tables and packs.

**One file outside it-en's two modules is edited**: `pack_sources.py`, whose `PINNED["it-en"]` gains
`ITALIAN_MARKIT`, the two sections it-fr's pin already names — as fr-es's refinement added UD French-GSD
to `PINNED["fr-es"]` (`refine-lingua-fr-es-glosses` D8). It is the pipeline's table of what a pair
fetches, in no pair's rule digest; no other pair's entry, pin or table moves. `reduce-it-en.py` reads
the two files from its work folder, through `held_out`, which refuses a test section.

### D2 — The order of a row: MarkIT for function words, then the page's own marks, then a reviewed table

**a. A function word's row opens on the part of speech MarkIT reads it as** — it-fr's D2 as written:
fr-en's rule (`ADP`, `DET`, `PRON`, `CCONJ`, `SCONJ`, `PART`, `ADV`; at least 10 and twice the page's
first; never a proper noun first; a content word never moved ahead of another). **16 rows, all of the
1,000**, 15 first senses: `su` « on, upon, onto » before « up, upstairs », `solo` « only, just »,
`spesso` « often » before « thick », `verso` « toward » before « verse », `dopo` « after », `proprio`
« really, quite », `uno` « an, a », `quello`, `quale`, `tanto`; `loro` « their » (34 determiners against
15 pronouns) and `come` « as soon as » (39 subordinating conjunctions against 16 adverbs) — `come`
worse, and both given a row of the table, as it-fr's. *Rejected — the boundary lifted for content
words*: 17 rows more, four worse (`fatto` « fact » before « done, made », `piacere` « pleasure » before
« to be pleasing », `diritto` « right side », `gente` « gens »): fr-en's owner kept the page's order for
content words (its Q2), and MarkIT is 30,153 words of school essays.

**b. A row that would open on an outdated sense opens on its first sense that is not.** The page marks
its senses; the shared round-robin ignores the marks. When the sense a row opens on — its first entry's
first meaning — is marked archaic, obsolete, dated or historical (it-fr's `_OLD` and historical), the
row opens instead on the first sense, in the page's order of entries and senses, that carries none of
the four marks: that sense first in its entry, that entry first; every other sense keeps its place.
**173 rows** (9 / 17 / 55 / 92), each in its first sense alone: `polizia` « police; polity »,
`università` « university; totality… », `ufficio` « office; moral duty… », `prodotto` « product »,
`parlamento` « public assembly or meeting », `avanti` « forward, ahead », `principalmente` « mainly »,
`dieta`, `sincero`, `folle` « mad, insane », `tortura`, `cornice` « frame », `esca` « bait »; among the
5,000 two read a right sense that is not the commonest — `contrastare` « to quarrel » before « to
oppose », `dì` « daytime » before « day » (« day » marked dated and literary) —, and five a regional,
religious or slang sense of no worse standing (`tratta` « seine fishing », `signore` « Lord, God », `pro`
« pro », `mica`, `negro`). *Narrowed*: every outdated sense written last changes 296 rows and reshuffles
whole rows; with « rare », « uncommon », « literary » and « poetic » the rule opens `grande` on « really
(intensifier) », `sito` on « region of a protein », `testo` on « an earthenware baking tray ». The page's
marks are its own, so a wrong mark moves a row wrongly: `gente` and `stazione`, whose every sense is
marked archaic, are rows of the table.

**c. The reviewed table, `FIRST_RUN`**, names for a row the rules miss or misread the headword's case,
the part of speech and the sense the row opens on — its entry first, that sense first in it — each with
its reason; the reduction fails on a row whose sense it does not find (an update that rewrites the page
names it), and a row moves only entries and senses, never adds or removes one. Keyed by a sense rather
than a part of speech alone, as change 17's could not: `testo` has two noun entries, `gente`'s and
`stazione`'s meanings sit inside one. **19 rows, proposed** (task 0.1):

| Row | Opens on | Reason |
|---|---|---|
| `come` | adverb « how » | MarkIT's rule would open on « as soon as » (held, as it-fr's row) |
| `loro` | pronoun « they, them » | MarkIT's rule opens on « their »; the pronoun first, as it-fr's row and card |
| `bene` | adverb « well, nicely, OK, right » | the adjective « upper-class, posh, high » first; MarkIT reads 12 nouns and 8 adverbs and moves no content word |
| `gente` | noun « people, guys, folks (body of human beings) » | the archaic adjective « noble » first; every noun sense is marked archaic and poetic too, so D2b cannot choose |
| `piano` | noun « floor, storey (of a building) » | the adjective's « plane » first; « plan » lost to the eight-sense cap, back with the noun first |
| `forte` | adjective « strong » | the noun « fort, fortress » first |
| `partito` | noun « party, political party » | the heraldic adjective « party per pale » first |
| `squadra` | noun « team, squad, group, side » | « set square » first, in the same entry |
| `presto` | adverb « soon » | the literary adjective « ready » first |
| `valore` | noun « value » | the adverb « as much as, as many as » first |
| `testo` | noun « text (writing) » | the other noun entry's « an earthenware lid » first |
| `usato` | adjective « used, worn » | « usual, customary » first |
| `morire` | verb « to die » | the noun « death » first |
| `medio` | adjective « mean, average » | the noun « middle finger » first |
| `prego` | interjection « you're welcome, don't mention it etc. » | the noun « prayer » first |
| `stazione` | noun « station (stopping place for ground transportation) » | « position, posture » first, every sense marked archaic |
| `tratta` | noun « draft (financial instrument) » | D2b would open on the regional « seine fishing » |
| `signore` | capitalised noun « Mister or Sir » | D2b opens on « Lord, God » |
| `compagno` (1,482) | noun « mate, companion, partner… » | the informal adjective « alike, like, similar » first |

**In this order**: the treebank's, then D2b's, then the table's, each ahead of the previous. MarkIT's
counts and the table never touch a proper noun's entry. *Not taken*: the Italian Wiktionary's first
translation as the signal for the sense a row opens on — its file repeats a word's translations under
every part of speech and carries typos (`pratica` « pratcice »).

### D3 — A pointer the edition does not know is a pointer; a short word's pointer lends

The English edition reads pointers by their tags, fields and wordings (`_FORM_OF`); the section writes
Italian's own as plain senses, which then gloss. it-en's pass reads as a pointer to its word:
- **a labelled form or spelling** — labels (archaic, regional, dialectal, obsolete, Old Italian,
  proscribed, censored, dominant, a place's name) before « form of X » or « spelling of X », the word
  last or before a colon: `che` « proscribed spelling of ché », `de` « Marche, Milan, Rome, and Venice
  form of di », `el`, `amo` « Rome form of abbiamo: … », `ao` « dominant spelling of ahó », `meo`, `eo`,
  `istoria` « Old Italian form of storia », `phon`;
- **a clitic compound** (kaikki's `compound-of`, or « compound of apri, the second-person singular
  imperative form of aprire, with le »), whose meaning after its closing full stop is read when it writes
  one (`vendesi` « it gets sold, for sale ») — 12 senses of ranked lemmas: `aprile`, `fallo`, `stivale`,
  `cambiale`, `portale`;
- **a truncation** (« truncation of X »), and an untagged « alternative form of X » no field names, X its
  last word (`itria` → *lontra*, « otter »).

The shared rules lend only from a word of three letters or more (`_MIN_BASE`, so that a letter's plural
lends nothing). A pointer to an Italian word of one or two letters — `di`, `il`, `ci`, `io`, `si`, `mi`
— **lends that word's meanings of its part of speech, in its place, when its entry holds no meaning of
its own** and it is not marked archaic, obsolete, dated or a misspelling: `de` reads *di*'s « of, ’s;
from; … », `el` « the; he », `ce` « us; ourselves, each other; to us; …; reinforcing particle, “do” »,
`eo` « I », `se`, `me`, `te`, `ne` their clitic readings. *Narrowed*: without the marks, `il` gained an
obsolete pronoun's « him; it » and `the` (rank 177, the English article in Italian text) « tea », its
section's « misspelling of tè ».

**29 rows** (10 / 3 / 9 / 7), 14 first senses, no lemma lost: `de` and `eo`, which a pointer alone left
unglossed, read their word's meanings.

### D4 — Meanings a pointer carries, beyond change 22's list

- **a. A meaning nested under a degree's wording** is read by its own gloss, as the English edition reads
  one nested under a pointer (`_own_glosses`), which does not name a degree: `minimo` « minimum, (the)
  least…; very small… », `minore` « less; shorter; lesser; lower; younger… » in place of « comparative
  degree of piccolo; superlative degree of piccolo », 29 superlatives in -issimo. **31 rows** (1 / 2 /
  4 / 24), all better.
- **b. A comparative's or a superlative's carried meaning is read wherever it stands**, no longer only
  for a word whose lowercase entries hold nothing else — a degree's meaning is the word's own: `più`
  « more; anymore, any longer », `bravissimo` « very good; bravissimo! », `gentilissimo` « very kind,
  very polite; dear… ». **5 rows**, all better.
- **c. A clipping's carried meaning** (« clipping of automobile: car »): `auto` « car; bus; auto (genre
  of dramatic literature)… », `moto` « motorcycle; movement… », `bici` « bike (bicycle) », `ser` « sir »,
  `primo` gaining « first course, starter ». The historical senses `auto` opened on are marked so, and
  come after the cars by the page's order once the clippings read. **7 rows**, all better or no worse.
- **d. A plural's carried meaning beside another part of speech only**: `guai` « woe!; trouble »,
  `vegetali`. Beside a sense of its own part of speech the narrowing of change 22 stands — `fusa` keeps
  « purr » alone, `colla` « glue », `mura` « city walls, battlements ». **2 rows.**

### D5 — Female equivalents: no rule; four rows for the owner

106 ranked rows hold a « female equivalent of X » that lends nothing, because the word glosses another
sense: mostly right as they read (`politica` « politics », `fisica` « physics », `critica`
« criticism »), harmful where the other sense is narrow — `amica` (882) « girlfriend » alone, `bambina`
(1,479) « babe », `compagna` (2,810) « wife, spouse », `spagnola` (2,541) « Spanish influenza; tit
fuck… ». *Rejected — the pointer lending X's noun senses beside the word's own* (it-fr's D6 widened):
the masculine's English is often a man's — `signora` « lord (ruler) », `bambina` « boy, young boy »,
`compagna` « husband », `stronza` « turd », `fisica` « physicist (male) », `francesca` « Frenchman » —
and lending X's first sense alone keeps half of them (107 rows). fr-en's owner settled that a female
equivalent is no word's meaning (its Q3). **Proposed** (task 0.3): four rows of `CORRECTIONS` (D8)
adding, before the section's own sense, the person's meaning in the words of a source — `amica`
« friend » (`amico`'s first sense), `bambina` « girl, little girl », `compagna` « companion, partner »
(the Italian Wiktionary's translations), `spagnola` « Spaniard » (`spagnolo`'s noun) — each reported
upstream; or the rows as they are.

### D6 — Names' notes: a surname where the row says more, a given name after a common word, a town off a common word

change 22 kept a name beside a common word's senses and left the notes to this change; it-fr's D5 and
D9 (change 17) drop a surname's, a first name's and a commune's note from a word with a lowercase entry.
Applied as first worded to the English section — a surname's or a given name's note off any word with a
lowercase entry — it reads `francesco` « French; Frenchman », `luigi` « louis (coin) », `alice`
« anchovy », `gennaro` « January », and leaves `diego`, `roberta` and 15 more with no gloss: the English
section gives many given names an archaic or rare common entry. Decided:
- **a surname's note** (« a surname », « a surname transferred from the nickname », « a habitational
  surname from… ») goes from a row holding any other meaning — a common word's, a pointer that lends, or
  another proper noun's sense: `volta`, `marzo`, `musica`, `guerra`, `milanese` « Milanese »,
  `ferrari` « an Italian car manufacturer », `ventimiglia` « a town and municipality of the province of
  Imperia… »; a name alone keeps it (`mattarella` « a surname ») — 3,429 senses, 1,672 ranked rows;
- **a given name's note** (« a male given name… », « a diminutive of the female given name… ») goes only
  where a common word's meaning comes first, the row not opening on the name: `primo`, `giusto`,
  `massimo`, `rosa`, `domenica` « Sunday », `delfino` « dolphin; dauphin; probable successor;
  hunchbacked »; `francesco`, `alice`, `diego`, which open on the name, keep it (fr-en's Q5) — 192
  senses, 177 rows;
- **a town's, a comune's, a village's, a hamlet's, a frazione's or a municipality's note** goes from a
  word with a common meaning: `noto`, `grado`, `cambio`, `bagno`, `fonte`, `mura` « city walls,
  battlements; Mur (a river…) » — unless it names the place a capital or a city (`napoli`, `torino`), or a
  « town and province » (a provincial capital: `fermo`, `matera`), or a reviewed list keeps it, `TOWNS_KEPT`:
  `massa` (755), the capital of Massa-Carrara, « a town in Tuscany », which would read « mass; crowd »,
  and `andria` (10,986), the capital of Barletta-Andria-Trani, « a municipality of… province », which
  would read a salamander's genus — task 0.2. 225 senses, 122 rows.

`obbiettivo`, a surname's note beside an alternative form of *obiettivo*, then borrows « objective ».
**1,852 rows** (82 / 304 / 669 / 797), 186 first senses, none of the 1,000; one lemma loses its only
gloss, `pinuccio` (37,110), whose pointer borrowed `Pino`'s given-name note.

### D7 — The page's notes and typography

- **An article's sense describing its form** (« the form of il that is used before the so-called impure
  consonants…; before a vowel it becomes l'; the ») reads its last clause: `lo` « the; him; it, this or
  that thing », and every `l'` card with it. 1 row; the only such sense.
- **« etc » takes its period** (« etc. »), in glosses and expressions — fr-en's `with_etc_period`
  (refine-lingua-fr-en-glosses D8), written again in it-en's module, which cannot import fr-en's
  reducer: 71 rows, 13 expressions.
- **A list marker** opening a sense goes (`itria` « # alternative form of lontra »).
- **A carried meaning the English edition leaves in its quotation marks** — « (“sea”) (used in poetry
  and in names of some seas) », what `read_as_meanings` writes for `mar`'s « apocopic form of mare
  (“sea”) (…) » — reads « sea (used in poetry and in names of some seas) ». The edition's reading is
  es-en's and fr-en's too, so it is read again here, not edited (D9).

### D8 — `CORRECTIONS`

it-fr's reviewed list (change 17's D10), for it-en: each entry the source, the headword, its part of
speech, the source's text as the pinned file writes it, the corrected text or none, the reason and the
page to report upstream; an update finding a correction firing on nothing names it. Proposed:
- `all'antica`, the Italian Wiktionary's English translations « ancient, anshent, antique »: « anshent »,
  a jocular spelling, out — a mechanical correction;
- the four rows of D5 if the owner takes them (task 0.3).

Not `USA` (D9): the acronym requirement forbids a gloss of a pair's own words for a row.

### D9 — Measured and left

- **Bracketed constructions** (154 rows, 7 / 25 / 50 / 72: `chiedere` « to ask [with di ‘about
  (someone)’] », `rispondere` « to answer [with a ‘to’] ») and **auxiliary notes** (12 rows: `morire`
  « to kill [auxiliary avere] »): they say which preposition or auxiliary a verb's meaning takes, which
  the card says nowhere else; kept, as change 17 kept a conjunction's usage note (its D8).
- **`USA`** reads `america`'s first sense, the supercontinent, before `stati uniti`'s, by the acronym
  table's order and its requirement (« each cut to its first sense, in the row's order »). Reordering the
  row's synonyms edits `reduce_italian_acronyms.py`, which it-fr imports too: it re-pins it-fr, whose
  gloss would read « États-Unis; Amérique (continent) ». Not this change's; recorded in the programme's
  backlog.
- **Offensive senses unlabelled** (`negro` « black; negro; nigger; ghostwriter », `negra`): the page marks
  them; the card shows no label, as es-en's refinement left labels to the card.
- **`dovere`**'s interjection, a description (« Dismisses gratitude by clarifying one acted for one's
  duty… »): 1 row, a right reading.
- **Names opening a row** (`svizzera` « Switzerland; hamburger », `francesco`, `alice`): fr-en's Q5,
  accepted until a card reads the token's capital.
- **`the`** (177), the English article in Italian text, stays unglossed (D3 lends no misspelling);
  **`meo`** (17,074) reads *meare*'s « to filter through » once its Old Italian pointer and its surname go.
- **The English edition's rules** would gain from D3's wordings, D4a and D7's quoted opener for es-en
  and fr-en too: a refinement of the edition re-pins both, a change of its own.
- **Species among the expressions** (change 22's list): as change 17's D13, a rule cannot tell a rare
  sense from the plant or animal a word names.
- **Italian's forms**: `brutta` and `amica`, ranked as nouns of their own, read as those nouns wherever
  the adjective's feminine is written — it-fr's tables, not it-en's.

### D10 — it-en re-reduced at its pin; what moves and what cannot

One owner dispatch (`lingua-pack-update`, mode `reduce`, it-en) on this change's branch, from the pinned
release `lingua-pack-sources-it-en-2026.10.25`, MarkIT's two sections fetched at `e210079d` and checked
against the sha256s it-fr's pin records. The pin keeps its snapshot and its two section files byte for
byte, gains MarkIT's two entries, and its rule digest moves (`reduce-it-en.py`, `reduce_italian_en.py`,
and `reduce_french_treebank.py` and `reduce_italian_treebank.py` joining it), with `pack_version`, the
pack and the layer; the base is it-fr's, byte for byte. `gloss_coverage.py --pair it-en` passes the
floor: predicted 91.3 / 84.2 / 75.8.

**What may move, and nothing else**: `tables/it-en/gloss.tsv`, `senses.tsv`, `mwe.tsv`, `manifest.json`,
`pin.json`, `README.md`; `SOURCES.md`; `it-en.golden` and change 22's `word-card-it-en.txt` (D11).
**What does not**: `tables/it/` — so `measure/it-ud.sh`, which reads the studied side through it-fr's
pack, reads UD Italian-PUD 99.27 / 91.37 / 99.90 as now: no rule here touches a form, a lemma or a
reading —; the seven other pairs' tables, pins, packs, goldens and snapshots; `reduce_common.py`,
`reduce_edition_en.py`, `reduce_italian_acronyms.py` and the treebank modules; the extension's source;
Italian's analyser version.

### D11 — The golden and the card, read probe by probe

`LINGUA_BLESS=1 cargo test -p lingua-wasm --test it_en_baseline` once, after the dispatch; the owner
approves the re-bless on the listed probes (task 4.1). Expected to move: the pack line, and the `gloss`
and `phrase-gloss` probes of `lo` (every `l'`), `che`, `più`, `bene`, `ne`, `su`, `solo`, `dopo`, `uno`,
`come` held, the `di` family untouched, and every page token whose lemma's row moves; the vocabulary
estimate does not move (dictionary words are it-fr's). The studied side equals it-fr's golden as before
(`the_golden_is_the_italian_one_on_the_studied_side`); `cross_native` passes unblessed, the two packs
differing in their glosses alone. Change 22's `word-card-it-en.txt` moves with the same rows and is
re-blessed once, every moved line listed.

### D12 — Order

| Change | Relation |
|---|---|
| 22 `add-lingua-pack-it-en` | before: its first requirement is MODIFIED here, so this change archives after it |
| 17 `refine-lingua-it-fr-glosses` | before: its forms are the ranks measured here; its D2's rule is imported, not copied |
| 24 `measure-lingua-it-en-translation-marks` | either side: its tier reads the engine's column alone, so no mark moves |
| 25 `enable-lingua-italian-for-english-speakers` | after: it lists it-en once this change has merged (its D2); it renames change 22's other requirement, which this change does not touch, and adds this change to its `archiveAfter` (its task 4.4) |
| 28 `refine-lingua-it-es-glosses` | may reuse D2–D7 as rules of it-es's reducer |

## For the owner

Three questions, each with a recommendation (tasks 0.1–0.3, written there in French): the 19 rows of
`FIRST_RUN` (D2c), with D2b's two rows that open on a right but less common sense; the two cities
`TOWNS_KEPT` keeps (D6); the four female forms (D5). Decided here, so not asked: MarkIT's rule as it-fr
reads it (D2a), D2b's four marks and not « rare » or « literary », the pointer wordings and the short
words' lending (D3), the degrees, clippings and plurals (D4), a surname's and a given name's notes as
D6 words them, the notes and typography (D7), `all'antica` (D8), and D9's list; the golden's re-bless
is approved on its listed probes after the implementation (task 4.1).

## Risks / Trade-offs

- **[The page's marks are its own]** → D2b follows them; a wrong mark moves a row wrongly, and the two
  rows where every sense is marked (`gente`, `stazione`) are the table's; every changed row of the 5,000
  is in the pull request's listing.
- **[MarkIT is thin]** → it moves 16 function words, all read; content words keep the page's order
  (fr-en's Q2).
- **[A reviewed table grows]** → `FIRST_RUN`, `TOWNS_KEPT` and `CORRECTIONS` each hold a reason per row,
  and a row whose sense or text the source no longer writes fails or names itself.
- **[A pin gains a source]** → MarkIT's two files are it-fr's, at the same commit and sha256s; they are
  read for counts alone, never glosses.
- **[es-en and fr-en keep what it-en loses]** → named (D9); they do not move here.
- **[A golden that moves]** → re-blessed once, each probe with its rule, approved on the list.

## Migration Plan

No reader holds an it-en pack before change 25: nothing to migrate. The tables arrive by the owner's
dispatch on this change's branch; rollback is a revert of the tables, the pin and the golden.

## Effort

2.5–3.5 ideal days, against the programme's 2–3.5 (48b): the pass reading a headword's entries together
and the rules D2–D7 with their tests 1–1.5; `FIRST_RUN`, `TOWNS_KEPT`, `CORRECTIONS` and MarkIT's pin
entry 0.5; the dispatch, the pin, the README, `SOURCES.md` and the measurements 0.5–0.75; the golden,
the card and the checks 0.25–0.5; the owner's reading 0.25. MarkIT's source and the sense-keyed table,
absent from 48b, account for the difference.
