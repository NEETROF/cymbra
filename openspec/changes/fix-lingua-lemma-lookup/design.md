# Design — fix-lingua-lemma-lookup

## Context

See proposal.md (Why). What exists, on `main` at `e3d6eb4a` (changes 41 and 43 merged, #832 and #829;
the files below are as they were at `da94a82e`, line for line):

| Where | What |
|---|---|
| `lingua-core/src/analysis/lexicon.rs` | `FstLexicon`: an `fst::Map` form → lemma id, the lemma pool (a `Vec<String>` the ids index) and a `HashSet` of its lemmas. `id_of(form)` (L116) is the form lookup; its comment says « a lemma resolves to its own id », true only while the forms read every lemma as itself. `lemma_of` (the `Lexicon` trait) is `id_of` then the pool; `contains_lemma` is the set. `from_slices` checks every id against the pool's length, and nothing of the pool's order |
| `build_lexicon_blobs` (L148) | Pool = the pairs' lemmas and the extra lemmas, sorted byte-wise and deduplicated. Entries = every pair `(form, id(lemma))` and every pool lemma's identity `(lemma, id(lemma))`, sorted as `(form, id)` and deduplicated by form (L176): **the byte-wise smaller lemma id wins**, whichever entry it comes from — while its comment says a lemma resolves as itself « unless the pairs already map that surface elsewhere » |
| `lingua-core/src/packs/pack.rs` | Every lemma-keyed read goes through `id_of(lemma)`: `gloss` (L289), `paradigm` (L321, under `readings` L339 and `other_readings` L355), `sense_runs` (L373), `is_dictionary_word` (L461), `rank` (L479), `level` (L489). The enumerations (`lemmas_in_rank_band`, `lemmas_at_level`, `dictionary_words`) walk ids and are right |
| `lingua-pack/src/lib.rs`, `lexical.rs` | `build_pack` keys ranks (L510), glosses (L523) and levels (L539) by `lex.id_of(lemma)`; `grammar_sections`'s `lemma_id` (L701) checks `contains_lemma`, then `id_of` — so a pool lemma whose spelling reads as another files its readings and runs there; `lexical_table` places a dictionary word at `id_of(word)` (lexical.rs L50), « the id the reader looks it up by » |
| The callers | `engine::analyse_page` glosses a Learning or Unknown token by `pack.gloss(token.lemma)` (engine.rs L141); `gloss_phrase` its tokens and parts (L420, L427); `word_grammar` (L505) the card's gloss, runs, readings and other dictionary forms; lingua-wasm's `gloss` (L837) and `readable_gloss` (review, L108: the pack's gloss, else the card's stored text); the agent's `vocab.rs` (L86). The extension asks `gloss` with a card's lemma when it creates the card (`selection-card.ts` `cardGloss`), asks the translator for a word card that has no gloss (`wordEngine`), and words the card's frequency line from `frequencyRank` (lingua-wasm L888, `rank`). Every other caller of `id_of` is a test's fixture builder (`engine.rs`'s and `pack.rs`'s tests, `tests/french_fixtures.rs`); graphify's call graph does not resolve these method calls (`affected "id_of"`: none), so this census is a search of the sources |
| The cascades | English (`lemmatize.rs`): irregulars, the forms, rule candidates checked by `contains_lemma`, then `out_of_lexicon_plural` — a singular returned **without looking it up**. Spanish (`spanish.rs`): the forms, the accentless retry, enclitics, then its `out_of_lexicon_plural`, likewise. French (`french.rs`, change 41): its unlisted plural only when the singular is unknown too (`!lexicon.contains(&singular)`), so its lemma is a lemma of the pack or no string the forms hold |
| The tables | Every forms table writes each lemma's identity row: en 75,315 forms for 40,685 lemmas, es 144,952 for 60,000, fr (change 43, #829) 124,096 for 60,000 |
| The reviews | Change 45's design D12 and open question 4 (the card's side, measured on the released packs: 2,002 Spanish plurals, 433 read masculine only, 12,129 English forms; `porte porte` read off *porter*'s paradigm once change 48 switches the French golden to the tables). Change 46's design D3 (the builder's side: on change 43's prototype tables, *venir* and *donner* took the levels and ranks of `venue` and `donnée`; its check *A level reaches the lemma it is written for*). Change 48's design (a ranked lemma whose own form reads as another « would lend its gloss to that word »; its task 3.1 checks none is glossed). Change 43 (merged) keeps a ranked lemma only when its own form reads as itself (its requirement: « Every ranked lemma SHALL be the lemma of its own form »), and `committed_tables.rs`'s `every_rank_lands_on_its_own_lemma_in_the_built_pack` checks each committed pack holds every rank on its own lemma, through `lemma_of` |
| The goldens | `crates/lingua-wasm/tests/baseline/`: `en-fr.golden` (`english_baseline.rs`), `es-fr.golden` (`spanish_baseline.rs`), `es-en.golden` (change 23), `en-es.golden` (change 24), `fr-en.golden` (change 39, over its hand-written fixture until change 48); `apps/lingua-extension/test/baseline/word-card-en-es.txt` and `word-card-es-en.txt` render their `word-grammar` probes |

