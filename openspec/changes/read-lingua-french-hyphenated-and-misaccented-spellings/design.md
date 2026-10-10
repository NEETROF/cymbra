# Design — read-lingua-french-hyphenated-and-misaccented-spellings

## Context

See proposal.md (Why). What exists, on `main` at `311f8fe3`, and what this change builds on:

| Where | What |
|---|---|
| `reduce-fr-en.py` | French's reference reducer (change 43). `UNMARKED_SPELLINGS` (change 43b, merged), form → (word, reason): a row's spelling is a form of its word alone — `Lexicon.read` makes the word its one candidate, `_inflections` skips it, `unmarked_spellings` seeds it whether or not an entry names it |
| Change 43e (`read-lingua-french-web-unmarked-spellings`, proposed, its questions settled) | `WEB_SPELLINGS`, 464 rows merged into `UNMARKED_SPELLINGS`, and the owner's four exceptions; the boundary check (the reduction fails on a row the section gives a meaning of its own or gives only as its accented word's spelling); an entry pointing at a row follows it; each spelling carries the readings of its word's forms it writes without their marks; Zipf ≥ 2.5 (its Q1); hyphenated words and wrong accents sent to a reviewed list of their own (its Q5, D7) — this change |
| The ranks (change 43's D6, `ranks_for`) | wordfreq's words that are French tokens, each the lemma of its own form: `déja` 4,125, `trés` 4,725, `aprés` 5,540 are ranked words of their own, unglossed, levelless. A hyphenated lemma is ranked when UD French-GSD attests it **as a lemma**, at the lower of wordfreq's estimate and GSD's frequency |
| The pre-pass (change 40, `push_french_run` in `analysis/tokenize.rs`) | a hyphenated run is read in order: a digit splits it; **a run the pack lists whole is one token** (`listed_whole`, the run lowercased in NFC); an elision on its first piece is split off; pieces after the first that are all pronouns are an inversion (`dit-il`); any other run is a compound the pack does not know — one token, its lemma the written run lowercased, its pieces resolved one by one only to classify it (`resolve_lemmas`) and shown as parts on the card |
| `tables/fr-en/`, `tables/fr-es/` | as 43e's prototype leaves them (below, *Measured*) |

## Goals / Non-Goals

**Goals:**
- French's hyphenated words written without their marks and its words written with wrong marks read
  as the word a reviewed list names, by rows a person reviews one by one (the owner's decision), never
  by a rule on marks alone.
- Each candidate list built from data and shown whole, with every group left out.
- What the rows move measured: ranks, forms, readings, dictionary words, glosses, coverage, treebanks,
  false readings in running text, goldens, packs, pins.

**Non-Goals:**
- Any byte of en-fr, es-fr, es-en or en-es, any shared or edition module, any core code; French's
  analyser version does not move.
- The dictionary's own spellings (change 43d's), the web's plain unaccented spellings (43e's).
- Making `elle-même` and its siblings forms of the tables (D8).

## Measured

A prototype in the scratchpad (never committed): `reduce-fr-en.py` of `311f8fe3` with 43e's 468 rows,
its redirect and its readings rule, then with this change's 84 rows on top, behind switches, over the
pinned `kaikki-French.jsonl` (sha256 `2d7bbe5f…`), GSD's training and development sections
(`4b9a87b1…`, `9221e508…`) and wordfreq 3.1.1. Switches off, it writes the committed `tables/fr/` and
`tables/fr-en/` byte for byte; fr-es's reducer over its pinned sources (`2e724e39…`, `bf44ebb4…`,
`bd14a5c4…`) writes the committed `tables/fr-es/` byte for byte. With 43e's switch alone it gives 43e's
own figures (forms 124,669, grammar 126,295, dictionary words 26,570, fr-en 30,165 glosses, fr-es's
four glosses out and 39 in, `level.tsv` byte for byte): **every figure below is this change's over
43e's**, the order the two land in. The goldens were blessed in a scratch checkout with D7's probes on
`main`'s tables, on 43e's and on 43e's plus this change's; every lingua-wasm test was run on the last;
the packs were measured on UD French-PUD and GSD's test section as `measure/fr-ud.sh` does. The
running text is change 43c's corpus file: UD's French sections, Tatoeba's 727,867 French sentences,
change 44c's Wikipedia sample and Gutenberg novels, change 39's pages — 6.36 M words as this change
counts them (a hyphenated run one word per piece).

