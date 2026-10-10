# Design — read-lingua-french-ca-as-ca-cedilla

## Context

See proposal.md (Why). What exists, on `main` at `ea3d89de` (after change 49b's implementation, which
re-pinned fr-es):

| Where | What |
|---|---|
| `reduce-fr-en.py` | French's reference reducer (change 43): `Lexicon.read` reads each entry of the English Wiktionary's French section, lowercased and in NFC (`nfc_lower`); an elided piece is read by the reviewed table `ELISIONS` alone (form → word, reason) and returns before its entry is read; `OVERRIDES` (three copy errors) is the first rule of `choose_lemma`, which applies a row only among a form's candidates; D7's `spelling_target` reads a post-1990 spelling and an ASCII spelling of a ligature as forms of their word. Then the ranks (wordfreq, D6), the readings (45), fr-en's glosses (48, 48b), French's dictionary words and levels (46, 48b) |
| `tables/fr/` | 124,096 forms, 60,000 ranked lemmas, 125,177 readings, 26,486 dictionary words, 8,302 levels (English's sizes in rank order: A1 1–1,085 … C2 9,475–10,820); `ca` → *ca* (rank 144, A1, a dictionary word), `ça` → *ça* (rank 26, A1), `ç'` → *ça* (`ELISIONS`); `age` → *age* (2,461, B1), `âge` (407, A1); `forcement` → *forcement* (9,588, C2), `forcément` (1,287, A2) |
| `tables/fr-en/` | `ca` « board of directors » (`NOUN:1`), `age` « beam (central bar of a plough); shaft », `forcement` « fixing number, cooking the books »; coverage 93.6 / 86.9 / 76.3 % (4,679 / 8,688 / 15,260) against `FLOORS["fr-en"]` 91.9 / 85.1 / 74.4; pinned at snapshot `2026.10.09`, the pack 2,537,386 B; rule digest `reduce-fr-en.py`, `reduce_common.py`, `reduce_edition_en.py`, `reduce_french_treebank.py` |
| `tables/fr-es/` | no gloss for `ca`, `age` or `forcement`; `ça` « (coloquial) Eso, esto, aquello », `âge` « Edad »; coverage 83.0 / 70.7 / 56.7 % (4,150 / 7,065 / 11,340); the pack 1,971,289 B; its pin records `tables/fr/`'s six files by sha256 (`studied`), and its `pack_version` a digest of them (`2026.10.10+4e7aa33.1cc934b`) |
| Change 41's cascade | `french::lemmatize` looks every token up lowercased and in NFC: `CA`, `Ca` and `ca` are one form to the pack. Its D2 measured and rejected « a capital read without its accent (`Ecole` → `école`) » as a cascade rule |
| Change 48b | its D9 measured `ca` and left it; its Q4, settled by the owner on 2026-10-10: « `ca` is to read as a spelling of `ça` — a change to French's forms table (change 43's), proposed later as a change of its own » |
| The number | change 43's design reserved « 43b » for a split it never needed (`refine-lingua-french-forms-tables`); 43 merged whole, so the row is this change's |

What the French section says of the words, read in the pinned `kaikki-French.jsonl` (sha256
`2d7bbe5f…`, the pin's):

| Entry | Part of speech | Senses |
|---|---|---|
| `CA` | noun, plural `CAs` | « initialism of conseil d'administration », « … of chiffre d'affaires », « … of courant alternatif ("AC") », « … of comptable agréé » (Canada) — each tagged abbreviation, initialism, alt-of |
| `ca` | preposition | « abbreviation of circa » |
| `ça` | pronoun; noun | thirteen pronoun senses (« that », « this », « it », Louisiana's « he, she, it »…); the noun « id » |
| `age` | noun, plural `ages` | « beam (central bar of a plough) », « shaft » |
| `ages` | noun | « plural of age » |
| `forcement` | noun, plural `forcements` | « fixing number; cooking the books » |

No entry says `ca`, `age` or `forcement` spells another word. Both `ca` entries are lemmas, lowercased
into one: `ca` is its own candidate, and `CAs`, lowercased, links `cas` to it (`cas` keeps *cas* by
GSD's counts).

## Goals / Non-Goals

**Goals:**
- `ca` read as `ça`, `age` as `âge` and `forcement` as `forcément` in French's tables, as the owner
  decided, by a rule a person reviews row by row.
- What the spellings' own words become decided and measured — the acronym, circa, the plough's beam,
  the fixing number, their ranks, levels, dictionary words and glosses — and what every table, pin,
  golden and snapshot shows moving.
- « CA » in capitals kept as the acronym, by a change of its own whose options this design sketches,
  and this change reaching readers only with it or after it.

**Non-Goals:**
- Any byte of en-fr, es-fr, es-en or en-es, and any shared or edition module.
- A core rule here: the acronym change is its own (D2); accents restored on capitals stay rejected
  (change 41's D2). French's analyser version does not move.
- The dictionary's own unmarked spellings and the web's unglossed ones: changes of their own (D6).
- fr-en's colliding `ça va` and `ça ira` keys (D5, *Found while measuring*).

## Measured

A prototype in the scratchpad (never committed): `reduce-fr-en.py` of `main` with the table of D1
behind a switch, its inputs the pinned release's `kaikki-French.jsonl` (sha256 `2d7bbe5f…`), GSD's
training and development sections at the pinned commit (sha256 `4b9a87b1…`, `9221e508…`, the pin's)
and wordfreq 3.1.1. Switch off, it writes `forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`,
`lexical.tsv`, `gloss.tsv`, `senses.tsv`, `mwe.tsv` and `NOTICE` byte for byte as committed; fr-es's
reducer of `main` over its pinned sources writes `tables/fr-es/` byte for byte, and the packs built
from both have their pins' sha256. Measured with `ca` alone first, then with the four rows the owner
settled; the figures below are the four rows'. fr-es was measured again once change 49b's
implementation reached `main` (`9eb63300`): its reducer of `ea3d89de`, which reads GSD's two sections
too, reproduces its committed tables and pin. The goldens were re-blessed in a scratch copy of `main`
with D5's four phrase probes, on `main`'s tables and on the prototype's.

## Decisions

### D1 — A reviewed table of spellings, read as the elided pieces are

`reduce-fr-en.py` gains `UNMARKED_SPELLINGS`, form → (word, reason), beside `ELISIONS`, with four
rows:

```python
UNMARKED_SPELLINGS = {
    "ca": ("ça", "ça written without its cedilla (the owner's decision of 2026-10-10): UD's treebanks "
                 "read every « ca » and « Ca » as ça; the section's own entries — CA's initialisms, "
                 "ca « abbreviation of circa » — leave the tables"),
    "age": ("âge", "âge written without its accent (the owner, 2026-10-10): UD reads both « age » as "
                   "âge; the section's « beam (central bar of a plough); shaft » leaves the tables"),
    "ages": ("âge", "âges written without its accent, age's row's plural (the owner, 2026-10-10)"),
    "forcement": ("forcément", "forcément written without its accent (the owner, 2026-10-10); the "
                               "section's « fixing number, cooking the books » leaves the tables"),
}
```

`Lexicon.read` reads a row's form as it reads an elided piece: the named word is its one candidate,
and the entry returns before anything else is read, so neither `CA` nor `ca` (nor `age`, `ages`,
`forcement`) makes it a lemma, links it to another word or lists an inflection (`CAs` no longer links
`cas` to it); `_inflections` skips the form wherever another entry lists it, as it skips a piece. The
implementation seeds every row's candidate once the section is read, as `own_words` seeds `du` and
`des`, so that a row holds whether or not an entry names its form (each of the four has one today).
The rest follows from change 43's rules, unchanged: a row's form has the named word as its first
choice, so it is no rank (D6 of change 43); wordfreq attests it (`ca` Zipf 5.60, `age` 4.30, `ages`
3.05, `forcement` 3.47), so it is a form. `forcements`, which only the noun's entry listed and which
`forcément`, an adverb, has no plural for, is no form any more; `ages` takes its row, as `âges` reads
as *âge*. `read_readings` reads no `CA` (a capitalised headword) and no `ca` (a preposition, no part of
speech it keeps); the readings `age`, `ages`, `forcement` and `forcements` had under their own words
leave with those words, and no reading is written under the named words for them (a card opened on
one shows its word's gloss and senses). Editing the table changes the rules' sha256, as
for any rule; its test holds that every row has a reason, that its word is a ranked lemma reading as
itself, and that its form is neither an elided piece nor `du` or `des`.

**Never a row**: `cote` (« rating »), `tache` (« stain »), `pale` (« blade »), `foret` (« drill bit »)
and `aine` (« groin ») — words a reader can mean, though their marked spelling is commoner (D6).

*Rejected — an `OVERRIDES` row `ca` → `ça`.* `choose_lemma` applies a row among a form's candidates
only (« An override naming a lemma the form cannot have is no decision », its test), and no entry
links `ca` to `ça`; making it a candidate would still read `CA`'s and circa's entries, keep `ca` a
lemma for the first choice, and leave `CAs` linking `cas` to it. An override chooses between what the
dictionary offers; this row says what the dictionary does not.
*Rejected — change 43's spelling rule (D7).* It reads what an entry says it spells; no entry says it
of these three (D6 records the entries that do say it of other words).
*Rejected — a core rule* (an unaccented word retried with its marks, as Spanish's cascade retries a
retired accent). Change 41 measured a capital read without its accent and rejected it (its D2); a
cascade rule would bump French's analyser version for words the tables can name.

### D2 — The spellings' own words are lost; « CA » in capitals waits for a change of its own

The rows make each spelling a form of its word alone, so the four initialisms of `CA`, circa, the
plough's beam and the fixing number leave the tables: no rank, no level, no dictionary word, no gloss
in fr-en (fr-es glossed none of them).

**Until the acronym change, `CA` in capitals reads as `ça`.** The pack keys its forms in lower case,
and French's cascade looks every token up lowercased (change 41): `CA`, `Ca` and `ca` are one form,
and no forms table can read them apart. Measured (D3), the case would tell them: every `ca` and `Ca`
that is a French word is `ça`, every `CA` the acronym — UD's three sports clubs (« CA Vitry », « CA
Peñarol », « CA Montreuil »), the Wikipedia sample's five (four of them Geneva's Conseil
administratif). The names rule (change 41) sets aside neither: a capital in mid-sentence is set aside
only when its lemma is no dictionary word, and *ca* was one as *ça* is. The owner settled on
2026-10-10 that « CA » in capitals stays the acronym, through a deeper analyser change of its own, and
that this change reaches readers only together with that change or after it (task 6.4), unless the
owner decides otherwise; the phrase probe « Le CA a voté le budget. » records what it reads until
then (D5), and moves with the acronym change.

**The acronym change's options**, sketched so that it can be proposed. The section lists 609 headwords
in capitals as initialisms or acronyms; lowercased, 433 are ranked in `tables/fr/`, and 60 share their
lower case with another word of the tables or read as another lemma — `LE`, `NE`, `SE`, `SA`, `SI`,
`CES`, `VA`, `EAU`, `CAS`, `MUR`, `BAC`, `TIR`, `ARC`, `CV`, `SVP`, `MDR`, and now `CA`. French writes
headlines, titles and shouted lines in capitals (« IL VA PLEUVOIR », « LE MONDE »).

| Option | What it does | Cost | Risk |
|---|---|---|---|
| A. **A small reviewed acronym table, looked up first** (recommended) | A token of two letters or more written wholly in capitals, in a block not itself written in capitals, is looked up first in a reviewed table of the pack — `CA` alone at first —, whose row names the acronym's own lemma (cased, `CA`) and its gloss (fr-en: the section's four initialisms); anything else is read as today | French's analyser (a version bump, every French golden's version line); a pack section or a cased form for the table's rows, the builder accepting a lemma with capitals; the reducer keeping `CA`'s entry for the table; a gloss under `CA` in fr-en (fr-es has none), a card, a status and a sync key holding capitals; the names rule reading the acronym as a word. About 2–3.5 ideal days | small: a row per acronym a person reviews; a headline in capitals is read as today by the block's case |
| B. **Cased forms for every acronym** | The pack keeps the section's 609 acronyms with their capitals, and a token in capitals is looked up as written first | as A, and a table of 609 rows to check | large: the 60 that share a word's spelling would read « LE », « VA », « SI », « EAU » as acronyms in any line written in capitals; a block-case guard narrows it, a review of 60 rows does not scale |
| C. **Set aside, no gloss** | A token in capitals whose lower case a reviewed row names (`ca`) is set aside as the names rule sets a name aside: no card, no count | French's analyser (a bump) and a pack list of the rows; about 1–1.5 ideal days | the reader learns nothing of « CA »: no wrong gloss, no right one either |

*Rejected here — keeping a lemma `ca` for the acronym beside the form.* A lemma is keyed by its own
form (change 43's D6, the builder's `id_of`): a form `ca` reading as *ça* and a lemma `ca` cannot both
be, the second losing its rank, gloss and level to the first (« one form, one lemma », M8). Option A
keys the acronym by its capitals instead.

### D3 — The spellings in the corpora

Each standalone token whose lower case is `ca`, by how it is written:

| Corpus | Words | `ça` (written so) | `ca`, `Ca` read as `ça` | `CA`, the acronym | Circa | No French word |
|---|---|---|---|---|---|---|
| UD French-GSD, training and development (fr-en's pin) | 390,368 | 36 | 3 (`Ca` 2, `ca` 1) | 3 (clubs, `PROPN`) | 0 | 0 |
| UD French-GSD, test | 10,017 | 4 | 1 (`Ca`), and `Cà` 1 | 0 | 0 | 0 |
| UD French-PUD | 24,726 | 9 | 2 (`Ca`, `ca`) | 0 | 0 | 0 |
| UD French FQB, ParTUT, ParisStories, Rhapsodie, Sequoia (test, and Sequoia's development) | 69,006 | 130 | 0 | 0 | 0 | 0 |
| Tatoeba, every French sentence (727,867) | 5,265,495 | 18,194 | 19, all `Ca` at a sentence's head | 0 | 0 | 0 |
| French Wikipedia (change 44c's sample) | 351,624 | 59 | 0 | 5 | 1 (« ca. 1893 ») | 81: Catalan's language code « (ca) » 69, `.ca` addresses 11, a quoted Catalan prefix 1 (« (CA) », Geneva's council, counted with the acronym) |
| Gutenberg (change 44c's four novels) | 209,690 | 191 | 0 | 0 | 0 | 2 `.ca` addresses |
| Change 39's corpus (`pages-fr.txt`) | 980 | 0 | 1 (« ca va aller ») | 0 | 0 | 0 |

UD's lemma column says each: GSD's « J'adore ce concept, et ils sont parfaits pour ca ! », « Ca fait
du bien. », « Ca fait un moment que ce restau a changé », PUD's « vous avez vu ca ? » all `ça`; its
`CA` all `CA`. **wordfreq** 3.1.1, whose French reads the web, subtitles and social media: `ca` Zipf
5.60, `ça` 6.53 — 12 % of `ça`'s frequency, `ca` the 208th word of its list and the 144th rank of the
tables. French's other cedilla-less spellings run at 2–4 % of their word (`francais` 4.07 against
5.78, `facon` 3.65 / 5.39, `garcon` 3.25 / 4.64, `recu` 3.24 / 5.01), so `ca`'s share is three to six
times theirs: the informal `ça` drops its cedilla more often, and the acronym and the non-words
(language codes, addresses) add to it. **No web forum corpus was in reach** — changes 42, 42b and 44c
measured on Wikipedia, Gutenberg, Tatoeba and UD, which are edited text; wordfreq is the web's
evidence, and the owner's.

**`age` and `forcement`**: UD's treebanks write `age` twice, both « Age » with a capital and both
lemmatised *âge*, and `forcement` never; wordfreq rates `age` at 14 % of `âge` (Zipf 4.30 against 5.16) and
`forcement` at 6 % of `forcément` (3.47 against 4.66).

The non-words read today as *ca* « board of directors » and will read as *ça*: wrong both ways, as
any language code or address the tokeniser takes for a word.

### D4 — What moves in the tables, the pins and the measurements

| File | Before → after | What |
|---|---|---|
| `tables/fr/forms.tsv` | 124,096 → 124,101 rows | `ca` → *ça*, `age` and `ages` → *âge*, `forcement` → *forcément* (each was its own word's); `forcements` out; `cussac`, `céphalonie` and `côtelé` (with `côtelée`, `côtelées`, `côtelés`) enter with their ranks |
| `tables/fr/freq.tsv` | 60,000 rows | `ca` (144), `age` (2,461) and `forcement` (9,588) out; every rank after each up by one, two or three (59,851 rows); `cussac`, `céphalonie`, `côtelé` in at 59,995–59,997, before the three nouns ending in a pronoun GSD never meets |
| `tables/fr/grammar.tsv` | 125,177 rows | `age`, `ages`, `forcement` and `forcements`'s readings out (their words leave); `côtelé`'s four in |
| `tables/fr/tags.tsv`, `studied.json` | byte for byte | |
| `tables/fr/lexical.tsv` | 26,486 → 26,484 words | `ca`, `age`, `forcement` out; `côtelé` in (`cussac` and `céphalonie` are glossed as names alone, no dictionary words) |
| `tables/fr/level.tsv` | 8,302 rows, English's sizes | `ca` (A1), `age` (B1) and `forcement` (C2) out; eight words up a band — `croissance` A2 → A1, `communiste` B1 → A2, `australien` and `boue` B2 → B1, `naïveté` and `parano` C1 → B2, `morphologie` and `nantais` C2 → C1 —; `crabe`, `cuisinière` and `dar` gain C2. The spans in ranks move by at most three (B2 from 4,836, C1 from 8,141) |
| `tables/fr-en/gloss.tsv`, `senses.tsv` | 30,067 rows | out: `ca` « board of directors », `age` « beam (central bar of a plough); shaft », `forcement` « fixing number, cooking the books »; in: `cussac` « a surname from Occitan », `céphalonie` « Cephalonia… » (names), `côtelé` « ribbed » |
| `tables/fr-en/mwe.tsv` | byte for byte | no headword holds `ca`, `age` or `forcement`, so no key moves |
| fr-en's coverage | 93.6 / 86.9 / 76.3 % → the same | 4,679 / 8,688 / 15,260 → 4,679 / 8,688 / 15,259 of the 5,000 / 10,000 / 20,000 commonest |
| `tables/fr-en/pin.json`, `manifest.json` | `reducer` and `pack` move, `pack_version` with the rule digest | `snapshot` and `sources` byte for byte |
| `tables/fr-es/gloss.tsv`, `senses.tsv` | 19,022 → 19,023 rows | every row it had byte for byte; `côtelé` « Pana » (`ADJ:1`) gained with the lemma entering the cut; fr-es glossed none of the three spellings |
| `tables/fr-es/mwe.tsv`, `NOTICE` | byte for byte | |
| fr-es's coverage | 83.0 / 70.7 / 56.7 % → 83.0 / 70.7 / 56.7 % | 4,150 / 7,065 / 11,340 → 4,150 / 7,067 / 11,342 (`digérer` and `doublure` enter the glossed top 10,000); against its floor 81.4 / 68.8 / 54.5 % |
| `tables/fr-es/pin.json`, `manifest.json` | `studied` (`forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`, `lexical.tsv`) and `pack` move, `pack_version`'s studied digest with them | its reducer digest and sources byte for byte |
| The packs (today's manifests) | fr-en 2,537,386 → 2,537,588 B; fr-es 1,971,289 → 1,971,114 B | the manifests' new `pack_version` adds or takes a byte or so |
| UD French-PUD (gated), GSD's test section (reported) | 99.13 / 96.41 / 99.90 %; 98.89 / 95.86 / 99.72 % — unchanged | the three `ca`/`Ca` of PUD and GSD's test section are pronouns, which resolved before and are no content words; neither section writes `age` or `forcement` |

With `ca` alone the prototype moved `forms.tsv` by one row, `level.tsv` by six (five words up a band,
`crabe` into C2), no reading, and fr-es's glosses not at all; `age` and `forcement` add the rest.

`ça`, `âge` and `forcément` keep their levels (A1, A1, A2), their readings and their glosses; `ça`
keeps its rank (26: wordfreq ranks a lemma by its own frequency, not its forms'), and `âge` (407 →
406) and `forcément` (1,287 → 1,286) move up one with every rank after `ca`.

**en-fr, es-fr, es-en and en-es cannot move.** The table lives in `reduce-fr-en.py`, which fr-en's
rule digest names and no other pair's does (`reduce-es-en.py`'s digest is it, `reduce_common.py` and
`reduce_edition_en.py`); `tables/en/`, `tables/es/` and their pairs' folders are not written; no core,
wasm or extension code changes. Over the prototype, the bless of every lingua-wasm test moved
`fr-en.golden` and `fr-es.golden` alone, and `cross_native.rs`, `parity.rs`, `card_gloss_language.rs`,
`deck.rs` and the other baselines passed as committed. `lingua-pack`'s committed-tables tests failed
only on fr-en's and fr-es's pins (their pack sha256 and fr-es's studied record), which the re-pin
records.

### D5 — The goldens and the snapshots

**Re-blessed on the prototype's tables, 9 of the 213 probes on `main` move in each golden**, the same
nine:

| Probe | fr-en | fr-es |
|---|---|---|
| `pack` | 2,537,386 → 2,537,588 B (and the new `pack_version`) | 1,971,289 → 1,971,114 B (and the new `pack_version`) |
| `analyse new-reader informel` | « ca » *ca* « board of directors » → *ça* « that (distal demonstrative pronoun)… »; its class (Unknown) and the page's count (37) unchanged | *ca*, no gloss → *ça* « (coloquial) Eso, esto, aquello » |
| `vocabulary-estimate new-reader` | universe 26,486 → 26,484 | the same |
| `vocabulary-estimate reader` | estimated 2,184 → 2,185, universe 26,485 → 26,483 | the same |
| `review-current first`, `second` | `croissance`, `célèbre` → `célèbre`, `davantage`: `croissance` is A1 now, so the deck seeded from A2's five commonest starts one word later | the same, in Spanish |
| `export-status-ops`, `export-card-ops`, `backup` | the seeded deck: `croissance`, `massacrer` and `miser` out, `exposition`, `morphologie` and `nantais` in (C1's three rarest glossed now `mono`, `morphologie`, `nantais`) | `croissance` and `marijuana` out, `exposition` and `morphologie` in (fr-es seeds its own glossed lemmas, change 49c) |

No `gloss`, `word-grammar` or `phrase-gloss` probe on `main` moves, and the studied side of the two
goldens stays one (`the_golden_is_the_french_one_on_the_studied_side` passes).

**Four phrase probes are added** to the French scenario (`support/french.rs`, before « l’homme »), so
the goldens show the rule where a reader meets it. Blessed on `main`'s tables and on the prototype's:

| Probe | fr-en, today → this change | fr-es, today → this change |
|---|---|---|
| « comme ca » | *ca* « board of directors », no expression → *ça*, the expression `comme ça` « like that/this… » | *ca* unglossed → *ça*; fr-es has no `comme ça` |
| « c'est ca » | `c'est` over two tokens → `c'est ça` « Confirmation of a question: that's right » over three | no expression → `c'est ça` « Así es o tener la razón, eso es; (irónico)… » |
| « Le CA a voté le budget. » | *ca* « board of directors » → *ça* « that… », flagged a function word — the record of D2, which the acronym change moves | *ca* unglossed → *ça* « (coloquial) Eso, esto, aquello » |
| « à mon age » | *age* « beam (central bar of a plough); shaft » → *âge* « age » | *age* unglossed → *âge* « Edad » |

Each golden gains 8 lines: `fr-en.golden` 370,503 → 376,285 B, `fr-es.golden` 248,267 → 250,296 B.
`test/word-card-fr-en.spec.ts` and `word-card-fr-es.spec.ts` render every grammar and phrase probe of
their golden: none of the probes they render before moves, so their snapshots gain the four cards and
nothing else, and their count of phrase probes goes 34 → 38.

*Rejected — a `gloss ca` probe.* A `gloss` probe asks a lemma's gloss, and `ca` is no lemma any more:
it would record nothing.

**Found while measuring, not this change's.** fr-en holds `ça va` (« how are you? how's it going? »)
and `ça ira` (« it will be all right »), both keyed *ça aller*, and the key answers `ça ira`: on `main`,
« ça va » already shows « it will be all right… », and « ca va » will. fr-es holds `ça va` alone. The
choice between headwords sharing a key is change 44's; this change only lets « ca va » reach it. It is
recorded here and in the pull request; the owner will be asked about it separately, and no probe of
this change records it.

