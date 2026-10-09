# Design — add-lingua-french-forms-tables

## Context

See proposal.md (Why). Where French stands, and what this design measured:

| What | Where, today |
|---|---|
| French in the core | `StudiedLanguage::French`, analyser `0.1.0`, the baseline analysis: the pack's form→lemma lookup, else the lowercased form (`lemmatize_baseline`); no tokenisation of its own, no NFC (add-lingua-french-baseline D2) |
| French tables | none: `scripts/lingua-data/testdata/fr-en/` is a hand-written fixture of about two hundred forms, which the French invariance baseline runs over until change 48 (change 39 D5, *Hand-over*) |
| The layout (M24) | `tables/<studied>/` holds `forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`, the pinned tag pool `tags.tsv`, the dictionary words `lexical.tsv` and `studied.json`; `tables/<pair>/` the native side, `pin.json`, `README.md`. A studied folder is written only by its reference pair's reduction, « the first pair built for » a language studied later (`pack_sources.py split`, which makes the first pair reduced the reference); `check_committed_tables` requires `tags.tsv` and `lexical.tsv` to exist, the second equal to the reference's glossed lemmas |
| The source | change 38's catalogue: `kaikki-French.jsonl`, `("entries", "fr")` of the English edition, « French's studied side and fr-en's glosses (changes 43, 45, 48) »; no pair registers it yet |
| Spanish's precedent | add-lingua-spanish-forms-tables: tagged inflections and form-of links, combined forms left to the analyser, one lemma by override / GSD counts / own entry / frequency / order, 60,000 lemmas by wordfreq and their attested forms, `lingua-pack-measure` over PUD held to 98.5 / 93.5 / 97 |
| The measurement inputs | the French section change 38's measurement derived on 2026-10-08 from the English dump regenerated on 2026-10-03 08:24 (dump sha256 `93b79aac…`, 25,614,284,530 B decompressed): 403,269 entries, 510,058,226 B, sha256 `2d7bbe5f…` — no dump was fetched again. UD French-GSD at `94d5b68e185fc22a9ef292040e84f476d36d9b0e` (train 25,555,018 B, dev 2,573,677 B, test 724,095 B; CC BY-SA 4.0) and UD French-PUD at `db260db10fe728853c549760801229ef4e7b16e1` (`fr_pud-ud-test.conllu`, sha256 `4dfed37b…`; CC BY-SA 3.0), both the default branch's head of 2026-05-06; wordfreq 3.1.1 under Python 3.12 |
| How it was measured | a prototype reducer in the scratchpad (never committed), its tables built into a fr-en pack by `lingua-pack-build` and measured by `lingua-pack-measure` from origin/main `03bdb0d3`, French at `0.1.0`; the step and cut tables below by the same lookup in Python (identical to the binary but for UD's inverted pronouns, D9) |

What the English Wiktionary's French section is like, as measured: 246,605 of its entries are verb
forms or verbs, 90,698 nouns, 47,011 adjectives, 10,111 names; 7,395 verbs carry a conjugation
table (`inflection-template`); 76,591 words have a sense that is not a form-of (the dictionary's
lemmas), 7,969 of them names only. A verb's table lists the masculine singular past participle
(`dirigé`) and none of its agreed forms: those hang under the participle's own entry, which is a
form of the verb. 12,548 listed inflections are tagged alternative, obsolete, archaic, rare,
dated, misspelt, nonstandard or abbreviated.

## Goals / Non-Goals

**Goals:**
- French's forms and ranks, committed once in `tables/fr/` and written by fr-en's reduction, good
  enough to pass Spanish's gates on a treebank the reduction never reads.
- One lemma per form by a stated rule, its cost measured (M8).
- Tables that hold every word change 40's pre-pass writes (M21), and every run it must find whole,
  whichever of changes 40 and 43 lands first.
- Every committed byte of en-fr, es-fr, es-en and en-es, and every golden, unchanged.

**Non-Goals:**
- The readings and the pinned tag pool (45), the levels (46), the glosses, expressions and
  dictionary words (48), fr-es (49). `tags.tsv` and `lexical.tsv` are committed empty, the files
  the folder contract requires, filled by 45 and 48.
- Any rule of the analyser (40, 41): the elision split, the contractions, the inversion, the
  cascade, NFC, names, function words. The tables are designed against M21 as change 40's proposed
  design writes it (`origin/docs/propose-add-lingua-french-tokenisation`), and checked against its
  lists once it is on `main` (D4).
- Lemma alternatives (M8: the optional `add-lingua-lemma-alternatives`, outside the counts).
- Shipping a French pack (52).

## Decisions

### D1 — fr-en is French's reference pair, and its reducer holds French's studied rules

`reduce-fr-en.py` reads the sources, applies French's forms rules (D3–D7) and writes `forms.tsv`,
`freq.tsv`, an empty `gloss.tsv`, `NOTICE` and `manifest.json` into its work folder; `split` files
them, makes fr-en French's reference on its first reduction (it writes `tables/fr/studied.json`)
and derives `tables/fr/lexical.tsv` from fr-en's glossed lemmas — none until change 48, so the file
is empty. `tables/fr/tags.tsv` is a person's pin that no reduction writes (`KEPT_INPUTS`); it is
committed empty here, because `record-build` and the committed-tables check require it, and change
45 pins French's pool in it. Change 48 adds the glosses to the same reducer; changes 45 and 46 add
the readings and the levels to it; change 49's `reduce-fr-es.py` reads `tables/fr/` as committed,
as es-en reads `tables/es/`.

French's rules live in the pair's own file, as Spanish's live in `reduce-es-fr.py`: a reducer named
with a hyphen cannot be imported, so fr-es can never load them, and fr-en's rule digest is that
file and `reduce_common.py` (change 48 adds the English edition's module for the glosses). **No
shared module is edited**: `reduce_common.py`'s `canonical_ranks` cannot express D6, so French's
ranking is written in the reducer, and every other pair's digest stays what its pin records (D11).

*Rejected — fr-es as the reference.* Its forms would come from the Spanish Wiktionary's French
section (11,862,917 B as kaikki's extract), whose inflections are not tagged, and the first French
pair built would be the second planned.
*Rejected — a module `reduce_studied_fr.py` shared by the French pairs.* It would be importable by
fr-es, whose reducer must load nothing of the reference's (add-lingua-pack-es-en D1).
*Rejected — a pin of the studied folder's own.* Split-lingua-pack-tables-by-language D3: the
reference's pin is the studied tables' provenance; a second pin would be written on every update.

### D2 — Sources: the English Wiktionary's French section, GSD's counts, wordfreq

- **kaikki**: `DUMPS["fr-en"] = {"en": ("kaikki-French.jsonl",)}` — the catalogue's file, no new
  derivation (change 38, *A pair of stage 3 registers what it reads*). Change 48 adds
  `kaikki-en-traductions-fr.jsonl` and `kaikki-fr-traductions-en.jsonl` for the glosses. Measured:
  510,058,226 B raw, 26,410,463 B as its zstd level-19 asset (change 38 estimated 35 MB), published
  under `lingua-pack-sources-fr-en-<snapshot>`.
- **UD French-GSD**, training and development sections (390,368 syntactic words, 43,244 distinct
  form–lemma pairs), at the commit above, as `PINNED["fr-en"]` (`gsd-train`, `gsd-dev`), checked by
  sha256 — read for one thing, how often a form stands for each lemma (D5). Its test section is
  never read by the reduction (D9).
- **wordfreq `fr` 3.1.1**, pinned by version and by `requirements-reduce.txt`'s hashes.

### D3 — What a form is

A French word, lowercased and in NFC, the typographic apostrophe read as `'`, matches
`[a-zàâäçéèêëîïôöùûüÿœæ]+(?:['-][a-zàâäçéèêëîïôöùûüÿœæ]+)*'?` — letters, words joined by hyphens or
by an inner apostrophe (`aujourd'hui`, `presqu'île`), and the elided pieces' final apostrophe. A
form's candidate lemmas come from:
- **the inflections a lemma's entry lists**, except kaikki's bookkeeping (`table-tags`,
  `inflection-template`, `class`, `romanization`, `canonical`), multi-word constructions
  (`avoir + past participle`), and an inflection tagged alternative, obsolete, archaic, rare,
  dated, uncommon, misspelt, nonstandard, proscribed, abbreviated or a pronunciation spelling — a
  form a source marks archaic, rarer or doubtful is no inflection (*Licence hygiene*);
- **the `form_of` targets** of a form's own entry, the target's first word (kaikki writes `beau used
  before a masculine noun…` as `bel`'s target);
- **a form of a form, along one part of speech.** When a candidate is itself read as a form of
  another word (the first choice of D5 maps it elsewhere), the form reaches that word too, if the
  entry linking the form to the candidate and the one linking the candidate to the word are of one
  part of speech: `dirigée` is the feminine of the participle `dirigé` (a verb entry), itself the
  past participle of *diriger* (a verb entry), so `dirigée` → *diriger*; `étés` is the plural of the
  noun `été` (a noun entry), and `été` reads as *être* through a verb entry, so `étés` reaches
  nothing and stays out: an unknown word rather than the verb. 6,367 forms of the table get
  their lemma this way. Without the rule, the tables fail the resolution gate (S0 below).

An entry with a sense that is not a form-of is a lemma, and its own candidate.

### D4 — The tables serve M21, as change 40 writes it

M21 settled French's tokenisation; change 40 (`add-lingua-french-tokenisation`, proposed) writes
it, and its design names what it asks of these tables. Its pre-pass hands the lookup **the word a
piece stands for**, not the piece: `l'` is read `le`, `d'` `de`, `j'` `je`, `n'` `ne`, `c'` `ce`, `ç'`
`ça`, `qu'` `que`, `jusqu'` `jusque`, `lorsqu'` `lorsque`, `puisqu'` `puisque`, `quoiqu'` `quoique`,
`s'` `si` before `il`/`ils` and `se` otherwise, `m'` and `t'` `moi` and `toi` right after a hyphen
and `me` and `te` otherwise; `au` is `à` + `le` and `aux` `à` + `les`; an inversion is its pieces
(`dit-il` → `dit` + `il`, the euphonic `t` no token), its pronouns among `je`, `tu`, `il`, `elle`,
`on`, `nous`, `vous`, `ils`, `elles`, `ce`, `le`, `la`, `les`, `lui`, `leur`, `moi`, `toi`, `y`, `en`;
and a hyphenated run the pack lists whole stays one token, read before any of this (its D5,
step 2). The tables hold what that pre-pass looks up, and nothing it never does:

- **Every word the pre-pass writes is a form.** Measured on the prototype's tables: all 32 —
  the 17 words of the elisions, `à`, `le`, `les`, and the pronouns — are forms, each a ranked lemma
  or a form of one (`les` and `la` of *le*); the rarest, `quoique`, is rank 3,328.
- **The pieces themselves are forms too**, by a closed, reviewed table in the reducer (`ELISIONS`:
  form, lemma, reason), each mapped to the first word the pre-pass reads it as: `l'` → *le*, `s'` →
  *se*, `m'` → *me*, `t'` → *te*, and the eleven others as above. The pre-pass never looks a piece
  up; the treebanks do: UD writes the piece as the word (`l'`, lemma `le`), and the measurement
  lemmatises a treebank's words as written (D9). The pieces are 5.53 % of PUD's measured words
  (1,118 of 20,232) and 5.55 % of GSD's test section — the 6.5 % of literary French the programme
  measured is of the same order —; without the table they would not resolve. The dictionary could
  not decide them: it writes `l'` as an alternative of *le* and *la*, `s'` and `n'` with no pointer,
  `qu'` under *que* and *qui*.
- **No plain word beginning with a piece is a form.** The pre-pass splits a word without a hyphen
  at its piece whatever the pack lists (`c'est`, `d'abord`, `l'on`, `jusqu'à`), so the whole string
  is never looked up: 180 dictionary entries and 609 of wordfreq's tokens are left out, and the
  expressions among them are expression keys (change 44). A **hyphenated run** beginning with a piece
  is read whole first, so it is a candidate like any compound (D6): `c'est-à-dire`, which GSD
  attests, is a form (rank 460). A word whose inner apostrophe is no piece's stays, as change 40
  keeps it whole: `aujourd'hui` (rank 136), `quelqu'un` (204), `presqu'île`, `prud'homme`,
  `main-d'œuvre`; `presqu'` is no piece, since *presque* elides in `presqu'île` alone.
- **A word ending in a pronoun is listed whole**, or the inversion rule would split it. The
  dictionary has 19 hyphenated words whose last piece is one of the pronouns above: 15 are verbs,
  phrases or interjections made of a verb and its pronouns (`est-il`, `a-t-il`, `allez-y`,
  `excusez-moi`), which the split reads right and the tables leave out; 4 are nouns —
  `rendez-vous` (GSD-attested, rank 1,587), `qu'en-dira-t-on`, `malgré-nous` and `non-moi` —, which
  the tables keep, the last three, which GSD never meets, at the 60,000 cut's last ranks. The
  demonstratives `celui-ci` (336) and `celui-là` (5,018) are lemmas, `celle-ci`, `ceux-là`, … their
  forms. `chez-moi`, which change 40's design names, is no entry of this dictionary: it reads
  `chez` + `moi`.
- **`au` and `aux` are no word**: neither a form nor a rank, since the pre-pass always splits them.
  Left in, they would be French's 15th and 40th words — at A1 once change 46 bands the ranks — for
  strings no page token can be. (Spanish's `al` and `del`, which its tokeniser splits too, keep the
  ranks es-fr gave them; nothing of es-fr moves here.)
- **`du` and `des` are words of their own**, since they stay whole: `du` maps to itself (its own
  partitive-article entry), and `des` — whose every sense is a pointer (the plural of *un*, *une*
  and *du*, the contraction of *de* + *les*), and which GSD's counts would read as *un* (1,730 of
  1,736 whole occurrences) — maps to itself by the same rule, the one place this design overrides
  the counts. A reader meeting « des » meets « des », not « un »; ranks 6 and 10.
- **A verb form joined to its clitic pronouns by hyphens** (`souviens-toi`, `portez-vous`,
  `sois-t'en`, 704 forms kaikki lists in reflexive conjugations) is left out: listed whole, it would
  stop the inversion split that reads it as a verb and its pronouns.

**The check is this change's**, as change 40's design leaves it (« change 43 runs the same check
on the real tables »): a test over the committed tables that every word the French pre-pass can
write is a form of `tables/fr/forms.tsv`, that every piece of `ELISIONS` maps to the first word the
pre-pass reads it as, that every hyphenated noun, adjective, adverb, pronoun or preposition of the
dictionary ending in one of its pronouns is a form, and that `au` and `aux` are not (task 3.3). It
reads change 40's lists from lingua-core when change 40 is on `main`, and holds them literally
until then; if change 40 settles other lists, its pull request moves the test and this reducer's
table with them.

### D5 — One lemma per form (M8), and what it costs

When a form has several candidates, the first rule that decides wins — Spanish's order:
1. **the override list** (`OVERRIDES`: form, lemma, reason), empty, as Spanish's;
2. **GSD's counts** of the form under each candidate;
3. **the form's own entry**, when it is a lemma;
4. **the candidate's wordfreq frequency**;
5. **alphabetical order**, so the result never depends on the source's order.

Before it, Spanish's names rule (fix-lingua-spanish-card-noise D1): a form that is a name and
another word's keeps only the commoner reading, by wordfreq — `cette` reads as *ce*, not the town;
`claire` as *clair*; `paris` stays the city (Zipf 5.71 against *pari*'s 4.12). It narrows 226
forms.

Measured on the tables: 3,599 forms keep more than one candidate after the cut; the counts decide
1,384, the own entry 1,314, frequency or order 901. **The cost M8 accepts** is a dictionary noun
whose own form a verb takes, which then leaves the pack, since a lemma is keyed by its own form:
- `porte`: GSD counts 39 *porter*, 23 *porte* — the noun « door » leaves the pack, `portes` reads as
  *porter* (it is also its second person), and « la porte » shows *porter*'s card;
- `été` (830 *être*, 51 *été*), `demande` (37 *demander*, 36 *demande*), `offre` (23, 15), `reste`
  (88, 37), `passé` (38, 12), `produit` (48, 22), `montre` (24, 3), `sort` (17, 9);
- in all, 29, 54 and 125 dictionary nouns among wordfreq's 1,000, 2,000 and 5,000 commonest words
  read as a verb. **Spanish's committed tables, by the same rule and the same count, have 30, 48 and
  95** (`cuenta` → *contar*, `pregunta` → *preguntar*): the rule costs French what it costs Spanish.

The alternative was measured too: **the form's own entry first** (a noun keeps its form) keeps those
nouns and breaks the auxiliaries — `est`, `été`, `suis`, `a` read as *est*, *été*, … —: 39.03 % of
PUD's auxiliaries and 91.50 % of its content words take the treebank's lemma, below two gates. A
list of overrides could keep a single noun (`porte` → *porte*, reading « il porte » as the door);
the list starts empty, as Spanish's did, and the owner may name a row (Open Questions).

### D6 — Lemmas and ranks: 60,000 by wordfreq, elision stems out, compounds by evidence

- **The ranks** are wordfreq `fr`'s order over its words that are not only inflected forms by the
  first choice of D5, dense, the 60,000 first kept. Words the dictionary does not know (names,
  loans, `etc`) are ranked as en-fr's and es-fr's are: a lowercase word outside the lexicon reads as
  unknown.
- **wordfreq's elision stems are no words.** Its tokeniser splits `l'homme` into `l` and `homme`,
  so `l`, `d`, `c`, `qu`, `j`, `n`, `s`, `t` and `m` carry the elided pieces' frequency and stand at
  ranks 5 to 63 of its list; ranked, the letter `l` would be French's fourth commonest word. They
  are skipped; the pieces are forms of their words (D4).
- **A hyphenated word is ranked by evidence.** wordfreq splits at the hyphen too and never lists
  `peut-être`; for a hyphenated string it answers the combination of its parts, which is about its
  rarest part's frequency. Ranked by that estimate, `est-il` and `a-t-il` — inversions the
  dictionary lists as entries — would be French's 17th and 49th words, `fait-tout` the 51st,
  `en-cas` the 133rd, and 2,760 compounds would enter the 60,000 (measured with S4's other rules).
  So a compound is ranked only when GSD's training sections attest it as a lemma, at the **lower**
  of wordfreq's estimate and GSD's own frequency (its count over 390,368 words, as a Zipf value) —
  and, whatever GSD says, when it is one of D4's nouns ending in a pronoun, at the cut's last
  ranks: 474 compounds, 70 of them in the 5,000 commonest — `lui-même` 277, `celui-ci` 336,
  `c'est-à-dire` 460, `peut-être` 941, `au-delà` 1,010, `rendez-vous` 1,587, `week-end` 1,674,
  `après-midi` 1,923, `grand-mère` 3,540 —, `en-cas` at 10,723, and no inversion. The lower of two sources never ranks
  a compound above what either supports; on the 77 compounds GSD meets five times or more,
  wordfreq's estimate runs a median 0.23 Zipf above GSD's own frequency, against −0.12 for single
  words, and the gap widens as the compound gets rarer. Resolution pays 0.07 points on PUD for the
  rule (99.28 % with the raw estimate). A word
  with an inner apostrophe needs no estimate: wordfreq lists `aujourd'hui` and `quelqu'un` whole.
- **Forms**: those whose chosen lemma is kept and that wordfreq attests (a Zipf frequency above
  zero), each kept lemma's identity form, and the elided pieces, as Spanish's.

### D7 — A spelling variant reads as the word it spells

An entry whose every sense only spells another word — an ASCII spelling of a ligature
(`coeur`, « nonstandard spelling of *cœur* », 52 entries) or a post-1990 spelling (`connait`,
`évènement`, `chaine`, `weekend`, « post-1990 spelling of … », 323 entries) — is read as a form of
that word, with its own inflections (`coeurs` → *cœur*). Otherwise each would be a lemma of its own
beside the word it spells — 20 and 81 of them among wordfreq's 60,000 commonest —, and a reader who
knows `boîte` would meet `boite` as new. The tables gain 209 forms and lose those lemmas' ranks to
other words; the gates do not move (S3). An alternative that is a word of its own (`clef`, whose
entry also glosses the musical clef) keeps its entry, as any lemma does.

### D8 — The cut, measured

With D3–D7 (the S-table measures the rules, the cut-table the size; the figures are the lookup's,
D9 gives the binary's):

| Rules, 60,000 lemmas, attested forms | Forms | `forms.tsv` | PUD resolved | content | AUX | GSD test resolved | content | AUX |
|---|---|---|---|---|---|---|---|---|
| S0 Spanish's rules transposed, with the pieces of D4 | 117,826 | 2.12 MB | 97.61 % | 93.86 % | 99.90 % | 97.80 % | 93.72 % | 99.72 % |
| S1 + a form of a form (D3) | 123,895 | 2.24 MB | 98.90 % | 96.30 % | 99.90 % | 98.88 % | 95.65 % | 99.72 % |
| S2 + compounds by evidence (D6) | 124,172 | 2.26 MB | 99.21 % | 96.39 % | 99.90 % | 99.12 % | 95.70 % | 99.72 % |
| S3 + spelling variants (D7) | 124,381 | 2.26 MB | 99.21 % | 96.40 % | 99.90 % | 99.12 % | 95.67 % | 99.72 % |
| **S4 + M21 as change 40 writes it (D4): this design** | **124,013** | **2.26 MB** | **99.21 %** | **96.40 %** | **99.90 %** | **99.12 %** | **95.67 %** | **99.72 %** |

| Cut (S4's rules) | Forms | `forms.tsv` | `freq.tsv` | PUD resolved | content | AUX | GSD test resolved | content | AUX |
|---|---|---|---|---|---|---|---|---|---|
| 20,000 lemmas, attested | 61,989 | 1.15 MB | 0.28 MB | 98.24 % | 95.65 % | 99.90 % | 97.70 % | 94.14 % | 99.72 % |
| 40,000 lemmas, attested | 97,013 | 1.78 MB | 0.56 MB | 99.05 % | 96.29 % | 99.90 % | 98.93 % | 95.41 % | 99.72 % |
| **60,000 lemmas, attested** | **124,013** | **2.26 MB** | **0.84 MB** | **99.21 %** | **96.40 %** | **99.90 %** | **99.12 %** | **95.67 %** | **99.72 %** |
| 80,000 lemmas, attested | 147,752 | 2.66 MB | 1.13 MB | 99.32 % | 96.46 % | 99.90 % | 99.23 % | 95.75 % | 99.72 % |
| 100,000 lemmas, attested | 170,208 | 3.04 MB | 1.41 MB | 99.37 % | 96.49 % | 99.90 % | 99.29 % | 95.83 % | 99.72 % |
| 60,000 lemmas, every form | 212,655 | 4.15 MB | 0.84 MB | 99.23 % | 96.44 % | 99.90 % | 99.12 % | 95.67 % | 99.72 % |
| 100,000 lemmas, every form | 283,848 | 5.49 MB | 1.41 MB | 99.39 % | 96.53 % | 99.90 % | 99.29 % | 95.83 % | 99.72 % |

**60,000 lemmas and their attested forms**, as Spanish: every form nobody writes would add 88,642
rows (1.9 MB) for 0.02 points; 40,000 passes too, with 0.16 points less resolution on PUD and
0.19 on GSD; 80,000 adds 0.4 MB of forms and 0.3 MB of ranks for 0.11. The analysis of change 41
resolves part of the long tail by rule, as Spanish's enclitic rule does.

**The pack** built from these two tables alone (no gloss, reading or level) is 1,240,362 B; the same
two tables of Spanish build 1,308,123 B. Both French packs (48, 49) carry it; their glosses,
readings and levels are measured against the 5 MiB budget by the changes that add them. No package
changes here.

### D9 — The measurement: PUD gated, GSD's test section reported

`scripts/lingua-data/measure/fr-ud.sh`, beside `es-pud.sh`: fetches UD French-PUD and GSD's
`fr_gsd-ud-test.conllu` at their commits, checks their sha256, builds fr-en's pack from the
committed tables (`build.sh fr-en`), and runs `lingua-pack-measure` — the core's own lemmatiser
with the pack's lexicon, whatever French's analyser version is when it runs — over each. PUD is held
to the gates (98.5 % resolved, 93.5 % content lemmas, 97 % auxiliaries, the programme's figures for
Spanish, which French meets); GSD's test section is reported beside it and not gated, since the
reduction reads GSD's other two sections and a held-out section of the same treebank is the
weaker test. Neither file is committed, and the reduction reads neither.

With the prototype's tables, French at `0.1.0`:

| | Words | Resolved | Content words | Auxiliaries |
|---|---|---|---|---|
| UD French-PUD (gated) | 20,232 | **99.12 %** | **96.36 %** of 9,573 | **99.90 %** of 1,030 |
| GSD test (reported) | 8,049 | 98.91 % | 95.65 % of 3,791 | 99.72 % of 359 |

The binary reads UD's words as UD writes them, without tokenising them: an elided piece is `l'`,
which `ELISIONS` resolves (D4), and an inverted pronoun is `-il` or `-t-il` (18 of PUD's measured
words, 17 of GSD's), which stays unresolved where change 40's pre-pass hands `il` to the lookup;
the Python lookup of D8, which strips their hyphens, reads 0.09 points more on PUD. The harness is
not changed to run a tokenisation: its figure does not depend on whether change 40 has landed, and
the gated figure is the conservative one.
`lingua-pack-measure` needs no French arm — it lemmatises through the pack's studied language —
and its doc line names both treebanks.

The PUD pin is the one change 50's marks measurement reads for French (`tool/marks/pud.mjs`), so
the two never measure different files.

### D10 — Committed tables and the pipeline

| Folder | Files |
|---|---|
| `tables/fr/` | `forms.tsv`, `freq.tsv`, `studied.json` (`{"reference": "fr-en"}`), `tags.tsv` (empty, 45), `lexical.tsv` (empty, 48); `grammar.tsv` and `level.tsv` come with 45 and 46 |
| `tables/fr-en/` | `gloss.tsv` (empty, 48), `NOTICE`, `manifest.json`, `pin.json`, `README.md` |

`manifest.json` studies `fr`, is glossed in `en`, and names French's analyser version as
`analysis/mod.rs` writes it (`FRENCH_ANALYZER_VERSION`, read by the reducer as es-fr's reads
Spanish's): `0.1.0` today. `NOTICE` and the manifest credit kaikki / the English Wiktionary
(CC BY-SA 4.0 + GFDL), wordfreq (CC BY-SA 4.0) and UD French-GSD (CC BY-SA 4.0, counts only).
`pin.json` records the derived file, its dump, GSD's two files, wordfreq, the rules and the pack.

The pipeline:
- `build.sh`: `max_lemmas` answers 60,000 for `fr-*`, so fr-es (49) caps French as fr-en does.
- `lingua-pack-update.yml`: `fr-en` among the dispatch options, and a concurrency group `fr`
  beside `en` and `es`, so an update of fr-en and of fr-es never propose `tables/fr/` on two
  branches; its header names fr-en. The monthly dry run and `pair=all` take fr-en through `pairs`,
  and the English dump they already read for es-fr and es-en derives `kaikki-French.jsonl` in the
  same pass (change 38 D4): one more reduction, no more download.
- `lingua-extension-check.yml`: the build loop (`tables/*-*/`) builds fr-en's pack against its pin;
  the reduce job re-reduces it from its pin (a 26 MB asset and GSD's two files), French's tables
  among the bytes it must reproduce. Nothing is added to either loop.
- **The first tables** come from `lingua-pack-update`, dispatched on the implementation branch in
  update mode for fr-en, as es-en's did: it publishes `lingua-pack-sources-fr-en-<snapshot>`, the
  sources release, and pushes the folders whole (« a pair's first update proposes the folder
  whole »). The branch commits `tables/fr/studied.json` and the empty `tags.tsv` first, which
  `record-build` reads. A sources release is tooling; no pack, package or listing is published
  (M18).
- `packs.json`, the extension's build and its packages are unchanged.

### D11 — What moves, and what cannot

- **fr-en.golden does not move.** The French baseline runs over the fixture pack
  (`PackSource::Testdata`) beside the real es-en pack until change 48 commits fr-en's glosses and
  switches it to the tables (change 39 D5): the tables this change commits gloss nothing, so a
  switch now would replace the fixture's seventy glosses and fifty-three levels with none and blind
  the golden until change 48. The fixture is kept; *The committed tables replace the fixture*
  happens in change 48, which commits fr-en's tables in the sense that scenario reads — its
  glosses. This change's spec says so (*French's forms and frequencies*, *The French baseline keeps
  its fixture*).
- **en-fr, es-fr, es-en and en-es cannot move**: no file of their rule digests changes —
  `reduce-en-fr.py`, `reduce-es-fr.py`, `reduce-es-en.py`, `reduce-en-es.py`, `reduce_common.py`,
  the `reduce_edition_*.py` — (`pack_sources.py` and `build.sh` are outside every digest, change 38),
  no table, pin or `tables/en/`, `tables/es/` file changes, and nothing of lingua-core, lingua-wasm
  or the extension changes. Their packs are rebuilt by the check job to the sha256 their pins
  record, and the four goldens and their beside lines pass without re-blessing. The gate is
  `git diff --stat origin/main --` over their folders and the golden files, empty, and the reduce
  job reproducing every committed byte (task 4.1).

OpenSpec: two ADDED requirements in `lingua-data-packs`, no MODIFIED one. *French's forms and
frequencies* reads change 38's catalogue (*Every kaikki source is derived from one dump per
Wiktionary edition*), change 39's French (*A pack names the language it studies*, MODIFIED there)
and change 40's pre-pass (the words it writes, its MODIFIED *French is a studied language served by
the baseline analysis*); the three are in `archiveAfter`, and `openspec_archive_order.py` exits 10
naming those still open on this branch's base — 38 and 39 today, 40 once its proposal is merged. Neither requirement of `lingua-analysis` that changes 39 and 40 hold is touched.

### D12 — What the later changes of the stage take from here

| Change | Takes |
|---|---|
| 40 tokenisation (M21) | every word its pre-pass writes as a form, the nouns ending in a pronoun and `c'est-à-dire` listed whole, `au`/`aux` absent, `du`/`des` words, no plain word beginning with a piece; the check of D4 (task 3.3), which reads its lists once it is on `main` |
| 41 analysis | the tables its cascade is designed and measured on; its version bump re-reduces fr-en (`manifest.json` alone moves, and the pack and pin with it), as each Spanish bump re-reduced es-fr |
| 44 expression keys | the plain words left out because they begin with a piece (`d'abord`, `c'est`, `l'on`) |
| 45 grammar | `reduce-fr-en.py`'s pass over the same entries, the forms it reads the readings of, `tags.tsv` to pin |
| 46 levels | the ranks (estimated, M7) |
| 48 fr-en | the reducer to add the glosses to, `lexical.tsv` filled, the golden's hand-over |
| 49 fr-es | `tables/fr/` as committed, French's 60,000 cap |
| 50 marks | the PUD pin |

## Risks / Trade-offs

- **[A common French noun leaves the pack]** (`porte`, `été`, `demande`) → M8 accepts it, measured at
  Spanish's rate (D5); the readings of change 45 cannot name a lemma that is not kept, so the
  optional lemma alternatives are where it returns; an override row is the owner's to name.
- **[Compound ranks are estimates]** → the lower of two sources, GSD-attested only; a compound GSD
  never met is classified from its parts by the tokeniser, as today, and an update's report lists
  the ranks that move.
- **[Change 40's pre-pass moves after this change is designed]** — another piece, another read
  word, another pronoun — → the check of D4 reads its lists from lingua-core once change 40 is on
  `main` and fails in the pull request that moves them; the reducer's table follows the tokeniser in
  it. Measured today against change 40's proposed design: every word it writes is a form.
- **[A compound with a pronoun tail is added to the dictionary]** → the check lists the dictionary's
  hyphenated nouns, adjectives, adverbs, pronouns and prepositions ending in a pronoun at each
  reduction, so an update that adds one keeps it whole or fails.
- **[The analyser version moves under committed tables]** → 40 and 41 re-reduce fr-en in their pull
  requests (D12); the reduce job fails until they do.
- **[kaikki's next regeneration differs from the measured one]** → the first tables are an update's,
  at the dispatch day's snapshot; task 2.2 measures them again and records the figures; the gates,
  not the prototype's figures, decide.
- **[The reduce job grows]** → one more pair, a 26 MB asset and two GSD files; the job measured
  3 min 38 s for three pairs (add-lingua-pack-es-en) and keeps its 45-minute timeout.
- **[GSD's counts are sparse beyond common words]** → steps 3 to 5 of D5 decide, towards the word's
  own entry, as Spanish's do.

## Migration Plan

Nothing to migrate: new folders, no existing table, pin, pack or baseline moves, no reader holds a
French pack. Rollback is a revert of the folders and the registrations; the sources release stays,
as every pair's does.

## Effort

7–10.5 ideal days, against the programme's 8.5–13.5: the reducer's rules 2.5–3.5, its tests 1–1.5,
the pipeline's registration and its Python tests 0.75–1.25, the dispatch, tables, pin, READMEs and
`SOURCES.md` 0.75–1.25, the measurement 0.5–1, the Rust committed-tables scenarios 0.5–1, the M8
report and the check of D4 0.5–1, spec and programme 0.5. The top is at the programme's ≈ 10.

**If it runs over, it splits** at the rules that only refine: **43a**, `add-lingua-french-forms-tables`
— D1–D5 (the four nouns ending in a pronoun included, which change 40 needs), D6 without the
GSD-attested compounds, D8–D12 — passes the gates alone (measured with the binary, before D4's
whole runs, which move no measured word but `c'est-à-dire`: PUD 98.81 % resolved, 96.27 % content
lemmas, 99.90 % auxiliaries; GSD test 98.65 / 95.65 / 99.72);
**43b**, `refine-lingua-french-forms-tables` — the compound ranks (D6) and the spelling variants
(D7), about 1.5–2 days — re-reduces fr-en alone, which no reader holds, and moves forms and ranks
only. 45, 46 and 48 would follow 43b.

## Open Questions

For the owner, none blocking:
1. **M8's cost, by name.** `porte` (GSD 39 against 23), `demande` (37 against 36) and `offre`
   (23 against 15) are the closest common cases; an override row keeps the noun and reads the verb
   form as it. The list starts empty, as Spanish's did; the owner may name rows, each with its
   reason, before the tables are reduced (task 2.2) or in any later reduction.
2. **`des` as a word of its own** (D4) rather than *un*, as GSD's counts read it: the one place the
   design overrides the counts, for M21's « du/des whole ».
3. **GSD's test section reported, not gated** (D9): gating it too is one line, at the risk of an
   update failing on a section 0.39 points above the gate.
