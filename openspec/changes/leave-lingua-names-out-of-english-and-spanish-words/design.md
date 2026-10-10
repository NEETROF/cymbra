# Design — leave-lingua-names-out-of-english-and-spanish-words

## Context

See proposal.md (Why). What exists, on `main` at `f3585580`:

| Where | What |
|---|---|
| `tables/<lang>/studied.json` | English's reference pair is en-fr, Spanish's es-fr, French's fr-en. The reference's reduction writes the studied tables, and its glossed lemmas, as its reduction writes them, are the language's dictionary words |
| `reduce-fr-en.py` `dictionary_words` (48b D2) | French's dictionary words: fr-en's glossed lemmas less those every sense run of which is `PROPN`, written to `lexical.tsv` (30,055 → 26,486). The levels read them (48b D3) |
| `reduce-en-fr.py`, `reduce-es-fr.py` | Write no `lexical.tsv`. Their rule digests are `reduce-<pair>.py`, `reduce_common.py` and `reduce_edition_fr.py`, the French Wiktionary's module, which no other pair loads |
| `pack_sources.py split` (48b) | Files a reference's own `lexical.tsv` when its reducer wrote one, else writes the reference's glossed lemmas. In no rule digest |
| `crates/lingua-pack/src/tables.rs` `check_studied`, `names_only` (48b) | A studied folder's `lexical.tsv` must be the reference's glossed lemmas, or those less every one whose `senses.tsv` runs are all `PROPN`; anything else fails, naming the lemma |
| `reduce-es-fr.py` `estimated_levels` | Spanish's estimated levels go to the ranked lemmas es-fr glosses « that is not only a proper noun's » (`madrid` takes none): the same set this change takes out |
| `reduce-en-fr.py` `reduce_levels` | English's levels: CEFR-J and Octanove, read from the lists, not estimated |
| `engine.rs` `document_names` | The names rule: Spanish's (add-lingua-spanish-names) and French's (change 41, with two French readings). `analyse_page` runs it for Spanish and French; « English's analysis does not change ». English sets aside only an out-of-lexicon proper noun (`percent.rs`) |
| `Pack::dictionary_words` | The vocabulary estimate's universe and a CEFR ladder's typical vocabularies: the ranked lemmas that are dictionary words or carry a level |
| `knowledge/vocabulary.rs` `ENGLISH_TYPICAL_VOCABULARY` | `[0, 1_292, 3_359, 7_988, 16_326, 20_556]`, frozen on 2026-10-07 from en-fr's pack. A ladder of estimated levels (Spanish, French) borrows it, and English's own ladder computes its own figures (generalise-lingua-native-language D6) |
| `ANALYZER_VERSION` | English `1.2.0` since `ignore-lingua-soft-hyphens` (42d), Spanish `1.5.0`, French `1.2.0` |
| The goldens | `en-fr.golden` (S0, 142 probes), `en-es.golden` (182), `es-fr.golden` (137, with a `beside en-fr` line), `es-en.golden` (176), `fr-en.golden` (213, `beside es-en`), `fr-es.golden` (213, `beside en-es`) |
| The shipped pairs | `packs.json` lists en-fr and es-fr. es-en and en-es are committed and pinned, and ship with changes 34 and 35 |

## Goals / Non-Goals

**Goals:**
- English's and Spanish's dictionary words are words: no lemma their references gloss as a name alone.
  This is the rule French follows since 48b, and its home is the reference's reduction.
- What a reader sees follows: the names are set aside in Spanish pages (its rule exists) and in English
  pages (its rule is added), and the vocabulary figures stop counting them.
- Every move measured and named: tables, pins, packs, goldens, fixtures, tests.

**Non-Goals:**
- Any gloss, sense, expression, form, rank, level, reading or tag. Each lemma keeps its gloss and a
  selection opens its card.
- French: its dictionary words, levels, analysis and version are 48b's and change 41's. fr-en and
  fr-es tables, pins and packs stay byte for byte.