### D6 — What is not read alike here, measured

wordfreq's French list holds 475 words of Zipf ≥ 3 that are another French word of the tables with
its accent, cedilla, diaeresis or circumflex dropped and less than half as frequent as it. By what the
tables make of them today, and what the owner settled on 2026-10-10:

**Real words, never read as another.** The marked word may be rarer or commoner; the unmarked one is
a word, and UD's lemmas say so:

| Spelling | Its word | The marked word | Zipf, spelling / marked | UD reads the spelling as the marked word |
|---|---|---|---|---|
| `ou` | « or » | `où` | 6.46 / 6.07 | 8 of 1,048 |
| `a` | *avoir* | `à` | 7.00 / 7.26 | 118 of 3,405, 100 of them a capital `A` (change 41's D2: the tokeniser's knowledge) |
| `la`, `des`, `du`, `sur` | *le*, *des*, *du*, *sur* | `là`, `dès`, `dû`, `sûr` | 7.43 / 5.99 … 6.77 / 5.27 | 2 of 13,503; 0 of 2,200; 3 of 133; 2 of 2,009 |
| `mais`, `mur`, `jeune`, `notre`, `votre` | their own | `maïs`, `mûr`, `jeûne`, `nôtre`, `vôtre` | the marked word 20 to 430 times rarer | 0 |
| `cote`, `tache`, `pale`, `metal` | « rating », « stain », « blade », « metal (music) » | `côté`, `tâche`, `pâle`, `métal` | 4.24 / 5.50, 3.88 / 4.46, 3.16 / 3.81, 3.83 / 4.34 | 1 of 3 (as *côte*), 0 of 5, —, 1 of 11 |