## Decisions

### D1 — The hyphenated rows: the whole run, estimated from the corpora

**The tokenisation decides the mechanism.** Measured on the goldens (D7), over 43e's tables, which
already read `meme` as *même* and `etre` stays 43d's:

| Written | Today, and after 43e | With a row for the whole run |
|---|---|---|
| « peut-etre » | one compound token `peut-etre`: no gloss, parts *pouvoir* + `etre` | `peut-être` « maybe, perhaps » |
| « lui-meme » | compound `lui-meme`: no gloss, parts *lui* + *même* | `lui-même` « himself, itself » |
| « c'est-a-dire » | `ce` + a compound `est-a-dire`, parts *être* + *avoir* + *dire* | `c'est-à-dire` « that is to say » |
| « grand-mere », « la-bas » | compounds, parts *grand* + `mere`, *le* + *bas* | `grand-mère` « grandmother », `là-bas` « there, over there » |
| « celle-la » | an inversion (`la` is a pronoun): `celle` (*celui*) + `la` (*le*) | `celui-là` (from `push_french_run`'s order: a listed run is read before the inversion) |

A row for a piece cannot do it: an unlisted compound keeps its written lemma whatever its pieces read,
and the pieces `a` and `la` are words of their own, never rows. A row for the whole run (`peut-etre`
→ *peut-être*) makes the pack list the run, and the pre-pass reads it whole, its span the run's, as it
reads `peut-être` — no core code, no analyser version.

**Counting them.** wordfreq splits a word at its hyphens and answers a mix of its pieces dominated by
the rarer: `peut-etre` 4.75 is `etre`'s 4.77, `lui-meme` 4.75 `meme`'s 4.77, `c'est-a-dire` 5.86 and
`la-bas` 5.39 are its common pieces' order. It cannot say how often the web writes a hyphenated
spelling, and the corpora at hand cannot either: edited text, they write 7 of them in 6.36 M words.
They do count the accented compounds. So the estimate:

- the spelling's Zipf = the accented compound's Zipf in the corpora + **−1.59**, the median over 466
  of 43e's rows of (wordfreq's Zipf for the spelling − the corpora's Zipf for its accented word),
  quartiles −1.86 / −1.25 — the web's share of unaccented spellings and the corpora's own bias,
  measured together on the words 43e settled;
- held to 43e's settled threshold, Zipf ≥ 2.5 (its Q1): the accented compound standing at least ~84
  times in the corpora.

| Step | Count | Examples |
|---|---|---|
| Hyphenated forms of `tables/fr/forms.tsv` written with marks | 275 | |
| — their spelling without marks no form of the tables already | 267 | 8 are: a form of another word (`sous-estime`, `court-circuite`), the post-1990 `ouvre-boite`, the dictionary's `etats-unis` |
| — named by the section | 1 | `court-circuites` |
| — of a name, or of no dictionary word | 39 | `ile-de-france`, `jesus-christ`, `saint-etienne`, `pyrenees-orientales` (estimated 2.61) |
| **Forms of dictionary words** | **227** | 43e's D7 counted the same 227 |
| — estimated Zipf ≥ 2.5 | **18** | below |
| — the paradigm's fourth form, taken with its siblings | 1 | `celles-la` 2.39 (`celui-la`, `celle-la`, `ceux-la` are rows), as 43e took `sures` |
| — under 2.5, left out | 208 | `vis-a-vis` 2.43, `bien-etre` 2.41, `beau-frere` 2.28, `jusque-la` 2.23, `beau-pere` 2.20 … |
| **Rows** | **19** | |

**The 19 rows** — spelling → accented form (and its word when another), the accented compound's count
in the corpora / the estimated Zipf:

