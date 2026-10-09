# Design — add-lingua-spanish-occitan-guard

## Context

`analysis/language.rs` on `main` since change 42 (`add-lingua-french-detection-guard`, `a656dbcb`):
- `detect(trimmed, languages)` answers whichlang's language when it is one of `languages`, unless
  that language's guard refuses the block: none for English, `iberian_neighbour` for Spanish,
  `romance_neighbour` for French. The gate asks `[studied]`, the vote the candidates.
- `iberian_neighbour` (`add-lingua-spanish-detection-guard`, 2026-10-03) splits a block on every
  character that is neither a letter nor the ASCII apostrophe, lowercases each word, counts a word
  opening on `l'`, `d'`, `s'` or `n'` as Catalan, and looks the others up in three sorted tables:
  Catalan's 32 markers, Galician's 32 and Spanish's 17 (`ahora`, `aunque`, `después`, `entonces`,
  `fue`, `había`, `hay`, `las`, `lo`, `los`, `muy`, `más`, `por`, `su`, `sus`, `también`, `y`). The
  block is refused when Catalan's count, or Galician's, is higher than Spanish's.

Spanish's analyser is `1.2.0` (`add-lingua-spanish-names`). Two packs study Spanish: es-fr, shipped,
and es-en, committed and shipped by change 34; the fixtures `testdata/es-fr` and `testdata/es-en`
build the small packs of the core's and the agent's tests.

whichlang 0.1.1 has no Occitan class. Change 42 measured, and left to a Spanish change (its open
question 2, settled so by the owner on 2026-10-09), that Spanish's guard lets 19.2 % of Occitan text
through as Spanish. The guard misses Occitan for a reason of its own: Occitan's articles `lo`, `los`,
`las` and its `sus` (*on*) are four of Spanish's seventeen markers. An Occitan sentence's own articles
count for Spanish, and its Occitan words count for no one. What the guard does refuse of Occitan, it
refuses through Catalan's elisions (`l'ostal`, `d'Occitània`).

## Goals / Non-Goals

**Goals:**
- Occitan blocks — Languedocien, Provençal, Gascon, Aranese — no longer analysed, counted or voted
  for as Spanish, measured.
- Spanish kept: no measurable false refusal on real Spanish, regional Spanish included — Latin
  American, Canary, rural.
- Catalan's and Galician's comparisons untouched: every block refused today stays refused.
- English and French detection unchanged; en-fr and en-es output byte for byte.
- A cost a reader of Spanish can ignore, and none for a reader who does not study Spanish.

**Non-Goals:**
- Recognising Occitan as a language of its own: no reader studies it.
- Asturian (70.3 % of its blocks still read as Spanish) and Aragonese: measured below, not guarded.
- The vote: a page whose blocks all read as Occitan or English, offered with English and Spanish as
  candidates, still goes to Spanish when a single Occitan block leaks (below, *Per article*).
- The existing guard's own refusals — 850 of the 453,821 Spanish blocks measured, 796 of them by
  Galician's `da` and `das`, Spanish's *gives* (« ¿Cuánto se da de propina en España? », « La vida
  empieza cuando te das cuenta de quién eres realmente. »), the rest by `polo`, `do`, « per se »,
  « ON », « Guns n'Roses » — and its reading of words: the curly apostrophe (U+2019) splits a word,
  and so does the soft hyphen (U+00AD) an e-book may hide inside its words, so that « vida » and
  « todo » read as `vi` `da` and `to` `do` (Risks). Changing either moves Catalan's and Galician's
  answers (open questions 4 and 5).

## Measurement

Fetched on 2026-10-09 into the scratchpad and kept out of the repository, as change 42's were; the
fixtures committed are sentences written for the change. Blocks are sentences or paragraphs, trimmed,
of 12 bytes or more (`MIN_BLOCK_BYTES`): change 42's 148,267, with Tatoeba's whole Spanish export in
place of its 10,000-sentence sample, and the sources below — 597,086 in all.