## Goals / Non-Goals

**Goals:**
- A card, a token's gloss and a caller of the engine read a dictionary form's own entry, or nothing —
  never another word's, in every language and pair.
- The builder files what it files under the lemma it is written for, and refuses a lexicon whose
  forms could make a lemma's entry unreachable or ambiguous.
- Measured: what moves (every golden and snapshot line), what cannot (every pack byte), and what a
  reader loses or gains.

**Non-Goals:**
- Any cascade: the English and Spanish plural fallbacks keep returning the singular they return
  (open question 2); no token's lemma changes, no analyser version moves.
- The estimates: rank, level and dictionary-word mark keep reading through the spelling (D3).
- Any table: none is re-reduced, none moves; French's tables stay change 43's, 45's and 46's.
- Lemma alternatives (M8, optional outside the counts): a later change, which D1 does not block.
- The memory of `FstLexicon`'s lemma set (D1 could replace it; not this change).

## Measured

On a prototype over `main` (scratch, never committed): D1–D5 applied to `lexicon.rs`, `pack.rs`,
`lib.rs` and `lexical.rs`; the five goldens blessed, the extension's whole suite run on the prototype's
WASM build, and a probe over each released pack's lexicon. Repeated on change 41's implementation
before it merged, then on `main` at `24bcbcd5` (change 43 merged) and on the implementation branches
of changes 42 and 44: the same 31 lines moved, and every Rust test passed.

**The tables meet D4 already.** For each pair, rebuilt from its committed tables as the builder
reads them (pool = the forms' lemmas, the ranked and the glossed):

| Tables | Forms | Lemmas | A form with two lemmas | A lemma whose spelling reads as another today | … that the forms map elsewhere (identity winning) | Rows filed under another lemma |
|---|---|---|---|---|---|---|
| en-fr, en-es (`tables/en/`) | 75,315 | 40,685 | 0 | 0 | 0 | 0 of the ranks, glosses, 8,302 levels, dictionary words, readings (20,205 lemmas), runs |
| es-fr, es-en (`tables/es/`) | 144,952 | 60,000 | 0 | 0 | 0 | 0 (27,233 lemmas with readings) |
| fr-en (`tables/fr/`, change 43, on `main` since #829) | 124,096 | 60,000 | 0 | 0 | 0 | 0 |
| fr-en, change 43's earlier prototype with change 46's 70 fixture glosses (its measurement) | 124,040 | 60,002 | 0 | 5: `bordée`, `donnée`, `retombée`, `tranchée`, `venue` | 2: `porte`, `saisie` | the 5 lemmas' ranks, and without change 46's rule 4 three levels (`donnée`, `tranchée`, `venue`) |

The prototype builder builds the five committed pairs **to the sha256 their pins record**, fr-en's
(1,241,733 B) included, and the five `testdata/` pairs byte for byte as today's builder does (en-fr's
is the extension's and lingua-wasm's fixture, es-fr's the agent's, fr-en's change 41's French
fixture, 5,439 B); it refuses change 43's earlier prototype, naming `bordée` (read as
*border*) first, and a fixture listing `porte` as `porte` and as `porter`. On a fixture where a
level table gives `donner` A1 then `donnée` B1 and the forms map `donnée` to *donner*, today's builder
makes *donner* B1, the prototype A1.

