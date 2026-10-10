# Design — seed-lingua-decks-with-glossed-lemmas

## Context

See proposal.md (Why). What exists, on `main` at `101b684d` (change 49's fr-es tables committed,
#862; change 48b proposed, #861, its implementation on a branch; change 49b proposed, #865):

| Where | What |
|---|---|
| `lingua-core/src/packs/pack.rs` | `lemmas_at_level(level)` (L452): every lemma whose level byte is `level`, in lemma-id order, each with `self.glosses.get(&id)` — `None` when the pack carries no gloss for it. The ladder (`levelLadder`) and the seeding both read it. `dictionary_words` (L484): the ranked lemmas that are dictionary words **or carry a level**, which the vocabulary estimate and the typical vocabularies count |
| `lingua-core/src/decks/review.rs` | `Deck::seed_lemmas(lang, lemmas, gloss_language, knowledge, cap, at)` (L75): for each `(lemma, Option<gloss>)` in the caller's order, stops at `cap` new cards, **skips (and refills)** a lemma that already has a card or an explicit status, else upserts `Card::seeded(lemma, gloss, …)` — with `gloss` `None` when the pack has none. Writes no status. Its test `seed_lemmas_caps_and_skips_tracked_lemmas` (L429) seeds `quixotic` with no gloss |
| `lingua-core/src/decks/card.rs` | `Card::seeded(lemma, gloss: Option<String>, gloss_language, at)` (L182): source `Import`, no sentence, the form the lemma |
| `lingua-wasm/src/lib.rs` | `seedLevel(level, count, order, at, language)` (L983): `lemmas_at_level`, sorted by rank (`common`) or by rank descending (`rare`), unranked last, then `seed_lemmas` with the engine's native language as the label. Returns the number added; 0 for an unknown level or a pack without levels. Review shows a card the gloss of the pack held for its language, else the card's own text (`readable_gloss`, L119): a seeded card without a gloss shows nothing |
| `apps/lingua-extension/src/stats/view.ts` | « Renforcer un niveau » (L296–317): a level select preset to the declared level, a count (default 20, `SEED_CAP` 50), commonest or rarest first; it reports « N cartes ajoutées » or `noCardsAdded` (« Aucune carte ajoutée — ces mots sont déjà suivis ou dans ton deck. », its English and Spanish versions alike). The extension's tests drive it through a fake port |
| The levels | One `level.tsv` per studied language (M24), 8,302 levelled lemmas each, shared by its pairs. English's from CEFR lists (not estimated), Spanish's (es-fr's dictionary words) and French's (change 46, estimated, M7) from frequency. Change 48b gives a French level only to a lemma fr-en glosses (its D3); nothing ties a level to the glosses of a second pair |
| The invariance tests | `tests/support/mod.rs` renders one scenario per pair; its reader seeds `A2 5 common` then `C1 3 rare` (L390–399) and then reviews (the first card graded, the second marked known) — 19 probes follow, from `start-review` to `backup`. `cross_native.rs` (`assert_probes_alike`, L130) answers each language's scenario through two packs of that language and compares **every probe** with glosses, senses and expressions stripped. `en_es_baseline.rs` and `es_en_baseline.rs` compare their committed golden with en-fr's and es-fr's the same way (`the_golden_is_the_*_one_on_the_studied_side`). Change 51 plans the same for `fr-es.golden` (its design, *fr-es's golden*) |
| The requirements | `lingua-decks-review` *Level-targeted deck seeding* (no open change holds it). `lingua-analysis` *An analysis does not depend on the native language*, held by change 49 (merged, not archived), whose test compares « every probe » |

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
- Cards already seeded without a gloss (open question 2).
- The control's copy (open question 1).

## Measured

On a prototype over `main` (scratch, never committed): `seed_lemmas` skipping a candidate without a
gloss; every committed pack built from its tables; the five goldens blessed; every Rust test of
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
- Change 51's `fr-es.golden` (rendered on the prototype as its design declares it,
  `Scenario { pair: "fr-es", beside: &["en-es"], ..FRENCH }`): **6 move** — C1 rarest first draws
  `carreau` « Baldosa » instead of `cheminement`, which fr-es does not gloss; the review marks
  `cavité` known where it marked `certainement`; `vocabulary-estimate reader` 2,304 → 2,305.
- Unchanged in all five: `level-ladder`, `vocabulary-estimate new-reader`, `has-levels`, both
  `seed-level` counts (5 and 3), `start-review`, `review-remaining`, `deck-count`, `due-count`,
  `calibration`, `declared-level`, `export-declared-levels`, every new reader's analysis.

**What the tests show**: on the rule alone, `seed_lemmas_caps_and_skips_tracked_lemmas` fails (it
seeds `quixotic` without a gloss), `cross_native.rs` fails for English and French (`review-current
first`: `digitalize` through en-fr, `eclectically` through en-es; `cavité` through fr-en, `carreau`
through fr-es) and passes for Spanish, and the English goldens move. With D4's tests on the
prototype, every Rust test passes on the re-blessed goldens; the seeding check sees 153 / 148 skips
among the first 50 drawn over the twelve level-orders through en-fr / en-es, 0 / 3 through es-fr /
es-en, 1 / 83 through fr-en / fr-es.

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
are byte for byte as before in all five goldens. The reader's estimate moves in en-fr's and en-es's
goldens (3,365 → 3,363) only because the scenario's review marks another card known once the deck
differs — the same rule, another history.