- A capital read by the card (23b's Q3): a common word whose row opens on a name (`jean`, `will`)
  keeps the page's order.
- New sources or a fetch beyond the pins. en-fr keeps its 2026-09-26 snapshot, es-fr its 2026-10-03.

## Measured

A prototype in the scratchpad, never committed, on a scratch checkout of `main` (`f3585580`):
- `tables/en/lexical.tsv` and `tables/es/lexical.tsv` rewritten by the rule over the committed
  `gloss.tsv` and `senses.tsv`. The rule reads nothing else, so a re-reduction writes the same bytes;
- English's names rule added to `engine.rs` with its two readings;
- the six goldens re-blessed, and every Lingua Rust test and the lingua-data Python suite run.

« Top 10k » means ranked 1–10,000 in the studied language's `freq.tsv`.

**The names-only lemmas**, by studied language and pair:

| Language (reference) | Glossed | Names only | Top 5k / 10k / 20k | Commonest |
|---|---|---|---|---|
| English (en-fr) | 24,799 | 3,046 | 388 / 993 / 1,868 | `york` 348, `london` 442, `america` 488, `june` 502, `april` 524, `july` 561, `al` 566, `september` 592, `david` 611, `england` 629, `uk` 640 |
| Spanish (es-fr) | 22,755 | 1,693 | 261 / 550 / 968 | `al` 16, `méxico` 149, `españa` 153, `madrid` 231, `juan` 253, `josé` 323, `venezuela` 342, `europa` 426, `colombia` 436, `carlos` 462 |
| French (fr-en, 48b) | 30,067 | 3,581 | 359 / 912 / 1,763 | `france` 63, `paris` 109: out since 48b, unchanged here |

The reader pairs do not decide anything (*A pack's dictionary words do not depend on its glosses*), but
they show the same names. en-es glosses 820 of English's 3,046 (545 of the top 10k), and 1,074 lemmas
by names alone of its own. es-en glosses 1,235 of Spanish's 1,693 (527), and 3,447 by names alone of
its own. fr-es's 583 of its own do not matter here.

**The dictionary words and the vocabulary estimate's universe:**

| Language | Dictionary words | Universe (ranked, a dictionary word or levelled) |
|---|---|---|
| English | 24,799 → 21,753 | 25,372 → 22,337 (the 11 levelled names still counted) |
| Spanish | 22,755 → 21,062 | 22,755 → 21,062 |
| French | 26,486, unchanged | 26,486, unchanged |

The reader pairs' READMEs follow. en-es: 2,501 → 3,321 lemmas it glosses that are no dictionary word,
and 5,336 → 3,110 dictionary words it does not gloss. es-en: 9,928 → 11,163 and 798 → 340.

**Levels.** English: 11 of the names-only lemmas carry a CEFR level. The eight months the French
Wiktionary files as proper nouns (`january`, `february`, `april`, `june`, `july`, `september`,
`november`, `december`; `may`, `march` and `august` have common senses) are A1, `olympics` A1,
`badminton` A2 and `ph` B2. Spanish: none, since its estimated levels already skip them. French: none,
since 48b D3.

**What « a name » leaves out by mistake.** A lemma the French Wiktionary files only as a proper noun
may be a word a learner wants. Measured two ways:
- *Against the other edition*. Of English's 3,046, 124 (55 of the top 10k) have a common sense in
  en-es's runs, read from the Spanish Wiktionary. Of Spanish's 1,693, 168 (81) have one in es-en's,
  read from the English Wiktionary. Most are names with a rarer homograph (`toledo` « toilet »,
  `luis` « louis (coin) », `moisés` « bassinet »), acronyms (`usa`, `fbi`, `otan`) or the months.
  The words a learner may want:
  - **English, words English always capitalises**: `easter`, `halloween`, `islam`, `judaism`,
    `ramadan`, `passover`, `renaissance`, `hebrew`, `hindi`, `sanskrit`, `urdu`, `telugu`, `croatian`,
    `esperanto` (out of the 124 only for some of them);
  - **English, mis-tagged**: `unseen` « Inédit; Inaperçu », `goofy`, `saline`, `heartland`, `tong`,
    `butte`, `glover`;
  - **Spanish**: `cristo`, `biblia`, `islam`; mis-tagged `títere` « Marionnette », `tea` (the
    section's « Théia… », Spanish's word for a torch), `parejo`, `portillo`, `pal`, `quemada`; and `al`
    « Amérique latine » (rank 16), which the pre-pass splits into `a` + `el` before any lookup, so only
    the universe loses it.
- *In context*. The corpora of D4 (below): of the 2,587 English tokens the data rule sets aside, 133
  (39 lemmas) have such a lemma, nearly all names where they stand (`Australia` 15, `Victoria` 9,
  `Hollywood` 8). The rest are `Islam` 8, `Esperanto` 5, `Easter` 3, `Hebrew` 2, `Renaissance` 2,
  `Hindi`, `Farsi`, `Judaism` and `Yule`. Of the 3,376 Spanish tokens, 379 (78 lemmas) have such a
  lemma, nearly all names (`María` 50, `Luis` 45, `Jorge` 18). The rest are `Cristo` 11, `Biblia` 7
  and `Islam` 1. The rule sets a form aside only where the document never writes it in lowercase, so
  `pisa`, `toledo` or `cortés` written in lowercase stay words.

**English's names rule** (D4), on two texts:
- *The English baseline* (`pages.txt`, en-fr and en-es alike): `Ruiz` (`news`, no gloss), `Margaret`
  (`fiction`, « Marguerite, satellite naturel… »), `HTTP` (`technical`, no gloss) and `Sam`
  (`informal`, « Prénom masculin ou feminin ») are set aside: 99 → 98, 94 → 93, 71 → 70 and 60 → 59
  counted words, and the reader's `news` goes from 85 % to 86 %. `Tuesday`, `March`, `Dr.`, `Tom`,
  `Mum`, `Smith`, `OK`, `CHAPTER ONE` and the title « The Lighthouse at the End of the World » stay
  words. Without English's rule, none of the four moves: only the vocabulary figures do.
- *A corpus*: 10,000 quotations of the English Wiktionary's English entries, from the cached dump.
  They are grouped 25 to a document, 400 documents, 296,336 tokens. The rule sets aside 4,347 tokens
  (1.5 %):
  - 2,587 (977 lemmas) are the names this change takes out: `London` 57, `England` 53, `France` 31,
    `America` 30, `York` 30;
  - 1,760 (1,278 lemmas) have no en-fr gloss and were underlined with no card: `II` 14, `Kong` 12,
    `Hong` 11, `Tory` 11, `God's` 10, `NASA` 10, `Trump's` 10;
  - among those 1,760 are possessives of capitalised words (`God's`, `Earth's`, `Queen's`), Title Case
    words (`Prelate`, `Children's`) and six `That's`/`It's` in quotations that drop a sentence's period.
- *Without the readings* the same corpus loses 4,895 tokens:
  - the pronoun `I`'s contractions, 233 tokens: `I'm` 132, `I'd` 36, `I'll` 33, `I've` 30. On the
    baseline, `I'll` and `I'd`;
  - levelled lemmas, 315 tokens: `Mr` 115, `Mrs` 47, the months 123, `Madame` 9, `Mama` and `Mamma`
    5, `Olympics` 3, `Coalition` and `Participle` 2 each, and nine words met once (`Depression`,
    `Gaol`, `Sulphur`).

**Spanish's names rule on the new dictionary words** (its code unchanged):
- *The Spanish baseline* (`pages-es.txt`, es-fr and es-en alike): on `nombres`, `Ana`, `García`,
  `Montevideo`, `Pedro`, `Sevilla`, `María`, `Madrid`, `Fernández` and `Guadalquivir` are set aside,
  so the page counts 29 words instead of 38, and the B1 reader's page goes from 76 % to 100 %. `Miró`
  (read as *mirar*) and `Argentina` (a common adjective) stay words. No other page moves.
- *A corpus*: 10,000 quotations of the English Wiktionary's Spanish entries, 400 documents, 312,678
  tokens. Set aside by the names rule: 4,907 → 8,283 tokens. The 3,376 added are all names-only
  lemmas (639): `España` 146, `José` 106, `Madrid` 100, `Juan` 73, `Barcelona` 65. Counted words go
  from 305,421 to 302,045 (−1.1 %).

**The vocabulary figures** (D5), on the baselines' readers:

| | Before | After |
|---|---|---|
| English, new reader's universe | 25,372 | 22,337 |
| English, B1 reader's estimate (S0) | 3,363 | 3,078 |
| Spanish, new reader's universe | 22,755 | 21,062 |
| Spanish, B1 reader's estimate | 2,285 (of 22,754) | 2,176 (of 21,061) |
| English's ladder, typical vocabulary A1–C2 | 0 / 1,292 / 3,359 / 7,988 / 16,326 / 20,556 | 0 / 1,213 / 3,074 / 7,155 / 14,433 / 18,123 |

**The packs** (built from the prototype's tables):

| Pair | Before | After |
|---|---|---|
| en-fr | 1,835,638 B, no lexical table | 1,840,736 B (+5,098: a lexical table) |
| es-fr | 2,224,439 B, no lexical table | 2,231,951 B (+7,512) |
| en-es | 1,688,931 B | same size: its lexical table's bits move, and so does its `pack_version` |
| es-en | 2,608,413 B | same size, likewise |
| fr-en, fr-es | — | byte for byte |

The shipped package grows by about 12.6 KB before compression (en-fr and es-fr).

## Decisions

### D1 — One rule for the three studied languages: a name is what the reference's edition files as a proper noun

A studied language's dictionary words are its reference pair's glossed lemmas, less those every sense
run of which (`senses.tsv`) is `PROPN`. The runs are the reference's: the French Wiktionary's English
and Spanish sections for en-fr and es-fr. Their `PROPN` is the page's « Nom propre », kaikki's `name`
part of speech. This is 48b's D2 for French, word for word, and `check_studied` already holds it for
any language. A lemma with a common sense beside a name's stays a word: `bill`, `mark`, `will`, `hope`
in English, `luna`, `sol`, `don` in Spanish. The lemma keeps its gloss: a reader who selects `Madrid`
or `London` reads it, and so does one who meets it written in lowercase.

*Rejected: any sense that is a name's* (a lemma with one `PROPN` run, or a sense tagged `surname`
or `given-name`). That would take out `will`, `bill`, `mark` and `hope`, which are words.
*Rejected: another edition's senses to rescue a word* (the en-es or es-en runs). A language's
dictionary words would then depend on a reader pair's glosses, which *A pack's dictionary words do not
depend on its glosses* forbids, and es-en alone names 3,447 lemmas by names only. The edition's slips
are measured instead (*Measured*) and put to the owner (Q3).
*Rejected: a capitalised headword.* The committed tables key lemmas in lowercase; the rule must be
readable off them, so that a check can hold it.

### D2 — The rule lives in the French Wiktionary's module

`reduce_edition_fr.py` gains `dictionary_words(glosses, runs)`: the glossed lemmas less those every run
of which is `PROPN`, byte-sorted. `reduce-en-fr.py` and `reduce-es-fr.py` write its result to their work
folder as `lexical.tsv`, after `senses.tsv`. 48b's `split` files it into `tables/en/` and `tables/es/`.
48b's `check_studied` accepts it, so there is no Rust change in lingua-pack. The builder, given
dictionary words that differ from the glossed lemmas, writes en-fr's and es-fr's lexical tables, as it
writes fr-en's.

The module is in exactly en-fr's and es-fr's rule digests, so only these two pairs re-pin by rule.
es-fr's `estimated_levels` keeps its own test and is not edited: its level table does not move (D3).

*Rejected: `reduce_common.py`.* It is every pair's digest, so fr-en's, fr-es's, es-en's and en-es's
`pack_version` would move for nothing.
*Rejected: one module for the three references.* fr-en would load it, so its digest, pin, pack and
golden's pack line would move. fr-en keeps 48b's copy.
*Rejected: a copy in each reducer, as fr-en has one.* That is two more copies of three lines, tested
twice. The French Wiktionary's module serves exactly the two pairs, and its test is one.

### D3 — Levels are not touched

English's levels are CEFR-J's and Octanove's. A list that levels a lemma says it is a word to learn,
whatever the page's part of speech. The eleven levelled names-only lemmas (the eight months,
`olympics`, `badminton`, `ph`) keep their level. *A vocabulary size counts dictionary words* already
counts « dictionary words or levelled » lemmas, so the universe keeps them, and English's names rule
reads a levelled lemma as a word (D4). Spanish's estimated levels go only to lemmas es-fr glosses as
more than a name, the same set, so every Spanish levelled lemma stays a dictionary word.
`committed_tables.rs` checks that, and lists English's eleven. French: 48b. No `level.tsv` moves, and
change 49c's seeding (glossed lemmas) is unaffected: every levelled lemma keeps its gloss.

*Rejected: English's levelled names stay dictionary words* (the data rule less the levelled ones). The
check would need a third set (*Names left out by halves* would read `level.tsv`), and the names rule
would still set aside `Mr` and `Mrs`, levelled and never glossed by en-fr. The reading in D4 covers
both.

### D4 — English sets a document's names aside

`analyse_page` runs `document_names` for English too, as written for Spanish. A plain token (an
unlisted hyphenated run is judged by its parts, as in Spanish) whose form the document never writes in
lowercase, capitalises at least once in mid-sentence, and whose lemma is no dictionary word, is set
aside like an out-of-lexicon proper noun. Every occurrence is set aside, but only where the knowledge
model reads it `Unknown`, so a word the reader marked keeps its status. English adds two readings:

1. **The pronoun I gives no evidence.** A form that is `i`, or opens on `i'` or `i’` (`I'm`, `I'll`,
   `I'd`, `I've`), is capitalised wherever it stands. Its contractions are lemmas en-fr does not gloss
   (`i'm`, rank 65), so they would be set aside: 233 tokens on the corpus, `I'll` and `I'd` on the
   baseline.
2. **A lemma that carries a CEFR level is a word.** `June`, `Mr`, `Mrs`, `Madame` are always
   capitalised. A CEFR list levels them, so they are words a reader learns: 315 tokens on the corpus.
   Spanish's and French's levels are estimated and go only to dictionary words, so the reading would
   change nothing for them. It is written for English alone, as French's elision reading is French's.

Neither of French's readings applies: an English capital after an apostrophe is a name's own
(`O'Brien`) or a contraction's, and a run is judged by its parts. The rule changes English's output,
so English's analyser version is bumped, `1.2.0` → `1.3.0` (*An analyser version per studied
language*). Spanish's and French's do not move. en-fr and en-es are built again with it: their
manifests' `analyzer_version`, the fixtures and the tests that write the version follow, as with 42d.

