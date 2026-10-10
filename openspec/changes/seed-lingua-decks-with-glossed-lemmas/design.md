# Design — seed-lingua-decks-with-glossed-lemmas

## Context

See proposal.md (Why). What exists, on `main` at `46450468` (change 49's fr-es tables committed,
#862; change 51 merged, #866, with `fr_es_baseline.rs` and `fr-es.golden`; change 48b proposed, #861,
its implementation on a branch; change 49b proposed, #865):

| Where | What |
|---|---|
| `lingua-core/src/packs/pack.rs` | `lemmas_at_level(level)` (L452): every lemma whose level byte is `level`, in lemma-id order, each with `self.glosses.get(&id)` — `None` when the pack carries no gloss for it. The ladder (`levelLadder`) and the seeding both read it. `dictionary_words` (L484): the ranked lemmas that are dictionary words **or carry a level**, which the vocabulary estimate and the typical vocabularies count |
| `lingua-core/src/decks/review.rs` | `Deck::seed_lemmas(lang, lemmas, gloss_language, knowledge, cap, at)` (L75): for each `(lemma, Option<gloss>)` in the caller's order, stops at `cap` new cards, **skips (and refills)** a lemma that already has a card or an explicit status, else upserts `Card::seeded(lemma, gloss, …)` — with `gloss` `None` when the pack has none. Writes no status. Its test `seed_lemmas_caps_and_skips_tracked_lemmas` (L429) seeds `quixotic` with no gloss |
| `lingua-core/src/decks/card.rs` | `Card::seeded(lemma, gloss: Option<String>, gloss_language, at)` (L182): source `Import`, no sentence, the form the lemma |
| `lingua-wasm/src/lib.rs` | `seedLevel(level, count, order, at, language)` (L983): `lemmas_at_level`, sorted by rank (`common`) or by rank descending (`rare`), unranked last, then `seed_lemmas` with the engine's native language as the label. Returns the number added; 0 for an unknown level or a pack without levels. Review shows a card the gloss of the pack held for its language, else the card's own text (`readable_gloss`, L119): a seeded card without a gloss shows nothing |
| `apps/lingua-extension/src/stats/view.ts` | « Renforcer un niveau » (L296–317): a level select preset to the declared level, a count (default 20, `SEED_CAP` 50), commonest or rarest first; it reports `cardsAdded` (« N cartes ajoutées au deck (niveau B1). ») or, when the seeding adds nothing, `noCardsAdded` (L315) into `#seed-result`, a `div.note.seed-result` (12 px, wrapping; `stats.css` L230, L289). The copy is `src/i18n/{fr,en,es}/stats.ts` (`noCardsAdded`: « Aucune carte ajoutée — ces mots sont déjà suivis ou dans ton deck. », "No cards added — these words are already tracked or in your deck.", « No se ha añadido ninguna tarjeta: estas palabras ya están seguidas o en tu mazo. »). The view lives in Chrome's side panel and in the in-page drawer (`styles/drawer.css`: 380 px wide, 14 px padding, at most 92vw). The extension's tests drive it through a fake port: `test/stats-view.spec.ts` (L255, `toContain("Aucune carte ajoutée")`) and `test/review-stats-copy.spec.ts` (L411, `esStats.noCardsAdded`); no snapshot holds the message |
| The levels | One `level.tsv` per studied language (M24), 8,302 levelled lemmas each, shared by its pairs. English's from CEFR lists (not estimated), Spanish's (es-fr's dictionary words) and French's (change 46, estimated, M7) from frequency. Change 48b gives a French level only to a lemma fr-en glosses (its D3); nothing ties a level to the glosses of a second pair |
| The invariance tests | `tests/support/mod.rs` renders one scenario per pair; its reader seeds `A2 5 common` then `C1 3 rare` (L390–399) and then reviews (the first card graded, the second marked known) — 19 probes follow, from `start-review` to `backup`. `cross_native.rs` (`assert_probes_alike`, L130) answers each language's scenario through two packs of that language and compares **every probe** with glosses, senses and expressions stripped. `en_es_baseline.rs`, `es_en_baseline.rs` and change 51's `fr_es_baseline.rs` compare their committed golden with en-fr's, es-fr's and fr-en's the same way (`the_golden_is_the_*_one_on_the_studied_side`). The extension's `word-card-*.txt` snapshots render the goldens' `word-grammar` probes, the French ones their `phrase-gloss` probes too |
| The requirements | `lingua-decks-review` *Level-targeted deck seeding* and `lingua-browser-extension` *Level-targeted deck feeding control* (no open change holds either). `lingua-analysis` *An analysis does not depend on the native language*, held by change 49 (merged, not archived), whose test compares « every probe » |

## Goals / Non-Goals

**Goals:**
- No card seeded from a level arrives without a gloss, in any pair.
- A seeding asked for N cards still gives N while the level can.
- Measured for every committed pair: what a level holds without a gloss, what a seeding draws today,
  what refilling and not refilling give, what moves.

**Non-Goals:**
- Any table, level or gloss. French's levels stay French's (change 46, 48b's refinement); no pair's
  glosses are touched (48b, 49b do that for theirs).
- The ladder, the vocabulary estimate, the presumed-known rule, the declared level and its sync: a
  lemma without a gloss is a word of the language all the same.
- Cards a reader added from a page (`addCard`), with or without a gloss, and the agent plugin's
  `add_words` (the words it is given; M17).
- Cards already seeded without a gloss: left as they are (the owner, Q2).
- Any interface text but the one message D7 rewords.

## Measured

On a prototype over `main` (scratch, never committed; first at `101b684d`, then again at `46450468`
with change 51's golden): `seed_lemmas` skipping a candidate without a
gloss; every committed pack built from its tables; the goldens blessed; every Rust test of
`lingua-core`, `lingua-pack` and `lingua-wasm` run. The same probe over the tables of 48b's
implementation branch (`0c6fb7fa`) for fr-en and fr-es. The pack's answers and a count over the TSV
tables (`level.tsv` against `gloss.tsv`) agree for every pair.

**Levelled lemmas the pack does not gloss**, of the 8,302 of each studied language:

| Level (lemmas) | en-fr | en-es | es-fr | es-en | fr-en | fr-es | fr-es after 48b |
|---|---|---|---|---|---|---|---|
| A1 (1,020) | 12 (1.2 %) | 6 (0.6 %) | 0 | 8 (0.8 %) | 5 (0.5 %) | 22 (2.2 %) | 18 (1.8 %) |
| A2 (1,158) | 19 (1.6 %) | 16 (1.4 %) | 0 | 4 (0.3 %) | 4 (0.3 %) | 67 (5.8 %) | 65 (5.6 %) |
| B1 (2,015) | 53 (2.6 %) | 60 (3.0 %) | 0 | 6 (0.3 %) | 12 (0.6 %) | 241 (12.0 %) | 230 (11.4 %) |
| B2 (2,347) | 150 (6.4 %) | 160 (6.8 %) | 0 | 12 (0.5 %) | 14 (0.6 %) | 428 (18.2 %) | 418 (17.8 %) |
| C1 (886) | 134 (15.1 %) | 112 (12.6 %) | 0 | 13 (1.5 %) | 3 (0.3 %) | 203 (22.9 %) | 199 (22.5 %) |
| C2 (876) | 205 (23.4 %) | 184 (21.0 %) | 0 | 8 (0.9 %) | 2 (0.2 %) | 227 (25.9 %) | 222 (25.3 %) |
| **All** | **573 (6.9 %)** | **538 (6.5 %)** | **0** | **51 (0.6 %)** | **40 (0.5 %)** | **1,188 (14.3 %)** | **1,152 (13.9 %)** |

fr-en after 48b: 0 — its gloss rules gloss 4 of the 40 (`burger`, `dev`, `ès`, `french`), its D3
takes the level from the 36 others and gives each slot to the next glossed lemma. Change 49b, after its own rules, counts 1,198 for fr-es (it loses glosses, gains none).
Every levelled lemma is ranked, so no « unranked last » case arises.

What they are: en-fr's and en-es's are hyphenated compounds and British spellings the dictionaries
file elsewhere (`take-off`, `hard-working`, `organisation`, `realise`), a few function words and
abbreviations (`an`, `mr`, `mrs`, `pm`) and rare derived adverbs (`unscathing`, `unenviably`); es-en's
letters and names es-fr glosses (`d`, `i`, `psoe`, `jack`, `lincoln`); fr-en's word pieces, abbreviations
and loans (`parce`, `quant`, `to`, `for`, `stp`); fr-es's everyday words the Spanish edition leaves out (`part`,
`lors`, `afin`, `taux`, `retrouver`, `forcément`, `réellement`, `classement`) and many loans and names
(`twitter`, `tv`, `bill`, `kevin`).

**What a seeding draws today**: cards without a gloss in a seeding of 20 / of 50 from a fresh deck
(« 0 » when both are 0), and the place of the first one in commonest-first order:

| Level | en-fr common · rare · first | en-es common · rare · first | es-en common · rare · first | fr-en common · rare · first | fr-es common · rare · first |
|---|---|---|---|---|---|
| A1 | 0/1 · 2/2 · 27th (`an`) | 0/0 · 2/3 · 671st | 0 · 0 · 164th (`d`) | 0 · 0 · 102nd (`parce`) | 0/0 · 0/1 · 102nd (`parce`) |
| A2 | 0/0 · 8/8 · 88th | 0/1 · 6/8 · 38th | 0 · 0 · 739th | 0 · 0 · 345th | 0/1 · 2/4 · 26th (`twitter`) |
| B1 | 4/5 · 4/12 · 1st (`take-off`) | 3/5 · 5/9 · 1st | 0 · 0 · 71st | 0 · 0 · 133rd | 4/8 · 1/5 · 7th |
| B2 | 2/5 · 16/29 · 3rd | 1/4 · 17/33 · 3rd | 0 · 0/1 · 153rd | 0/1 · 0 · 34th | 1/5 · 1/9 · 6th |
| C1 | 1/1 · **20/43** · 2nd | 3/3 · 18/38 · 1st | 0/1 · 0 · 33rd | 0 · 0 · 150th | 5/9 · 2/10 · 1st |
| C2 | 2/3 · 16/44 · 1st | 2/4 · 17/40 · 1st | 0 · 0/1 · 79th | 0 · 0 · 145th | 2/9 · 10/22 · 16th |

es-fr draws none. After 48b, fr-es: A1 0/0 · 0/1, A2 0/2 · 3/6, B1 3/8 · 1/3, B2 1/7 · 3/9, C1 3/5 ·
6/9, C2 3/6 · 2/7.

**The two packs of a language leave out different lemmas** — what makes the seeded deck native side
(D4): English, 287 lemmas neither en-fr nor en-es glosses, 286 only en-fr leaves out, 251 only en-es
(537 levelled lemmas one pack seeds and the other not); Spanish, the 51 of es-en; French, 35 neither
glosses, 5 only fr-en leaves out, 1,153 only fr-es (1,158).

**Refill or a shorter deck** (D2). Seeding 20 at a time from a fresh deck, until a seeding adds
nothing. Refilled, every glossed lemma of the level is reached, 20 a seeding, whatever the order.
Shorter — the next 20 untracked lemmas, the glossed ones carded —, the lemmas without a gloss stay
untracked and stay first: once 20 of them fill the window, every seeding adds 0, for good:

| Pair, level, order | Glossed lemmas in the level | Reached with a shorter deck |
|---|---|---|
| en-fr C1, rarest first | 752 | **0** (the first seeding adds nothing) |
| en-fr B2, rarest first | 2,197 | 7 |
| en-fr B1, commonest first | 1,962 | 1,424 |
| en-es C1, rarest first | 774 | 2 |
| fr-es A2, commonest first | 1,091 | 510 |
| fr-es B1, commonest first | 1,774 | **150** |
| fr-es C1, commonest first | 683 | 92 |
| es-en, fr-en, any level and order | all | all |

**What the goldens show** (the rule alone, blessed on the prototype):

- `en-fr.golden` (142 probes): **9 move**, 72 lines in, 70 out. The reader's `seed-level C1 3 rare`
  still answers 3, but its cards are `digitalize` « Digitaliser, numériser », `exhilarate`
  « Ragaillardir », `impishly` « Espièglement » instead of `unenviably`, `unmusically`,
  `unscathing`, which had no gloss. The review then opens on `digitalize` (graded) and marks `even`
  known where it marked `expedition`, so: `review-current first`, `review-current second`,
  `tracked-count` (10 → 11: `even` gains a status, `expedition` already had one), `analyse reader
  homographs` (`expedition` Known → Learning, known 89 → 88, 96 → 95 %), `phrase-gloss reader She
  will lead the expedition` (`expedition`'s class), `vocabulary-estimate reader` (3,365 → 3,363),
  `export-status-ops`, `export-card-ops`, `backup`.
- `en-es.golden` (182 probes): the **same 9**, 59 lines in, 57 out — `eclectically` « Eclécticamente »,
  `irately` « Airadamente », `tastebud` « Papila gustativa ».
- `es-fr.golden`, `es-en.golden`, `fr-en.golden`: **none**. Their two seedings draw glossed lemmas
  only (es-fr glosses every levelled lemma; es-en's and fr-en's draws are glossed), and fr-en's still
  do on 48b's tables (no levelled lemma without a gloss).
- `fr-es.golden` (change 51, merged, 209 probes): **6 move**, 22 lines in, 22 out — C1 rarest first
  draws `carreau` « Baldosa » instead of `cheminement`, which fr-es does not gloss; the review opens
  on `carreau` and marks `cavité` known where it marked `certainement`: `review-current first`,
  `review-current second`, `vocabulary-estimate reader` (2,304 → 2,305), `export-status-ops`,
  `export-card-ops`, `backup`. Measured first on a render of 51's design before it merged, then on
  the committed golden: the same 6.
- No word-card snapshot moves: `word-card-en-es.txt` and `word-card-es-en.txt` read the
  `word-grammar` probes, `word-card-fr-en.txt` and `word-card-fr-es.txt` those and the `phrase-gloss`
  probes — none of which moves (fr-es's `phrase-gloss reader Le vieux marin ferma les fenêtres du
  phare` is byte for byte; fr-en.golden does not move).
- Unchanged in all six: `level-ladder`, `vocabulary-estimate new-reader`, `has-levels`, both
  `seed-level` counts (5 and 3), `start-review`, `review-remaining`, `deck-count`, `due-count`,
  `calibration`, `declared-level`, `export-declared-levels`, every new reader's analysis.

**What the tests show**: on the rule alone, `seed_lemmas_caps_and_skips_tracked_lemmas` fails (it
seeds `quixotic` without a gloss), `cross_native.rs` fails for English and French (`review-current
first`: `digitalize` through en-fr, `eclectically` through en-es; `cavité` through fr-en, `carreau`
through fr-es) and passes for Spanish, and the English goldens and fr-es's move. With D4's tests on
the prototype (`fr_es_baseline.rs`'s comparison stopping at the seeding too), every test of
`lingua-wasm` passes on the three re-blessed goldens; the seeding check sees 153 / 148 skips
among the first 50 drawn over the twelve level-orders through en-fr / en-es, 0 / 3 through es-fr /
es-en, 1 / 83 through fr-en / fr-es.

**The message** (D7), set in the panel's font (`system-ui` at 12 px, macOS's system font measured
with AppKit) and wrapped word by word at the drawer's text width (352 px: 380 px less 14 px each
side) and at a narrow panel's (292 px):

| | Today | Characters | Width | Lines at 352 / 292 px |
|---|---|---|---|---|
| fr | « Aucune carte ajoutée — ces mots sont déjà suivis ou dans ton deck. » | 66 | 383 px | 2 / 2 |
| en | "No cards added — these words are already tracked or in your deck." | 65 | 379 px | 2 / 2 |
| es | « No se ha añadido ninguna tarjeta: estas palabras ya están seguidas o en tu mazo. » | 80 | 460 px | 2 / 2 |
| | **Draft** | | | |
| fr | « Aucune carte ajoutée — ces mots sont déjà suivis, dans ton deck ou sans traduction. » | 83 | 478 px | 2 / 2 |
| en | "No cards added — these words are already tracked or in your deck, or have no translation." | 89 | 508 px | 2 / 2 |
| es | « No se ha añadido ninguna tarjeta: estas palabras ya están seguidas, en tu mazo o sin traducción. » | 96 | 547 px | 2 / 2 |

A first Spanish draft, « … o en tu mazo, o no tienen traducción. » (104 characters), took a third
line at 292 px and was shortened.

## Decisions

### D1 — The deck skips a lemma its pack does not gloss

`Deck::seed_lemmas` passes over a candidate whose gloss is absent, before the cap counts it, as it
passes over a lemma already carded or holding an explicit status; it writes no card and no status
for it. The rule lives in the core, where the deck's seeding requirement lives, so that every caller
— the extension's `seedLevel` today, any other later — gets it. `Card::seeded` takes the gloss itself
(a `String`), not an `Option`: a seeded card without a gloss can no longer be built, and the
signature says so. The pack is the one the engine holds for the studied language, glossed in the
engine's native language — the reader's (*An engine serves one native language*).

*Rejected — filter in `seedLevel` (lingua-wasm)*: the requirement is the deck's, and a second caller
would have to remember it. *Rejected — leave the seeding and let review translate a card without a
gloss*: review has no translator; a card's answer is its gloss (M5: written by a person).

### D2 — A skipped lemma's place is taken by the next one

A lemma skipped for want of a gloss takes no place under the cap; the next lemma of the level in the
order chosen takes it. The core already does exactly this for a tracked lemma (the loop counts only
the cards it adds), so the rule is one more reason to skip, not a new mechanism; the spec's scenario
*Seed a level into the deck*, which said « at most 15 » for 20 asked with 5 tracked, is rewritten to
say what the core does.

The measure decides it: with a shorter deck, the skipped lemmas never get a card, so they stay
first in the order, and once 20 of them fill a seeding the level gives nothing more — en-fr's C1
rarest first gives 0 cards from the first seeding (its 20 rarest lemmas have no French gloss),
fr-es's B1 commonest first stops after 150 of 1,774 glossed lemmas (*Measured*). Refilled, a seeding
gives the count asked: every level of every pair holds at least 649 glossed lemmas (fr-es's C2), and
the cap is 50.

*Rejected — mark the skipped lemmas so a later seeding passes them*: it would write a status the
reader never chose, which moves the ladder, the estimate and the sync.

### D3 — What the reader knows does not move

A lemma without a gloss is still a word of the language at its level: it keeps its level, its row
in the ladder (`lemmas_at_level` is unchanged), its place among the dictionary words the estimate
counts (`dictionary_words` counts levelled lemmas whatever their gloss), and its presumption below a
declared level (`KnowledgeState` reads levels, not glosses). Seeding writes no status, before or
after. So the vocabulary estimate's rule does not change: a fresh reader's estimate and every ladder
are byte for byte as before in all six goldens. The reader's estimate moves in en-fr's and en-es's
goldens (3,365 → 3,363) and fr-es's (2,304 → 2,305) only because the scenario's review marks another card known once the deck
differs — the same rule, another history.

The cost, said plainly: a level can show words « to learn » in the ladder that seeding will never
add — fr-es's C2 shows 227 once its 649 glossed lemmas are carded. The ladder is the studied side
(*An analysis does not depend on the native language*) and stays so; the control says why (D7).

### D4 — The seeded deck is the native side; the invariance tests say so

The cards a level seeds now depend on the pack's glosses, so two packs of one language seed different
decks (537 levelled lemmas for English, 1,158 for French). Every probe that follows the reader's
seeding — the review, the statuses it writes, the pages analysed with them, the exports and the
backup — can differ through no leak of the native side into the studied side. The tests keep
checking both, apart:

- `tests/support/mod.rs`: `render_with` keeps the reader's seeding (the goldens do not change shape);
  `render_unseeded` leaves out the two `seed-level` probes and their seedings, the deck holding the
  scenario's three cards; `follows_seeding(names, name)` names the probes from `start-review` on.
- `cross_native.rs`: `assert_probes_alike` answers the scenario unseeded and compares every probe as
  before — all 19 of the reader's later probes included, through engines. A new check,
  `seeding_follows_each_packs_glosses`, builds each language's two committed packs and seeds, from a
  fresh engine, every level in both orders, 50 at a time: the cards are the first 50 lemmas of that
  level, in that order (read from the reference pack, whose studied sections the other's equal), that
  the pack glosses, each with the pack's gloss and labelled with its native language. The gloss
  filter is then the seeding's one native dependency, and it is checked through every level, not
  only the scenario's two.
- `en_es_baseline.rs`, `es_en_baseline.rs`, `fr_es_baseline.rs`: the golden-to-golden comparison
  stops at the seeding — every probe before `start-review`, both seeding counts included; the 19
  after are compared through engines by `cross_native.rs`, and pinned exactly by each golden.

*Rejected — compare the 19 when both goldens seeded the same lemmas* (es-fr's and es-en's do today,
en-fr's and en-es's do not): a test whose reach depends on the data it reads says less than one that
always compares the same probes. *Rejected — change the scenario's seedings to levels both packs
gloss*: every golden would move, es-fr's with no reason, and the next gloss update could break it.
*Rejected — skip a lemma any pack of the language does not gloss*: the deck of an en-fr reader would
then depend on en-es's tables.

### D5 — What moves, and what cannot

Moves: `en-fr.golden` and `en-es.golden` on the 9 probes of *Measured*, `fr-es.golden` on 6,
re-blessed with the owner's approval (2026-10-10, task 0.1) — en-fr's under the programme's rule
« en-fr and es-fr output does not move », en-es's a shipped pair's; `fr_es_baseline.rs`'s comparison
(D4); `seed_lemmas_caps_and_skips_tracked_lemmas` and `card.rs`'s seeded-card test, on the new
signature; the one message of D7.

Cannot move: `es-fr.golden` (es-fr glosses every levelled lemma, so its seeding draws what it drew),
`es-en.golden` and `fr-en.golden` (their scenario draws are glossed; fr-en's on 48b's tables too —
it has no levelled lemma without a gloss there); every pack byte and pin, since no table and no
builder moves; the four word-card snapshots, whose `word-grammar` and `phrase-gloss` probes do not
move; every extension test but the two that pin D7's message, the seed control being driven through
a fake port.

Change 51 landed first (#866): its comparison stops at the seeding here, and its golden is re-blessed
on its 6 probes; its snapshot `word-card-fr-es.txt` does not move.

### D6 — No analyser version, no stored format

*An analyser version per studied language* moves a language's version with its analysis' output.
Seeding changes which cards a reader's deck holds, not how any page is analysed for a given reader:
`analyse reader homographs` moves in en-fr's golden because the scenario's reader marked another card
known, under the same rules (as change 41b bumped nothing for a fix of how every pack is read). No
pack byte moves, so no `pack_version`; no field, label or schema of the backup or the sync moves (a
seeded card is the card it was, with a gloss).

### D7 — The control says when only words without a translation are left

Settled by the owner on 2026-10-10 (Q1): the message shown when a seeding adds nothing is reworded,
in the three languages, so that it is also true when the level's remaining words have no translation
in the reader's pack. The drafts, in the extension's register (tu in French, tú in Spanish) and with
the word the word card uses for a missing gloss (`card.noGloss`: « Pas de traduction dans le pack. »,
"No translation in the pack.", « No hay traducción en el paquete. »):

- fr: « Aucune carte ajoutée — ces mots sont déjà suivis, dans ton deck ou sans traduction. »
- en: "No cards added — these words are already tracked or in your deck, or have no translation."
- es: « No se ha añadido ninguna tarjeta: estas palabras ya están seguidas, en tu mazo o sin
  traducción. »

Each keeps today's opening and sentence and adds the one case; « ces mots » are the level's words, as
today. Each stays on the two lines today's message takes in the drawer and in a narrow panel
(*Measured*: 83, 89 and 96 characters for 66, 65 and 80). The owner reads the English and Spanish
drafts (M9, task 0.3); a word changed there joins this change before it merges.

**One message for both cases.** The core can tell them apart — `seed_lemmas` sees each lemma it skips,
and why — but saying it is not cheap: `seedLevel` returns a count, so the reason would take a new
return shape through the binding (`lingua-wasm`'s `seedLevel`), `analyzer/engine.ts`,
`analyzer/port.ts`, `analyzer/messaging-port.ts` and every fake port of the extension's tests
(`test/helpers.ts`, `stats-view.spec.ts`, `review-stats-copy.spec.ts`), and a fourth message in three
languages for the owner to read. The case comes only once every glossed word of a level is in the
deck (13 seedings of 50 at fr-es's C2), and the reader's next step is the same either way: another
level, or the other order. Recommended, and taken: one message.

**What moves.** The `noCardsAdded` entry of `src/i18n/fr/stats.ts`, `en/stats.ts` and `es/stats.ts`,
nothing else: no key, no slot, no other text, no surface code. It moves the French interface by one
message — an owner's decision, an exception to the programme's « French interface byte for byte »
(M23). Tests: `test/stats-view.spec.ts` pins the whole French text (it checks the prefix « Aucune
carte ajoutée » today, which holds); `test/review-stats-copy.spec.ts` pins the English and Spanish
texts after a seeding that adds nothing (its Spanish case reads `esStats.noCardsAdded` today);
`test/i18n.spec.ts` (shapes, slots, nothing left in French) and `test/lint-copy.spec.ts` pass
unchanged. No snapshot holds the message (`test/baseline/`), and no listing or guide quotes it
(`STORE-LISTING.md`, `TRANSLATION.md`, `REVIEWERS.md`).

*Rejected — name the level in the message* (« … à ce niveau »): a slot message for one case, against
a sentence that reads right without it.

## Risks / Trade-offs

- [A level runs dry before its ladder does] → measured (D3): only after every glossed lemma of the
  level is carded — at 50 a seeding, at least 13 seedings for fr-es's C2. The ladder stays the
  studied side; the control's message says why (D7).
- [The French interface moves by one message] → the owner's decision (Q1); one entry of one module,
  pinned by its test, the English and Spanish read by the owner (M9).
- [The invariance test compares the reader unseeded] → the seeding is checked apart, through every
  level and both orders, with its one native dependency named; the goldens still pin the seeded
  history exactly.
- [A reader of two natives on one account] → unchanged: a card synced from a device of another
  native shows the current pack's gloss in review, else its own text (*Review shows a gloss the
  reader can read*); this rule only stops cards with no text being created.
- [fr-es's coverage hides behind the skip] → the skip removes empty cards, not the gap: fr-es's
  coverage stays published (M6) and its own refinement (49b) its concern.

## Migration Plan

Nothing stored changes. The rule applies to seedings after the update; cards a reader already holds
are left as they are, those without a gloss included (the owner, Q2). Rollback: revert the commit; no data written under the rule
needs undoing.

## Order and dependencies

- **Before change 52** lists fr-en or fr-es (the owner, 2026-10-10). It joins 52's `archiveAfter`,
  as 48b and 49b do (52's design, *At or above*); 52 adds it when it next moves.
- **48b and 49b**, either side: no table moves here; their tables move none of this change's probes.
- **51**, merged first (#866): its `fr_es_baseline.rs` and `fr-es.golden` follow here (D4, D5).
- `archiveAfter`: `add-lingua-pack-fr-es`, which holds *An analysis does not depend on the native
  language* as this change modifies it; `add-lingua-french-levels`, whose French levels this change's
  scenarios read.

## Settled by the owner (2026-10-10, in session)

1. **The message when only words without a translation are left — reworded**, against the
   recommendation to leave it: in French, English and Spanish, one message for both cases, within the
   space the panel gives it today (D7). The panel said « Aucune carte ajoutée — ces mots sont déjà
   suivis ou dans ton deck. » even when the words left had no translation: a Spanish speaker who has
   added all 649 C2 French words that have a Spanish gloss still sees 227 C2 words to learn in the
   ladder, and adding C2 words adds none. The English and Spanish drafts are the owner's to read
   (M9, task 0.3).
2. **Cards without a gloss already in decks — left as they are**, as recommended. Seeded before this
   change, mostly by French readers of English who chose « rarest first » at C1 or C2 (today 20 of the
   20 rarest C1 words, 16 of C2's). They are the reader's cards, some already reviewed; review shows
   them without an answer, and the reader can mark them known or put them aside. Retiring them would
   write a sync change the reader never made.

The re-bless of `en-fr.golden`, `en-es.golden` and `fr-es.golden` (task 0.1) was approved the same
day, before the implementation.
