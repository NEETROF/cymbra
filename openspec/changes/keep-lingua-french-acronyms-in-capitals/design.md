# Design — keep-lingua-french-acronyms-in-capitals

## Context

See proposal.md (Why). What exists, on `main` at `32c0edc0` with change 43b's proposal
(`read-lingua-french-ca-as-ca-cedilla`, #877) and its prototype tables:

| Where | What |
|---|---|
| Change 43b | A reviewed table of fr-en's reducer, `UNMARKED_SPELLINGS`, reads `ca` as a form of *ça* (with `age`, `ages` → *âge*, `forcement` → *forcément*); the English Wiktionary's French `CA` (four initialisms) and `ca` (« abbreviation of circa ») leave the tables. Its requirement reads a named spelling as its word « whatever its case, until a rule of French's analysis reads a word written in capitals apart ». Its D2 sketched three options for « CA » in capitals; its task 6.4: it reaches readers only with that rule or after it. Its implementation is in progress |
| `analysis/french.rs` | French's cascade (change 41): every token lowercased (`’` read as `'`) and in NFC, then the pack's forms, an unlisted lowercase plural, the form. Its spec: « the core SHALL produce a lowercase lemma » |
| `analysis/pipeline.rs` | `resolve_lemmas(token, studied, lexicon)` resolves one token, with no context; `analyse_document` calls it for each token of a block, `headword_reading` for each token of an expression's headword (the builder's keys, change 44), `engine::gloss_phrase` for each token of a selection, `engine::word_grammar` for the written word of a card |
| `analysis/lexicon.rs` | The forms map, an `fst::Map` keyed by bytes, and the lemma pool, sorted by bytes (change 41b); every lookup of the core lowercases its string first (`french_lowercase`, the three cascades, `word_grammar`'s `lemma.to_lowercase()`) |
| `engine.rs` | The names rule (change 41's D4) sets aside a form capitalised in mid-sentence, never written in lowercase, whose lemma is no dictionary word |
| `packs/pack.rs` | `gloss`, `sense_runs`, `readings` read a lemma by its place in the pool; `rank`, `level`, `is_dictionary_word` through the forms map; `dictionary_words` (the vocabulary estimate) counts the ranked dictionary words |
| `tables/fr/` (43b's prototype) | 124,101 forms, 60,000 ranked lemmas, 26,484 dictionary words, 8,302 levels; no form, rank or dictionary word holds a capital |
| `tables/fr-en/`, `tables/fr-es/` | Expressions: `conseil d'administration` « board of directors » / « Administración, consejo de administración », `chiffre d'affaires` « revenue, turnover, net sales » / « Ingreso, cifra de negocios, facturación », `courant alternatif` « alternating current; domestic electricity supply » / « Corriente alterna »; `comptable agréé` in neither |
| `apps/lingua-extension` | `reading/session.ts` keys a gesture's status and card by `g.lemma.toLowerCase()`; `reading/selection-card.ts` asks a word card's gloss for `g.lemma.toLowerCase()` |
| Versions | English `1.2.0`, Spanish `1.5.0`, French `1.2.0` (change 42d) |

What the dictionaries say of `CA`: the English Wiktionary's French section (fr-en's source) — a
masculine noun, « initialism of conseil d'administration », « … of chiffre d'affaires », « … of
courant alternatif ("AC or alternating current") », « … of comptable agréé ("chartered accountant") »
(Canada); the French Wiktionary — seven senses, those four and *corps d'armée*, *congés annuels*,
*communauté d'agglomération*. The Spanish Wiktionary and the French Wiktionary's translation tables,
fr-es's sources, have no entry `CA`. 43b counted 609 headwords of the English Wiktionary's French
section written in capitals as initialisms or acronyms; 60 of them share their lower case with
another word of the tables (`LE`, `NE`, `SE`, `SA`, `SI`, `VA`, `EAU`, `CAS`, `MUR`, `BAC`, `CV`,
`MDR`… and `CA`).

## Goals / Non-Goals

**Goals:**
- « CA » written in capitals, in a line that is not itself in capitals, read as the acronym, as the
  owner decided (2026-10-10, option A), by a table a person reviews row by row — `CA` alone.
- What the table holds, how the pack holds it, what each pair glosses it with, and how its card,
  status and key keep the capitals, decided and measured.
- What moves, measured on a prototype: the analyser version, the tables, pins, packs, goldens and
  snapshots; and proof that en-fr, es-fr, es-en and en-es cannot move.

**Non-Goals:**
- Any other acronym: the table is the mechanism; a row is a person's decision, and the owner asked
  for `CA` alone at first (*Not here*).
- Any byte of en-fr, es-fr, es-en or en-es; any English or Spanish rule (English's « US » reads as
  `us` for the same reason, which no one has asked to change).
- A line on the card spelling the acronym out in French (*Not here*).

## Measured

A prototype in the scratchpad (never committed): a worktree of `main` at `32c0edc0` with 43b's
prototype tables and probes (those of its design, which reproduce its committed figures), then:
- the rule of D1 in `analysis/french.rs` and `analysis/pipeline.rs`, read by the four callers, and
  `word_grammar`'s lemma kept (D6);
- French's analyser version `1.3.0` in the core, `tables/fr-en/manifest.json`,
  `tables/fr-es/manifest.json` and `testdata/fr-en/manifest.json`;
- the rows D2 and D4 specify, written into `tables/fr/forms.tsv`, `tables/fr/lexical.tsv`,
  `tables/fr-en/` and `tables/fr-es/` (`gloss.tsv`, `senses.tsv`) by hand: the reducers' rule was not
  prototyped, and the implementation's re-reduction must write exactly these rows (task 2.4);
- the four probes of D7, then `LINGUA_BLESS=1` once over the two French goldens, and every test of
  `lingua-core`, `lingua-pack` and `lingua-wasm` without re-blessing.

The corpora: UD's seven French treebanks (every section at hand: GSD's training, development and
test sections — the training section from fr-en's pin —, PUD, FQB, ParTUT, ParisStories, Rhapsodie,
Sequoia's development and test sections), Tatoeba's 727,867 French sentences, the French Wikipedia
sample and the four Gutenberg novels of change 44c, change 39's corpus, and the usage examples of
French entries in the French Wiktionary (595,982 sentences, quotations of the press and of books
among them) and in the English Wiktionary's French section (18,375): 23,467,957 tokens as the
prototype's engine reads them, analysed block by block with the prototype's fr-en pack, the rule on
and off.

## Decisions

### D1 — A word standing alone in capitals is looked up as written, before the cascade

For each token of the text French's analysis is given, before French's cascade:

1. **Written wholly in capitals**: the token is one word (not a hyphenated run, whose parts are read
   as today), and its source — its span, read without soft hyphens — holds two letters or more and
   no lowercase letter. `CA`, `ÉTÉ`, `ONU` are, and so is the `CA` of « CA-14 », a run holding a
   digit, which the tokeniser reads as a word of its own; `Ca`, `A`, `L'` are not, nor « LS-CA », a
   hyphenated run read by its parts.
2. **Standing alone in capitals**: the nearest token before it and the nearest token after it, in the
   same text, whose source holds two letters or more and is not the token's own span (`au` → `à` +
   `le` share one), are each absent or not written wholly in capitals. Single letters (`A`, `L'`,
   `W`), numbers and punctuation are passed over.
