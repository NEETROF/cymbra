# Design — read-lingua-french-ca-as-ca-cedilla

## Context

See proposal.md (Why). What exists, on `main` at `f3585580` (change 44c's implementation, #873):

| Where | What |
|---|---|
| `reduce-fr-en.py` | French's reference reducer (change 43): `Lexicon.read` reads each entry of the English Wiktionary's French section, lowercased and in NFC (`nfc_lower`); an elided piece is read by the reviewed table `ELISIONS` alone (form → word, reason) and returns before its entry is read; `OVERRIDES` (three copy errors) is the first rule of `choose_lemma`, which applies a row only among a form's candidates; D7's `spelling_target` reads a post-1990 spelling and an ASCII spelling of a ligature as forms of their word. Then the ranks (wordfreq, D6), the readings (45), fr-en's glosses (48, 48b), French's dictionary words and levels (46, 48b) |
| `tables/fr/` | 124,096 forms, 60,000 ranked lemmas, 125,177 readings, 26,486 dictionary words, 8,302 levels (English's sizes in rank order: A1 1–1,085 … C2 9,475–10,820); `ca` → *ca* (rank 144, A1, a dictionary word), `ça` → *ça* (rank 26, A1), `ç'` → *ça* (`ELISIONS`) |
| `tables/fr-en/` | `ca` « board of directors » (`NOUN:1`); `ça` « that (distal demonstrative pronoun); this…; it… »; coverage 93.6 / 86.9 / 76.3 % (4,679 / 8,688 / 15,260) against `FLOORS["fr-en"]` 91.9 / 85.1 / 74.4; pinned at snapshot `2026.10.09`, the pack 2,537,386 B; rule digest `reduce-fr-en.py`, `reduce_common.py`, `reduce_edition_en.py`, `reduce_french_treebank.py` |
| `tables/fr-es/` | `ca` has no gloss, `ça` « Eso, esto, aquello »; coverage 83.2 / 70.8 / 56.8 % (4,161 / 7,082 / 11,359); its pin records `tables/fr/`'s six files by sha256 (`studied`), and its `pack_version` a digest of them (`2026.10.10+3f293e8.1cc934b`) |
| Change 41's cascade | `french::lemmatize` looks every token up lowercased and in NFC: `CA`, `Ca` and `ca` are one form to the pack. Its D2 measured and rejected « a capital read without its accent (`Ecole` → `école`) » as a cascade rule |
| Change 48b | its D9 measured `ca` and left it; its Q4, settled by the owner on 2026-10-10: « `ca` is to read as a spelling of `ça` — a change to French's forms table (change 43's), proposed later as a change of its own » |
| The number | change 43's design reserved « 43b » for a split it never needed (`refine-lingua-french-forms-tables`); 43 merged whole, so the row is this change's |

What the French section says of the word, read in the pinned `kaikki-French.jsonl` (sha256
`2d7bbe5f…`, the pin's):

| Entry | Part of speech | Senses |
|---|---|---|
| `CA` | noun, plural `CAs` | « initialism of conseil d'administration », « … of chiffre d'affaires », « … of courant alternatif ("AC") », « … of comptable agréé » (Canada) — each tagged abbreviation, initialism, alt-of |
| `ca` | preposition | « abbreviation of circa » |
| `ça` | pronoun; noun | thirteen pronoun senses (« that », « this », « it », Louisiana's « he, she, it »…); the noun « id » |
| `ç'` | — | read by `ELISIONS` as *ça* |

No entry says `ca` spells `ça`. Both `ca` entries are lemmas, lowercased into one: `ca` is its own
candidate, and `CAs`, lowercased, links `cas` to it (`cas` keeps *cas* by GSD's counts).

## Goals / Non-Goals

**Goals:**
- `ca` read as `ça` in French's tables, as the owner decided, by a rule a person reviews row by row.
- What `ca`'s own words become decided and measured — the acronym, circa, the rank, the level, the
  dictionary word, the gloss — and what every table, pin, golden and snapshot shows moving.
- Every other unmarked spelling measured and listed for the owner, none read alike without a decision.

**Non-Goals:**
- Any byte of en-fr, es-fr, es-en or en-es, and any shared or edition module.
- A core rule: case-sensitive lookup, an acronym in capitals read apart, accents restored on capitals
  (change 41's D2). French's analyser version does not move.
- Other unmarked spellings (D6, open questions 2–4).
- fr-en's colliding `ça va` and `ça ira` keys (D5, *Found while measuring*).

## Measured

A prototype in the scratchpad (never committed): `reduce-fr-en.py` of `main` with the table of D1
behind a switch, its inputs the pinned release's `kaikki-French.jsonl` (sha256 `2d7bbe5f…`), GSD's
training and development sections at the pinned commit (sha256 `4b9a87b1…`, `9221e508…`, the pin's)
and wordfreq 3.1.1. Switch off, it writes `forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`,
`lexical.tsv`, `gloss.tsv`, `senses.tsv`, `mwe.tsv` and `NOTICE` byte for byte as committed; fr-es's
reducer of `main` over its pinned sources writes `tables/fr-es/` byte for byte, and the packs built
from both have their pins' sha256. The goldens were re-blessed in a scratch copy of `main` over the
prototype's tables, then once more with D5's three phrase probes on `main`'s tables and on the
prototype's.

## Decisions

### D1 — A reviewed table of spellings, read as the elided pieces are

`reduce-fr-en.py` gains `UNMARKED_SPELLINGS`, form → (word, reason), beside `ELISIONS`, with one row:

```python
UNMARKED_SPELLINGS = {
    "ca": ("ça", "ça written without its cedilla (the owner's decision of 2026-10-10): UD's treebanks "
                 "read every « ca » and « Ca » as ça; the section's own entries — CA's initialisms, "
                 "ca « abbreviation of circa » — leave the tables"),
}
```

`Lexicon.read` reads a row's form as it reads an elided piece: the named word is its one candidate,
and the entry returns before anything else is read, so neither `CA` nor `ca` makes it a lemma, links
it to another word or lists an inflection (`CAs` no longer links `cas` to it); `_inflections` skips
the form wherever another entry lists it, as it skips a piece. The rest follows from change 43's
rules, unchanged: the form's first choice is the named word, so it is no rank (D6 of change 43); it
is attested by wordfreq (Zipf 5.60), so it is a form; `forms.tsv` maps `ca` to *ça*. `read_readings`
already reads neither entry (a capitalised headword; a preposition, no part of speech it keeps), so no
reading moves. Editing the table changes the rules' sha256, as for any rule; its test holds that every
row has a reason, that its word is a ranked lemma reading as itself, and that its form is not one of
the elided pieces.

*Rejected — an `OVERRIDES` row `ca` → `ça`.* `choose_lemma` applies a row among a form's candidates
only (« An override naming a lemma the form cannot have is no decision », its test), and no entry
links `ca` to `ça`; making it a candidate would still read `CA`'s and circa's entries, keep `ca` a
lemma for the first choice, and leave `CAs` linking `cas` to it. An override chooses between what the
dictionary offers; this row says what the dictionary does not.
*Rejected — change 43's spelling rule (D7).* It reads what an entry says it spells; no entry says it
of `ca` (D6 measures the entries that do say it of other words).
*Rejected — a core rule* (an unaccented word retried with its marks, as Spanish's cascade retries a
retired accent). Change 41 measured a capital read without its accent and rejected it (its D2); a
cascade rule would bump French's analyser version for a word the tables can name.

### D2 — `ca`'s own words are lost, in capitals too

The row makes `ca` a form of *ça* alone, so the four initialisms of `CA` and circa leave the tables:
no rank, no level, no dictionary word, no gloss in fr-en (fr-es glossed neither). « board of
directors » leaves the pack, and so does the vocabulary's word.

**`CA` in capitals reads as `ça` too.** The pack keys its forms in lower case, and French's cascade
looks every token up lowercased (change 41): `CA`, `Ca` and `ca` are one form, and no forms table can
read them apart. Measured (D3), the case would tell them: every `ca` and `Ca` that is a French word is
`ça`, every `CA` the acronym. Keeping the acronym would take a core rule reading a word in capitals
apart — French's analyser, and every acronym it meets (`SNCF`, `ONU`, `UE`), not `CA` alone —, which
this change does not propose (open question 1). What it costs, on the corpora: UD's three `CA`, sports
clubs (« CA Vitry », « CA Peñarol », « CA Montreuil »), for which « board of directors » was as wrong
as `ça` is; the Wikipedia sample's five, four of them Geneva's Conseil administratif, for which it was
right. The names rule (change 41) sets aside neither: a capital in mid-sentence is set aside only when
its lemma is no dictionary word, and *ca* was one as *ça* is. The phrase probe « Le CA a voté le
budget. » records the cost (D5).

*Rejected — keeping a lemma `ca` for the acronym beside the form.* A lemma is keyed by its own form
(change 43's D6, the builder's `id_of`): a form `ca` reading as *ça* and a lemma `ca` cannot both be,
the second losing its rank, gloss and level to the first (« one form, one lemma », M8).

### D3 — `ca` in the corpora

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

The non-words read today as *ca* « board of directors » and will read as *ça*: wrong both ways, as
any language code or address the tokeniser takes for a word.

### D4 — What moves in the tables, the pins and the measurements

| File | Before → after | What |
|---|---|---|
| `tables/fr/forms.tsv` | 124,096 → 124,097 rows | `ca` → *ça* (was *ca*); `cussac` → *cussac* enters with its rank |
| `tables/fr/freq.tsv` | 60,000 rows | `ca` (144) out; every rank after it up by one (59,853 rows); `cussac` in at 59,997, before the three nouns ending in a pronoun GSD never meets |
| `tables/fr/grammar.tsv`, `tags.tsv`, `studied.json` | byte for byte | no entry gave `ca` a reading |
| `tables/fr/lexical.tsv` | 26,486 → 26,485 words | `ca` out; `cussac` is glossed as a name alone, no dictionary word |
| `tables/fr/level.tsv` | 8,302 rows, English's sizes | `ca` (A1) out; one word up at each boundary: `croissance` A2 → A1, `communiste` B1 → A2, `australien` B2 → B1, `naïveté` C1 → B2, `morphologie` C2 → C1; `crabe` gains C2. The spans in ranks stay (A1 1–1,085 … C2 9,475–10,820; C1 from 8,141) |
| `tables/fr-en/gloss.tsv`, `senses.tsv` | 30,067 rows | `ca` « board of directors » (`NOUN:1`) out, `cussac` « a surname from Occitan » (`PROPN:1`) in |
| `tables/fr-en/mwe.tsv` | byte for byte | no headword holds the word `ca`, so no key moves |
| fr-en's coverage | 93.6 / 86.9 / 76.3 % → the same | 4,679 / 8,688 / 15,260 → 4,679 / 8,688 / 15,259 of the 5,000 / 10,000 / 20,000 commonest |
| `tables/fr-en/pin.json`, `manifest.json` | `reducer` and `pack` move, `pack_version` with the rule digest | `snapshot` and `sources` byte for byte |
| `tables/fr-es/gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE` | byte for byte | fr-es glossed no `ca` |
| fr-es's coverage | 83.2 / 70.8 / 56.8 % → 83.2 / 70.8 / 56.8 % | 4,161 / 7,082 / 11,359 → 4,161 / 7,083 / 11,359 (`digérer` enters the glossed top 10,000); against its floor 81.4 / 68.8 / 54.5 % |
| `tables/fr-es/pin.json`, `manifest.json` | `studied` (`forms.tsv`, `freq.tsv`, `level.tsv`, `lexical.tsv`) and `pack` move, `pack_version`'s studied digest with them | its reducer digest and sources byte for byte |
| The packs (today's manifests) | fr-en 2,537,386 → 2,537,284 B; fr-es 1,973,407 → 1,973,085 B | the manifests' new `pack_version` adds or takes a byte or so |
| UD French-PUD (gated), GSD's test section (reported) | 99.13 / 96.41 / 99.90 %; 98.89 / 95.86 / 99.72 % — unchanged | the three `ca`/`Ca` of PUD and GSD's test section are pronouns, which resolved before and are no content words |

`ça` keeps its rank (26: wordfreq ranks a lemma by its own frequency, not its forms'), its level (A1),
its readings and its glosses. A card opened on `ca` shows `ça`'s gloss and senses and no reading: no entry gives the form `ca`
one (`ça`'s own card shows the one reading the section gives it, the noun « id »'s).

**en-fr, es-fr, es-en and en-es cannot move.** The table lives in `reduce-fr-en.py`, which fr-en's
rule digest names and no other pair's does (`reduce-es-en.py`'s digest is it, `reduce_common.py` and
`reduce_edition_en.py`); `tables/en/`, `tables/es/` and their pairs' folders are not written; no core,
wasm or extension code changes. Over the prototype, the bless of every lingua-wasm test moved
`fr-en.golden` and `fr-es.golden` alone, and `cross_native.rs`, `parity.rs`, `card_gloss_language.rs`,
`deck.rs` and the other baselines passed as committed. `lingua-pack`'s committed-tables tests failed
only on fr-en's and fr-es's pins (their pack sha256 and fr-es's studied record), which the re-pin
records.

### D5 — The goldens and the snapshots

**Re-blessed on the prototype's tables, 9 of 213 probes move in each golden**, the same nine:

| Probe | fr-en | fr-es |
|---|---|---|
| `pack` | 2,537,386 → 2,537,284 B (and the new `pack_version`) | 1,973,407 → 1,973,085 B (and the new `pack_version`) |
| `analyse new-reader informel` | « ca » *ca* « board of directors » → *ça* « that (distal demonstrative pronoun)… »; its class (Unknown) and the page's count (37) unchanged | *ca*, no gloss → *ça* « Eso, esto, aquello » |
| `vocabulary-estimate new-reader` | universe 26,486 → 26,485 | the same |
| `vocabulary-estimate reader` | estimated 2,184 → 2,185, universe 26,485 → 26,484 | the same |
| `review-current first`, `second` | `croissance`, `célèbre` → `célèbre`, `davantage`: `croissance` is A1 now, so the deck seeded from A2's five commonest starts one word later | the same, in Spanish |
| `export-status-ops`, `export-card-ops`, `backup` | the seeded deck: `croissance` and `massacrer` out, `exposition` and `morphologie` in (C1's three rarest now end on `morphologie`) | the same |

No `gloss`, `word-grammar` or `phrase-gloss` probe moves, and the studied side of the two goldens stays
one (`the_golden_is_the_french_one_on_the_studied_side` passes).

**Three phrase probes are added** to the French scenario (`support/french.rs`, before « l’homme »), so
the goldens show the rule where a reader meets it. Blessed on `main`'s tables and on the prototype's:

| Probe | fr-en, today → this change | fr-es, today → this change |
|---|---|---|
| « comme ca » | *ca* « board of directors », no expression → *ça*, the expression `comme ça` « like that/this… » | *ca* unglossed → *ça*; fr-es has no `comme ça` |
| « c'est ca » | `c'est` over two tokens → `c'est ça` « Confirmation of a question: that's right » over three | no expression → `c'est ça` « Así es o tener la razón, eso es… » |
| « Le CA a voté le budget. » | *ca* « board of directors » → *ça* « that… », flagged a function word (the acronym's cost, D2) | *ca* unglossed → *ça* « Eso, esto, aquello » |

Each golden gains 6 lines. `test/word-card-fr-en.spec.ts` and `word-card-fr-es.spec.ts` render every
grammar and phrase probe of their golden: none of the probes they render before moves, so their
snapshots gain the three cards and nothing else, and their count of phrase probes goes 34 → 37.

*Rejected — a `gloss ca` probe.* A `gloss` probe asks a lemma's gloss, and `ca` is no lemma any more:
it would record nothing.

**Found while measuring, not this change's.** fr-en holds `ça va` (« how are you? how's it going? »)
and `ça ira` (« it will be all right »), both keyed *ça aller*, and the key answers `ça ira`: on `main`,
« ça va » already shows « it will be all right… », and « ca va » will. fr-es holds `ça va` alone. The
choice between headwords sharing a key is change 44's; this change only lets « ca va » reach it. It is
named in the pull request as a follow-up for the owner, and no probe of this change records it.

### D6 — What is not read alike, measured

wordfreq's French list holds 475 words of Zipf ≥ 3 that are another French word of the tables with
its accent, cedilla, diaeresis or circumflex dropped and less than half as frequent as it. By what the
tables make of them today:

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

| Spelling | Today | The marked word | Ratio | UD |
|---|---|---|---|---|
| `age` | rank 2,461, B1, « beam (central bar of a plough); shaft » | `âge` | 0.14 | 2 of 2 read as *âge* |
| `forcement` | rank 9,588, C2, « fixing number, cooking the books » | `forcément` | 0.06 | — |
| `foret`, `aine`, `envoye`, `genie`, `alle` | ranks 12,780–18,735, no level: « drill bit », « groin », « slowworm », a diminutive of Eugénie, « she » (North America) | `forêt`, `aîné`, `envoyé`, `génie`, `allé` | 0.02–0.11 | — |

Measured on the prototype with `age` → *âge* and `forcement` → *forcément* beside `ca`: two more
ranked lemmas, dictionary words and levels out (B1 and C2; `boue`, `parano`, `nantais` up a band,
`cuisinière` and `dar` gain C2), coverage unchanged, and their plurals `ages` and `forcements`, which
only their own entries listed, become no form unless a row names them too. Left to the owner (open
question 3).

**The dictionary's own unmarked spellings.** 186 entries of the section are only another word written
without its marks — « alternative spelling of État », « obsolete spelling of être » (Internet,
colloquial), « nonstandard spelling of Écosse » —; 51 are ranked: `etat` 490, `etats` 868, `etre`
1,015, `etats-unis` 1,355, `etait` 1,996, `arreter` 8,060, `siecle` 15,372… They are words of their
own today, glossed through 48b's pointer meanings (`etre` « to be… ») and levelless (46's rule 3). Change
43's D7 reads two kinds of the dictionary's spellings as forms of their word; reading this third kind
alike would be a rule on the dictionary's own word, not a person's row — but its direction needs a
condition: `diesel` « alternative spelling of diésel » is 78 times commoner than `diésel` (Zipf 4.05
against 2.16), `ego` than `égo` (3.87 / 3.22), and both are levelled (B1, B2). Left to the owner
(open question 2).

**Spellings no entry glosses.** 301 of the 475 carry no gloss: ranked today as unglossed words of
their own, or no form at all: `meme` 4.77, `tres` 4.52, `etait` 4.42, `apres` 4.33,
`ecole` 4.21, `deja` 4.06, and the cedilla's: `francais` (rank 3,740), `facon` (7,296), `francaise`
(9,145), `francois` (10,513), `garcon` (13,328), `recu` (13,564). A reader meets each as an unknown word
with no gloss. A rule reading them by their marks would be blind — English words (`video`, `hotel`,
`education`, `general`, `these` → *thèse*), names (`jesus`, `noel`, `grace`, `israel`), capitals
French sometimes writes unaccented (`Etat`, `Eglise`: change 41's D2) —; rows by name would not.
Left to the owner (open question 4).

**This change reads `ca` alone**: the owner's decision, and the one unmarked spelling that is a
dictionary word, levelled A1 and ranked among the 1,000 commonest.

## Risks / Trade-offs

- **[The acronym is lost, in capitals too]** → measured (D3): 3 of UD's 9 `ca`-forms and 5 in the
  Wikipedia sample, against `ça` everywhere a reader writes informally; the phrase probe records it;
  a core rule is the owner's question (1).
- **[A language code or a web address reads as `ça`]** → it read as « board of directors » before:
  wrong both ways, as any non-word the tokeniser keeps.
- **[One row hides the others]** → D6 lists them with their figures; each further row is the owner's.
- **[Change 49b re-reduces fr-es]** (`refine-lingua-fr-es-glosses`, proposed) → whichever of 49b and
  this change lands second re-reduces fr-es on both and re-blesses `fr-es.golden`; neither edits the
  other's files.
- **[Change 52 ships French before this lands]** → then a reader may mark `ca` known as « board of
  directors »; this change is listed before 52's release in the programme's row, as 48b and 49b are.
- **[wordfreq's next version]** → the row does not depend on frequencies; an update re-reduces with
  it and the test holds the row's word ranked.

## Migration Plan

Nothing to migrate: no package lists a French pair, no reader holds a status, card or level for `ca`.
Rollback is a revert of the reducer's row, the re-reduced tables, the pins and the re-blessed files.

## Effort

0.75–1.25 ideal days, outside the programme's estimates: the table and its tests 0.25, the
re-reduction of fr-en and fr-es with their pins, READMEs and `SOURCES.md` 0.25–0.5, the probes, the
two re-blesses and the snapshots 0.25, the committed-tables test and the gates 0.25.

## Open Questions

For the owner — the first blocks nothing, the others are follow-ups:
1. **Keep `CA` in capitals?** The tables cannot: the pack has no case. A later core change could
   read a word written in capitals apart when its lower case is an unmarked spelling — French's
   analyser, a version bump. Example: « Le CA a voté le budget » shows `ça` « that » with this change,
   « board of directors » before. Recommended: no — 3 club names in UD's 390,368 words, 5 in the
   Wikipedia sample, and `CA` also means turnover, alternating current and chartered accountant, so the
   old gloss was often wrong too.
2. **The dictionary's own unmarked spellings** (`etre` « obsolete spelling of être », `etat`, `etait`,
   `arreter`, `siecle`: 51 ranked): read them as their word, by a rule on what the dictionary says,
   only when the marked word is the commoner (so that `diesel` stays `diesel`)? Example: « etre »
   would show *être*'s card and count as *être* in the reader's vocabulary instead of a second word.
   Recommended: yes, as a change of its own, measured like this one.
3. **Rare words an unmarked spelling swamps** (`age` « beam of a plough », B1; `forcement` « fixing
   number », C2): name them in `UNMARKED_SPELLINGS` like `ca`, each with its plural? Example: « à mon
   age » shows « beam (central bar of a plough) » today. Recommended: `age` and `forcement`, if the
   owner agrees, in that change; never `cote`, `tache`, `pale`, `foret`, `aine`, which are words a
   reader can mean.
4. **Spellings no entry knows**, the cedilla's first (`francais`, `facon`, `garcon`, `recu`) and the
   commonest accent-less ones (`meme`, `tres`, `apres`, `deja`): rows by name, or nothing? Example:
   « c'etait deja tres bien » shows three unknown words with no gloss. Recommended: rows by name for
   the cedilla's family and the ten commonest, measured in the same follow-up as 2; never a rule by
   marks alone (`video`, `hotel`, `these` would read as French words they are not).