The cost, said plainly: a level can show words « to learn » in the ladder that seeding will never
add — fr-es's C2 shows 227 once its 649 glossed lemmas are carded. The ladder is the studied side
(*An analysis does not depend on the native language*) and stays so; what the control says then is
open question 1.

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
- `en_es_baseline.rs`, `es_en_baseline.rs`: the golden-to-golden comparison stops at the seeding —
  every probe before `start-review`, both seeding counts included; the 19 after are compared through
  engines by `cross_native.rs`, and pinned exactly by each golden.

*Rejected — compare the 19 when both goldens seeded the same lemmas* (es-fr's and es-en's do today,
en-fr's and en-es's do not): a test whose reach depends on the data it reads says less than one that
always compares the same probes. *Rejected — change the scenario's seedings to levels both packs
gloss*: every golden would move, es-fr's with no reason, and the next gloss update could break it.
*Rejected — skip a lemma any pack of the language does not gloss*: the deck of an en-fr reader would
then depend on en-es's tables.

### D5 — What moves, and what cannot

Moves: `en-fr.golden` and `en-es.golden` on the 9 probes of *Measured*, re-blessed with the owner's
approval for en-fr (the programme's rule « en-fr and es-fr output does not move ») and for en-es
(a shipped pair); `seed_lemmas_caps_and_skips_tracked_lemmas` and `card.rs`'s seeded-card test, on
the new signature.

Cannot move: `es-fr.golden` (es-fr glosses every levelled lemma, so its seeding draws what it drew),
`es-en.golden` and `fr-en.golden` (their scenario draws are glossed; fr-en's on 48b's tables too —
it has no levelled lemma without a gloss there); every pack byte and pin, since no table and no
builder moves; `word-card-en-es.txt` and `word-card-es-en.txt`, which read the `word-grammar` probes
alone; every extension test, which drives the seed control through a fake port.

Change 51: if its `fr_es_baseline.rs` and `fr-es.golden` are on `main` first, this change has its
comparison stop at the seeding and re-blesses its 6 probes; otherwise 51 writes them with
`follows_seeding` and blesses on this rule.

### D6 — No analyser version, no stored format

*An analyser version per studied language* moves a language's version with its analysis' output.
Seeding changes which cards a reader's deck holds, not how any page is analysed for a given reader:
`analyse reader homographs` moves in en-fr's golden because the scenario's reader marked another card
known, under the same rules (as change 41b bumped nothing for a fix of how every pack is read). No
pack byte moves, so no `pack_version`; no field, label or schema of the backup or the sync moves (a
seeded card is the card it was, with a gloss).

## Risks / Trade-offs

- [A level runs dry before its ladder does] → measured (D3): only after every glossed lemma of the
  level is carded — at 50 a seeding, at least 13 seedings for fr-es's C2. The ladder stays the
  studied side; open question 1 asks whether the message should say why.
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
are left as they are (open question 2). Rollback: revert the commit; no data written under the rule
needs undoing.

## Order and dependencies

- **Before change 52** lists fr-en or fr-es (the owner, 2026-10-10). It joins 52's `archiveAfter`,
  as 48b and 49b do (52's design, *At or above*); 52 adds it when it next moves.
- **48b and 49b**, either side: no table moves here; their tables move none of this change's probes.
- **51**, either side (D5).
- `archiveAfter`: `add-lingua-pack-fr-es`, which holds *An analysis does not depend on the native
  language* as this change modifies it; `add-lingua-french-levels`, whose French levels this change's
  scenarios read.

## Open questions

1. **The message when only words without a translation are left.** The panel says « Aucune carte
   ajoutée — ces mots sont déjà suivis ou dans ton deck. » (and its English and Spanish versions).
   After this change it can also mean « the words left at this level have no translation in your
   pack »: a Spanish speaker who has added all 649 C2 French words that have a Spanish gloss still
   sees 227 C2 words to learn in the ladder, and adding C2 words adds none. Reword it in the three
   languages here (M9: the owner reviews the English and Spanish), or leave it? *Recommendation:
   leave it* — the case needs every glossed word of a level in the deck first (13 seedings of 50 at
   fr-es's C2), and a wording change belongs with the interface's copy.
2. **Cards without a gloss already in decks.** Seeded before this change, mostly by French readers of
   English who chose « rarest first » at C1 or C2 (today 20 of the 20 rarest C1 words, 16 of C2's).
   Leave them, or retire them on update? *Recommendation: leave them* — they are the reader's cards,
   some already reviewed; review shows them without an answer, and the reader can mark them known or
   put them aside. Retiring them would write a sync change the reader never made.