**Rare words a commoner unmarked spelling swamps** — `ca`'s kind, a levelled or glossed word whose
meaning a reader almost never means:

| Spelling | Today | The marked word | Ratio | UD | Settled |
|---|---|---|---|---|---|
| `age` | rank 2,461, B1, « beam (central bar of a plough); shaft » | `âge` | 0.14 | 2 of 2 read as *âge* | a row here, with `ages` |
| `forcement` | rank 9,588, C2, « fixing number, cooking the books » | `forcément` | 0.06 | — | a row here; `forcements` no form |
| `foret`, `aine`, `envoye`, `genie`, `alle` | ranks 12,780–18,735, no level: « drill bit », « groin », « slowworm », a diminutive of Eugénie, « she » (North America) | `forêt`, `aîné`, `envoyé`, `génie`, `allé` | 0.02–0.11 | — | not here; `foret` and `aine` never |

**The dictionary's own unmarked spellings — a change of its own** (settled 2026-10-10: yes). 186
entries of the section are only another word written without its marks — « alternative spelling of
État », « obsolete spelling of être » (Internet, colloquial), « nonstandard spelling of Écosse » —; 51
are ranked: `etat` 490, `etats` 868, `etre` 1,015, `etats-unis` 1,355, `etait` 1,996, `arreter`
8,060, `siecle` 15,372… They are words of their own today, glossed through 48b's pointer meanings
(`etre` « to be… ») and levelless (46's rule 3). Change 43's D7 reads two kinds of the dictionary's
spellings as forms of their word; that change reads this third kind alike **when the marked word is
the commoner**: `diesel` « alternative spelling of diésel » is 78 times commoner than `diésel` (Zipf
4.05 against 2.16), `ego` than `égo` (3.87 / 3.22), both levelled (B1, B2), and keep their own.
Proposed later, measured as this one (e.g. `read-lingua-french-dictionary-unmarked-spellings`).

**The web's unglossed spellings — a reviewed list by name, a change of its own** (settled
2026-10-10: yes). 301 of the 475 carry no gloss: ranked today as unglossed words of their own, or no
form at all — `meme` 4.77, `tres` 4.52, `etait` 4.42, `apres` 4.33, `ecole` 4.21, `deja` 4.06, and the
cedilla's: `francais` (rank 3,740), `facon` (7,296), `francaise` (9,145), `francois` (10,513),
`garcon` (13,328), `recu` (13,564). A reader meets each as an unknown word with no gloss. The list is
reviewed by name, never a rule by marks alone: it would read English words (`video`, `hotel`,
`education`, `general`, `these` → *thèse*), names (`jesus`, `noel`, `grace`, `israel`) and capitals
French sometimes writes unaccented (`Etat`, `Eglise`: change 41's D2) as French words they are not.
Proposed later, with this change's table as its mechanism (e.g.
`read-lingua-french-web-unmarked-spellings`).