**What a string that is no lemma reads today.** Every form of a pack's lexicon that is no lemma
(en 34,630; es 84,952; fr 64,096) reads another word's entry when asked as a lemma. The
analysis asks so through the plural fallbacks: the probe enumerated every word whose cascade lemma is
such a form — its `-s`, `-es`, `-ies`, `-ces` and `-iones` plurals, read by `lemmatize` itself (a
lower bound: a plural whose fallback strips an accent is not generated) — and the card the engine
answers today:

| | English (en-fr / en-es) | Spanish (es-fr / es-en) |
|---|---|---|
| Words | 15,215 | 32,372 |
| … with a frequency in wordfreq (Zipf ≥ 1), unhyphenated | 625 (84 at Zipf ≥ 3) | 1,005 (15 at Zipf ≥ 3) |
| Another word's readings | 12,129: 3,173 a verb's only, 1,079 a noun's only, 7,877 both — the `-s` form's reading of the lemma, put on another word | 2,103: 1,128 agree with the word's ending by chance (`abuelitas` feminine plural, off *abuela*), 434 `-as` plurals read masculine only (`ablativas`), 13 `-os` read feminine only, 306 a verb's reading (300 on no word wordfreq lists), 222 other |
| Another word's gloss | 14,231 / 13,886 (7,478 / 7,397 of a top-10,000 lemma) | 29,830 / 32,369 |
| Another word's senses, grouped by its parts of speech | every word with a gloss | every word with a gloss |

French has none: its plural rule never returns a string the pack holds. The reviewers' counts (2,002,
433) required the plural edit itself among the lemma's readings; this probe counts every borrowed
reading.

The commonest real words, and what their card reads today (en-fr, es-fr):

| Word | Read as | Through | Today's readings | Today's gloss opens on | Right? |
|---|---|---|---|---|---|
| `thoughts` (Zipf 4.75) | `thought` | *think* | present, 3rd sg | « Penser, croire » | no |
| `buildings` (4.69) | `building` | *build* | plural noun; present, 3rd sg | « Construire, édifier » | no |
| `shots` (4.63) | `shot` | *shoot* | both | « Pousse » | no |
| `settings` (4.23) | `setting` | *set* | both | « Prêt » | no |
| `wounds` (4.07) | `wound` | *wind* | both | « Vent » | no |
| `rankings` (3.97) | `ranking` | *rank* | both | « Virulent » | no |
| `paintings`, `recordings`, `readings`, `listings` | | *paint*, *record*, *read*, *list* | both | « Peinture », « Enregistrement », « Lecture », « Liste » | gloss near, readings no |
| `videos` (4.78) | `video` | *vídeo* | masc. plural | « Vidéo » | yes |
| `peores` (4.44) | `peor` | *mal* | masc. plural noun | « Mal, douleur… » | no |
| `cuentos` (4.35), `tomos`, `trazos` | `cuento`, `tomo`, `trazo` | *contar*, *tomar*, *trazar* | none | « Compter », « Prendre », « Tracer » | no |
| `galletitas`, `añitos`, `besitos`, `ojitos` | the diminutive | *galleta*, *año*, *beso*, *ojo* | fem./masc. plural | « Biscuit », « Année », « Baiser », « Œil » | near |
| `ablativas` | `ablativa` | *ablativo* | ADJ masc. plural | *ablativo*'s | readings no |

**What moves, by the lookups changed** (lines of each golden; a line is one probe's answer):

| Lookups by the dictionary form itself | en-fr | en-es | es-fr | es-en | fr-en |
|---|---|---|---|---|---|
| readings, other readings, runs (gloss through the spelling) | 4 | 4 | 0 | 0 | 0 |
| **+ gloss (this change)** | **10** | **10** | **3** | **3** | **5** |
| + rank, level, dictionary-word mark | 12 | 12 | 3 | 3 | 5 |

- The first row is change 45's « four probes each »: `more more`, `thought thought`, `lay lay`,
  `saw saw` lose their readings, and their gloss — still *many*'s, *think*'s, *lie*'s, *see*'s —
  falls into one group with no part of speech.
- This change's 10: those four with no gloss either; `gloss more`, `gloss are`, `gloss has` read
  « (none) »; the new reader's `news`, `academic` and `register` pages give `lowers`, `findings`
  and `strangers` no gloss (*low*'s « Bas; Dépression… », *find*'s « Trouver… », *strange*'s
  « Ignoré, inconnu… » before). es-fr's and es-en's 3: `gloss cuenta`, `gloss llama` and the
  `quijote` page's `quebrantos` (*quebrantar*'s « Casser, briser… »). fr-en's 5: `gloss vis`, `as`,
  `été`, `est` and `word-grammar été été`. No token's lemma, class, count or percentage moves; no
  phrase gloss, level, ladder, estimate, review, export or backup moves.
