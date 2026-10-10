# Design — refine-lingua-spanish-galician-markers

## Context

`analysis/language.rs` on `main` since change 42b (`add-lingua-spanish-occitan-guard`, #856):
- `detect(trimmed, languages)` (change 42) runs Spanish's guard, `iberian_neighbour`, only when
  Spanish is asked about, in the gate and in the vote alike.
- `iberian_neighbour` splits a block on every character that is neither a letter nor the ASCII
  apostrophe, lowercases each word, counts a word opening on `l'`, `d'`, `s'` or `n'` as Catalan, and
  looks the others up in sorted tables: Catalan's 32 markers, Galician's 32, Spanish's 17 and
  Occitan's 121 (with its elisions `qu'`, `m'`, `t'`). A block is refused when Catalan's count, or
  Galician's, is higher than Spanish's, or Occitan's higher than the Spanish markers Occitan does
  not write.
- Galician's 32 (`add-lingua-spanish-detection-guard`, 2026-10-03): `ao`, `aos`, `aínda`, `cando`,
  `coa`, `coas`, `da`, `das`, `do`, `elas`, `eles`, `foi`, `hai`, `iso`, `isto`, `lle`, `lles`, `moi`,
  `máis`, `non`, `nun`, `nunha`, `onde`, `pola`, `polas`, `polo`, `polos`, `súa`, `tamén`, `unha`,
  `unhas`, `xa`. In Galician `da` and `das` are *de* + *a*, *de* + *as* (« of the »). Spanish writes
  both as forms of *dar*: *da*, « gives » (and the imperative « give! »), *das*, « you give »
  (Galician spells its own verb *dá*, *dás*).

Spanish's analyser is `1.3.0` (42b). Two packs study Spanish: es-fr, shipped, and es-en, committed
(change 34 lists it); the fixtures `testdata/es-fr` and `testdata/es-en` build the small packs of the
core's and the agent's tests.

Change 42b measured that today's guard refuses 850 of the Spanish blocks the detector reads as
Spanish, 796 of them for `da` and `das`, and left it to a change of its own (its open question 4,
settled so by the owner on 2026-10-10). whichlang 0.1.1 has no Galician class; it reads 42.1 % of
Galician blocks as Spanish, and the guard keeps 21.2 % of them as Spanish.

## Goals / Non-Goals

**Goals:**
- A Spanish line refused only for Spanish's *da* or *das* analysed as Spanish, measured.
- Galician kept out as far as the guard can: the smallest leak for what Spanish gets back.
- Catalan's and Occitan's comparisons untouched; every block the guard keeps today stays kept.
- English and French detection unchanged; en-fr and en-es output byte for byte.
- No cost a reader notices.

**Non-Goals:**
- The guard's other refusals of Spanish — 56 blocks after this change: `polo` (14, « Polo Norte »),
  `do` (7, « el do de pecho », « Banco do Brasil »), `súa`, `non` (« persona non grata »), `pola`,
  and Catalan and Occitan words in quotations and bibliographies (« per se », « ON », « Guns
  n'Roses », « Institut d'Estudis Andorrans »). Each is a Spanish noun, a name or a quotation;
  changing them moves Catalan's or Galician's answers otherwise than this change measures.
- The guard's reading of words: the soft hyphen (U+00AD) an e-book hides inside a word, which makes
  « vida » `vi` `da` and « todo » `to` `do` (change 42b's open question 5, a change of its own), and
  the curly apostrophe.
- Galician words the table lacks: `é` (« is ») is measured below and offered (open question 2).
- Asturian and Aragonese (42b's open question 3: left as they are).

## Measurement

The corpus is change 42b's, fetched on 2026-10-09 into the scratchpad and kept out of the
repository: 597,086 blocks of 12 bytes or more — UD, Tatoeba (Spanish whole, Galician, Catalan,
Occitan, Portuguese and the others), Spanish Wikipedia (Spain, Catalan and Occitan subjects, Latin
America, the Canaries), Occitan Wikipedia, five Argentine, Peruvian and Uruguayan books; its
sources and licences are 42b's design's *Measurement*. whichlang reads 475,412 of them as Spanish.

The guard's variants ran in a scratch copy of `main`'s `lingua-core` (`eddaf712`, after #856), today's
checked against `main`'s own `block_is_studied` on all 475,412 blocks (the same answer every time).
The rule D1 adopts was then applied to a scratch checkout of `main`, where it gives the
measurement's answer on all 475,412 blocks, and every Lingua test and baseline ran (D5).

**Spanish refused**, of the blocks whichlang reads as Spanish:

| Spanish corpus | Read as Spanish | Refused today | With the rule |
|---|---|---|---|
| UD AnCora (test, dev) | 3,324 | 5 | 4 |
| UD GSD and PUD | 1,420 | 0 | 0 |
| UD COSER: rural Spain / Canaries / Colombia | 404 / 26 / 63 | 1 / 0 / 0 | 1 / 0 / 0 |
| Tatoeba | 433,536 | 816 | 28 |
| Wikipedia: Spain / Catalan subjects | 2,405 / 1,468 | 3 / 6 | 2 / 6 |
| Wikipedia: Latin America / the Canaries | 4,136 / 1,248 | 6 / 0 | 4 / 0 |
| Wikipedia: Occitan subjects | 921 | 14 | 11 |
| Books: Argentina / Peru / Uruguay | 2,152 / 1,065 / 1,653 | 0 / 1 / 0 | 0 / 0 / 0 |
| **All** | **453,821** | **852** (0.19 %) | **56** (0.012 %) |

Today's 852 are change 42b's 850 and the two quotations its Occitan comparison refuses. Of them,
820 are refused by Galician's comparison alone, 796 for `da` or `das` (`da` 700, `das` 95, both 1).
The rule gives back all 796: « ¿Cuánto se da de propina en España? », « No me da tiempo. », « Da un
paso atrás. », « ¿Me da una factura? », « Se le da bien la natación. », « …hasta que te das cuenta… »
(Tatoeba, 788); « Da la impresión de que desean que ETA actúe". » (AnCora); « …el referéndum sobre
la reforma política, que da paso a la democracia en España. », « La sede del Ayuntamiento que da
nombre a una gran plaza… » (Wikipedia, 6); « …el nombre que hoy le da la ciencia: _Chinchona_. »
(Palma). Two of the six Wikipedia lines are not prose: a heading (« === Atentado contra el edificio
del DAS === ») and a Galician example quoted inside a Spanish line (« 'Funme da casa' (da = de + a
→ da = de + la) »).

**Galician read as Spanish**, share of the blocks (share of the text in brackets):

| Galician corpus | Blocks | whichlang | Today | With the rule |
|---|---|---|---|---|
| UD CTG (paragraphs) | 861 | 55.3 % (55.2) | 10.9 % (10.6) | 15.0 % (14.5) |
| Tatoeba (short sentences) | 8,072 | 40.7 % (41.1) | 22.3 % (18.3) | 23.0 % (19.1) |
| Pooled | 8,933 | 42.1 % (45.3) | 21.2 % (16.0) | 22.2 % (17.7) |

The rule lets 91 more Galician blocks through (35 of UD, 56 of Tatoeba), and by construction every
one is a Galician line whose only Galician words of the table are `da` or `das`, outnumbering its
Spanish ones: « A esperanza é a razón da vida. », « Todos os alumnos da clase están presentes. »,
« Son o xefe da mesa de información. », « Esta lei mantén os criterios xerais para a atribución da
competencia territorial… ».

**The other languages.** Portuguese: 98 → 100 of 11,070 blocks read as Spanish (« Está no lado
esquerdo da rua. », « Joaquín é da Argentina. Ele é argentino. »); one Latin and one Romansh line
(« Si tua das cunctis omnia, multa feres. », « Bellas festas da Nadal! »). Catalan (13.7 %), Occitan
(15.8 %, and Aranese 9.5 %), Asturian (70.3 %) and Aragonese (46.2 %) do not move, block for block:
Galician's count enters none of their comparisons.

## Decisions

### D1 — `da` and `das` count for Galician only beside another Galician word

In `iberian_neighbour`, `da` and `das` are counted apart from Galician's 30 other markers. When the
block holds at least one of the 30, they are added to Galician's count, as today; when it holds
none, they count for no one. The comparison is today's: Galician's count against Spanish's. A block
whose Galician words are `da` and `das` alone is therefore never refused by Galician's comparison;
Catalan's and Occitan's still apply to it.

Galician prose writes its contractions among its other function words — `do`, `non`, `unha`, `xa`,
`máis`, `polo` —; a Spanish line writes *da* or *das* as a verb, among Spanish words. Of the 3,758
Galician blocks whichlang reads as Spanish, 326 write `da` or `das`, and 225 of them another
Galician word of the table; of the 796 Spanish blocks refused for them, none. The rule reads exactly
that difference. Both words stay in `GALICIAN_MARKERS`, so the table, its sortedness test and the
archived guard's 32 words do not move; a second constant names the two Spanish writes
too, as 42b's `SPANISH_MARKERS_OCCITAN_WRITES` names Occitan's four.

The guard only keeps more: Galician's count can only fall, so every block kept today is kept after,
and a block refused after was refused today.

Measured, every option the owner named and two more (Spanish blocks refused of 453,821; Galician
read as Spanish, blocks and text; Galician blocks more than today):

| Option | Spanish refused | Galician as Spanish | Galician more |
|---|---|---|---|
| Today | 852 | 21.2 % (16.0) | — |
| `da` out of the table | 152 | 22.1 % (17.9) | +76 |
| `da` and `das` out of the table | 56 | 22.4 % (18.5) | +110 |
| Both out when the block holds a Spanish marker | 847 | 21.4 % (16.8) | +20 |
| Both weighing one half | 847 | 21.2 % (16.0) | +1 |
| Both ignored right after a clitic (`me`, `te`, `se`, `le`, `les`, `nos`, `os`, `lo`, `la`, `los`, `las`) | 303 | 21.2 % (16.0) | 0 |
| **Both counted only beside another Galician word** | **56** | **22.2 % (17.7)** | **+91** |
| The same, with Galician's `é` (« is ») added (open question 2) | 58 | 19.7 % (15.4) | −134 |

- **Dropping `da` alone** gives back 700 of the 796 and leaves 96 blocks refused for `das` (« ¿Te
  das cuenta…? »); it lets 76 Galician blocks through.
- **Dropping both** gives back the same 796 blocks as the rule — the same 56 stay refused, one for
  one — but lets 19 more Galician blocks through: those where `da` or `das` stands beside one other
  Galician word and as many Spanish ones, Galician writing `por` too (« O profesor Smith é recoñecido
  por ser un dos máis grandes eruditos da filoloxía inglesa. »: `máis` and `da` against `por`).
- **Dropping them only where the block holds a Spanish marker, or weighing them one half**, gives
  back 5: 791 of the 796 hold no Spanish marker at all, and one Galician word, or half of one, still
  outnumbers none.
- **Ignoring them right after a clitic** gives back 549 at no Galician cost (« Me da miedo… »,
  « ¿Te das cuenta…? », « Se le da bien… »), but not the 247 where the verb follows anything else —
  a noun, a name, `que` — or opens the line (« Da un paso atrás. », « El padre de Bob da clase en un
  colegio de niñas. », « …que da paso a la democracia… », « Da la impresión de que… »); and it puts a
  list of Spanish pronouns and a word-order rule into a guard that otherwise counts words. Against the rule: 247 Spanish
  blocks refused for 91 Galician blocks kept out. The rule is the default; open question 1 offers
  this one.

*Rejected — a detector with a Galician class*, as changes 19, 42 and 42b rejected a second detector.

### D2 — The guard reads words as today

The split, the lowercasing, Catalan's elisions and Occitan's lowercase rule do not change, nor do the
four tables. `da` and `das` are matched as today, whatever their case (« Da un paso atrás. » opens on
`Da`, « el edificio del DAS » is an acronym): Galician's comparison never had a casing rule, and the
measurement above counts as the guard reads.

### D3 — What it costs

One counter and one comparison more, the same lookups. Timed on the Spanish corpora's blocks (UD,
40,000 Tatoeba sentences, the Spanish Wikipedia articles, the books; Apple M2 Max, release build,
three runs, the first a warm-up): Spanish's guard (`block_is_studied` less whichlang) takes 27.3–30.6
µs per KB of Spanish text today and 27.9–31.0 with the rule, within the machine's run-to-run noise.
Not timed in WebAssembly, which runs the same loop. A reader who does not study Spanish pays nothing
(change 42's `detect`).

### D4 — Spanish's analyser version: bumped, `1.3.0` → `1.4.0`

The rule changes which Spanish blocks are analysed, which the analyser version exists to signal
(*An analyser version per studied language*); English's and French's do not move. The requirement
states the bump, not the number (changes 41's and 42b's lesson). The bump moves what a Spanish bump
moves (42b did the same at `1.3.0`): `SPANISH_ANALYZER_VERSION` and its doc line, the four Spanish
manifests, the two pins, the fixtures' recorded digests, the agent's fixture, one literal of
`tests/languages.rs`, and the goldens (D5). It moves es-fr's output, which the programme's rule holds
still: the owner approves it in this change's pull request (task 6.1).

### D5 — What moves, and what cannot

Applied to a scratch checkout of `main` (`eddaf712`):
- **The rule alone**, before the bump: `english_baseline`, `spanish_baseline`, `es_en_baseline`,
  `en_es_baseline`, `french_baseline`, `cross_native` and `languages` pass without re-blessing, and the
  51 tests of `analysis::language` pass unchanged. `pages-es.txt` writes neither `da` nor `das`; its
  `mixto` page's Galician block is read as Portuguese, as before.
- **With the bump**, re-blessed once (`LINGUA_BLESS=1`, `spanish_baseline`, `es_en_baseline`,
  `french_baseline`):
  - `es-fr.golden` moves on 18 lines, and only on the version: the `pack` line and the 17 analyses
    (`analyse new-reader` of its 13 pages, `analyse reader` of `noticias`, `homografos`, `nombres`
    and `mixto`) — every moved line, its version set back, is the line before;
  - `es-en.golden`, the same 18 lines;
  - `fr-en.golden`, its `beside es-en` line (`analyzer_version "1.4.0"`); no French analysis moves;
  - `en-fr.golden` and `en-es.golden` do not move: `english_baseline`, `en_es_baseline` and
    `cross_native` pass without re-blessing.
- **The packs**: `tables/es-fr/` and `tables/es-en/` re-reduced from their pinned sources; the reducers
  read `SPANISH_ANALYZER_VERSION` from `analysis/mod.rs`, so `manifest.json` (`analyzer_version`) and
  `pin.json` (the pack's sha256) alone move. Built from the committed tables at `1.4.0`: es-fr
  `d617856d…` and es-en `507abaae…`, 2,224,439 and 2,608,413 bytes as before, `pack_version` the same;
  `scripts/lingua-data/build.sh` then builds all five committed pairs to their pins.
- **The fixtures**: `testdata/es-fr` and `testdata/es-en` manifests; `crates/lingua-pack/tests/
  pipeline_testdata.rs` records the Spanish fixtures' bytes (`652ab141…` and `636e693e…`, 1,342 and
  1,356 bytes as before; en-fr's `d5c85ef9…` and en-es's `516afb7f…` unchanged);
  `apps/lingua-agent/rust/tests/fixtures/es-fr.lingua` is the es-fr fixture's build — without it,
  eight of the agent's tests fail, the core refusing a `1.3.0` pack.
- With all of it, lingua-core, lingua-pack, lingua-agent and lingua-wasm pass (726 tests), and the
  lingua-data Python suite passes (459 tests): nothing it reads moves.

The corpus is not touched: a Spanish line with *da* in `pages-es.txt` would move es-fr's output
beyond the version; the unit tests carry the sentences (D6), as 42b's carry Occitan.

Other changes moving the same files: any change that bumps Spanish's version or re-records the
Spanish pins — the hidden hyphens' (42b's open question 5), should it be proposed, or an es-fr
update (`migrate-lingua-pack-sources-to-raw-dumps`' 5.1) — meets a textual conflict on the same
golden lines and pins. Whichever merges second re-reduces or re-records the pins on top of the other
and re-blesses once; two Spanish bumps landing together may share one version, if the owner says so.

### D6 — Tests

In `language.rs`, beside Spanish's guard's tests: the spec's new scenarios, each sentence first
asserted to be read as Spanish by whichlang (a whichlang update that stops reading it so fails the
test rather than passing it for the wrong reason); the two shared words in `GALICIAN_MARKERS`;
« ¿Cuánto se da de propina en España? » and « ¿Te das cuenta de la hora que es? » kept, and
`iberian_neighbour` false on them; `da` counted beside another Galician word — « O profesor Smith é
recoñecido por ser un dos máis grandes eruditos da filoloxía inglesa. » refused, and the bare
`da xa por` refused where `xa por` ties; the leak « A esperanza é a razón da vida. » kept; the
rule's limit, « Me da igual: nunca jugué al polo. » (`da` beside `polo`) refused; the Catalan,
Galician and Occitan sentences of the existing tests refused as before. Every existing test of
`analysis::language` passes unchanged, the scenarios of *Catalan and Galician are not read as
Spanish* and *Occitan is not read as Spanish* among them.

All of this answered as stated in the scratch checkout (the scenario sentences through
`block_is_studied`, on `main` and with the rule).

### D7 — Gates and OpenSpec

1. en-fr and en-es do not move: `git diff --stat origin/main --
   crates/lingua-wasm/tests/baseline/{en-fr,en-es}.golden` is empty; `es-fr.golden`, `es-en.golden`
   and `fr-en.golden` move on the version alone (D5), and any other line is a review failure.
2. The workspace gates (fmt, clippy `-D warnings`, tests — `lingua-agent`'s among them —, `llvm-cov`
   ≥ 80 %, `wasm-pack test --node`), the extension's, and the lingua-data Python suite.

OpenSpec: one MODIFIED requirement in `lingua-analysis`, *Catalan and Galician are not read as
Spanish*. Change 42b, implemented but still open (its 6.3), MODIFIES it too; the programme's rule —
never MODIFY a requirement an open change holds — is met by archiving after it: this delta is 42b's
text with the rule added, so archived second it replaces 42b's requirement with 42b's own words plus
the rule. `.openspec.yaml` names `add-lingua-spanish-occitan-guard` in `archiveAfter`, and
`openspec_archive_order.py` exits 10 naming it. The requirement's name and its five scenarios' names
and words are kept; the first sentence gains the rule for `da` and `das`; « a tie, or a block with no
such function word » gains « counted »; « adding it SHALL bump Spanish's analyser version » becomes
« adding it, or changing what it counts, SHALL bump… » (D4); the stated leak gains the Galician line
whose only words are `da` and `das`. Four scenarios are added: Spanish's *gives*, `da` beside another
Galician word, Catalan and Occitan refused as before, and the new leak. *Occitan is not read as
Spanish* is 42b's and is not touched.

## Risks / Trade-offs

- [A Galician line whose only Galician words are `da` or `das`] → read as Spanish: 91 more Galician
  blocks of 8,933 (21.2 → 22.2 %), 56 of them short Tatoeba sentences. A reader of Spanish on a
  Galician page sees such a line highlighted as Spanish, as the 21.2 % already are. Accepted; open
  question 2 would more than offset it.
- [A Spanish line with *da* beside a Spanish word the Galician table holds] → still refused: « Me da
  igual: nunca jugué al polo. » (`da` and `polo`), as today. None among the corpora's Spanish blocks.
- [Portuguese] → 2 more of 11,070 blocks read as Spanish (100 instead of 98); whichlang reads only
  103 of them as Spanish to begin with.
- [Hidden hyphens] → in an e-book that hides a soft hyphen inside « vida », `da` alone no longer
  refuses the line, but `do` (« todo », « cuando ») still does, and beside it `da` counts again: such
  lines are refused less often, not never (42b's open question 5).
- [A whichlang update] → the tests check that each scenario sentence is still read as Spanish
  first, and a bump re-runs the measurement.
- [An installed agent] → the agent (outside the programme, M17) rebuilt at `1.4.0` skips an es-fr pack
  of `1.3.0` (« Pack ignoré ») until the new pack is copied beside it, as its README says.

## Migration Plan

Nothing to migrate: no stored format, wire field or table row moves; statuses, counts and backups do
not name the analyser version. The es-fr pack is rebuilt with the extension at its next release.
Rollback is a revert, the goldens, manifests, pins and fixtures with it.

## Open Questions

For the owner, none blocking. Each has a default, which the implementation follows unless the owner
answers otherwise.

1. **Which rule for *da* and *das*** (D1) — today a short Spanish line such as « ¿Cuánto se da de
   propina en España? » is set aside as Galician because *da* is also Galician's « of the ». Four
   ways to fix it, measured on 453,821 Spanish and 8,933 Galician sentences and paragraphs:
   - count *da* and *das* only when the line holds another Galician word such as `non`, `xa`,
     `unha` (the default): Spanish set aside 852 → 56, Galician taken for Spanish 21.2 → 22.2 %
     (« A esperanza é a razón da vida. » now passes as Spanish);
   - take *da* out of the Galician list: 152 still set aside (« ¿Te das cuenta…? »), Galician
     22.1 %;
   - take both out: 56 set aside, as the default, but Galician 22.4 % (« O profesor Smith é
     recoñecido por ser un dos máis grandes eruditos da filoloxía inglesa. » passes too);
   - ignore them only right after `me`, `te`, `se`, `le`… (« No me da tiempo. »): 303 still set
     aside (« Da un paso atrás. », « El padre de Bob da clase… »), Galician unchanged at 21.2 %.
2. **Add Galician's `é` (« is »)** (D1) — Spanish no longer writes the word *é*, Galician writes it
   in most sentences. Added to the Galician list with the default rule: Galician taken for Spanish
   21.2 → 19.7 % instead of 22.2 % (better than today), and Spanish set aside 56 → 58 — a list of
   Catalan accents (« Acentos: à, è, é, í, ò, ó, ú. ») and a line of Gascon verb forms in the article
   on Occitan. The risk: 19th-century Spanish wrote the conjunction « é » (« una manera tan estrecha é
   intima », *La vuelta de Martín Fierro*'s preface; *Facundo*); the four such blocks measured are
   kept by their other Spanish words, but a short line of old Spanish with « é » and none would be
   set aside. Add it here (one word, one test, the measurement re-run; es-fr's output still moves
   on the version alone, the `mixto` page's Galician line being read as Portuguese), or leave it for
   later (the default: this change is the owner's *da*/*das* fix, and nothing more)?