## Risks / Trade-offs

- **[« CA » reads as `ça` until the acronym change]** → this change reaches readers only together with
  that change or after it (task 6.4), unless the owner decides otherwise; the probe records it; D2
  sketches the options.
- **[A language code or a web address reads as `ça`]** → it read as « board of directors » before:
  wrong both ways, as any non-word the tokeniser keeps.
- **[`forcements` loses its form]** → Zipf 1.01, the plural of the fixing number only; an unknown word,
  as any word the cut leaves out.
- **[A few rows hide the others]** → D6 records the two follow-ups the owner settled and the words
  that are never rows.
- **[fr-es moves again before this lands]** → change 49b's implementation already did, and fr-es was
  measured again on it (*Measured*); a later fr-es change re-reduces on whichever `tables/fr/` is
  committed, and the second to land re-blesses `fr-es.golden`.
- **[wordfreq's next version]** → the rows do not depend on frequencies; an update re-reduces with it
  and the test holds each row's word ranked.

## Migration Plan

Nothing to migrate: no package lists a French pair, no reader holds a status, card or level for `ca`,
`age` or `forcement`. Rollback is a revert of the reducer's rows, the re-reduced tables, the pins and
the re-blessed files.

## Effort

0.75–1.25 ideal days, outside the programme's estimates: the table and its tests 0.25, the
re-reduction of fr-en and fr-es with their pins, READMEs and `SOURCES.md` 0.25–0.5, the probes, the
two re-blesses and the snapshots 0.25, the committed-tables test and the gates 0.25. The acronym
change (D2, option A) is estimated at 2–3.5 ideal days of its own.

## Settled by the owner (2026-10-10, in session)

- **Q1 — « CA » in capitals.** *Settled*: kept as the acronym, through a deeper analyser change of its
  own (D2 sketches its options; A recommended). Until it lands, « CA » reads as `ça` here, and this
  change reaches readers only together with it or after it, unless the owner decides otherwise
  (task 6.4). The probe « Le CA a voté le budget. » stays as the record.
- **Q2 — The dictionary's own unmarked spellings** (186 entries, 51 ranked: `etre`, `etat`, `etait`).
  *Settled*: read as their accented word when it is the commoner, in a change of its own (D6).
- **Q3 — Rare words an unmarked spelling swamps.** *Settled*: `age` → *âge* (with `ages`) and
  `forcement` → *forcément* join this change's table (D1, re-measured in D4 and D5); `cote`, `tache`,
  `pale`, `foret` and `aine` are never rows.
- **Q4 — The web's unglossed spellings** (301 at Zipf ≥ 3: `francais`, `tres`, `deja`, `meme`).
  *Settled*: a reviewed list by name, in a change of its own (D6).
- **The `ça va` / `ça ira` shared key** (D5, *Found while measuring*): recorded; the owner is asked
  separately.