3. **Looked up as written**: the token's text, in NFC, not lowercased, in the pack's forms. A form
   holding a capital is only ever a row of the acronym table and its own lemma (D2): a hit makes the
   token's dictionary form that acronym, capitals kept, and the cascade does not run.

Anything else goes to French's cascade as today — `ça`'s `ca` row (43b) for « CA » in a headline, in
a shout, or for « Ca ». English and Spanish never run it: the function that reads the neighbours
answers « no » for every token of their texts without reading them.

**The owner's « line not itself in capitals », read at the word's neighbours.** The text French's
analysis is given is a page's block, a reader's selection, a card's written word or an expression's
headword; it has no lines or sentences of its own (the names rule reads a sentence's head from the
character before a word, change 41's `mid_sentence`). Read at the whole block, the rule would take a
shouted « ça » inside a line of lowercase words for the acronym; read at the word's neighbours, it
does not, and on the corpora it loses nothing (D3):

| Reading of « a line not in capitals » | « CA » read as the acronym | of them shouted « ça » | « CA » beside an acronym (« CA HT ») kept |
|---|---|---|---|
| **The word's neighbours are not in capitals** (chosen) | 31 | 0 | no — none written in the corpora |
| The block holds a word in lowercase | 33 | 2 | yes |
| At most two words in capitals in a row | 31 | 0 | yes — but « CA VA » in a line of lowercase words would read as the acronym; none written either |

It also reads a selection as the page: a word's neighbours are read in the text the analysis is
given, so « CA SUFFIT » selected from « Il a crié : CA SUFFIT ! » reads as the page does; only a
selection cut right at the word (« CA » alone, from that shout) reads it alone (*Risks*).

**One function for the four callers.** `alone_in_capitals(text, tokens, studied)` in
`analysis/pipeline.rs` answers, for each token, steps 1 and 2; `resolve_lemmas` takes the answer and,
for French, runs step 3 (`french::acronym`) before the cascade. `analyse_document`,
`headword_reading`, `gloss_phrase` and `word_grammar` call it over their own text, so a page, a
selection, a card and an expression key read « CA » alike.

*Rejected — the block.* The owner's words read literally; measured above, two shouted « ça » of 35
read as the acronym (« LES RÉSEAUX SOCIAUX CA RENDS LES GENS FOU… », « OUAH CA CHAUFFE…… »), and
a selection holding only words in capitals would read them otherwise than its page.
*Rejected — a run of at most two words.* It would keep « CA HT » (chiffre d'affaires hors taxes) and
read « CA VA » as the acronym; neither is written in 23.5 million tokens, and the rule is simpler to
state with no count. Open question 1.
*Rejected — a cased lookup of every word in capitals.* 43b's option B: with the section's 609
acronyms in the pack, `LE`, `SI`, `VA`, `EAU` of a line in capitals read as acronyms unless the
line is checked, and 60 rows that collide with a word need a review each anyway.

### D2 — A reviewed table; in the pack, a form in capitals that is its own lemma

**The table** is `ACRONYMS` in `scripts/lingua-data/reduce_french_acronyms.py`, a French rule module
of its own, imported by fr-en's reducer (French's reference, which writes `tables/fr/`) and by
fr-es's (which glosses it, D4) — as `reduce_french_treebank.py` is a rule module both read
(refine-lingua-fr-es-glosses D8). It is a rule file of each (`pack_sources.rule_files`): editing it
re-pins fr-en and fr-es, and no other pair. One row:

```python
ACRONYMS = {
    "CA": (
        ("conseil d'administration", "chiffre d'affaires", "courant alternatif", "comptable agréé"),
        "NOUN",
        "CA written in capitals is the acronym (the owner's decision of 2026-10-10, option A of "
        "read-lingua-french-ca-as-ca-cedilla D2): its expansions as the English Wiktionary's French "
        "CA entry names them; « ca » and « Ca » read as ça (UNMARKED_SPELLINGS)",
    ),
}
```

A row is the acronym as written, its expansions in the order the section's entry names them, the
part of speech its entry gives, and its reason. Its test holds that every expansion is one the
section's own entry points at, so the table chooses among the dictionary's meanings and never writes
one of its own; that the acronym is wholly in capitals, two letters or more; and that its lower case
is a form of the tables (a row is only needed where the lowercase reading would be another word's).

**What fr-en's reducer writes.** The section's entry headed `CA`, exactly so, is read for the table
and nowhere else: `tables/fr/forms.tsv` gains `CA` → `CA`, `tables/fr/lexical.tsv` gains `CA` (a
dictionary word: the names rule then keeps it, a capital in mid-sentence being otherwise a name, and
the knowledge model counts it); `freq.tsv`, `level.tsv` and `grammar.tsv` gain nothing (D5). 43b's
`ca` row is unchanged — `ca` and `Ca` stay forms of *ça*, `CAs` no form.

**In the pack: the existing forms map, keyed by the capitals.** The map is keyed by bytes and the
pool sorted by bytes: `CA` → `CA` is a form like any other, `CA` a lemma like any other (it sorts
before every lowercase lemma, id 0), and it reads as itself, as change 41b's builder requires of
every lemma. Every other lookup of the core lowercases its string first, so only step 3 of D1 ever
reaches a form in capitals; a core that predates the rule never would, and refuses the pack anyway
(the analyser version, D7). **The builder** accepts a form or a lemma holding a capital only in a pack
studying French, only when it is wholly in capitals, two letters or more, and is its own lemma; any
other capital in a table fails the build and names the form (a stray capital is an error, as today
it would silently be a form no lookup reaches).

*Rejected — a pack section of its own* (`acronyms`, acronym → lemma id): a section name, a reader, a
writer and a format test for what the forms map already does.
*Rejected — a lowercase lemma with a mark* (`ca.`, `c.a.`): a card and a status would show a spelling
no page writes, and the key could not be told from a word's.
*Rejected — the lemma `ca`*: 43b's D2 — one form, one lemma (M8); `ca` is *ça*'s form.
*Rejected — the table in fr-en's reducer alone, with a studied table for fr-es*
(`tables/fr/acronyms.tsv`): a seventh studied file for `pack_sources.py`, the builder and fr-es's
studied record, for one row a module both reducers import carries.

### D3 — « CA » in the corpora

Each token whose source is `CA`, by where it is written, through the prototype's engine:

| Corpus | Words | Words in capitals | Lines in capitals | `CA` alone among lowercase words | `CA` in shouted text |
|---|---|---|---|---|---|
| UD, seven French treebanks | 420,218 | 2,258 | 27 | 4 | 0 |
| Tatoeba, every French sentence | 5,767,417 | 1,561 | 4 | 0 | 0 |
| French Wikipedia (change 44c's sample) | 376,000 | 2,548 | 6 | 5 | 0 |
| Gutenberg (change 44c's four novels) | 226,576 | 873 | 108 | 0 | 0 |
| Change 39's corpus | 1,093 | 4 | 0 | 0 | 0 |
| French Wiktionary, usage examples | 16,590,790 | 37,502 | 84 | 22 | 4 (2 in lines in capitals, 2 in a run of capitals) |
| English Wiktionary's French section, usage examples | 254,519 | 479 | 3 | 0 | 0 |
| **All** | **23,636,613** | **45,225** | **232** | **31** | **4** |

(Words and lines as letter runs and corpus lines count them; the engine reads 23,467,957 tokens, 43,515
of them in capitals. One more `CA` stands inside a hyphenated model number, « LS-CA 1149 », read as
before.) **No headline or line in capitals writes the acronym**; the four shouted ones are all « ça »
(« OSKOUR IL GRÊLE NEIGE A PARIS CA FAIT BOBO SUR LA TÊTE », « LE BOUG EST COMME CA »).

**What the 31 mean**, read one by one:

| Meaning | Count | fr-en's gloss of `CA` (D4) |
|---|---|---|
| *conseil d'administration*, an association's board (« Le CA remerciait ensuite… », « son conseil d'administration (CA) ») | 3 | right |
| *chiffre d'affaires* (« 4,1 MF de CA en 1998 », « +6 chiffres de CA ») | 2 | right |
| *comptable agréé*, a professional title | 1 | the dictionary's, but no pair glosses it |
| Geneva's *Conseil administratif*, a city's executive (« le CA de déposer un projet d'arrêté ») | 4 | close (« board ») |
| A sports club, *club athlétique* (« CA Peñarol », « CA Vitry », « CA Montreuil 93 », « CA Brive »…) | 6 | wrong: a name |
| *Corps d'armée* (« le 10e CA ») | 2 | wrong (the French Wiktionary's sense, open question 2) |
| The Canadian dollar (« 737 500 $ CA ») | 2 | wrong |
| A certificate authority, English (« autorité de certification (CA, Certificate Authority) ») | 2 | wrong |
| The Latin syllable (« la palatalisation des CA et GA ») | 2 | wrong |
| A geometry segment (« toute droite CA ») | 2 | wrong |
| A cruiser's hull code (« Chicago (CA-14) »), a catalogue code, a notation | 3 | wrong |
| *Charbons actifs* (« des CA [charbons actifs] ») | 1 | wrong |
| Unclear (« tendu comme un CA de l'Asso ») | 1 | — |

With 43b alone all 31 read as `ça`, « that », a function word: wrong every time. Before 43b they read
as fr-en's old `ca`, « board of directors »: right for the three boards. With this change the card
names the right meaning 5 times, a close one 4 times, and the acronym is at least read as a noun to
learn rather than a pronoun. These corpora are edited text; no business press, where the shares of
the meanings may differ, was in reach.

**Its frequency**: 31 in 23.5 million tokens, 1.3 per million (Zipf ≈ 3.1), about the tables' rank
16,000 (`académicien`, 16,278, has Zipf 3.10). wordfreq cannot tell it apart: it lowercases, and `ca`
(Zipf 5.60) is mostly `ça`.

### D4 — Each pair glosses the acronym from its own glosses of the expansions

A pair's gloss of an acronym row is, for each expansion in the row's order, the pair's own gloss of
it — its expression table's for a headword with a space, its word gloss otherwise — cut to its first
sense (up to the first « ; »), the senses joined by « ; », one run of the row's part of speech; an
expansion the pair does not gloss is passed over, and a row none of whose expansions it glosses has no
gloss (the card shows the acronym without one). Measured on the committed tables:

| Pair | Gloss of `CA` | Runs |
|---|---|---|
| fr-en | « board of directors; revenue, turnover, net sales; alternating current » | `NOUN:3` |
| fr-es | « Administración, consejo de administración; Ingreso, cifra de negocios, facturación; Corriente alterna » | `NOUN:3` |

`comptable agréé` has no gloss in either; `courant alternatif`'s second sense, « domestic electricity
supply », is cut. Each sense is a gloss people wrote in a dictionary, in the reader's language, for
the expansion the acronym stands for (M5, D4 of the Spanish programme): nothing is translated or
pivoted, and the French expansion itself is not shown — a gloss is never in the studied language.

*Rejected — the shared rules as they are.* `reduce_common.reduce_gloss` lends a pointer's word the
gloss of its first base that has one: « board of directors » alone, fr-en's old `ca`, losing
*chiffre d'affaires*; and fr-es, which never reads the English section, would have no base at all.
*Rejected — a gloss written in the table per native language.* A person's own gloss is the optional
`add-lingua-curated-word-glosses`'s question, outside the programme's counts; M5 keeps glosses to
what dictionaries say.
*Rejected — every sense of every expansion.* « board of directors; revenue, turnover, net sales;
alternating current; domestic electricity supply »: the electricity supply is not what « CA » means.

### D5 — No rank, no level; a dictionary word

`CA` has no rank: wordfreq lowercases, and a rank counted from these corpora (about 16,000, D3) would
be the tables' one rank not from wordfreq, and inserting it would move every rank after it (43b moved
59,851 ranks for three removals). So `CA`:
- is never presumed known — calibration reads a rank, a declared level reads a level — and stays
  unknown until the reader says otherwise;
- carries no level and is never seeded into a deck by level;
- stays outside the vocabulary estimate, which counts ranked dictionary words (the golden's universes
  do not move);
- is still counted on a page, as any word the reader does not know;
- has a card that says « Rare — beyond the 20,000 most frequent words », the card-frequency
  requirement's unranked case (open question 3).

It is a dictionary word (`lexical.tsv`), so the names rule keeps it — « Le CA de l'association… » has
`CA` in mid-sentence, never in lowercase — and it is glossed. It is the one lemma fr-en glosses that
is not ranked: change 48's scenario *A gloss is its lemma's* (« every lemma `tables/fr-en/gloss.tsv`
glosses is a ranked lemma that `tables/fr/forms.tsv` reads as itself ») holds for every other, and the
acronym rows are its exception, which this change's requirement states (48 is open: no MODIFY).

### D6 — The card, the status and the key keep the capitals

- **The core.** `word_grammar` lowercases the dictionary form it is asked about; it keeps it as asked
  when the pack holds it as a lemma (`CA`) and lowercases it otherwise, as before — every other lemma
  is lowercase already, so every other answer is byte for byte. The card of « CA » (measured): the
  gloss of D4 under one noun group, no reading, no other dictionary form, no pieces.
- **The extension.** Today `reading/session.ts` keys a gesture's status and card by
  `g.lemma.toLowerCase()`, and `reading/selection-card.ts` asks a word card's gloss for
  `g.lemma.toLowerCase()`, which would file `CA` under `ca` and ask the gloss of a lemma the pack does
  not hold. With this change a word keeps the dictionary form the engine gave, and only an
  expression's key — built from the selection's text — is lowercased, as before. Every lemma the
  engine gives today is lowercase, so nothing else moves. The card shows « CA » as its headword, no « forme vue » line (the same letters, case aside),
  one listen button.
- **Further on.** The knowledge state, the deck, review, sync (cards and statuses are stored as sent)
  and backup v3 carry `CA` as any lemma: no format, proto or field changes. No reader holds a French
  status, so nothing is migrated.

### D7 — French's analyser version is bumped; what moves

**The version.** `FRENCH_ANALYZER_VERSION` `1.2.0` → `1.3.0` on today's `main` (the next number if
another French bump lands first; the spec names none), its doc line « `1.3.0` since
keep-lingua-french-acronyms-in-capitals reads a word standing alone in capitals as the acronym its
pack lists ». English and Spanish keep theirs. French packs at `1.2.0` are refused, as any.

**The tables, pins and packs** (on 43b's prototype tables):

| File | Before → after |
|---|---|
| `tables/fr/forms.tsv` | 124,101 → 124,102 forms: `CA` → `CA`, the first line |
| `tables/fr/lexical.tsv` | 26,484 → 26,485 words: `CA` |
| `tables/fr/freq.tsv`, `level.tsv`, `grammar.tsv`, `tags.tsv`, `studied.json` | byte for byte |
| `tables/fr-en/gloss.tsv`, `senses.tsv` | `CA` « board of directors; revenue, turnover, net sales; alternating current », `NOUN:3` |
| `tables/fr-es/gloss.tsv`, `senses.tsv` | `CA` « Administración, consejo de administración; Ingreso, cifra de negocios, facturación; Corriente alterna », `NOUN:3` |
| both `mwe.tsv`, `NOTICE` | byte for byte: no headword of fr-en's 17,635 or fr-es's 12,174 holds a word standing alone in capitals, so no key moves |
| Coverage | unchanged (it counts the ranked top 5,000 / 10,000 / 20,000) |
| `manifest.json` | `analyzer_version`, and `pack_version` with the rule digests (the new module is a rule file of both) |
| `pin.json` | fr-en: `reducer`, `pack`; fr-es: `reducer`, `studied` (`forms.tsv`, `lexical.tsv`), `pack` |
| The packs | fr-en 2,537,588 → 2,535,748 B; fr-es 1,971,114 → 1,969,484 B. `CA` takes lemma id 0 and every other id moves by one, which the forms map (−746 B) and the paradigms (−750 B) encode slightly tighter; the glosses −348 B in fr-en, +33 B in fr-es; the new rows +9 B |
| `testdata/fr-en/manifest.json` | `analyzer_version` |

**The goldens.** Re-blessed once on the prototype, each of `fr-en.golden` and `fr-es.golden` moves
19 lines and gains 8:

| Probe | fr-en | fr-es |
|---|---|---|
| `pack` | the bytes above, the version, `pack_version` | the same |
| the 17 `analyse` probes | `analyzer_version` alone | the same |
| `phrase-gloss Le CA a voté le budget.` | `CA`: `ça` « that (distal demonstrative pronoun)… », a function word → `CA` « board of directors; revenue, turnover, net sales; alternating current », no function word | `ça` « (coloquial) Eso, esto, aquello » → `CA` and its Spanish gloss |
| *added* `phrase-gloss LE CA A VOTÉ LE BUDGET` | a headline in capitals: `CA` reads as `ça`, as 43b reads it | the same |
| *added* `phrase-gloss Il a crié : CA SUFFIT !` | a shout in a line of lowercase words: `CA` reads as `ça`, and the expression `ça suffit` is met | the same |
| *added* `phrase-gloss Ca fait du bien.` | a capital initial: `ça`, and `faire du bien` | `ça`, no expression |
| *added* `word-grammar CA CA` | the card: the gloss under one noun group, no reading | the same, in Spanish |

`fr-en.golden` 376,285 → 381,018 B, `fr-es.golden` 250,296 → 252,967 B (on 43b's prototype).
Every other probe is byte for byte — the vocabulary estimates and the decks among them — and the
studied side stays one (`the_golden_is_the_french_one_on_the_studied_side`). The phrases go before
« l’homme », after 43b's four; change 44d's four (merged since, #881) stand earlier in the list, so the
implementation re-blesses on a `main` holding them, its goldens larger than the prototype's by theirs.
`french_baseline.rs` follows: « this core is 1.3.0 », and 32 reference grammar probes. The extension's
`test/word-card-fr-en.spec.ts` and `word-card-fr-es.spec.ts` render every grammar and phrase probe:
their snapshots gain the four cards and nothing else, their counts one grammar probe (92 → 93) and
three phrases.

**en-fr, es-fr, es-en and en-es cannot move**, by construction and on the prototype:
- the rule runs only for a text read as French (`alone_in_capitals` answers no for every English and
  Spanish token, and `resolve_lemmas` runs `french::acronym` for French alone);
- the form in capitals is written only to `tables/fr/`; no table of theirs changes;
- `reduce_french_acronyms.py` is imported by `reduce-fr-en.py` and `reduce-fr-es.py` alone, so only
  fr-en's and fr-es's rule digests move (`pack_sources.rule_files`); `reduce_common.py` and the
  edition modules are untouched;
- `word_grammar` keeps a lemma's capitals only when the pack holds it so: every English and Spanish
  lemma is lowercase, so every answer is the same; the extension's key change is a no-op for every
  lemma the engine gives them;
- English's and Spanish's analyser versions do not move;
- on the prototype, without re-blessing: `english_baseline`, `spanish_baseline`, `es_en_baseline`,
  `en_es_baseline`, `cross_native`, `parity`, `card_gloss_language`, `deck`, `languages`,
  `soft_hyphens`, `statuses`, `spanish_expression_keys` and `reprofile` pass; `lingua-core`'s 487 unit
  tests and its fixture suites pass; `lingua-pack`'s `committed_tables.rs` fails only on fr-en's and
  fr-es's pins, fr-es's studied record, and change 48's ranked-gloss check (D5), which the re-pin and
  the test's exception record.

### D8 — What it costs

The prototype read every token's letters to tell its case; the analysis of French, detection
included, over the corpora, rule on and off, interleaved, on a machine shared with other sessions:

| Run | Corpus | Off | On | |
|---|---|---|---|---|
| best of 5 | UD, Tatoeba, Wikipedia, Gutenberg (6.74 M tokens) | 6.778 s | 6.836 s | +0.9 % |
| best of 5 | the same without Tatoeba (1.01 M) | 1.046 s | 1.078 s | +3.1 % |
| best of 5 | the Wiktionaries' examples (16.7 M) | 17.491 s | 18.100 s | +3.5 % |
| median of 9 | UD, Tatoeba, Wikipedia, Gutenberg | 7.025 s | 7.110 s | +1.2 % |
| median of 9 | without Tatoeba | 1.087 s | 1.126 s | +3.6 % |

So +0.9 to +3.6 % of French analysis time natively; WebAssembly was not measured. The implementation
stops reading a token at its first lowercase letter — most words at their first — so the prototype's
figure is an upper bound. A reader of English or Spanish pays nothing. A word card and a selection add
one pass over a few tokens.

## Risks / Trade-offs

- **[The acronym's meaning is often another]** → measured (D3): the dictionary's meanings fit 6 of 31,
  the gloss 5; a club or a code reads with them. Still a noun to learn, never « that »; open
  question 2 offers *corps d'armée*.
- **[A selection cut right at the word]** → « CA » selected alone out of a shout reads as the acronym
  while the page read it as `ça`; a reader selecting one word of a shout is rare, and the card of a
  click (the page's token) reads as the page.
- **[An acronym beside another in capitals]** → « CA HT », « le CA SNCF » read as `ça`; none in 23.5
  million tokens (open question 1).
- **[A one-word block]** → a table's header « CA » stands alone, and reads as the acronym (rightly
  in a financial table).
- **[More rows later]** → each row is a person's decision with its measurement; the 59 other
  acronyms that read as a common word in lower case are recorded, not proposed (*Not here*).
- **[43b and this change land apart]** → 43b's 6.4: no package lists a French pair until both have
  merged; this change is written on 43b's tables and archives after it.

## Migration Plan

Nothing to migrate: no package lists a French pair, no reader holds a French status, card or level.
Rollback is a revert of the rule, the version, the module, the re-reduced tables, the pins and the
re-blessed files.

## Not here

- **Other acronyms.** 43b counted the 60 the section writes in capitals that read as another word in
  lower case: `LE`, `NE`, `SE`, `SA` (*société anonyme*, read as *son*), `SI`, `VA`, `EAU`, `CAS`,
  `MUR`, `BAC`, `TIR`, `ARC`, `CV`, `SVP`, `MDR`… A row each is a person's decision, measured as `CA`
  was; the owner asked for `CA` alone at first. The section's other acronyms read through their lower
  case as before, as a word of their own where the tables rank one (`ONU` as `onu`).
- **The expansion on the card** (« CA — conseil d'administration »): a line in French on the card is a
  card change of its own, and a gloss is never in the studied language.
- **English and Spanish acronyms** (« US » read as `us`): the same mechanism could serve them; it would
  move en-fr or es-fr, which the programme's rule keeps still, and no one has asked.
- **« C.A. » with its periods**: the tokeniser reads two single letters.

## Effort

2–3.5 ideal days, outside the programme's estimates, as 43b's design sketched option A: the rule,
`word_grammar` and their tests 0.5–1; the module, both reducers, the builder's guard, the re-reductions
with their pins, READMEs and `SOURCES.md` 0.75–1.25; the extension's key and its tests 0.25–0.5; the
probes, the re-blesses, the snapshots and the gates 0.5–0.75.

## Open questions for the owner

1. **Where does « a line not in capitals » stop?** Recommended: at the word's neighbours — « CA »
   reads as the acronym when the words right before and after it are not in capitals. So « Il a crié :
   CA SUFFIT ! » reads « ça » (the shout), and « Le CA a voté » the acronym. The price: « un CA HT de
   2 M€ » reads « ça » too, because « HT » is in capitals; none of the 23.5 million tokens writes it.
   The alternatives: the whole paragraph (then the shout reads as the acronym — 2 such lines in the
   corpora), or allowing one other word in capitals beside it (« CA HT » kept, but « dis-moi si CA
   VA » would read as the acronym).
2. **What should the card say « CA » means?** As designed, the English dictionary's four meanings,
   each in the reader's language from that pair's own dictionary: in English « board of directors;
   revenue, turnover, net sales; alternating current », in Spanish « Administración, consejo de
   administración; Ingreso, cifra de negocios, facturación; Corriente alterna » (« comptable agréé »
   has no gloss in either). In the corpora this is right 5 times in 31 and close 4 (Geneva's council);
   clubs (« CA Brive ») are the commonest use. Keep the four? Add the French dictionary's « corps
   d'armée » (seen twice; English « corps », no Spanish gloss)? Drop « courant alternatif » (never
   seen)?
3. **« CA » has no frequency rank.** Its card says « Rare — beyond the 20,000 most frequent words »,
   no reader is assumed to know it, and no deck adds it by level. Measured, it is about the
   16,000th word. Acceptable as designed, or should a rank be estimated (which moves every rank after
   it, and every golden line that shows one)?