What it sets aside beyond the names this change takes out are words en-fr has never glossed, written
with a capital: unglossed names (`Hong Kong`, `NASA`), possessives (`Trump's`, `God's`), Title Case
words (`Prelate`). They were underlined with no card. As add-lingua-spanish-names D2 says of Spanish,
they no longer count against the reader.

*Rejected: no English rule* (the data alone). `London`, `Margaret` and `Sam` would stay underlined,
so only the vocabulary estimate would change, and the owner's reason for the change (« `london`
highlighted as unknown ») would stay true. Asked as Q1.
*Rejected: the rule without its readings.* It moves 548 more tokens of the corpus (D4's figures) and
sets aside `I'll`, `I'd`, `June` and `Mr`.
*Rejected: a list of English's capitalised words* (days, months, languages). The days are common nouns
in en-fr (`monday` « Lundi »), the months are levelled, and a language's adjective is a common word
(`english`, `french`, `spanish`). The data and the levels already decide; a list would be a third
source to keep in step.

### D5 — The vocabulary figures, and English's frozen typical vocabularies

The universes and estimates follow the dictionary words with no code (*Measured*). English's own ladder
computes its typical vocabularies from its pack: 1,213 / 3,074 / 7,155 / 14,433 / 18,123 at A2–C2.
The Spanish and French ladders borrow `ENGLISH_TYPICAL_VOCABULARY`, which generalise-lingua-native-
language D6 froze so that « an English dictionary update » would not move them. This change is no
dictionary update: it changes what all three languages count as a word. Left frozen, the borrowed
figures would count names that no ladder counts any more, and a French reader studying English and
Spanish would read B1 « 3 074 » on English's ladder and « 3 359, taken from English » on Spanish's.
The constant is frozen again on the new figures, with its provenance (« over en-fr's pack after
leave-lingua-names-out-of-english-and-spanish-words »). The four borrowed ladders move by their
`level-ladder` probe alone. Asked as Q2. Kept frozen, the four ladders and the browser extension's
requirement do not move.

### D6 — The re-pin: what moves and what cannot

`build.sh --reduce en-fr` and `--reduce es-fr` from their pins, nothing fetched beyond the pins'
assets. Each pin keeps its `snapshot` and `sources`, and its `reducer` digest moves
(`reduce-<pair>.py`, `reduce_edition_fr.py`), with `pack_version` and the pack's sha256 and size.
Then en-es and es-en are reduced again on the new studied tables (*A change to a studied language's
tables reaches every pair of that language*). Their glosses, senses and expressions stay byte for
byte. Their pins' studied record (`lexical.tsv`), `pack_version` (its studied digest) and pack move.
The English version bump moves en-fr's and en-es's `analyzer_version`.

**Moves:**
- `tables/en/lexical.tsv` (24,799 → 21,753), `tables/es/lexical.tsv` (22,755 → 21,062);
- `tables/en-fr/` and `tables/es-fr/` `manifest.json`, `pin.json`, `README.md`; `tables/en-es/` and
  `tables/es-en/` likewise;
- the testdata manifests of en-fr and en-es (the version) and their built bytes in
  `pipeline_testdata.rs`;
- the six goldens (D7), and the fixtures and tests that write English's version: lingua-wasm's
  `pack.lingua` and `golden.json`, the extension's `en-fr.testdata.lingua`, the agent's English
  `pack.lingua`, `language.rs`, `engine.rs`, `pack.rs` and `languages.rs`;
- the tests that pin a figure: `cross_native.rs` (25,372 / 3,359 → 22,337 / 3,074; 22,755 → 21,062),
  `vocabulary.rs` (the frozen constant, D5), and `committed_tables.rs`. In `committed_tables.rs`,
  *The shipped packs keep their bytes* asserted that en-fr and es-fr carry no lexical table, and
  *A pair left behind* assumed Spanish's dictionary words were es-fr's glossed lemmas;
- the lingua-data Python suite's `test_the_committed_dictionary_words_are_the_reference_s_glossed_lemmas`,
  which now applies the names rule to all three languages.

**Cannot move**, checked by the gates:
- every other file of `tables/en/` and `tables/es/`;
- every `gloss.tsv`, `senses.tsv` and `mwe.tsv`;
- `tables/fr/`, `tables/fr-en/` and `tables/fr-es/`, byte for byte, with their pins and packs;
- `reduce_common.py`, `reduce_edition_en.py`, `reduce_edition_es.py` and `reduce-fr-en.py`;
- Spanish's and French's analyser versions;
- the extension's sources and snapshots (`word-card-*.txt`, `selection-rows-fr.txt`: glosses and word
  grammar), and the site's coverage figures.