- The third row adds the reader's `news` and `register` pages: `lowers` and `strangers` go from
  Known to Unknown (known 84 → 83, 42 → 41; 85 → 84 %, 86 → 84 %) — D3 keeps them.
- `word-card-en-es.txt`: the four probes, 21 lines out and 4 in (`gloss: (none)`), among them
  « también puede ser el comparativo de more » and « … el pasado simple de saw ».
  `word-card-es-en.txt`, `selection-rows-fr.txt` and `voice-ranking.txt` unchanged; the extension's
  whole suite on the prototype's WASM: 122 of 123 files pass, the 123rd that snapshot.
- Every Rust test of lingua-core, lingua-pack, lingua-wasm and lingua-agent passes with the goldens
  re-blessed (37 test binaries), `cross_native.rs` and the studied-side comparisons of changes 23
  and 24 included, and so does the WASM lane (`wasm-pack test --node crates/lingua-wasm`).

## Decisions

### D1 — A lemma is found among the lemmas

`FstLexicon::lemma_id(lemma)` finds the lemma in the pool by binary search: its id is its place in
the pool, whatever the forms say of its spelling. The builder writes the pool sorted byte-wise with no
repeat (`build_lexicon_blobs` has since the first pack); `from_slices` now checks it once, at load, and
refuses a pool out of order or with a repeated lemma, as it refuses a form pointing outside the pool
(`LexiconError`, `PackError::Lexicon`: no partial analysis). `id_of` stays the form lookup, its
comment corrected. A lookup costs about sixteen string comparisons over 60,000 lemmas, the check one
pass over the pool when a pack loads; no memory is added.