`peut-etre` → peut-être 2,061/3.92, `la-bas` → là-bas 1,326/3.73, `grand-mere` → grand-mère 823/3.52,
`lui-meme` → lui-même 661/3.43, `grand-pere` → grand-père 537/3.34, `apres-midi` → après-midi
515/3.32, `moi-meme` → moi-même 441/3.25, `petit-dejeuner` → petit-déjeuner 380/3.19, `celui-la` →
celui-là 305/3.09, `c'est-a-dire` → c'est-à-dire 219/2.95, `celle-la` → celle-là, *celui-là*
184/2.87, `au-dela` → au-delà 171/2.84, `eux-memes` → eux-mêmes 162/2.82, `ceux-la` → ceux-là,
*celui-là* 153/2.79, `soi-meme` → soi-même 147/2.77, `la-dessus` → là-dessus 142/2.76,
`apres-demain` → après-demain 92/2.57, `belle-mere` → belle-mère 87/2.55, `celles-la` → celles-là,
*celui-là* 60/2.39.

Every row's word is a ranked lemma reading as itself (43b's test); 7 of the 19 rows read as A1–A2
words (`peut-être`, `lui-même`, `c'est-à-dire`, `au-delà` A1; `grand-père`, `après-midi`, `eux-mêmes`
A2). No row is a spelling another language writes:
a hyphenated run spelled exactly as a French compound less its marks is that compound.

**Wrong accents inside a hyphenated word** (`lui-mème`, `la-bàs`, `sous-éstimées`, `sous-éstimés`):
once each in the corpora, no estimate possible; none is a row.

*Rejected — rows for the pieces* (`etre`, `meme`): the compound keeps its written lemma and no gloss
(measured above), and `a`, `la` cannot be rows. *Rejected — a core rule retrying an unlisted run with
the marks of a listed one*: a rule on marks, which the owner ruled out, and an analyser change.
*Rejected — every one of the 227*: the threshold is the owner's for the web's spellings, and the
estimate puts 208 of them under it; a row nobody meets costs a review for nothing. *Rejected —
wordfreq's figure*: it is a piece's.

### D2 — The misaccented rows

Every word wordfreq rates at Zipf ≥ 2.5 that carries an accent, a cedilla or a diaeresis, whose
letters (marks dropped) are those of a form of `tables/fr/forms.tsv` other than itself and its
unmarked spelling, and which is no form of another word today: 516. Then, as 43e's tests read them:

