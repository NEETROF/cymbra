# Design — read-lingua-french-dictionary-unmarked-spellings

## Context

See proposal.md (Why). What exists, on `main` at `8ef47870`, change 43b's implementation (#884)
merged — the tables this change starts from:

| Where | What |
|---|---|
| `reduce-fr-en.py` | French's reference reducer (change 43). Its spelling rule (D7), `spelling_target`, reads two kinds of an entry's sense as « this entry only spells another word »: a post-1990 spelling (`connait`, « post-1990 spelling of connaît ») and an ASCII spelling of a ligature (`coeur`, whose `alt_of` is *cœur*). `Lexicon.read` makes an entry every sense of which only spells another word a form of that word, its inflections forms of it; `read_readings` files such an entry's readings under the word it spells (`coeurs` → *cœur*). 43b's reviewed table `UNMARKED_SPELLINGS` (`ca` → *ça*, `age`/`ages` → *âge*, `forcement` → *forcément*) reads four spellings no entry links to their word, as `ELISIONS` reads an elided piece. Level rule 3 (change 46) gives no level to a lemma every sense of which only spells another word (`etre`, `parceque`): 33 ranked lemmas left out |
| `tables/fr/` | 124,101 forms, 60,000 ranked lemmas, 125,177 readings, 26,484 dictionary words, 8,302 levels (A1 1–1,085 … C2 9,475–10,820); `etat` → *etat* (rank 489, a dictionary word, no level), `etre` → *etre* (1,014), `etait` → *etait* (1,995, no gloss), `diesel` → *diesel* (3,855, B1) |
| `tables/fr-en/` | `etat` « state, condition; state, nation; government », `etre` « to be; to be located… » (48b's pointer meanings), `siecle` « century (period of 100 years) »; coverage 93.6 / 86.9 / 76.3 % (4,679 / 8,688 / 15,259) against `FLOORS["fr-en"]` 91.9 / 85.1 / 74.4; the pack 2,537,588 B; rule digest `reduce-fr-en.py`, `reduce_common.py`, `reduce_edition_en.py`, `reduce_french_treebank.py` |
| `tables/fr-es/` | no gloss for any of the dictionary's accent-less spellings but `uberisation` « Uberización »; coverage 83.0 / 70.7 / 56.7 % (4,150 / 7,067 / 11,342) against 81.4 / 68.8 / 54.5; the pack 1,971,114 B; its pin records `tables/fr/`'s six files by sha256 (`studied`) |
| Change 43b | its D6 counted « 186 entries only spelling another word without its marks, 51 ranked » and its Q2, settled by the owner on 2026-10-10: « read as their accented word when it is the commoner, in a change of its own » — this one, the example name its own |
| Change 41's cascade | `french::lemmatize` looks every token up lowercased and in NFC: « Etat », « ETAT » and « etat » are one form to the pack. Its D2 rejected a capital read without its accent as a cascade rule |

What the French section says, read in the pinned `kaikki-French.jsonl` (sha256 `2d7bbe5f…`):

| Entry | Part of speech | Its one sense |
|---|---|---|
| `etre` | verb | « obsolete spelling of être » (Internet, colloquial, obsolete; `alt_of` *être*) |
| `Etat` | noun, plural `Etats` | « alternative spelling of État » |
| `etait` | verb | « obsolete spelling of était » (a form of *être*) |
| `siecle` | noun, plural `siecles` | « obsolete form of siècle » |
| `repondre` | verb, its whole conjugation (`repond`, `reponds`, `repondu` …) | « obsolete spelling of répondre » |
| `diesel` | noun, plural `diesels` | « alternative form of diésel » |
| `teter` | verb; `tete` « inflection of teter » | « alternative form of téter » |
| `surs` | adjective (two entries) | « masculine plural of sur » (« sour »); « alternative spelling of sûrs » |

## Goals / Non-Goals

**Goals:**
- The dictionary's own accent-less spellings read as their accented word when wordfreq rates it the
  commoner, as the owner decided — by change 43's spelling rule, which already reads what the
  dictionary says a word spells.
- Every candidate listed with its fate (D6), what each read spelling's own word becomes (D3), and what
  every table, pin, golden and snapshot shows moving (D4, D5).

**Non-Goals:**
- Any byte of en-fr, es-fr, es-en or en-es, and any shared or edition module.
- The web's spellings the dictionary does not name (`tete`, `francais`, `tres`, `deja`): the reviewed
  list by name the owner settled as a change of its own (43b's D6).
- 43b's table: no row added or removed; `cote`, `tache`, `pale`, `foret`, `aine`, `ou`, `a`, `la`,
  `des`, `du`, `sur` are none of this rule's words.
- A core rule (change 41's D2 rejected a capital read without its accent); French's analyser version
  does not move.

## Measured

A prototype in the scratchpad (never committed): `reduce-fr-en.py` of `8ef47870` with D1's rule behind a
switch, its inputs the pinned release's `kaikki-French.jsonl` (sha256 `2d7bbe5f…`), GSD's training and
development sections at the pinned commit (sha256 `4b9a87b1…`, `9221e508…`) and wordfreq 3.1.1.
Switch off, it writes `forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`, `lexical.tsv`, `gloss.tsv`,
`senses.tsv`, `mwe.tsv`, `NOTICE` and `manifest.json` byte for byte as committed; fr-es's reducer over
its pinned sources writes `tables/fr-es/` byte for byte, and the packs built from both have their
pins' sha256. Four variants were measured: the rule read per entry, the rule read per word, the rule
per word with the forms guard (D2, the one proposed), and 43b's table mechanism with rows generated
from the same tags (D1, rejected). fr-es was reduced again on the proposed variant's `tables/fr/`.
The goldens and the word-card snapshots were re-blessed in scratch copies of the tree, with and
without D5's four phrase probes, on the committed tables and on the prototype's; UD French-PUD and
GSD's test section were measured on packs built from both.

## Decisions

### D1 — Change 43's spelling rule reads a third kind, per word

`reduce-fr-en.py` reads, before the forms, which words the section gives only as a marked word
written without its marks:

```python
def dictionary_unmarked_spellings(path, frequency):
    """`spelling → marked word`: each word every sense of every entry of which is an alternative,
    obsolete, archaic, nonstandard or misspelt spelling or form (`alt_of`) of one word that, written
    without its accents, cedillas, diaereses and circumflexes (`_unmarked`), is it — and which wordfreq
    rates commoner than the spelling and than each of the spelling's forms (D2)."""
```

`_unmarked` drops the combining marks of the word's NFD and reads `œ`/`æ` as `oe`/`ae`; the word and
its target are read as everywhere else (`nfc_lower`). A word's forms are the inflections its entries
list (less the bookkeeping, the doubtful ones and the markers `Lexicon` already skips) and the entries
a form-of sense of which points at it. `spelling_target(sense, word, unmarked)` gains the third kind:
the sense's `alt_of` target when `unmarked` names it for the word. `Lexicon.read` and `read_readings`
pass the table, so the rest is change 43's and 45's rules, unchanged: the spelling's entry is a form
of the marked word, its inflections link to it and follow it to the marked word's lemma along their
part of speech (`repond` → `repondre` → *répondre*; `etait` → *était* → *être*), a spelling is never
ranked (its first choice is another word), and its readings are filed under the marked word. A
spelling 43b's table names is read by the table first; none of the 81 is one. The reduction's stderr
line says how many spellings were read and kept.

**Per word, not per entry.** Change 43's rule reads each entry alone; read so, an entry that only
spells beside another entry of the same word makes the word a form of both, and `choose_lemma`'s
frequency tie-break picks the commoner lemma. Measured, per entry moves three more forms: `surs` and
`sures` (*sur* « sour »'s plural forms, and `sûrs`, `sûres` without their circumflex) from their own
« sour » to *sur* « on, upon », and `entrainant` (the participle of the post-1990 `entrainer`) to
*entraîner*, losing `entrainante(s)`. Read per word, a word with an entry of its own keeps the
reading change 43's rules give it — `sure`, `sures`, `surs`, `entrainant`, `chouia` (« brochette »),
`colon` (« colonist »), `mercedes`, `to` — and the web's reviewed list (D6) can name them.

*Rejected — 43b's table mechanism, rows generated from the same tags.* A row makes its form a form
of a ranked lemma alone and drops the form's entries whole (D1 of 43b): measured with the 64 rows
whose word is a ranked lemma, the spellings' inflections and the entries pointing at them are no
longer read, and 20 of them become ranked words of their own with no gloss — `tete` (5,532),
`egyptiens` (15,256), `repond` (23,193), `reponds`, `repondu`, `siecles`, `matieres`, `methodes`,
`icones`, `criteres` — where the rule reads them as their word; a row must name a ranked lemma, so
`etait`'s would name *être*, not the *était* the dictionary names, and the 18 spellings whose word is
no ranked lemma would have none. A generated table also has to be
generated again at each dictionary update, which the rule reads as it is. 43b's table stays the place
for spellings the dictionary does not link to their word.
*Rejected — a table written by hand.* The owner chose the dictionary's own statement and wordfreq's
verdict as the rule; the 81 rows would be a copy of what the section says. The rows a reduction
moves are reviewed in its diff (`forms.tsv`), as every rule's are.
*Rejected — a core rule* (an unknown unaccented word retried with marks): change 41's D2; it would
bump French's analyser version for words the tables can name.

### D2 — The frequency condition: the marked word is the commoner, than the spelling and its forms

The marked word, as the section names it (`alt_of`, lowercased and in NFC: `État` → `état`), must
have a strictly higher wordfreq Zipf frequency than the spelling — the owner's words, « only when the
accented word is the more common one » — and than each of the spelling's forms. wordfreq is the one
the reducer already reads (3.1.1, the pin's), through the same `frequency` function; no margin.

The second half catches one word, measured over the 85 spellings the first half accepts: `teter`
« alternative form of téter » (Zipf 1.56 against 2.52), whose entries list `tete` (3.83) and `tète`
(2.97) — `tete` being how the web writes *tête*, « head ». Without it `tete`, `tetes` and `tetons`
would read as *téter* « to suckle » (they are no form today). With it `teter` and its forms read as
today, and `tete` waits for the web's list (D6).

Near the line, as wordfreq rates them: read — `uberisation` 2.31 / 2.39, `cameraman` 2.90 / 2.99,
`etats-unis` 4.70 / 4.92, `haiku` 2.25 / 2.55; kept — `impresario` 2.47 / 2.45, `ethiopie` 3.27 /
3.24, `egypte` 4.13 / 4.10, `angelus` 2.68 / 2.54, `etienne` 4.15 / 3.95, `eve` 3.73 / 3.44. Far from
it: `etre` 4.77 / 6.38 (41 times), `etait` (65 times), `siecle` (110 times); `diesel` 4.05 / 2.16, 78
times commoner than `diésel`, `ego` 3.87 / 3.22.

29 spellings wordfreq attests in neither form (Zipf 0 against 0) are no form either way: neither is
commoner.

### D3 — What a read spelling's own word becomes: nothing of its own

The spelling's entries make it no lemma: no rank, no level, no dictionary word, no gloss in fr-en
(fr-es glossed one, `uberisation`). A reader who opens it sees its marked word's card. Of the 33
ranked:

| Spelling | Today | Reads as |
|---|---|---|
| `etat`, `etats` | 489 and 867; `etat` a dictionary word « state, condition… », no level; `etats` unglossed | *état* (140, A1) |
| `etre`, `etait`, `etaient` | 1,014 « to be… » (a pointer meaning, 48b), 1,995 and 9,434 unglossed; no level | *être* (36, A1) |
| `etats-unis`, `ecosse`, `chloe`, `equateur` | names, 1,354 … 14,881 | *états-unis*, *écosse*, *chloé*, *équateur* (« Ecuador…; equator… », C1) |
| `benoit` | 5,557, **B2**, « blessed, fortunate; sanctimonious… » | *benoît* (2,684, B1), whose gloss opens with « a male given name… », then the same senses |
| `derniere`, `arreter`, `repondre`, `siecle`, `matiere`, `methode`, `colere`, `cooperation`, `scenario`, `referendum`, `rebellion`, `ecossais`, `egyptien`, `icone`, `electro`, `poele`, `gacher`, `experimental`, `cameraman`, `haiku`, `pecho`, `comparaitre`, `uberisation` | 7,841 … 52,474, no level; all but `derniere` glossed, with their word's senses or some of them (`ecossais` lacks « Scottish ») | their marked word (*dernier* A1 … *gâcher* B2, *ubérisation* unlevelled) |

Only `benoit` had a level; 25 dictionary words leave French's dictionary words; fr-en's 29 glosses of
the 33 leave its tables. **No form loses its fr-en gloss**: each marked word has one, and 34 forms
that had none gain it (`etait`, `etaient`, `etats`, `derniere`, and 30 newly read). **fr-es** gains a
gloss on 85 forms (`etat` « Estado », `etre` « Ser; (être + participio) Haber; Estar », `siecle`
« Siglo », `repondre` « Contestar, responder ») and loses one: `uberisation` « Uberización », whose
word *ubérisation* fr-es does not gloss (Q2).

The marked words keep their levels, readings and glosses, and their ranks but for the shift of every
rank after a removed spelling (*répondre* 591 → 590, *arrêter* 526 → 525; *être*, *état* and
*siècle* unmoved).

One form changes word that read as a correctly spelt word: `rebellions`, the imperfect of the verb
*se rebeller* (« nous nous rebellions »), is also `rebellion`'s plural; the noun's entry now reaches
*rébellion*, and change 43's choice among a form's words (M8, « one form, one lemma ») takes the
commoner, *rébellion* (Zipf 3.91) (Q1).

### D4 — What moves in the tables, the pins and the measurements

| File | Before → after | What |
|---|---|---|
| `tables/fr/forms.tsv` | 124,101 → 124,184 rows | 100 forms now read as a marked word: 69 that read as one of the 33 spellings, 30 that were no form (`critere`, `criteres`, `babord`, `pylone`, `melancolie`, `chome` …), and `rebellions` (*rebeller* → *rébellion*); 53 forms of the 33 lemmas entering the cut; `désossé`, a lemma entering it, reads as itself, no longer as *désosser* |
| `tables/fr/freq.tsv` | 60,000 rows | the 33 spellings out; every rank after each up by one to 33 (59,476 rows); `dagon` … `eamon` in at 59,965–59,997 (wordfreq's next words at the cut's frequency: `dashboard`, `depth`, `déboîter`, `désarçonner`, `dînette` …), before the three nouns ending in a pronoun |
| `tables/fr/grammar.tsv` | 125,177 → 125,303 rows | out: the 28 readings filed under the spellings' own words, `rebellions`' as *rebeller*'s and `désossé`'s as *désosser*'s; in: the moved forms' readings under their marked words (`siecles` under *siècle*, fourteen of *répondre*'s, nine of *gâcher*'s) and the entering lemmas' |
| `tables/fr/tags.tsv`, `studied.json` | byte for byte | every new reading's tag is in the pinned pool (the packs build) |
| `tables/fr/lexical.tsv` | 26,484 → 26,471 words | out: `arreter`, `benoit`, `cameraman`, `colere`, `comparaitre`, `cooperation`, `ecossais`, `egyptien`, `electro`, `etat`, `etre`, `experimental`, `gacher`, `haiku`, `icone`, `matiere`, `methode`, `pecho`, `poele`, `rebellion`, `referendum`, `repondre`, `scenario`, `siecle`, `uberisation`; in: `dilapidation`, `doctrinaire`, `drette`, `débaptiser`, `déboîter`, `décalque`, `dég`, `déjection`, `déplétion`, `désarçonner`, `désossé`, `dînette` |
| `tables/fr/level.tsv` | 8,302 rows, English's sizes | `benoit` (B2) out; `patriotique` C1 → B2, `opter` C2 → C1, `digestif` into C2; the spans A1 1–1,082, A2 1,083–2,380, B1 2,381–4,827, B2 4,831–8,132, C1 8,134–9,465, C2 9,466–10,812; rule 3 (« spelling ») leaves out 26 instead of 33 (`parceque`, `okay`, `hazard`, `isle` …) |
| `tables/fr-en/gloss.tsv`, `senses.tsv` | 30,067 → 30,050 rows | out: the 25 dictionary words above and four names (`chloe`, `ecosse`, `equateur`, `etats-unis`); in: the 12 entering dictionary words |
| `tables/fr-en/mwe.tsv` | byte for byte (17,635) | no headword is gained or lost; a key is the builder's (change 44), so `il était une fois` is met through the new forms |
| fr-en's coverage | 93.6 / 86.9 / 76.3 % → the same | 4,679 / 8,688 / 15,259 → 4,681 / 8,688 / 15,254 of the 5,000 / 10,000 / 20,000 commonest |
| `tables/fr-en/pin.json`, `manifest.json` | `reducer` and `pack` move, `pack_version` with the rule digest | `snapshot` and `sources` byte for byte |
| `tables/fr-es/gloss.tsv`, `senses.tsv` | 19,023 → 19,026 rows | every other row byte for byte; out `uberisation` « Uberización »; in `doctrinaire` « Doctrinario », `débaptiser` « Desbautizar », `déjection` « Deyección », `désarçonner` « Desarzonar, desconcertar, confundir » |
| `tables/fr-es/mwe.tsv`, `NOTICE` | byte for byte (12,174) | |
| fr-es's coverage | 83.0 / 70.7 / 56.7 % → 83.1 / 70.7 / 56.7 % | 4,150 / 7,067 / 11,342 → 4,153 / 7,071 / 11,347; floor 81.4 / 68.8 / 54.5 % |
| `tables/fr-es/pin.json`, `manifest.json` | `studied` (`forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`, `lexical.tsv`) and `pack` move, `pack_version`'s studied digest with them | its reducer digest and sources byte for byte |
| The packs | fr-en 2,537,588 → 2,537,640 B; fr-es 1,971,114 → 1,971,811 B | measured with today's manifests; the new `pack_version`s add a byte or none |
| UD French-PUD (gated), GSD's test section (reported) | 99.13 / 96.41 / 99.90 % → 99.13 / 96.36 / 99.90 %; 98.89 / 95.86 / 99.72 % → 98.91 / 95.94 / 99.72 % | PUD writes « Etats-Unis » four times and lemmatises it as written, which *états-unis* no longer matches (9,229 → 9,225 of 9,573 content words); GSD lemmatises « Etat » (twice) and « Etats » as *état*, now matched, and `dilapidation` resolves. The gates (98.5 / 93.5 / 97) hold |

**en-fr, es-fr, es-en and en-es cannot move.** The rule lives in `reduce-fr-en.py`, which fr-en's rule
digest names and no other pair's does; `tables/en/`, `tables/es/` and their pairs' folders are not
written; no core, wasm or extension code changes. Over the prototype, `english_baseline`,
`spanish_baseline`, `es_en_baseline`, `en_es_baseline`, `cross_native`, `parity`,
`card_gloss_language`, `deck`, `languages`, `spanish_expression_keys`, `statuses`, `soft_hyphens` and
`reprofile` passed without re-blessing, and `row-gloss-tables.spec.ts` without `-u`.
`lingua-pack`'s committed-tables tests failed only on fr-en's and fr-es's pins (their pack sha256 and
fr-es's studied record), which the re-pin records, and on the levels test's example of a spelling,
`etre`, no longer a ranked lemma: `parceque` takes its place (change 46's scenario says `etre` has no
level, which stays true).

### D5 — The goldens and the snapshots

**Re-blessed on the prototype's tables, 6 of the 221 probes move in each golden**, the same six:

| Probe | fr-en | fr-es |
|---|---|---|
| `pack` | 2,537,588 → 2,537,640 B (and the new `pack_version`) | 1,971,114 → 1,971,811 B (and the new `pack_version`) |
| `analyse new-reader informel` | « c'etait »: *etait*, no gloss → *être* « to be; to be located… »; its class and the page's count unchanged | *etait*, no gloss → *être* « Ser; (être + participio) Haber; Estar… » |
| `vocabulary-estimate new-reader` | universe 26,484 → 26,471 | the same |
| `vocabulary-estimate reader` | estimated 2,185 → 2,183, universe 26,483 → 26,470 | the same |
| `export-card-ops`, `backup` | the seeded deck: `mono` out, `opter` in (`opter` is C1 now, one of C1's three rarest glossed) | `massacrer` out, `opter` in |

No `gloss`, `word-grammar`, `review-current` or existing `phrase-gloss` probe moves, and the studied
side of the two goldens stays one.

**Four phrase probes are added** to the French scenario (`support/french.rs`, after « à mon age »),
so the goldens show the rule where a reader meets it. Blessed on the committed tables and on the
prototype's:

| Probe | fr-en, today → this change | fr-es, today → this change |
|---|---|---|
| « il etait une fois » | *etait* unglossed, the expression `une fois` → *être*, the expression `il était une fois` « there was once…; once upon a time… » | *etait* unglossed, no expression → *être*, `il était une fois` « Érase una vez, había una vez » |
| « un coup d'Etat » | *etat* « state, condition… », no expression → *état*, `coup d'état` over three tokens | *etat* unglossed → *état* « Estado », `coup d'état` « Golpe de estado » |
| « Il faut etre patient. » | *etre* « to be… », not a function word → *être*, the function word | *etre* unglossed → *être* « Ser… » |
| « un moteur diesel » | *diesel* « diesel », unchanged: the frequency rule keeps it | *diesel*, the expression `moteur diesel` « Motor diésel… », unchanged |

« un coup d'Etat » reads a capital without its accent after an elided piece: *état* is a dictionary
word, so the names rule does not set it aside. Each golden gains the four probes: `fr-en.golden`
380,843 → 384,506 B, `fr-es.golden` 253,589 → 256,302 B (221 → 225 probes).
`test/word-card-fr-en.spec.ts` and `word-card-fr-es.spec.ts` render every grammar and phrase probe of
their golden: none they render before moves, so their snapshots gain the four cards and nothing else
(`word-card-fr-en.txt` 31,025 → 31,430 B, `word-card-fr-es.txt` 20,189 → 20,591 B), and their count of
phrase probes goes 42 → 46. Against the same cards blessed on today's tables, « il etait une fois »
opens the expression `il était une fois`, « un coup d'Etat » shows `coup d'état` as one row, and « Il
faut etre patient. » loses its `etre` row, *être* being a function word.

### D6 — Every candidate, by what happens to it

The owner's « about 186 » are, in the pinned section, **186 words every sense of every entry of which
spells another word without its marks: 141 accent-less spellings and 45 ASCII spellings of a
ligature**. Beside them, **13 words** have such a sense among others. By their fate:

| Group | Count | The words |
|---|---|---|
| **Read as their word, ranked today** | 33 | `etat` 489, `etats` 867, `etre` 1,014, `etats-unis` 1,354, `etait` 1,995, `benoit` 5,557, `ecosse` 7,074, `derniere` 7,841, `arreter` 8,058, `etaient` 9,434, `electro` 11,580, `chloe` 14,398, `equateur` 14,881, `repondre` 15,357, `siecle` 15,369, `scenario` 19,624, `rebellion` 20,678, `methode` 20,938, `matiere` 21,191, `cameraman` 21,329, `ecossais` 21,634, `referendum` 22,892, `colere` 23,346, `egyptien` 24,597, `icone` 26,516, `cooperation` 34,632, `poele` 36,146, `gacher` 36,473, `experimental` 37,343, `uberisation` 44,560, `pecho` 46,118, `haiku` 47,745, `comparaitre` 52,474 |
| Read as their word, a form not ranked | 21 | already a form: `ecossaise`, `complait`; a form now: `babord`, `phylloxera`, `ethiopien`, `critere`, `calisse`, `surcout`, `transparaitre`, `cezanne`, `camescope`, `pylone`, `isaie`, `melancolie`, `spermatozoide`, `murement`, `beche`, `michaelle`, `arithmetique`, `machicoulis`, `equatorien` |
| Read as their word, no form either way | 27 | wordfreq does not attest the spelling, or its word is not among the 60,000 (`chomer`, whose forms `chome`, `choma`, `chomé` do read as *chômer*; `geologique`, `reconsiderer`, `taboule`, `plait-il`, `zaouia`, `meson` …) |
| **Kept by the frequency rule, ranked** | 18 | `etienne` 3,229 (4.15 / 3.95), `egypte` 3,351 (4.13 / 4.10), `diesel` 3,855 B1 (4.05 / 2.16), `ego` 5,134 B2 (3.87 / 3.22), `eve` 6,390, `ethiopie` 12,940, `seb` 13,945, `benelux` 19,949, `cot` 20,820, `alena` 23,288, `pedigree` 23,790, `epernay` 24,909, `belize` 26,755, `angelus` 28,107, `impresario` 36,524 (2.47 / 2.45), `amha` 41,461, `djo` 51,266, `boro` 57,765 |
| Kept by the frequency rule, not ranked | 38 | 9 attested (`retrogaming`, `kafir`, `pomelo`, `iambe`, `bezef` …) and 29 wordfreq attests in neither spelling (`abigeat`, `tolkienien`, `ypreau` …) |
| Kept by the forms guard (D2) | 1 | `teter` (its form `tete` is *tête*'s) |
| No word form | 3 | « de main de maitre », « emirats arabes unis », « ltee. » (spaces, a period: the tokeniser never hands them whole) |
| Already read, change 43's D7 | 45 | ASCII spellings of a ligature: `coeur`, `oeuvre`, `soeur`, `boeuf`, `noeud`, `oeil`, `voeu`, `coup d'oeil` … |
| Not only a spelling (D1) | 13 | an entry of their own: `sure`, `sures`, `surs`, `entrainant`, `chouia`, `colon`, `mercedes`, `to`; another sense in the same entry: `-ide`, `cappe`, `cordeliere`, `grande ile`, `grande-ile` |

Of the 51 ranked, 33 are read and 18 kept. **Found while measuring**: wordfreq rates `tete` (3.83),
`tetes`, `francais` and many accent-less words the dictionary does not name as frequent web
spellings; the owner's reviewed list (43b's D6) is their place, and `sure`, `surs`, `sures` (→
*sûr*) and `entrainant` belong on it too.

## Risks / Trade-offs

- **[A rule, not a reviewed table]** → the rule reads only what the dictionary says, and only when
  wordfreq agrees; the owner reviews the 81 words and the moved rows before the merge (task 6.2), and
  every later update shows the rule's moves in its `forms.tsv` diff. The test pins the decisive words
  (`etre`, `etat`, `diesel`, `teter`, `surs`).
- **[A dictionary update adds an entry]** → a new accent-less spelling of a commoner word is read as
  its word at the next update, as change 43's other spelling kinds are; the guard keeps out a spelling
  whose form is commoner than its word.
- **[`rebellions` reads as the noun]** → change 43's choice of the commoner word among a form's
  words; an override row can keep the verb (Q1).
- **[`uberisation` loses its Spanish gloss]** → one word at rank 40,429; fr-es can lend it later
  (Q2).
- **[PUD's content lemmas 96.41 → 96.36 %]** → four « Etats-Unis » PUD lemmatises as written; GSD,
  which the reduction reads, lemmatises « Etat » as *état*; the gate holds with three points to spare.
- **[43b, 43c or a fr-es change moves the tables first]** → this change is measured on 43b's tables;
  whichever lands second re-reduces on the committed `tables/fr/` and re-blesses `fr-en.golden` and
  `fr-es.golden`.
- **[wordfreq's next version]** → the reducer reads the pinned version; an update re-reduces with it
  and its diff shows any spelling that crosses the line.

## Migration Plan

Nothing to migrate: no package lists a French pair, no reader holds a status, card or level for a
spelling. Rollback is a revert of the rule, the re-reduced tables, the pins and the re-blessed files.

## Effort

1–1.5 ideal days, outside the programme's estimates: the pre-pass, the third kind and their tests
0.5, the re-reduction of fr-en and fr-es with their pins, READMEs and `SOURCES.md` 0.25–0.5, the
probes, the two re-blesses and the snapshots 0.25, the committed-tables test and the gates 0.25.

## Open questions for the owner

- **Q1 — « rebellions ».** Written so, it is the verb *se rebeller* correctly spelt (« nous nous
  rebellions ») and the plural of *rébellion* without its accent. The tables now read it as
  *rébellion*, the commoner word. Keep that (recommended), or add a reviewed override row so that it
  stays the verb's form?
- **Q2 — `uberisation` in fr-es.** It had a Spanish gloss, « Uberización »; read as *ubérisation*,
  which fr-es does not gloss, it has none. Accept (recommended: one rare word), or let fr-es lend a
  spelling's gloss to its word when the word has none?
- **Q3 — The words with an entry of their own.** `surs` and `sures` (« sour », and *sûrs*, *sûres*
  without their circumflex), `sure` (read as *sur*), `entrainant` stay as they are. Hand them to the
  web's reviewed list (recommended), or read them by this rule too (`surs` would then read as *sur*
  « on », not *sûr*)?