| Source | What | Licence |
|---|---|---|
| Change 42's corpus | UD Occitan TTB, Catalan AnCora, Galician CTG and the others; Tatoeba's Occitan (25,112), Catalan, Galician, Asturian and Aragonese exports, whole; Occitan Wikipedia, 22 articles (Languedocien and Provençal) | as change 42 lists them |
| Universal Dependencies (`# text`, `master`) | Spanish AnCora (test, dev), GSD (test), PUD, COSER (test: rural Spain, 30 Canary and 65 Colombian transcriptions) | per treebank |
| Tatoeba | Spanish, the whole export: 442,407 sentences | CC BY 2.0 FR |
| Spanish Wikipedia, plain-text extracts | 58 articles: Spain (8: España, Madrid, Sevilla, Salamanca, Historia de España…), Catalan subjects (7), Occitan subjects (15: Idioma occitano, Valle de Arán, Aranés, Gascón, Felibrige, Toulouse, Marsella…), Latin America (18: Buenos Aires, Ciudad de México, Bogotá, Lima, Montevideo, Santiago de Chile, La Habana, Caracas, Español rioplatense, Voseo, Lunfardo, Español mexicano, caribeño, andino, chileno, García Márquez, Borges, Tango), the Canaries (10: Español de Canarias, Canarias, Las Palmas, Santa Cruz de Tenerife, Silbo gomero, Gastronomía de Canarias…) | CC BY-SA 4.0 |
| Occitan Wikipedia | 10 Aranese and Gascon articles: Val d'Aran, Vielha e Mijaran, Aranés, Naut Aran, Bossòst, Conselh Generau d'Aran, Gascon, Bearn, Bigòrra, Les | CC BY-SA 4.0 |
| Project Gutenberg | José Hernández, *El gaucho Martín Fierro* (#14765) and *La vuelta de Martín Fierro* (#15066); Domingo F. Sarmiento, *Facundo* (#33267); Ricardo Palma, *Tradiciones peruanas* (#21282); Horacio Quiroga, *Cuentos de amor de locura y de muerte* (#13507) | public domain |

A Python prototype chose and ablated the table; a Rust one (whichlang 0.1.1, the comparison as D1–D4
write it) agrees with it on all 475,412 blocks whichlang reads as Spanish, and timed it (D5). Then the
comparison was applied to a scratch checkout of `main` (`f43c6035`, change 42 merged) and every
Lingua test and baseline run (D7).

**Before and after**, share of blocks read as Spanish (share of the text in brackets); *today* is
`main`'s guard:

| Language | Corpus | Blocks | whichlang | Today | With Occitan |
|---|---|---|---|---|---|
| Occitan | UD TTB | 1,467 | 23.1 % (25.4) | 17.4 % (17.3) | 8.0 % (5.8) |
| | Tatoeba (short sentences) | 24,098 | 28.4 % (29.4) | 25.1 % (24.8) | 16.7 % (14.8) |
| | Wikipedia (paragraphs) | 2,065 | 16.9 % (12.4) | 13.0 % (6.9) | 10.2 % (3.6) |
| | Aranese and Gascon Wikipedia | 739 | 15.4 % (12.6) | 11.9 % (6.7) | 9.5 % (2.3) |
| Catalan | UD, Tatoeba, Wikipedia | 16,553 | 48.6 % (60.1) | 15.5 % (6.0) | 13.7 % (5.1) |
| Galician | UD, Tatoeba | 8,933 | 42.1 % (45.3) | 21.4 % (16.2) | 21.2 % (16.0) |
| *Not guarded* | Asturian, Tatoeba | 697 | 88.1 % (91.7) | 71.3 % (73.9) | 70.3 % (72.4) |
| | Aragonese, Tatoeba | 65 | 52.3 % (54.5) | 49.2 % (50.7) | 46.2 % (48.3) |

Pooled over change 42's three Occitan sources: 23.7 → 15.8 % of the blocks, 19.2 → 10.9 % of the
text. What leaks is 4,361 blocks: 3,565 without an Occitan word of the table or an elision (« Lo libre
es sus la taula. », « La morfologia es redusida. » — `es`, `son`, `la`, `de`, `un` are Spanish too),
and 796 whose only ones are written with a capital, mostly opening the sentence (« Siá pacienta. »,
« Dempuèi los ans 1980… », « M'agradan los esquiròls. », D3).

**Per article**, on the 32 Occitan Wikipedia articles (Languedocien, Provençal, Gascon, Aranese): the
text the Spanish gate keeps falls from 6.9 to 3.2 % (« Occitan » 19.8 → 4.6 %, « Tolosa » 13.7 →
6.1 %, « Naut Aran » 18.9 → 3.1 %); 25 articles of the 32 still hold ten words read as Spanish, and
among English and Spanish the vote still gives 29 of them to Spanish (31 today) — the leak decides it,
since English reads almost none of their blocks.

**Spanish refused.** Of 459,668 Spanish blocks, whichlang reads 453,821 as Spanish and today's guard
keeps 452,971. The Occitan comparison refuses 2 of them (0.0004 %; 0.0008 % of the text):

| Spanish corpus | Blocks | Read as Spanish | Kept today | Refused |
|---|---|---|---|---|
| UD AnCora (test, dev) | 3,354 | 3,324 | 3,319 | 0 |
| UD GSD and PUD | 1,427 | 1,420 | 1,420 | 0 |
| UD COSER: rural Spain / Canaries / Colombia | 409 / 26 / 63 | 404 / 26 / 63 | 403 / 26 / 63 | 0 |
| Tatoeba | 438,132 | 433,536 | 432,720 | 0 |
| Wikipedia: Spain / Catalan subjects | 2,612 / 1,628 | 2,405 / 1,468 | 2,402 / 1,462 | 0 |
| Wikipedia: Latin America / the Canaries | 4,551 / 1,346 | 4,136 / 1,248 | 4,130 / 1,248 | 0 |
| Wikipedia: Occitan subjects | 1,139 | 921 | 909 | 2 |
| Books: Argentina / Peru / Uruguay | 2,237 / 1,070 / 1,674 | 2,152 / 1,065 / 1,653 | 2,152 / 1,064 / 1,653 | 0 |

Both refused blocks are in the article on Occitan, and neither is Spanish prose: a Catalan sentence
quoted (« La rata que menja el raïm que neda dins el riu (catalán) ») and a line setting Catalan,
Spanish and Languedocien forms side by side (« … qüan venga (cat. esp. id; lang. quand vendrá). »).
The Rioplatense, Chilean, Mexican, Caribbean and Canary articles, the gaucho verse of *Martín Fierro*
and COSER's Canary and Colombian speech lose nothing.

## Decisions

### D1 — A third comparison: Occitan against the Spanish markers Occitan does not write

For a block whichlang reads as Spanish, `iberian_neighbour` counts, beside its three counts, the
block's Occitan markers (D2, D3, D4) and its Spanish markers other than `lo`, `los`, `las` and `sus`.
The block is refused when Catalan's count or Galician's is higher than Spanish's — as today — or when
Occitan's is higher than those Spanish markers. A tie, or no marker, keeps it Spanish. This is the
archived guard's own rule read per neighbour: each neighbour's function words against the Spanish
ones it does not write. Catalan writes none of Spanish's seventeen and Galician one (`por`); Occitan
writes four, its articles among them.

Since the first two comparisons are today's, the guard only refuses more: a block refused today is
refused after, and Catalan's and Galician's tables and their counts do not move.

Measured alternatives, with D2's table:
- **Against Spanish's whole table**: Occitan 18.3 % of the blocks (14.4 % of the text) instead of
  15.8 % (10.9 %), one Spanish block refused instead of two. Occitan's articles defend it: « Los
  dròlles son totjorn dins lo jardin. » ties, two against two.
- **Pooled with Catalan** (Catalan's and Occitan's counts together against Spanish's): 17.8 % (13.3 %),
  for the same reason.
- **`lo`, `los`, `las` and `sus` out of Spanish's table for all three comparisons**: 14.8 % (9.5 %),
  but 259 Spanish blocks refused — Galician's table holds `da`, Spanish's *gives* (« Me da lo
  mismo. », « A los animales les da miedo el fuego. »), which today's articles outweigh.

*Rejected — a second detector with an Occitan class*, as changes 19 and 42 rejected one.

### D2 — The table, measured word by word

121 closed-class words of Languedocien, Provençal, Gascon and Aranese:
- adverbs and negation (27): `pas`, `jamai`, `sonque`, `tanben`, `totjorn`, `sovent`, `fòrça`,
  `tròp`, `puèi`, `pasmens`, `alara`, `atau`, `uèi`, `ièr`, `deman`, `ongan`, `vaquí`, `aicí`, `ací`,
  `lèu`, `quora`, `cossí`, `abans`, `darrièr`, `davant`, `defòra`, `dempuèi`;
- pronouns and determiners (36): `ieu`, `nosautres`, `lor`, `lors`, `li`, `çò`, `aquò`, `aquò's`,
  `quicòm`, `degun`, `aqueste`, `aquestes`, `aquestas`, `aqueles`, `aqueth`, `aquera`, `aqueras`,
  `mon`, `ma`, `sas`, `tas`, `meu`, `nòstre`, `nòstres`, `nòstra`, `nòstras`, `vòstre`, `vòstres`,
  `vòstra`, `vòstras`, `tota`, `totas`, `totes`, `totis`, `qual`, `quin`;
- articles and their contractions (14): Gascon and Aranese `eth`, `deth`, `dera`, `deras`, `ena`,
  `ua`, `ues`, `deu`, `deus`, `peu`, `peus`, Languedocien `dau`, `sul`, `als`;
- prepositions and conjunctions (20): `dins`, `dens`, `sens`, `jos`, `ambe`, `dambe`, `damb`, `dab`,
  `tà`, `entà`, `dinc`, `entrò`, `vèrs`, `mès`, `doncas`, `perque`, `perqué`, `pr'amor`, `quan`,
  `quand`;
- the copula, the auxiliaries and the modals (24): `èsser`, `èstre`, `èi`, `sèm`, `sètz`, `èra`,
  `èran`, `foguèt`, `foguèron`, `siá`, `siás`, `sia`, `seriá`, `aviá`, `avèm`, `avètz`, `pòt`, `pòdi`,
  `pòdes`, `pòdon`, `vòl`, `vòli`, `vòls`, `vòlon`.

Each is written by Occitan blocks whichlang reads as Spanish (ten candidates no such block writes were
dropped: `alavètz`, `gaire`, `èri`, `vosautres`, `eths`, `deths`, `aqueths`, `enes`, `enta`, `dap`),
and none is written as Spanish in the Spanish corpora: the table's 17 lowercase occurrences in the
Spanish blocks today's guard keeps, in 14 blocks, are quotations — Aranese `eth`, `dera`, `aqueth`
and Languedocien `quand` and `li` (« parla-li ») in the article on Occitan, Catalan `dins` and
Alghero's `qual` and `quin`, toki pona's `li` —, a URL (`www.ua.gov.pl`), and Quiroga's syllables
« de--li--rio »; the two refused blocks of the Measurement are two of them. The Spanish words and
regional spellings the corpora write are left out; English words a Spanish text quotes are left out
too (`an`, `as`, `car`, `far`, `fan`). Kept, though a Spanish line can write them where the corpora
never do: three French words, for their weight (`pas`, `mon`, `quand`, and the elision `t'`), Latin
`deus` (« un deus ex machina »), colloquial `ma` (*mamá*) and `tas` (*estás*), and `aqueste` and
`aquestas`, Spanish demonstratives of the Golden Age (open question 1). Without `pas`, 476 more
Occitan blocks would read as Spanish (1.7 points); without `dins`, 71; without the eight others
(`mon`, `quand`, `t'`, `deus`, `ma`, `tas`, `aqueste`, `aquestas`), 166 (0.6 point), one Spanish block
fewer refused.

What was left out, and why, each measured by adding it alone to the table above (*caught*: Occitan
blocks it would refuse; *refused*: Spanish blocks kept today it would refuse):

| Word | Caught | Refused | Reason |
|---|---|---|---|
| `e` (*and*) | 128 | +517 | Spanish's *e* before *i-*: « geografía e historia » |
| `as` | 9 | +27 | « sacar un as », English « as » |
| `fan` | 10 | +19 | « soy fan de los Giants » |
| `deis` | 0 | +6 | Spanish subjunctive of *dar* |
| `cal` | 96 | +3 | « la cal » (*lime*) |
| `ton`, `pus` | 17, 5 | +3, +3 | « sin ton ni son », « la herida soltó pus » (and Mexican « pus ») |
| `per` | 39 | +2 | « per cápita » (Catalan's table already has it) |
| `ta` | 15 | +2 | « ¡Ya 'ta lista! », « ta-te-ti » |
| `ne`, `ath` | 2, 0 | +2, +2 | « je ne sais quoi », a quotation |
| `cap`, `ara`, `au`, `aus`, `car`, `res` | 0–26 | +1 each | « cap. » and English « clay cap », « el ara » (*altar*), « au-au », German « aus », « muscle car », « carne de res » |
| `an`, `fa`, `far` | 24, 20, 30 | 0 | English « an » and « far »; « en fa mayor », « por fa »; `fa` and `far` are verbs, not closed-class |
| `mai` | 35 | 0 | « Mi mai cocinó yautía… », Caribbean Spanish (Tatoeba); `ma` is kept, never met as Spanish (Risks) |
| `mos`, `ei`, `dei`, `aquela`, `aquelas`, `ai`, `soi` | 0–16 | 0–1 | regional Spanish in the corpora: rural « Ya mos… » (*nos*, COSER), *Martín Fierro*'s « ei ser », « dei que », « las llanuras aquelas », old « ai » (*hay*), Chilean voseo « soi » (refused: « La variación "eríh"/"soi"… ») |
| `vos`, `sos`, `pa`, `es`, `son`, `era`, `eras`, `unas`, `coma`, `plan`, `mas`, `i`, `aquel`, `quina`, `mea`, `jo`, `lei`, `leis` | — | — | Spanish words or spellings, not tried: voseo, Aranese's articles `era` and `eras`, « coma », « plan », « quina », « mea culpa », « ¡jo! », Romanian *lei* |
| `ben`, `sa`, `ren` | 15, 15, 0 | 0 | « David ben Gurión »; the Balearic article in « playa de sa Coma »; Confucian *ren* |
| `amb`, `per`, `pels`, `dels`, `encara`, `tot`, `pel` | 0.4 point together | +2 | Catalan's table holds the first six; in Occitan's comparison a Spanish line naming a Catalan thing (« Junts pel Sí », « Llibre dels fets ») meets only the Spanish markers Occitan does not write; measured: 15.4 % (10.3 %) of Occitan through, 4 Spanish blocks refused, two of them « per cápita » |

### D3 — An Occitan word counts only as written in lowercase

As French's guard counts its neighbours (change 42, D4): a capital makes a name. Counting capitals too
lets 12.9 % of Occitan blocks through instead of 15.8 % (9.1 % of the text instead of 10.9 %), but
refuses 10 Spanish blocks instead of 2: « Mi maestra es la señora Li. », « Jet Li es una estrella de
cine chino. », « el vuelo número UA111 », « Ma. Teresa », and « Pas de la Casa », « Mon » or « Siá »
would join them on any page naming them. The cost is a sentence opening on its only marker (796 of the
4,361 leaking blocks).

*Measured and not adopted — a capitalised word counted when it holds `à`, `è`, `ò` or `ç`*, letters
Spanish never writes (« Siá » no, « Aquò », « Dempuèi », « Sèm » yes): 15.8 → 15.0 % of the blocks
(10.9 → 10.4 % of the text), no further Spanish block refused — a second casing rule for 0.8 point
(open question 2).

### D4 — How the guard reads a block: as today, with Occitan's elisions

The words are read as `iberian_neighbour` reads them today — split on every character that is neither
a letter nor the ASCII apostrophe, lowercased, Catalan's elisions counted first — so that Catalan's
and Galician's counts cannot move. One addition: a word opening on `qu'`, `m'` or `t'` and longer than
it (`qu'ei`, `m'agrada`, `t'agrada`) is an Occitan marker, under D3's casing: 16.4 → 15.8 % of the
blocks, no further Spanish block refused. Spanish writes none of them; a French quotation does
(« je t'aime », Risks).

Not adopted, measured:
- **Catalan's elisions counted for Occitan as well** (`l'`, `d'`, `s'`, `n'`, which Occitan writes):
  15.2 % of the blocks (10.0 % of the text), but 12 Spanish blocks refused — « l'USAP Perpignan »,
  « Commandeur de l'Ordre des Lettres », « Ostal d'Occitània », « langue d'oc »: Spanish names French
  and Catalan things with them, and against Occitan's comparison they meet fewer Spanish markers.
- **French's reading** (change 42's D4: joiners, digits, NFC): it would move Catalan's and Galician's
  counts on Spanish text, which this change does not touch.
- **The curly apostrophe** (U+2019), which today's reading takes for a separator: 18 of the 4,361
  leaking blocks hold one.

### D5 — Inside `iberian_neighbour`; what it costs

The comparison is part of `iberian_neighbour`, so change 42's shape carries it: `detect` runs Spanish's
guard only when Spanish is asked about, in the gate and in the vote alike, which never disagree. A
reader studying English or French alone pays nothing; English's and French's answers cannot move.

Timed on the Spanish corpora's blocks (UD, 40,000 Tatoeba sentences, the Spanish Wikipedia articles,
the books; Apple M2 Max, release build; WebAssembly under Node 22), three runs, outliers of a busy
machine dropped:

| Spanish text, µs per KB | whichlang | Today's guard | With Occitan | Merged table |
|---|---|---|---|---|
| native | 16.7–17.7 | 21.5–25.7 | 27.5–31.0 (+4.7–6.8) | 16.3–19.1 |
| WebAssembly | 14.2–15.4 | 19.6–23.0 | 25.5–29.2 (+5.7–6.7) | 16.4–19.4 |

A page analysis costs 233–251 µs per KB natively (change 42's measurement, es-fr), so the comparison
adds 2–3 % on Spanish pages; a 20 KB Spanish article, about 0.1 ms. The *merged table* is one sorted
table of every guard word with the kinds it belongs to (Catalan, Galician, Spanish, Occitan, the
Spanish markers Occitan writes), built once from the guard's tables, one binary search per word
instead of four: it answers the same on all 475,412 blocks and costs less than today's guard. The
implementation may take that shape; the tests do not change.

### D6 — Spanish's analyser version: bumped, `1.2.0` → `1.3.0`

The comparison changes which Spanish blocks are analysed, which the analyser version exists to signal
(*An analyser version per studied language*); English's and French's do not move (D5). The
requirement states the bump, not the number, so that a later Spanish bump does not contradict it
(change 41's lesson: Spanish's requirements still name `1.0.0` and `1.1.0` beside `1.2.0`). *Catalan
and Galician are not read as Spanish*, which this change modifies (D9), loses its `1.1.0` the same
way, stale since the names change.

The bump moves what a Spanish bump moves (`add-lingua-spanish-names` did the same at `1.2.0`, before
the agent's fixture existed): `SPANISH_ANALYZER_VERSION` and its doc line, the four Spanish manifests,
the two pins, the fixtures' recorded digests, the agent's fixture, one literal of
`tests/languages.rs`, and the goldens (D7). It moves es-fr's output, which the programme's rule holds
still: the owner approves it in this change's pull request (task 6.1).

### D7 — What moves, and what cannot

Applied to a scratch checkout of `main` (`f43c6035`):
- **The comparison alone**, before the bump: `english_baseline`, `spanish_baseline`,
  `es_en_baseline`, `en_es_baseline`, `french_baseline`, `cross_native` and `languages` pass without
  re-blessing, and the 38 tests of `analysis::language`. No block of any corpus changes its answer:
  `pages-es.txt` holds no Occitan; its `mixto` page's Catalan block is read as Spanish and refused as
  before, its Galician block read as Portuguese.
- **With the bump**, re-blessed once:
  - `es-fr.golden` moves on 18 lines, and only on the version: the `pack` line and the 17 analyses —
    `analyse new-reader` of its 13 pages, `analyse reader` of `noticias`, `homografos`, `nombres` and
    `mixto`;
  - `es-en.golden`, the same 18 lines;
  - `fr-en.golden`, its `beside es-en` line (`analyzer_version "1.3.0"`); no French analysis moves;
  - `en-fr.golden` and `en-es.golden` do not move: `english_baseline`, `en_es_baseline` and
    `cross_native` pass without re-blessing — their engines are asked no Spanish question.
- **The packs**: `tables/es-fr/` and `tables/es-en/` re-reduced from their pinned sources; their
  tables are byte for byte, their `pack_version` the same (snapshot and rules), and only
  `manifest.json` (`analyzer_version`) and `pin.json` (the pack's sha256) move — es-fr `ce03a605…`
  and es-en `70030bf8…` as the scratch builds them on today's `main`, 2,190,188 and 2,567,804 bytes
  as before. `scripts/lingua-data/build.sh` builds all five committed pairs to their pins. Should
  change 44b (`add-lingua-spanish-expression-keys`) land first, the packs are its 2,224,439 and
  2,608,413 bytes and the sha256 are re-recorded on top of it; the size stays what 44b made it.
- **The fixtures**: `testdata/es-fr` and `testdata/es-en` manifests; `crates/lingua-pack/tests/
  pipeline_testdata.rs` records the Spanish fixtures' bytes (`ac75501f…`, `d63a846a…`, 1,342 and
  1,356 bytes); `apps/lingua-agent/rust/tests/fixtures/es-fr.lingua` is the es-fr fixture's build
  (`build.sh --testdata es-fr`) — without it, eight of the agent's pipeline tests fail, the core
  refusing a `1.2.0` pack.
- With all of it, lingua-core, lingua-pack, lingua-agent and lingua-wasm pass (639 tests, a scratch
  check of the scenario sentences among them), clippy is clean, and the lingua-data Python suite
  passes (405 tests): nothing it reads moves.

The corpus is not touched: an Occitan block in `mixto` would shift the block numbers of its analyses
and move es-fr's output beyond the version; the unit tests carry Occitan (D8), as change 42's carry
Catalan and Romanian.

Two Spanish changes in flight move the same files, on overlapping lines:
- **Change 41b** (`fix-lingua-lemma-lookup`, PR #848) moves 3 lines of `es-fr.golden` and of
  `es-en.golden` — `gloss cuenta`, `gloss llama`, and the `quijote` page's `analyse new-reader` line
  (« quebrantos » loses a gloss) —, the last one among this change's 18.
- **Change 44b** (`add-lingua-spanish-expression-keys`) moves both goldens' `pack` line and
  `fr-en.golden`'s `beside es-en` line (the packs' size), both pins (sha256 and size), and 7 phrase
  probes of each golden.

Whichever merges second meets a textual conflict on those lines and pins, and re-blesses once on top
of the other, each line then carrying both changes — the version from here, the gloss or the size
from there. Landing after 44b, this change re-reduces es-fr and es-en on top of it; landing before
it, 44b records the packs again on top of this one. Neither moves the other's lines otherwise.

### D8 — Tests

In `language.rs`, beside Spanish's guard's tests: the spec's scenarios, each sentence first asserted
to be read as Spanish by whichlang (a whichlang update that stops reading it so fails the test
rather than passing it for the wrong reason); the Occitan table sorted, disjoint from Spanish's,
sharing with Catalan's `aquestes` alone (Languedocien and Catalan both write it, and it counts for
both: leaving it out of Occitan's table changes no answer on the 597,086 blocks), and the Spanish
markers Occitan writes a part of Spanish's table; Occitan's own articles not defending it (« Los
dròlles son totjorn dins lo jardin. », a tie against the whole table); the elisions counted, a
capitalised one not; « Pas de la Casa » and « Vielha e Mijaran » kept; regional Spanish kept
(Rioplatense, Canary, « mi mai »), and « Junts pel Sí »; the leak kept; the Catalan and Galician
sentences of today's tests refused as before; and a page of Occitan blocks among English and Spanish
giving Spanish no vote. The existing tests pass unchanged, the scenarios of *Catalan and Galician
are not read as Spanish* among them.

All of this ran green in the scratch checkout (the scenario sentences through `block_is_studied` and
`detect_document_language`).

### D9 — Gates and OpenSpec

1. en-fr and en-es do not move: `git diff --stat origin/main --
   crates/lingua-wasm/tests/baseline/{en-fr,en-es}.golden` is empty; `es-fr.golden`, `es-en.golden`
   and `fr-en.golden` move on the version alone (D7), and any other line is a review failure.
2. The workspace gates (fmt, clippy `-D warnings`, tests — `lingua-agent`'s among them —, `llvm-cov`
   ≥ 80 %, `wasm-pack test --node`), the extension's, and the lingua-data Python suite.

OpenSpec: one ADDED requirement in `lingua-analysis`, *Occitan is not read as Spanish*, which names
no version number. One MODIFIED: *Catalan and Galician are not read as Spanish* says « a tie, or a
block with no such function word, SHALL stay Spanish », which « Los dròlles son totjorn dins lo
jardin. » (no Catalan or Galician word) would contradict once refused; it gains « unless *Occitan is
not read as Spanish* refuses it », and its « Spanish's analyser version SHALL be `1.1.0` », stale
since `1.2.0`, becomes « adding it SHALL bump Spanish's analyser version and no other » (D6). Its
scenarios are kept word for word, and no open change holds it (the programme's rule). Change 42's
*Catalan, Occitan and Romanian are not read as French*, which says that the French guard leaves
Spanish's detection alone, is change 42's and is not modified. `archiveAfter` names
`add-lingua-french-detection-guard`, whose `detect` this builds on; `openspec_archive_order.py` exits
10 naming it.

## Risks / Trade-offs

- [French or Latin words in a Spanish line] → « Bailaron un pas de deux en el último acto. » is
  refused (the prototype says so): `pas` with no Spanish marker but `lo`, `los`, `las` or `sus`.
  « Cometió un faux pas », « Le dijo mon amour », « Me dijo je t'aime » and « es un deus ex machina »
  alike. None among the 453,821 Spanish blocks; `pas` alone is 1.7 points of Occitan. Accepted (open
  question 1).
- [A text about Occitan] → its quotations are refused (the two measured blocks). They are not Spanish.
- [Colloquial and old Spanish] → the table leaves out every regional spelling the corpora hold (D2),
  but keeps four Spanish words they never write: colloquial `ma` (*mamá*) and `tas` (*estás*), and
  the Golden Age's `aqueste` and `aquestas`. « Mi ma no está. », « Hola, ma, ¿qué hacemos hoy? »,
  « ¿Dónde tas, mi amor? Te espero. » and « Mas en aqueste valle umbroso reposa el pastor. » are
  refused (the prototype says so); a Spanish marker outside the four keeps a line (« Por aquestas
  montañas anduve solo. »). Open question 1.
- [Soft hyphens] → an e-book may hide a soft hyphen (U+00AD) inside its words; the guard splits on it
  as on any non-letter, and reads syllables. Today's guard already refuses such Spanish lines through
  Galician's `da` and `do` (« vida », « todo », « cada », « cuando »: three lines tried, three
  refused); Occitan's short words add their own syllables (`ma` in « mañana », `sas` in « cosas »,
  `lor` in « valor », `jos` in « lejos »). The corpora hold three such blocks read as Spanish, whose
  answers skipping the soft hyphen would not change. Open question 5.
- [The leak] → short lines without an Occitan function word, nearly one in five of them holding one
  written with a capital; and the vote, which still gives most Occitan articles to Spanish among
  English and Spanish (Measurement). The spec says so.
- [A reader of Spanish and French] (after change 52) → change 42's risk reversed: of Occitan text,
  French keeps 16.4 % and Spanish 10.9 % instead of 19.2 %, so an Occitan page votes French again;
  the two together read 27.3 % of it instead of 35.6 %.
- [A whichlang update] → the table was chosen against whichlang 0.1.1's answers; the tests check that
  each scenario sentence is still read as Spanish first, and a bump re-runs the measurement.
- [An installed agent] → the agent (outside the programme, M17) rebuilt at `1.3.0` skips an es-fr pack
  of `1.2.0` (« Pack ignoré ») until the new pack is copied beside it, as its README says.

## Migration Plan

Nothing to migrate: no stored format, wire field or table row moves; statuses, counts and backups do
not name the analyser version. The es-fr pack is rebuilt with the extension at its next release.
Rollback is a revert, the goldens, manifests, pins and fixtures with it.

## Open Questions

For the owner, none blocking. Each has a default, which the implementation follows unless the owner
answers otherwise.

1. **Words a Spanish sentence can borrow** (D2, Risks) — a few words of the Occitan list can also
   appear in Spanish: French `pas`, `mon`, `quand` and `t'` (« Bailaron un pas de deux », « Me dijo
   je t'aime »), Latin `deus` (« un deus ex machina »), everyday `ma` and `tas` (« Hola, ma »,
   « ¿Dónde tas? ») and the old `aqueste` and `aquestas`. A short Spanish sentence holding one of
   them and none of the list's own Spanish words (`y`, `por`, `muy`, `más`…) is then not analysed;
   none of the 452,971 Spanish sentences and paragraphs measured writes one as Spanish. In exchange
   they stop Occitan: `pas` alone keeps 476 of the 27,630 Occitan sentences and paragraphs from
   passing as Spanish (1.7 %), the eight others 166 (0.6 %). Keep all nine (the default), drop the
   eight and keep `pas` (Occitan read as Spanish 15.8 → 16.4 %), or drop all nine (18.2 %)?
2. **An Occitan word that starts a sentence** (D3) — an Occitan word written with a capital does not
   count, so that a place such as « Pas de la Casa » (in Andorra) never counts against Spanish. The
   price: an Occitan sentence whose only Occitan word comes first (« Siá pacienta. », « Dempuèi los
   ans 1980… ») stays Spanish. One more rule could count such a word anyway when it holds `à`, `è`,
   `ò` or `ç`, letters Spanish never writes (« Dempuèi », « Aquò », « Sèm » — not « Siá », whose `á`
   Spanish writes): Occitan read as Spanish 15.8 → 15.0 %, no more Spanish refused, one more rule and
   test to keep. Add it, or keep the single rule (the default)?
3. **Asturian and Aragonese** — two regional languages of Spain, close to Spanish (in Asturias, and
   in Aragon's Pyrenees). The detector takes 70 % of Asturian sentences and 46 % of Aragonese ones
   for Spanish (Tatoeba: 697 and 65 sentences), and this change does not touch them. A change of
   their own later, or leave them as they are (the default)?
4. **Spanish sentences set aside as Galician** (Non-goals) — today's guard counts `da` and `das` as
   Galician words, but they are Spanish too (*da*, « gives »; *das*, « you give »), so « ¿Cuánto se da
   de propina en España? » is set aside as Galician. Of the 453,821 Spanish sentences and paragraphs
   measured, today's guard sets aside 850, 796 of them for these two words (0.18 %). Taking `da` off
   the Galician list brings the 850 to 150, taking `das` too to 54; in exchange a little more Galician
   passes as Spanish (21.4 → 22.2 % of Galician sentences without `da`, 22.6 % without both). Fix it
   here, where es-fr's output then moves once for both, or in a change of its own (the default, as
   the owner scoped this change to Occitan)?
5. **E-books with hidden hyphens** (Risks) — some e-books hide an invisible hyphen inside words so a
   line can break there: « vida » is stored as « vi », the hyphen, « da ». The guard reads each piece
   as a word, so today « vida » and « todo » already look Galician (`da`, `do`) and such Spanish
   lines are set aside; Occitan's short words add a few more (`ma` in « mañana », `sas` in
   « cosas »). Ignoring the invisible hyphen when reading fixes it and changes no answer on the text
   measured. Here, or in a change of its own (the default)?
