# Design — add-lingua-french-levels

## Context

See proposal.md (Why). Where levels stand, and what this design measured:

| What | Where, today |
|---|---|
| The level machinery | `add-lingua-cefr-levels` (archived): a pack's optional `levels` section, one byte per lemma of its pool, from `level.tsv` (`lemma<TAB>A1…C2`); the core presumes a lemma known below the declared level, folds each level's lemmas into the ladder, seeds a deck from a level, and counts levelled lemmas among the dictionary words of the vocabulary estimate (`Pack::dictionary_words`) |
| Estimated levels | `add-lingua-spanish-levels` (archived, D1 of the Spanish programme): `reduce-es-fr.py` gives English's level sizes (`ENGLISH_BANDS`: 1,020, 1,158, 2,015, 2,347, 886, 876) to Spanish's commonest lemmas, in rank order, skipping a lemma without a French gloss or with a proper noun's only; the manifest says `levels_estimated`; `PackMeta.levels_estimated`, `Pack::levels_estimated()`, the engine's `levelsEstimated` |
| The labels | `localise-lingua-*` and `add-lingua-native-language-labels`: `levelTitleEstimated`, `estimatedLevelsNote`, `borrowedTypicalNote`, `levelNameEstimated` in `src/i18n/{fr,en,es}/languages.ts`, the scale `levelScale` « CEFR » in French and English, « MCER » in Spanish (M19); the ladder's column « courants » for estimated levels |
| The ladder | `LinguaEngine::level_ladder`: a pack of another language than English whose levels are estimated gives each level `ENGLISH_TYPICAL_VOCABULARY` (`[0, 1_292, 3_359, 7_988, 16_326, 20_556]`, frozen in lingua-core by `generalise-lingua-native-language` D6) and says `typicalFrom: "en"` |
| French's tables | change 43 (`add-lingua-french-forms-tables`, proposal merged, implementation in progress): `reduce-fr-en.py`, fr-en French's reference pair, writes `tables/fr/forms.tsv` and `freq.tsv` (60,000 lemmas by wordfreq, `au`/`aux` no word, `du`/`des` words of their own, compounds ranked when GSD attests them) from the English Wiktionary's French section; « `grammar.tsv` and `level.tsv` come with 45 and 46 », « 46 levels: the ranks (estimated, M7) » (its D10, D12); `lexical.tsv` empty until change 48 glosses fr-en |
| The French golden | change 39 (`add-lingua-french-baseline`): `fr-en.golden` over the hand-written fixture `testdata/fr-en/` (53 hand-written levels, no `levels_estimated`, « the levels are a fixture's, not a decision (M7) ») until change 48 commits fr-en's glosses and switches it to the tables (its D5 *Hand-over*; change 43's D11) |
| The extension's French | `StudiedLanguage` is `en \| es` until change 52; change 47 (merged) brought the catalogue's `french` words forward for the voice block, the level messages gain French with change 52 (change 47's D4) |
| The measurement inputs | change 43's prototype tables (the 124,040 forms and 60,000 ranks its design reports, in their deterministic order), the English Wiktionary's French and Spanish sections derived on 2026-10-08 from the English dump (`kaikki-French.jsonl`, 510,058,226 B; `kaikki-Spanish.jsonl`), the committed `tables/en`, `tables/es` and `tables/es-en`, UD French-, Spanish- and English-PUD (the same 1,000 sentences, CC BY-SA 3.0, never committed), wordfreq 3.1.1; a scratch copy of `origin/main` `35faf774` (French's tokenisation at `0.2.0`, change 40) for the pack and golden figures |

No openly licensed French CEFR word list was found: FLELex (CEFRLex) is non-commercial, the
*Français fondamental* lists are not published under an open licence, and the CEFR-tagged lists of
teaching sites state none. FLELex was neither read nor measured against: its licence does not admit
the use, even for a figure in a design.

## Goals / Non-Goals

**Goals:**
- A French level table, so a reader of French declares a level, gets words presumed known below
  it, has a ladder and can strengthen a level — derived as Spanish's is (M7, D1).
- Measured before it ships, with open data only, and like-for-like with Spanish's.
- A level never lands on another lemma than the one it is written for.
- Every pack studying French says its levels are estimated, so every surface labels them.

**Non-Goals:**
- A licensed French list. If the request succeeds, its table replaces this one (D8).
- Changing Spanish's levels, even where this design departs from Spanish's rule (D2): es-fr and
  es-en must not move.
- The French labels and their tests on the extension's surfaces (change 52); the site and listings
  sentences on French levels (change 53).
- The French golden's hand-over to the committed tables (change 48).
- Levels per sense, as for English.

## Decisions

### D1 — English's level sizes, over French's commonest lemmas (M7 = D1)

The derivation is Spanish's: the lemmas `tables/fr/freq.tsv` ranks, in rank order, those a CEFR
list would hold (D2), take English's level sizes — 1,020 A1, 1,158 A2, 2,015 B1, 2,347 B2, 886 C1,
876 C2: 8,302 lemmas. The sizes are constants in `reduce-fr-en.py` (`ENGLISH_BANDS`), equal to
es-fr's by a test, never read from English's tables at reduction time: an English dictionary update
must not move French's levels unannounced (add-lingua-spanish-levels D2). They are written into
fr-en's reducer because French's rules live there (change 43's D1) and no shared module is edited:
moving the constant into `reduce_common.py` would move every pair's rule digest.

Measured on English, where the truth is known, on the committed tables (the study's figures,
reproduced): given English's 8,302 CEFR lemmas in their own rank order, these sizes agree with the
lists for **39.8 %** of them and within one level for **82.6 %**.

On change 43's prototype ranks, with D2's rules:

| Level | Lemmas | Ranks | First words | Spanish's committed ranks |
|---|---|---|---|---|
| A1 | 1,020 | 1–1,080 | de, le, et, à, en, des, un, que, pas, du, il, pour | 1–1,088 |
| A2 | 1,158 | 1,081–2,376 | certainement, clairement, collection, conscience, content, croissance | 1,089–2,423 |
| B1 | 2,015 | 2,377–4,814 | alimentaire, annuel, apparence, autrefois, bénéfice, clinique | 2,424–5,103 |
| B2 | 2,347 | 4,815–8,086 | rébellion, rédacteur, réserver, simultanément, slogan, tasse, tigre | 5,104–8,885 |
| C1 | 886 | 8,088–9,418 | confidentialité, conjoncture, consolider, contradictoire, croquis | 8,888–10,474 |
| C2 | 876 | 9,419–10,768 | clandestin, commentateur, contrefaçon, convaincant, diaspora | 10,479–12,107 |

French's levels end earlier than Spanish's because fewer of its commonest words are left out
(2,466 ranked lemmas within French's span, 3,805 within Spanish's): the English Wiktionary's French
section knows more of wordfreq's French words than the French Wiktionary's Spanish entries gloss of
Spanish's.

Every word change 40's pre-pass writes has a level: the read elided pieces, `à`, `le`, `les` and the
inversion's pronouns are A1 but `jusque` (A2) and `quoique` (B1); `au` and `aux`, no words of
French's tables, have none; `du` and `des` are A1. M8's cost carries over: `porte` is no lemma, so
the door has no level and « il porte » reads as *porter* (A1).

*Rejected — English's rank cutoffs* (A1 up to rank 1,096 … C2 up to 40,612), as for Spanish
(add-lingua-spanish-levels D1): every French lemma within them would be levelled, and a declared
level would presume far more words known than an English reader's.

*Rejected — ordering by the summed frequency of each lemma's forms* rather than by rank, so that a
noun said mostly in the plural (`cheveu`, rank 9,419, C2, while `cheveux` is common) ranks by its
forms. Measured: on English's lists, 39.6 % exact and 83.3 % within one, against 39.8 % and 82.6 %
by rank — no gain where the truth is known; on French's translation proxy (D4), 29.0 % and 67.7 %
against 28.3 % and 66.7 %, with 3,857 levels changed. It would part French's method from Spanish's
for no measured gain; a change covering both could take it up.

### D2 — What a CEFR list would leave out, read from the English Wiktionary's French section

Spanish skips a lemma without a French gloss, or with a proper noun's only: a CEFR list leaves out
`the`, `twitter`, `madrid`. French's gloss is fr-en's, which change 48 commits after this change.
So the rule reads the source fr-en's forms already come from and its glosses will come from, the
English Wiktionary's French section, which `reduce-fr-en.py` already reads. A ranked lemma takes no
level when:

1. **the section gives it no sense that is not a form of another word** — an unknown word, mostly
   English or a code (`the`, `etc`, `in`, `km`, `http`) — **or only a name's** (`france`, `paris`,
   `québec`, `facebook`);
2. **it is a single character the section gives no word's sense**: every sense a letter's name, a
   symbol's or an abbreviation (`b`, `e`, `h`, `p`, `ç`). `à` and `y`, a preposition and a pronoun,
   `x` (a stool, X-rated) and `ô` (an interjection) keep their place;
3. **every sense the section gives it only spells another word** — an obsolete, archaic, rare or
   alternative spelling, a letter-case form, a misspelling, a pronunciation spelling (`etre`,
   `etat`, `etait`, `parceque`, `derniere`, `arreter`): a reader learns the word it spells, which
   has a level of its own;
4. **its own form reads as another lemma** in `tables/fr/forms.tsv` (D3).

Measured on the prototype, within the levelled span (ranks 1–10,768): 2,466 ranked lemmas take no
level — 1,390 unknown to the section, 1,022 names, 33 spellings, 17 letters, 4 lemmas whose own
form reads as another. Every levelled lemma has a sense the English Wiktionary glosses (0 without
gloss text), so the levelled lemmas are, as far as a reduction before change 48 can tell, words
fr-en will gloss; change 48 checks it on its glosses.

The second and third rules are **departures from Spanish's outcome**, stated. The English
Wiktionary writes an entry for a letter and for a misspelling; the French Wiktionary's Spanish
entries, which decide Spanish's levels, rarely do, so Spanish's rule leaves most such words out
without naming them — but not letters: Spanish's committed table levels 19 single letters (`b`,
`d`, `h`, `k`, … A1), which « Renforcer un niveau » can seed as cards. Without rules 2 and 3,
French's A1 would hold 13 letters and `etat`, `etats`, `etre`. Spanish's table does not move here
(es-fr and es-en must not); aligning it is a change of its own (Open Questions).

*Rejected — waiting for fr-en's glosses (Spanish's rule word for word).* Change 46 precedes change
48: the table would be empty until then, and would move when the glosses land. Read from the
section, the table moves with the section or the ranks only, as the studied side should
(architecture: « the studied side does not depend on the native side »).

*Rejected — no rule (every ranked lemma), or names only.* Measured: with no rule, A1 holds
`france`, `paris`, `the`, `etc` and ends at rank 1,020; with names only, `the`, `of`, `and`, `km`
stay A1.

### D3 — A level reaches the lemma it is written for

The builder writes a lemma's level at `lex.id_of(lemma)`, the form lookup (`FstLexicon::id_of`). A
ranked lemma whose own form the forms table maps to another lemma hands its level to that lemma.
Change 43's prototype has four such levelled lemmas — `venue` (→ *venir*), `donnée` (→ *donner*),
`saisie` (→ *saisir*), `tranchée` (→ *trancher*): ranked because their plurals still read as them
(change 43's D6, « a word another of its forms still reaches keeps its rank »), while their own
form reads as the verb. Built as they are, the pack gives *venir* and *donner* the A2 of `venue`
and `donnée` instead of their own A1, and the nouns' own entries lose theirs (measured: 8,299 of
the pack's lemmas carry a level instead of 8,302; the A1 ladder counts 1,018, B2 2,346). Rule 4 of
D2 leaves them out, and the next lemmas take their places.

**A check over every committed pair** (*A level reaches the lemma it is written for*): the
committed-tables test builds each pair's pack and requires every lemma of its studied language's
`level.tsv` to carry that level, and every lemma the pack gives a level to carry the table's — the
second half catches a level landing on a lemma the table leaves without one. Measured on
`origin/main`: en-fr, es-fr, es-en and en-es pass both ways, 8,302 levels each; French's table
passes with rule 4, and fails without it, naming `donner` and `venir`.

**The same keying moves ranks, which is change 43's.** Built from the prototype's tables, the
pack ranks *donner* 1,711 instead of 225, *venir* 1,637 instead of 388, *trancher* 7,931 instead of
6,654 and *retomber* 18,045 instead of 7,691 — the ranks of `donnée`, `venue`, `tranchée` and
`retombée` — and those nouns, read through their plurals, carry no rank of their own (five of the
60,000 ranks are not the table's, measured). Change 43's implementation is in progress; its
committed-tables test can hold every rank as this change's holds every level, and its reducer can
keep a lemma only when its own form reads as itself. If it does, rule 4 leaves nothing out and
stays as a guard. A gloss is keyed the same way, which change 48 meets if 43 keeps such lemmas.

### D4 — Six levels: the scale is monotone

D1 of the Spanish programme falls back to three bands when the derived scale is not monotone. It is
monotone where the truth is known: the mean true level of each estimated English level rises from
A1 to C2 — 1.67, 2.41, 3.12, 3.89, 4.67, 5.03 — and the median rank of each true level too — 930,
2,217, 4,055, 8,165, 15,296, 26,611. French has no open truth, so this design measured French
against two open proxies, with Spanish's committed table as the control:

| Proxy | French (prototype) | Spanish (committed) |
|---|---|---|
| A levelled lemma's translation — the head word of its first sense in the English Wiktionary — levelled by English's CEFR lists: exact / within one | 28.3 % / 66.7 % of 5,950 (71.7 % covered) | 29.4 % / 65.9 % of 6,140 (74.0 %) |
| Mean English level of the translations, by estimated level A1 → C2 | 1.80, 2.40, 2.75, 3.06, 3.12, 3.06 | 1.83, 2.37, 2.74, 3.04, 3.04, 3.08 |
| A1 words whose translation is English A1 / A1–A2 | 51.6 % / 76.5 % | 49.6 % / 73.8 % |
| UD PUD's words at A1, A2, B1, B2, C1, C2, none (gold lemmas) | 78.4, 7.8, 5.7, 2.7, 0.6, 0.6, 4.3 % | 76.6, 8.1, 6.0, 2.6, 0.8, 0.4, 5.5 % |
| PUD's words presumed known when declaring A2, B1, B2, C1, C2 | 78.4, 86.2, 91.9, 94.6, 95.2 % | 76.6, 84.7, 90.7, 93.3, 94.1 % |

English's lists on the same PUD sentences: 65.8, 12.2, 9.0, 3.9, 0.5, 0.2, 8.3 %, so a B1 reader is
presumed to know 78.0 % of the words. The estimate presumes more than English's lists below each
level, in French as in Spanish: its A1 is the 1,020 commonest words. The translation proxy rises
from A1 to C1 in both languages and cannot tell C1 from C2 in either (a rare French word's
translation is often a common English word), so it is no evidence against the scale's top; the
method's monotonicity is measured on English. French keeps the six levels.

### D5 — Every pack studying French says its levels are estimated

`reduce-fr-en.py`'s manifest carries `"levels_estimated": true`, as es-fr's and es-en's do; the
builder reads it from the manifest's `meta`, `PackMeta` keeps it, the engine reports it. fr-en's
`NOTICE` says the levels are estimated from wordfreq's ranks and the English Wiktionary's French
section, citing no CEFR list. The requirement binds every pack studying French: fr-es (change 49)
reads `tables/fr/level.tsv` as committed, writes `levels_estimated` in its own manifest, and its
NOTICE credits the English Wiktionary's French section for which lemmas take a level, as es-en's
credits the French Wiktionary for Spanish's.

### D6 — What the reader sees, from change 52 on: nothing to build here

No code changes for French's levels to read as Spanish's do:
- **The ladder.** `level_ladder` borrows English's frozen typical vocabularies for any language but
  English whose levels are estimated, and says `typicalFrom: "en"` (*An estimated ladder shows
  English's typical vocabularies*); French qualifies by its flag. Its legend reads « taken from
  English, whose level sizes French borrows », true by D1.
- **The labels.** The extension labels estimated levels from `levelsEstimated`, in the interface
  language, with the scale's name M19 sets: in English « Estimated French level », « Levels
  estimated from word frequency, as no freely licensed CEFR list exists for French. », « B1
  (estimated) »; in Spanish « Nivel de francés estimado », « … a falta de una lista MCER de uso
  libre para el francés. », « B1 (estimado) »; the ladder's « courants » column. The French
  interface's scale is « CEFR » (`src/i18n/fr/languages.ts`, M19: « French unchanged »), and it
  never names French levels: a French-native reader cannot study French. The `french` words are
  in the catalogue since change 47 (its D4); the level messages' key widens with change 52, whose
  surfaces' tests in English and Spanish cover them.
- The site's French text says « CECR » (`apps/site/src/lib/lingua-text.ts`, `spanishLevels`)
  where the extension's says « CEFR »; change 53 writes French's sentence beside Spanish's and
  chooses.

### D7 — The French golden does not move; what change 48's hand-over will show

The French invariance baseline runs over its fixture until change 48 commits fr-en's glosses and
switches it to the tables (change 39's D5, change 43's D11). This change commits French's level
table and leaves the fixture alone — its 53 hand-written levels and its manifest without
`levels_estimated` — so `fr-en.golden` does not move, and neither do the four other goldens.

*Rejected — flagging the fixture's manifest now.* Measured: it would move two probes, the `pack`
line (5,139 → 5,163 B) and `level-ladder` (English's figures and `typicalFrom: "en"` instead of the
fixture's own 0, 27, 46, 54, 55, 55). It would show M7 in the golden one change early, but change
39's requirement names the reasons the golden moves — a French rule, the fixture replaced by the
committed tables, the beside pack's update — and a flag on the fixture is none of them; change 43
kept the fixture whole for the same reason.

**What the levels will move at the hand-over**, measured by blessing the golden in a scratch copy
over change 43's prototype forms and ranks with the fixture's glosses, with and without this table
and its flag: **23 of the 141 probes** move — the `pack` line (+60,037 B), `has-levels` (which the
fixture already answers `true`, so it does not move at the hand-over), `level-ladder` (totals
1,020, 1,158, 2,015, 2,347, 886, 876, English's figures, `typicalFrom: "en"`), both vocabulary
estimates (a universe of about 8,300 words before change 48's lexical table, the levelled lemmas
being dictionary words; the reader's estimate 2,179), `promote-by-exposure` (0 → 3), both
`seed-level` probes and the review, deck and card probes that follow from the cards they seed,
the four reader pages (312 of their 343 tokens read as known for the B1 reader, against 146 of 342
with the fixture and 1 with no table), the reader's phrase gloss, the status and card exports and
the backup. Change 48's re-bless shows these 22 with the forms, readings and glosses; its pull
request can name them as the levels'.

### D8 — If a licence is granted

The owner sends a licence request to UCLouvain's CENTAL for FLELex, together with ELELex's
(`[manual]`). For it to be usable, the grant must cover what the pipeline does with every table:
deriving a lemma → level table from the list, **committing it in this public repository** (the
reduction is reproducible only from committed tables and pinned sources), and shipping it in the
packages the stores distribute, for commercial use, which the list's non-commercial terms
exclude. A grant for use without redistribution of the derived table would not be enough.

If granted, it is a data change, not a format change:
- `reduce-fr-en.py` reads FLELex at a pinned version: each lemma takes the first level at which the
  list attests it, the lowest on collisions (English's `reduce_levels` rule), joined to French's
  lemmas through its forms table; the source is credited in fr-en's (and fr-es's) NOTICE and
  manifest under the granted terms. The licence guard denies `NonCommercial`, so the grant is
  recorded as a licence category the guard admits, naming the grant — a small change to
  `lingua-pack`'s `licence.rs` and its tests.
- The manifests drop `levels_estimated` (absent is false): the extension's labels lose « estimé »
  and the ladder computes French's own typical vocabularies from its list, as English's does,
  instead of borrowing English's; no code changes for either.
- The figures this change cannot measure legitimately are measured then: this estimate against
  FLELex, exact and within one level, kept in fr-en's README.
- fr-en and fr-es are re-reduced in one pull request; their packs, pins and the French golden's
  level lines move. An ELELex grant does the same for es-fr and es-en — and moves es-fr's output,
  with the owner's approval in that pull request.

### D9 — Size

The levels section is one byte per lemma of the pack's pool. Measured on the prototype: the fr-en
pack built from forms and ranks alone is 1,239,104 B (change 43's figure); with this table and the
flag 1,299,139 B (+60,035 B). fr-es carries the same section. Spanish's packs carry theirs; both
French packs stay far below the 5 MiB budget.

### D10 — Tests and what cannot move

- **Python** (`test_reduce_fr_en.py`, on fixture entries shaped as the English dump writes them):
  the bands given in rank order; each rule of D2 — `paris` and `france` names only, `the` unknown,
  `b` and `e` letters while `à` and `y` are levelled, `etre` a spelling, `donnée` read as *donner*;
  `du` and `des` levelled; the sizes summing to 8,302 and equal to `reduce-es-fr.py`'s
  `ENGLISH_BANDS`; the manifest's `levels_estimated`.
- **Rust**, `crates/lingua-pack/tests/committed_tables.rs`: `tables/fr/` holds `level.tsv` beside
  change 43's files; 8,302 lemmas, English's sizes per level, each a ranked lemma whose own form
  reads as itself; fr-en's manifest and pack say estimated, its NOTICE says so; and, over every
  committed pair, every lemma of the studied language's level table carries that level in the pack.
  `crates/lingua-wasm/tests/languages.rs`: an engine on es-en with fr-en from the committed tables
  reports French's levels as estimated, and its ladder's rows carry English's typical vocabularies,
  `typicalFrom: "en"` and the six sizes as totals.
- **What cannot move.** No file of en-fr's, es-fr's, es-en's or en-es's rule digests changes
  (`reduce_common.py`, `reduce_edition_*.py`, their reducers); no table or pin of theirs; nothing of
  lingua-core, lingua-wasm's sources or the extension. The tests added read the committed tables
  and build packs in memory. The gate is `git diff --stat origin/main --` over their folders, the
  shared modules, `testdata/fr-en/` and the five goldens, empty; the five baselines pass without
  re-blessing; `check-reducer` passes for the four; the reduce job reproduces every committed byte.

OpenSpec: two ADDED requirements in `lingua-data-packs`, no MODIFIED one. *French's estimated
levels* reads change 43's *French's forms and frequencies* (the table it bands, the reducer it is
written by) and leaves change 39's baseline alone; both changes are in `archiveAfter`, and
`openspec_archive_order.py` exits 10 naming them. No open change holds *A pack says when its levels
are estimated*, *The Spanish pack's estimated levels* or *An estimated ladder shows English's typical
vocabularies*, and none is modified.

### D11 — What the later changes take from here

| Change | Takes |
|---|---|
| 45 grammar | nothing; both add to `reduce-fr-en.py`, and whichever merges second re-reduces on top of the other |
| 48 fr-en | the table, as committed; the golden's level lines at its hand-over (D7); a check that every levelled lemma is a dictionary word once `lexical.tsv` holds fr-en's glossed lemmas |
| 49 fr-es | `tables/fr/level.tsv` as committed; `levels_estimated` in its manifest; its NOTICE's credit (D5) |
| 52 enable | the labels' French words and their tests in English and Spanish (D6) |
| 53 listings and site | French's sentence on estimated levels beside Spanish's; « CECR » or « CEFR » in the French site text (D6) |

## Risks / Trade-offs

- **[Estimated levels are about 40 % exact]** → the labels say so, as for Spanish; within one level
  83 % of the time on English, and French reads like Spanish on every open proxy (D4).
- **[Numbers written as words]** → frequency places them high: `onze`, `treize`, `soixante` are B1;
  hyphenated ones take change 43's GSD ranks — `dix-sept` B1, `vingt-quatre` B2, `dix-neuf` and
  `soixante-dix` C2, `trente-deux` none — where English's lists put every number at A1. Spanish
  has the same weakness (`diecinueve` C1). A reviewed list of numbers at A1 for every estimated
  language is a change of its own, since Spanish's table would move (Open Questions).
- **[C2 ends inside a block of equal frequencies]** → change 43 ranks a compound GSD meets once at
  that one occurrence's frequency, Zipf 3.41, after the single words of that frequency and
  alphabetically among themselves: 326 ranked lemmas share it, and C2 ends among them — 208 of C2's
  876 lemmas are in the block, 124 of them such compounds, from `afro-américain` to
  `sous-officier`, while the block's 28 lemmas past the edge have no level.
  Spanish's C2 ends inside a block of 194 equal frequencies too (7 at C2); French's is larger because
  GSD's single occurrences share one frequency. A C2 word is presumed known by no declared level, so
  only the C2 ladder and C2 seeding see the difference. Alternative in Open Questions.
- **[A word ranked by a name or a brand]** → the names rule of change 43 keeps a word that is also a
  name, ranked by the string: `jean` (the cloth) is A1 by the given name's frequency, `twitter` (a
  French verb) A2, as Spanish's rule would keep them.
- **[A noun said mostly in the plural ranks low]** → the ranks are the lemma's own string's
  (Spanish's too): `cheveu` C2, `œil` A2. Ordering by the summed forms measured no gain on English
  (D1).
- **[Change 43's tables move]** → its first tables are an update at the dispatch day's snapshot;
  this change re-measures D1, D2 and D4 on the committed ranks (task 2.2), and the sizes, not the
  figures, are the requirement.
- **[A rule of D2 misreads the section]** → each rule is tested on entries shaped as the dump
  writes them; the reduction reports how many lemmas each rule leaves out; an update's report lists
  the levels that move (*A change to a studied language's tables reaches every pair of that
  language*).
- **[Change 45 and this change both edit `reduce-fr-en.py`]** → disjoint functions; the second to
  merge re-reduces fr-en on top of the first, and the reduce job fails until it does.

## Migration Plan

Nothing to migrate: no reader holds a French pack before change 52. `tables/fr/level.tsv` is new;
fr-en's manifest, NOTICE, pack and pin move; no other pair's. Rollback is a revert and a
re-reduction of fr-en.

## Effort

1.75–2.75 ideal days, against the programme's 1.5–3: the reducer's derivation and D2's rules
0.5–0.75, their Python tests 0.25–0.5, the re-reduction, pin, NOTICE, README and `SOURCES.md`
0.25–0.5, the Rust tests 0.25–0.5, the re-measurement on the committed ranks 0.25, spec and
programme 0.25.

## Open Questions

For the owner, none blocking:
1. **The departures from Spanish's outcome** (D2: letters and spellings take no level in French).
   Spanish's committed table levels 19 single letters, which « Renforcer un niveau » can seed;
   aligning Spanish is a change of its own that moves es-fr's output.
2. **Numbers.** Accept D1's weakness (`dix-neuf` C2), or a reviewed list putting numbers at A1 for
   every estimated language, in a change of its own.
3. **C2's edge through the compounds GSD meets once** (Risks): accept, or give such a compound no
   level, so that C2 takes single words in their place.
4. **The licence request** (D8): ask for deriving a level table, committing it in the public
   repository and shipping it in the packages for commercial use, not only for use — for FLELex and
   ELELex alike.