### D7 — The goldens, probe by probe

`LINGUA_BLESS=1 cargo test -p lingua-wasm --test english_baseline --test en_es_baseline --test
spanish_baseline --test es_en_baseline --test french_baseline --test fr_es_baseline` once, approved by
the owner beforehand (task 0.2). The prototype re-blessed all six on `main` at `f3585580`. The
`pack_version` and analyser version moves below are the implementation's: the prototype re-pinned
nothing.

| Golden | Probes moved | Which, and why |
|---|---|---|
| `en-fr.golden` | 19 of 142 | `pack` (+5,098 B, digest, `1.3.0`); the 15 `analyse` probes on the version string, and `analyse new-reader news`, `fiction`, `technical`, `informal` and `analyse reader news` on `Ruiz`, `Margaret`, `HTTP`, `Sam` too (D4); `level-ladder` (English's own figures); both `vocabulary-estimate` |
| `en-es.golden` | 19 of 182 | the same, its `pack` line on `pack_version` and the version |
| `es-fr.golden` | 7 of 137 | `pack` (+7,512 B, digest); `beside en-fr` (en-fr's line); `analyse new-reader nombres` and `analyse reader nombres` (the nine names); both `vocabulary-estimate`; `level-ladder` (D5) |
| `es-en.golden` | 6 of 176 | `pack` (`pack_version`); the two `nombres` analyses; both estimates; `level-ladder` |
| `fr-en.golden` | 2 of 213 | `beside es-en` (es-en's `pack_version`); `level-ladder` |
| `fr-es.golden` | 2 of 213 | `beside en-es` (en-es's `pack_version` and version); `level-ladder` |

Kept frozen (Q2), the four `level-ladder` lines of es-fr, es-en, fr-en and fr-es do not move. The
pull request runs a comparison script: the probe names in order, every unmoved probe byte for byte,
each moved analysis listing its tokens whose class moved, and every other moved line differing only by
the figures named here.

### D8 — Order, and the specs

| Change | Relation |
|---|---|
| 48b `refine-lingua-fr-en-glosses` | before, on `main`: `split`, `check_studied`, the rule for French. Its two requirements are MODIFIED here, so it is in `archiveAfter` |
| 41 `add-lingua-french-analysis` | before, on `main`: its names requirement says English's analysis does not change; MODIFIED here, in `archiveAfter` |
| 42d `ignore-lingua-soft-hyphens` | before, on `main`: English at `1.2.0`, and the version rule this change follows |
| 21, 22, 23, 24, 23b, 24b | before: es-en's and en-es's tables, pins and goldens, which this change re-pins and re-blesses under their rules; in `archiveAfter` |
| 34, 35 (enable es-en, en-es) | either side: they list the packs, with whichever dictionary words are on `main` |
| 52 (enable French) | either side: no French byte moves |

The specs: *A pack's dictionary words do not depend on its glosses* and *A studied language's tables
are kept once* (48b's) and *Only the reference pair's reduction writes its studied language's tables*
state the one rule. The scenario *The shipped packs carry no lexical table* is renamed *The reference
packs carry a lexical table*, because its old name no longer holds; its test is renamed with it.
*An English document's names are set aside* is ADDED beside Spanish's and French's. Spanish's loses
its « The English analysis SHALL NOT change », and French's its « and English's analysis ». The
knowledge model's and the extension's figures are MODIFIED.

## Open questions for the owner

- **Q1. English's names rule, here or later?** Without it, this change takes `london` out of the words
  English counts, but an English page still underlines `London`, `Margaret` and `Sam` as unknown words.
  With it (recommended), they are set aside as `Madrid` is in Spanish. The price: English's analyser
  version moves, and every page analysis of the English baseline moves on its version string.
- **Q2. English's « typical » figures for the other ladders.** Today every ladder says B1 « 3 359 ». After
  the change English's own ladder says « 3 074 ». Recommended: Spanish's and French's, which borrow
  English's figures, say « 3 074 » too. Otherwise they keep « 3 359, taken from English » beside
  English's « 3 074 ».
- **Q3. Words the French Wiktionary files only as names.** For example `Easter`, `Islam`, `Hebrew`,
  `Esperanto`, `unseen` (a mis-filed adjective) in English, and `Cristo`, `Biblia`, `títere` (« puppet »,
  mis-filed) in Spanish. In a text they would no longer be underlined, though a selection still shows
  their gloss. Recommended: accept them, one rule read off the tables. The other answer is a reviewed
  list kept as words, in the French Wiktionary's module, and the check of a studied folder would have
  to learn it.
- **Q4. English's two readings.** `I'm`, `I'll`, `I'd`, `I've` are never names (otherwise « I'll » is set
  aside wherever it follows a comma). A word a CEFR list levels is never a name (otherwise `June`, `Mr`
  and `Mrs` are). Recommended: both.
- **Q5. Existing readers.** On the release, an English reader's estimate drops by about 8 % (the
  baseline's B1 reader: 3,363 → 3,078 words), a Spanish reader's by about 5 % (2,285 → 2,176), because
  names are no longer counted. Say so in the release notes, or not?

## Risks / Trade-offs

- **[A word a learner wants is set aside]**: it keeps its gloss, a selection opens it, and it is only
  set aside where the document never writes it in lowercase. The cases are measured and listed (Q3).
- **[An English capital that is no name]**: the readings cover the pronoun I and the levelled words.
  What is left is unglossed: possessives, Title Case, quotations without a period. Those had no card,
  and the corpus measures them.
- **[Shipped output moves]**: en-fr and es-fr are shipped. The programme's rule needs the owner's
  approval (task 0.2), and the next release waits for the owner's go-ahead (M18).
- **[The agent plugin]**: a rebuilt plugin refuses an installed English pack of `1.2.0`, naming both
  versions. The owner reinstalls `~/.lingua/pack.lingua` from the new build.
- **[A re-reduction moves more than `lexical.tsv`]**: the reduction is pinned and reproduced by the
  reduce job. If any other byte of `tables/en/` or `tables/es/` moves, the implementation stops and
  reports it.

## Migration Plan

Nothing to migrate. Statuses, cards, exposures and backups are keyed by lemma. A pack of the old
version is refused by name, and the extension ships its packs with its core. Rollback is a revert of
the tables, the pins, the core's version and readings, and the goldens.

## Effort

2.5–4.5 ideal days:
- `dictionary_words`, the two reducers, their tests and the re-pins of the four pairs: 0.5–0.75;
- English's names rule, its readings and tests: 0.75–1.25;
- the version bump, fixtures and tests that write it: 0.5–0.75;
- the frozen figures: 0.25;
- the goldens, the comparison, the specs' tests, READMEs and `SOURCES.md`: 0.5–1.25;
- the owner's reading: 0.25.