| Step | Zipf ≥ 3 | 2.5–3 | Examples |
|---|---|---|---|
| Candidates | 350 | 166 | |
| — the section names it: a word or a form of its own | 323 | 85 | the accented words themselves; `élève` beside `élevé`, `pâté` beside `pâte` |
| — the section gives it only as another word's spelling | 5 | 8 | `problême`, `précisement`, `sureté`, `réglement`, `complétement`, `protége` (D8) |
| — the word its letters point at is a name or no dictionary word | 6 | 8 | `ème` (→ `éme`), `jérome`, `andrés`, `león`, `malgrès` (its letters are `malgrés`', itself a misspelling) |
| — as common as that word or more | 3 | 1 | `elysées`, `barbès`, `pêle` (a piece of `pêle-mêle`), `vergès` |
| — another language writes it more often (wordfreq's en, es, pt, it, ca, de, nl) | 1 | 9 | `á` (Portuguese 4.71 / French 3.47), `lá`, `ã`, `å`, `cà`, `és`, `amé`, `accés`, `procés`, `succés` (Catalan) |
| — doubtful, left out | 0 | 2 | `çe` (its letters are `ce`'s; a row would read the rare `cé`), `abbès` (« Sidi Bel Abbès », both its occurrences) |
| **Rows** | **12** | **53** | **65** |

**Which word a spelling reads as.** A wrong accent keeps some of its word's marks in place: `agé`
keeps *âgé*'s final `é` and none of *âge*'s; `elève` keeps *élève*'s `è`, `elevé` *élevé*'s final
`é`; `géné` *gêné*'s. So a row reads as the form whose marks it keeps in the most places, then the
commoner — 43e's Q4 for the rest: `régle` keeps one mark of *règle* and one of *réglé*, and reads
*règle*, the commoner (as `règle` itself reads under M8). A plain « commoner » rule would read `agé`
as *âge* and `elève` as *élevé*. The row then names that form's word in `forms.tsv`.

**The 65 rows**, by Zipf band — `spelling` → form (and its word when another), the spelling's Zipf /
the form's:

| Zipf | Rows | Spellings |
|---|---|---|
| 3.50– | 3 | `déja` → déjà 4.00/5.84, `trés` → très 3.92/6.12, `aprés` → après 3.82/6.12 |
| 3.00–3.50 | 9 | `egalité` → égalité 3.26/4.63, `étre` → être 3.23/6.38, `oú` → où 3.22/6.07, `electricité` → électricité 3.21/4.57, `mére` → mère 3.15/5.45, `eté` → été, *être* 3.14/6.22, `théatre` → théâtre 3.09/4.85, `verité` → vérité 3.08/5.02, `agé` → âgé 3.00/4.25 |
| 2.75–3.00 | 16 | `méme` → même 2.97/6.32, `desolé` → désolé 2.93/4.78, `interêt` → intérêt 2.91/5.13, `dejà` → déjà 2.90/5.84, `arreté` → arrêté, *arrêter* 2.87/4.91, `dégats` → dégâts, *dégât* 2.86/4.25, `etrangères` → étrangères, *étranger* 2.86/4.55, `lâ` → là 2.86/5.99, `laché` → lâché, *lâcher* 2.85/3.98, `securité` → sécurité 2.85/5.24, `eléments` → éléments, *élément* 2.84/4.96, `pére` → père 2.84/5.44, `intéret` → intérêt 2.81/5.13, `evêque` → évêque 2.78/4.38, `arrété` → arrêté, *arrêter* 2.77/4.91, `trève` → trêve 2.76/3.84 |
| 2.50–2.75 | 37 | `agée` → âgée, *âgé* 2.74/4.04, `probléme` → problème 2.73/5.41, `interessé` → intéressé, *intéresser* 2.72/4.36, `frére` → frère 2.71/5.18, `régles` → règles, *règle* 2.70/4.92, `societé` → société 2.69/5.51, `extrème` → extrême 2.67/4.74, `oû` → où 2.67/6.07, `premiére` → première, *premier* 2.67/5.68, `préfére` → préfère, *préférer* 2.67/4.76, `éspère` → espère, *espérer* 2.67/5.13, `elevé` → élevé 2.66/4.76, `realité` → réalité 2.66/5.04, `régle` → règle 2.66/4.72, `dépot` → dépôt 2.65/4.40, `elève` → élève, *élever* 2.65/4.67, `epée` → épée 2.63/4.20, `ètre` → être 2.63/6.38, `evénement` → événement 2.62/4.66, `agées` → âgées, *âgé* 2.61/4.05, `arréter` → arrêter 2.61/5.05, `étes` → êtes, *être* 2.61/5.37, `systéme` → système 2.59/5.40, `agés` → âgés, *âgé* 2.57/3.98, `béta` → bêta 2.57/3.56, `génant` → gênant 2.55/3.91, `géné` → gêné 2.55/3.74, `espére` → espère, *espérer* 2.54/5.13, `évenement` → événement 2.54/4.66, `récuperer` → récupérer 2.53/4.55, `crême` → crème 2.52/4.39, `reservé` → réservé, *réserver* 2.52/4.32, `decidé` → décidé, *décider* 2.51/4.90, `faché` → fâché 2.51/3.55, `énormement` → énormément 2.51/4.38, `elèves` → élèves, *élever* 2.50/4.85, `extrèmement` → extrêmement 2.50/4.60 |

The words they read as: 27 A1 (*être*, *même*, *très*, *déjà*, *père*), 17 A2, 8 B1, 9 B2, 2 C1, 2 C2,
`dégât` levelless. `oú`, `oû` and `lâ` are short but no other language's (wordfreq's seven rate them
under French); `á` and `lá` are Portuguese's first.

*Rejected — a rule swapping marks* (any word retried with its letters' accented form): the owner's
decision, and it would read `élève`/`élevé`, `pâté`/`pâte`, Catalan's `accés` and Portuguese's `lá`
as the words they are not.

### D3 — The mechanism: 43e's table and its check

`reduce-fr-en.py` gains two literals, spelling → word: `WEB_HYPHENATED` (D1's 19) and
`WEB_MISACCENTED` (D2's 65), alphabetical, each merged into `UNMARKED_SPELLINGS` with one shared
reason (« a hyphenated word written on the web without its marks; the owner's reviewed list », « a
word written with marks not its own; the owner's reviewed list »), and read exactly as 43e's
`WEB_SPELLINGS` is: the named word its one candidate, its own entries and any entry's inflection
never making it another word's form, seeded whether or not an entry names it, an entry pointing at it
following it (measured: none does). 43e's boundary check reads the two literals too: the reduction
fails, naming them, on a row the section gives a meaning of its own or gives only as a spelling of
another word — so `élève`, `problême` or a dictionary update naming a row stops for a person. Each
row's word is the lemma `forms.tsv` gives its form today (D1, D2), so a row follows change 43's
choices and makes none. `is_form` already admits a hyphenated run and an elided start inside one
(`c'est-à-dire`).

*Rejected — rows in `WEB_SPELLINGS`*: one literal, but the two lists were chosen by different rules,
reviewed apart, and named apart in the reason a reader of the table sees. *Rejected — a data file*:
43e's reason (the rule digest names the reducer's modules).

### D4 — The readings

43e's rule (D3 there): a spelling carries the readings of every form of its word it writes without
their marks. A hyphenated row follows it unchanged (`grand-mere` the feminine singular noun of
`grand-mère`, `apres-midi` its four). A misaccented one writes its word's letters with other marks,
so the rule compares the letters, marks dropped, and when the spelling keeps marks of some of those
forms in place it takes theirs alone — D2's choice, read the same way: `arreté` and `arrété` the past
participle of *arrêter* (`arrêté`), not the present `arrête` 43e's `arrete` also carries; `agé` the
adjective `âgé`; `régle` *règle*'s noun and *régler*'s present (`other`), as `règle` carries; `géné`
the adjective `gêné` and *gêner*'s participle (`other`). Measured: **109 readings for 64 of the 84
rows** (11 for 8 hyphenated rows; the 20 without are adverbs and pronouns whose word carries none:
`déja`, `trés`, `peut-etre`, `lui-meme`). The tag pool is byte for byte.

### D5 — What moves in the tables, the pins and the measurements (over 43e's)

| File | Before → after | What |
|---|---|---|
| `tables/fr/forms.tsv` | 124,670 → 124,765 lines | the 84 rows (64 of them were their own lemma, 20 no form), and the 11 forms the entering lemmas bring beside their own |
| `tables/fr/freq.tsv` | 60,000 lines | 64 misaccented rows leave the ranks — `déja` 4,125, `trés` 4,725 (top 5,000), `aprés` 5,540 (top 10,000), 12 in the top 20,000 —; 64 lemmas enter at the cut's end (59,934–59,997: `théosophie`, `tilapia`, `tâtonner`, `vertébré` … `xanadu`); 55,809 ranks between move up by 1 to 64. No hyphenated row was ranked |
| `tables/fr/grammar.tsv` | 126,295 → 126,434 | D4's 109, and 30 of the entering lemmas' |
| `tables/fr/lexical.tsv` | 26,570 → 26,582 | 12 entering lemmas (`théosophie`, `tâtonner`, `vertébré`) |
| `tables/fr/level.tsv`, `tags.tsv`, `studied.json` | byte for byte | no row was a levelled word; the bands keep their counts |
| fr-en `gloss.tsv`, `senses.tsv` | 30,165 → 30,179 glosses | 14 entering lemmas; none lost or changed; `mwe.tsv` byte for byte |
| fr-en coverage | 93.8 / 87.2 / 76.8 → 93.9 / 87.2 / 76.8 % | 4,691 / 8,715 / 15,350 → 4,693 / 8,716 / 15,357 of the 5,000 / 10,000 / 20,000 commonest |
| fr-es `gloss.tsv`, `senses.tsv` | 19,058 → 19,064 | 7 entering lemmas in; `trève` « Tregua » out — the French Wiktionary's old spelling, now read through *trêve* « Tregua »; every other row and `mwe.tsv` byte for byte |
| fr-es coverage | 83.1 / 70.8 / 57.0 % unchanged | 4,156 / 7,083 / 11,404 → 4,157 / 7,084 / 11,407 |
| The packs | fr-en 2,547,436 → 2,547,127 B; fr-es 1,980,036 → 1,979,249 B | the pins record them |
| UD French-PUD (gated) | 99.13 / 96.40 / 100.00 → 99.13 / 96.41 / 100.00 % | « tâtonnant », now *tâtonner*, an entering lemma; no row stands in PUD |
| UD French-GSD, test (reported) | 98.91 / 95.94 / 99.72 → 98.91 / 96.02 / 99.72 % | « trés » ×3, now *très* (« la qualité du service est trés trés trés… bonne ») |
| Readings on the treebanks (reported) | PUD participles 974 → 975 words read | « tâtonnant » |

**en-fr, es-fr, es-en and en-es cannot move.** The literals, their check and the readings rule live in
`reduce-fr-en.py`, which fr-en's rule digest names and no other pair's; `tables/en/`, `tables/es/` and
their pairs' folders are not written; no core, wasm or extension code changes. Over the prototype every
lingua-wasm test passed — `english_baseline`, `spanish_baseline`, `es_en_baseline`, `en_es_baseline`,
`cross_native`, `parity`, `card_gloss_language`, `deck` unmoved, the two French goldens once
re-blessed.

### D6 — The rows in running text

Every occurrence of a row's spelling as a word in the corpora (*Measured*):

| Rows | Met | The word the row names | False |
|---|---|---|---|
| The 19 hyphenated | 2 | 2 (« Celui-la est un traître. », « Juba est arrivé la-bas ») | 0 |
| The 12 misaccented at Zipf ≥ 3 | 30 | 30 (« trés » ×9, « déja » ×6, « agé » ×5, « oú » ×3, « théatre » ×3 …) | 0 |
| The 53 from 2.5 | 40 | 36 (« pére » ×7, « régle » ×4, « oû » ×4, « interêt » ×3, « agée » ×3) | 4: « Monsieur Faché » (a name), Walloon « benet/béta » and « oû » ×2, quoted in Wikipedia |
| **All 84** | **72** | **68** | **4** |

By corpus: Tatoeba, its users' sentences, 54 met, 53 right (« Monsieur Faché »); UD 13 of 13 (GSD's reviews: « trés » ×9);
Wikipedia 4 met, 1 right; Gutenberg 1 of 1 (a title, « THÉATRE »). A false reading reads a name or a
neighbour language's word as a French word — today the same token reads as an unknown word of its own,
or, a capital in mid-sentence, as a name set aside —; the hyphenated rows meet no such word.

### D7 — The goldens and the snapshots

**Re-blessed on the prototype's tables over 43e's, 3 of the 233 probes move in each golden** — the
pack line and the two vocabulary universes, 26,570 → 26,582 (`new-reader`) and 26,569 → 26,581
(`reader`). No page of the corpus writes a row's spelling, so no `analyse` probe moves.

**Six phrase probes and two grammar probes are added** to the French scenario
(`support/french.rs`: the phrases after 43e's, the grammar probes at the end of `REFERENCE_GRAMMAR`):

| Probe | fr-en, after 43e → this change | fr-es |
|---|---|---|
| « peut-etre qu'il viendra » | compound `peut-etre`, no gloss, parts *pouvoir* + `etre` → `peut-être` « maybe, perhaps » | → « Quizá, tal vez, quizás… » |
| « il l'a fait lui-meme » | compound `lui-meme`, parts *lui* + *même* → `lui-même` « himself, itself » | → `lui-même`, which fr-es does not gloss |
| « c'est-a-dire demain » | `ce` + compound `est-a-dire` → one token `c'est-à-dire` « that is to say » | → « A saber, en otras palabras… » |
| « ma grand-mere habite la-bas » | compounds `grand-mere`, `la-bas` → `grand-mère` « grandmother », `là-bas` « there, over there » | → « Abuela », « Allí, allá » |
| « c'est trés bien, je l'ai déja vu » | `trés`, `déja` unknown, no gloss → *très* « very », *déjà* « already… », and the expression `très bien` | → *très*, *déjà* « Ya » (fr-es holds no `très bien`) |
| « elle l'a fait elle-meme » | the record of D8: compound `elle-meme`, parts *elle* + *même* — unmoved | unmoved |
| `arreté` as *arrêter* | → « past participle » alone (D4) | the same |
| `grand-mere` as *grand-mère* | → the feminine singular noun | the same |

`french_baseline.rs`'s `the_probes_are_the_references_then_the_cards_then_forty_lemmas` counts two
more reference probes. `test/word-card-fr-en.spec.ts` and `word-card-fr-es.spec.ts` render every
grammar and phrase probe: their counts gain 2 and 6, their snapshots gain the eight cards and move
nothing else.

### D8 — Not here, and the changes around this one

- **`elle-meme`, the owner's example, and its siblings — a change to the ranks, later.** `elle-même`
  (266 times in the corpora), `toi-même` (336), `vous-même` (186), `vous-mêmes` and `nous-mêmes` (95
  each), `elles-mêmes` (40) are **no forms of the tables**: the section gives each its own entry, GSD
  lemmatises `elle-même` as *lui-même* (23 times) and `elles-mêmes` as *eux-mêmes*, and change 43
  ranks a hyphenated lemma only by its count as a GSD lemma, so none is ranked, and none is a form of
  a ranked word. A row needs a ranked word, so their spellings cannot be rows; the accented words
  themselves read today as unknown compounds (parts *elle* + *même*). Recorded for a change of French's
  ranks (a compound the section heads and GSD writes, counted by its written form); the probe
  « elle l'a fait elle-meme » records it.
- **The dictionary's own misaccented spellings.** 13 at Zipf ≥ 2.5 (`problême` « alternative form of
  problème », `précisement`, `sureté`, `réglement`, `complétement`, `protége`, `allègement`) are words
  of their own in the tables today: change 43d's rule reads a spelling the section gives of its word
  with the marks dropped, not with other marks, and this change's check refuses them. Recorded for 43d
  and the owner.
- **Change 43e** (proposed, settled): this change extends its table, its check and its readings rule,
  and lands after it; its prototype is the base of every figure above.
- **Change 43d** (proposed) and **change 43c** (merged): 43d also edits `reduce-fr-en.py` and re-pins
  fr-en and fr-es; whichever lands later re-reduces on the other's tables and re-blesses the two French
  goldens. No row of this list is a spelling 43d reads (D3's check). 43c: a misaccented word alone in
  capitals that the acronym table does not hold falls through to the cascade, lowercased, so « TRÉS »
  reads *très*.

## Risks / Trade-offs

- **[The hyphenated threshold is an estimate]** → it reads the accented compound's real count and a
  share measured on 43e's own rows; the quartiles (−1.86 / −1.25) would make it 15 or 24 rows
  (and `celles-la`); the list is reviewed by name (task 0.1), and a row's cost is nil in running text (D6).
- **[A name or another language's word spelled like a misaccented row]** → 4 of 72 occurrences (D6);
  the spellings other languages write more often are out (D2).
- **[A wrong accent two words share]** → read by the marks it keeps, then the commoner (D2); `régle`
  carries *régler*'s present as `other`.
- **[A dictionary update names a row]** → the reduction fails, naming it (D3).
- **[Another French table change lands first]** (43d, a fr-es refinement) → the later one re-reduces
  and re-blesses; this change's counts are re-measured on the tables it lands on (tasks 2.1, 4.2).
- **[wordfreq's next version]** → the rows do not depend on frequencies once reviewed; an update
  re-pins as any source does.

## Migration Plan

None: no package lists a French pair, no reader holds a French pack. Rollback is reverting the
commit; the next reduction writes the tables back.

## Effort

0.75–1.25 ideal days: the two literals and the readings rule (0.25), the tests (0.25), the re-pin of
fr-en and fr-es and the treebank measure (0.25), the goldens and the snapshots (0.25), the pull
request's comparison (0–0.25). Outside the 57 changes' estimate.

## Open Questions

None. The owner's reading of the 84 rows (task 0.1) strikes any by name; a row struck changes D5's
and D7's counts, which the implementation measures again.