*Rejected — `id_of` then `lemma_at(id) == lemma`.* Exact on every pack D4 lets the builder write, and
no memory; but the lookup would depend on the forms reading every lemma as itself, and a lemma
alternative (M8's optional change) is precisely a lemma whose spelling reads as another: it would be
unreachable. *Rejected — a map lemma → id.* Exact, but a second copy of every lemma in each loaded
pack (the set already holds one), for what a sorted pool answers.

### D2 — A card describes the dictionary form it names

`Pack::readings`, `other_readings` (through `paradigm`), `sense_runs` and **`gloss`** read by
`lemma_id`; a string the pool does not hold as a lemma reads nothing. The readings are the bug
proper: they apply another lemma's inflection edits to this string, so `ablativas` reads masculine
and `buildings` a verb. The runs group a gloss by its word's parts of speech, so they follow the
gloss.

**The gloss is decided: its own, or none.** Measured both ways (table above). Through the spelling,
it is right for a share of Spanish diminutives and variants (`videos` « Vidéo », `ojitos` « Œil »)
and wrong for English nouns read through a verb (`buildings` « Construire », `settings` « Prêt »,
`wounds` « Vent »), Spanish nouns read through a verb (`cuentos` « Compter ») and every
probe that asks a homograph (`saw` « Voir »). Kept, it would sit under a headword it does not
describe, without the parts of speech its senses had, beside no reading: the dictionary's gloss of
one word shown as another's, which no table wrote and no reviewer of a table sees. A word card with no gloss is
translated in its sentence where the reader has translation (`wordEngine`), as any word the pack does
not gloss already is; a review card keeps the gloss it stored (`readable_gloss`). The owner settled it
so on 2026-10-10, for every language (open question 1).

### D3 — An estimate keeps reading through the spelling

`rank`, `level` and `is_dictionary_word` keep `id_of`: for a lemma of the pack it is the lemma's own
id (D4), and for a form it is the lemma the forms read it as. They estimate whether the reader knows a
word and whether a capitalised word is a name; a reader who knows *strange* is estimated to know
`strangers` as before, and no Spanish name is set aside anew. Their comments say so. Reading them by
`lemma_id` too would move the reader's pages in en-fr and en-es (the third row above) — a change of
what counts as known, for the owner and a change of its own if wanted.

The word card's frequency line reads `rank` too (`frequencyRank`, add-lingua-card-frequency): a card
D2 leaves without a gloss or a reading still says how common the word its spelling reads as is —
`building` *build*'s, `saw` *see*'s. That is the estimate the token's class is computed from, so the
line keeps agreeing with the colour the page gave the word; it names no other word and shows none of
its dictionary text.

### D4 — The builder files by the lemma's own place and refuses an ambiguous lexicon

`build_pack` refuses, naming them: a form the forms table lists with two lemmas (today the byte-wise
first wins in silence; M8 keeps « one form, one lemma » in every language, so no table may write
one); and a lemma of the pool — of the forms table, the ranks or the glosses — whose own spelling
the forms table reads as another (`venue` → *venir*). Then ranks, glosses, levels, the lexical
table, readings (their key, the « also » entries' key and target) and sense runs are keyed by
`lex.lemma_id`. A level, a dictionary word, a reading or a run naming a string that is no lemma is
filed nowhere: the lexical table already refuses such a word when it writes one; a reading or a run
is dropped as `grammar_sections` drops one whose lemma the lexicon lacks; a level is dropped, and
change 46's check names it. After the two refusals, `lemma_id` and `id_of` agree on every lemma of a
pack that builds, so keying by `lemma_id` moves no byte; it makes the builder's place the reader's
by construction rather than by the refusal. Change 43's committed-tables check
(`every_rank_lands_on_its_own_lemma_in_the_built_pack`) stays, its comment no longer describing the
builder as keying a rank through `id_of`; this rule makes its condition a build refusal for every
pair.

*Rejected — refuse only a ranked lemma* (the reviewers' wording). A glossed lemma or a forms-table
lemma whose spelling reads as another is as unreachable, and every committed table passes the wider
rule (0 of each).

### D5 — `build_lexicon_blobs` honours its comment

An entry carries whether it is a lemma's identity; sorted after the pairs' for the same form, so a
form the pairs map reads as the pairs' lemma, as the comment says. Between two pairs for one form —
which `build_pack` now refuses, but which a test fixture may write — the byte-wise first lemma, as
today, documented. No key moves on any committed table or on change 43's (identity rows always agree
with the forms there); on change 43's earlier prototype, `porte` and `saisie` would have read as
*porter* and *saisir*, as its forms table says, instead of as themselves — and D4 refuses those
tables anyway. It is also what lets D4's check, made on the lexicon the builder writes, see such a
lemma whatever the byte order: today `porte` sorts before `porter`, its identity wins, and it would
read as itself unseen.

### D6 — No version moves, no pack byte moves

Every token's lemma, class, span and count, and every percentage, stay (measured); only a token's
gloss moves — what the pack answers for its lemma, as a dictionary update moves it — so no analyser
version is bumped: a version gates which packs a core loads, by exact match, and bumping it
would refuse every shipped pack for no change in how a page is read. No pack byte moves, so no
`pack_version` and no pin. The goldens move as a dictionary update moves them — through what the pack
answers — and are re-blessed in the pull request, every line listed with its before and after; en-fr's
and es-fr's with the owner's approval, the programme's rule. Change 39's *Nothing of French changed*
is read with this change counted among the rules French runs: it moves five of its probes and says
why, as its requirement asks of a pull request that moves the golden. That requirement's list of
reasons (a French rule that bumps French's analyser version, the fixture replaced, the beside pack's
update) does not name a fix of how the core reads every pack: a departure, stated, since changes 39,
40 and 41 hold the requirement and this change cannot modify it.

### D7 — Order, and what the next changes read

After change 41 (merged; the prototype moved the same five French lines on its implementation as on
`main`) and change 43 (merged, #829: its tables pass D4, its pack keeps its pin). Before the
implementations of 45, 46 and 48 and before 52; in either order with 42 and 44, measured on their
implementation branches (the prototype applied, the five goldens blessed, every Rust test run):
- **Changes 42 and 44** move other lines of `fr-en.golden` (42: French's analyser version and the
  pages at `1.1.0`; 44: the fixture pack's header and phrase-gloss probes), never the five this
  change moves, and this change moves the same 31 lines on either branch. Whichever lands second
  re-blesses `fr-en.golden` and lists its own lines only. 44's `french_expression_key` reads
  `contains_lemma`, the lemma set, which this change leaves as it is; neither adds a caller of
  `id_of`.
- **Change 45** (its open question 4) hands this change the lookup; its readings are filed under the
  lemma's own place, and change 48's switch of the French golden to the tables shows `été été` and
  `porte porte` with nothing instead of *être*'s and *porter*'s paradigms read against them (on
  `tables/fr/`, `été` is a form of *être* and `porte` of *porter*, neither a lemma). Whether 48 keeps
  those probes is 48's.
- **Change 46**: its check *A level reaches the lemma it is written for* stays, as the committed-tables
  test. Its scenario *A level written for a lemma whose form reads as another* expects the check to
  fail « naming the pair and `donner`, which the pack gives the level written for `donnée` »; after
  this change the builder files nothing under `donner` for `donnée`, `donner` keeps its A1, and the
  check fails naming the pair and `donnée`, whose B1 no lemma of the pack carries — and if `donnée`
  is a lemma of the pack (ranked, glossed or another form's lemma), the build fails before, naming
  `donnée` and `donner` (D4). 46's
  implementation words that scenario so, and its D3's account of `id_of` as the builder's keying
  becomes history; its rule 4 stays as the reducer's guard.
- **Change 48**: its glosses are keyed by their own lemma; its task 3.1 (no glossed lemma reads as
  another) is what D4 now refuses at build time.
- **Changes 34 and 35** ship es-en and en-es with this fix if it lands first; nothing of theirs moves.

## Risks / Trade-offs

- **[A Spanish diminutive or variant loses an approximate gloss]** (`videos`, `galletitas`,
  `añitos`; 1,005 Spanish words with a frequency) → translated in its sentence where translation is
  on; the owner chose this over the spelling's gloss on 2026-10-10 (open question 1).
- **[An English plural loses a gloss that was mostly wrong]** (`buildings`, `settings`, `wounds`)
  → the same; without translation, the card shows no gloss rather than a wrong one.
- **[A pack whose pool is out of order is refused]** → `build_lexicon_blobs` has sorted and
  deduplicated the pool, unchanged, since lingua-core's first commit (`c3717545`), and every pack
  builder has written the pool through it (`lingua-pack` since `26886e0b`), so a pack a reader or the
  agent holds loads; every committed fixture pack of the history (five blobs) and the five committed
  pairs pass the check (all Rust and extension tests pass).
- **[A future table writes a lemma whose spelling reads as another]** → the build fails, named, in
  the pull request that writes it; a reducer that wants such a lemma (lemma alternatives) needs a
  change of its own, which D1 makes possible.
- **[Cards created before]** keep the gloss they stored; review shows it when the pack has none.
- **[The programme's invariance rule]** → en-fr's 10 and es-fr's 3 lines are a deliberate move,
  approved by the owner on 2026-10-10 (task 4.1); the pull request lists them.

## Migration Plan

Nothing stored changes: no pack, no table, no backup or sync format. The fix reaches readers with
the next extension release (Chromium, Firefox, Safari's host app) and the agent plugin's — the
owner's. Rollback is a revert, the goldens and the snapshot reverting with it.

## Effort

1.25–2 ideal days: the core's lookup and its load check 0.25–0.5, the builder's refusals and keying
0.25–0.5, the tests 0.5–0.75, the re-bless and the pull request's line-by-line list 0.25.

## Open Questions

For the owner:
1. **The gloss of a word the pack holds only as a form** (D2). **Settled by the owner on 2026-10-10:
   none, as proposed** (the card offers the sentence's translation). The alternative was the gloss of
   the word its spelling reads as, for Spanish alone (diminutives and variants right, verbs' nouns
   wrong), shown as that word's — a card naming the other word, which would be change 51's.
2. **The English and Spanish plural fallbacks** return a singular the pack holds only as a form
   (`ojitos` → `ojito`, `videos` → `video`, `buildings` → `building`). Read it through that form's
   lemma (*ojo*, *vídeo*, *build* — the same token then counts as known with its base), or not at
   all (French's rule: `buildings` stays `buildings`)? Either moves those languages' analysis and
   bumps their analyser versions: a change of its own per language, not this one. **Left open by the
   owner on 2026-10-10, outside this change**, which it does not block.
