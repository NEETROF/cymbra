# Design

## Context

See proposal.md — Why. What constrains the shape:

- **Who decided.** No one on the team reads Italian (I2). An Italian-specialist review commissioned
  by the owner on 2026-10-24 — not I2's reviewer agent — read, on his behalf, what changes 5, 6 and
  7 left the owner to read: `pages-it.txt` and the two
  excerpts' attributions (change 5's task 5.1), D3's elided forms, D4's articulated prepositions
  and D5's truncations (change 6's 4.1), D3's closed classes and their left-out words, D4's names
  rule and D2's cascade (change 7's 9.1). The owner, Guillaume Fortin, approved every
  recommendation on 2026-10-24 (« Tout appliquer »), the optional ones included. Each decision
  below gives the specialist's reason and an example; what the review approved as it stands is
  listed at the end of D6.
- **Where things stand on main** (2026-10-24): changes 5–11 are implemented, none archived;
  Italian's analyser is at `1.3.0` (change 11). `tokenize.rs` holds `ITALIAN_ELISIONS` (38),
  `ITALIAN_CONTRACTIONS` (26), `ITALIAN_ELIDED_PREPOSITIONS` (5), `ITALIAN_TRUNCATIONS` (8) and
  change 10's `ITALIAN_APOCOPES` (16: `a`, `ca`, `co`, `de`, `fe`, `fra`, `i`, `me`, `mi`, `ne`,
  `pe`, `que`, `su`, `to`, `ve`, `vo`), the words its accent rule never reads. A truncation's read
  word is final; `diss'` is the one reading that looks at the next word (`dissi` before `io`).
- **The requirements this change modifies** are held by open changes: *Italian tokenisation
  pre-pass* by change 10 (which modified change 6's), *The apostrophe that stands for an accent…*
  by change 10, *Italian closed classes* by change 7. Each MODIFIED block here copies the latest
  holder's whole and the change archives after 6, 7 and 10.
- **The reducer mirrors change 6's tables.** `reduce-it-fr.py` keeps `ELISIONS`,
  `ELIDED_PREPOSITIONS`, `CONTRACTIONS` and `TRUNCATIONS` as reviewed tables, and
  `test_reduce_it_fr.py` (`test_the_closed_tables_are_change_6_s`) holds them to `tokenize.rs`.
  Change 9's requirement makes every word the pre-pass writes a form of a ranked lemma, each elided
  piece and truncation a form of the word it stands for, no plain word beginning with an elided
  piece a form, and the pieces' wordfreq stems no ranked words; `committed_tables.rs`
  (`spec_scenario_every_word_italian_s_pre_pass_writes_is_a_form`) checks it on the committed
  tables. So a new piece is not done until it-fr is reduced again — the owner's `lingua-pack-update`
  dispatch in mode `reduce`, as changes 10 and 11 had (runs 40002448797, 40013770231). The analyser
  version, which the it-fr manifest records, needs the same dispatch.
- **Figures.** Zipf values are wordfreq 3.1.1's Italian list (the reducer's pinned version), which
  keeps an elided word whole (`trent'anni`, not `trent` + `anni`). Ranks are the committed
  `tables/it/freq.tsv`'s on 2026-10-24. Book counts are change 10's corpus (its task 2.2: the
  `pages-it` baseline, change 8's two Gutenberg novels, its Italian Wikipedia sample, its
  20,000-sentence Tatoeba sample and MarkIT's training and development sections, 990,502 words).

## Goals / Non-Goals

**Goals:**

- The test pages read as Italian writes, so that the golden probes Italian, not oddities.
- Every elision and truncation the review found is read as the words it stands for, its pieces
  glossed, ranked and counted as the words they are.
- `mo'` and `dai` read by their context where one rule tells their readings apart, measured before
  it ships.
- The closed classes hold the words the review found missing, and keep out the words it confirmed.
- English, Spanish, French, the six goldens and every pin of the six pairs do not move.

**Non-Goals:**

- Other context rules (`s'` read `se`, `d'` read `da`, `pover'` read `povera`): the review raised
  none; change 6's readings stand.
- `pe'` (*pei*, 3 in change 10's corpus): it stays one of the apocopes. The review did not name it,
  and change 6 keeps `pel` and `pei` out of `ITALIAN_CONTRACTIONS` — archaic, read whole —: reading
  `pe'` as `per` + `i` would split the truncation of a word whose full form the pre-pass keeps whole.
- The noun homographs the closed classes hide (D6, *Cost*): unchanged, as in Spanish and French.
- The gloss of the new pieces' words: it-fr's glosses are change 16's.

## Decisions

### D1 — The test pages: four sentences made natural, four optional ones taken

The corpus keeps its thirteen pages, their phenomena and every word a probe reads (`LEMMAS`,
`PHRASES`, `GRAMMAR` in `support/italian.rs`); the lines below are replaced or added.

| Page | Today | After | The specialist's reason |
|---|---|---|---|
| `grammatica` | « Le lettere che ho scritto sono arrivate ieri; le ho lette subito. » | « Le lettere che mi hai scritto sono arrivate ieri; le ho lette subito. » | one's own letters arriving and being read at once is a scene nobody writes; the letters someone wrote to me are. The participles' agreement (`scritto`, `arrivate`, `lette`) and the clitic `le` stay, `mi` and `hai` join |
| `ricetta` | « Aggiungete 200 g di farina, 1 l di latte e sale q.b., poi mescolate per 10 minuti a fuoco basso. » | « Sbattete 6 uova con 50 g di parmigiano e sale q.b., poi cuocete per 10 minuti a fuoco basso. » | a *frittata* is eggs beaten and cooked, never flour and milk (that is a batter). The imperatives, `g`, `q.b.`, `sale`, the minutes and the low heat stay, and so does the next line's `frittata` (a `LEMMAS` probe); `farina` stays on `preposizioni`; `1 l`, which no probe reads, leaves |
| `tecnico` (the NFD block) | « … e l'attività del servizio è più stabile di prima. » | « … e l'affidabilità del servizio è più stabile di prima. » | a service's *activity* is not « stable »; its reliability is. The elision `l'` and a final accent written as a combining mark stay: the block is still wholly NFD |
| `elisioni` | « Com'è possibile che sia sparita nell'armadio? » | « Com'è possibile che non sia più nell'armadio? » | a key does not « vanish into » a wardrobe; it is no longer in it. `com'è`, `sia`, `nell'` stay, `non … più` joins |
| `elisioni` (optional) | « Verrai alla festa? Senz'altro! E la serata è stata tutt'altro che noiosa. » | « Verrai alla festa? Senz'altro! E sarà tutt'altro che noiosa. » | the answer to a question about the future cannot be in the past. `senz'altro` and `tutt'altro` stay |
| `notizie` (optional) | « dello 0,3 % », « al 3,5 % » | « dello 0,3% », « al 3,5% » | Italian sets the sign with no space (I6: 96%) |
| `informale` (optional, added) | — | « aspetta un pò, qual'è il problema? » | two of the web's commonest misspellings: `pò` (read *poco* by change 10's owner row) and `qual'è` (D2) |
| `grammatica` (optional, added) | — | « La ringraziamo, Le risponderemo al più presto. » | the courtesy form, with its capital in mid-sentence, which no page wrote; the page writes `le` in lowercase too, so the names rule does not set « Le » aside |

*Judgement, recorded:* the specialist named « 0,3% »; « 3,5% » on the same line of the same page
follows the same rule and is changed with it. The courtesy sentence goes to `grammatica`, the page
of verb forms and pronouns.

The specialist also read the two excerpts' attribution in `support/italian.rs` and Dante's phrase
probe: the sources and licences are right — Manzoni's 1840 edition (Manzoni died in 1873), Collodi's
1883 *Pinocchio* (Collodi died in 1890) and Dante (died in 1321) are in the public domain.

### D2 — Eighteen elisions and two elided prepositions

Each is an elision Italian writes before a vowel; each, missing from the closed list, is read today
as one unknown word with no gloss, and the commonest are ranked words of the committed tables.

| Elided | Read as | Example | Zipf (whole word) | Rank today |
|---|---|---|---|---|
| `trent'` | `trenta` | « trent'anni » | 4.15 | 3,522 |
| `qual'` | `quale` | « qual'è » — a misspelling of « qual è » (no elision: `qual` is an apocope), the web's commonest | 4.03 | 4,273 (`qual'e` 21,141, `qual'era` 26,819, `qual'é` 50,051) |
| `quarant'` | `quaranta` | « quarant'anni » | 3.72 | 7,076 |
| `cinquant'` | `cinquanta` | « cinquant'anni » | 3.67 | 7,569 |
| `tant'` | `tanto` | « tant'è », « tant'altro » | 3.66 | 7,781 |
| `fin'` | `fino` | « fin'ora », « fin'allora » | 3.37 | 11,870 |
| `gliel'` | `glielo` | « gliel'ho detto » | 3.31 | 12,955 (`gliel'ha` 15,707, `gliel'hanno` 23,640, `gliel'hai` 38,790, `gliel'aveva` 45,495) |
| `sessant'` | `sessanta` | « sessant'anni » | 3.27 | 13,788 |
| `ultim'` | `ultima` | « l'ultim'ora », its one common use, feminine | 3.26 | 14,006 |
| `null'` | `nulla` | « null'altro » | 3.12 | 16,864 |
| `ottant'` | `ottanta` | « ottant'anni » | 3.09 | 17,558 |
| `settant'` | `settanta` | « settant'anni » | 3.09 | 17,601 |
| `neanch'` | `neanche` | « neanch'io » | 3.01 | 19,538 |
| `prim'` | `primo` | « prim'ordine », « prim'attore » — the masculine, as `l'` reads `lo` | 2.99 | 20,093 |
| `ventiquattr'` | `ventiquattro` | « ventiquattr'ore » | 2.96 | 20,952 |
| `diciott'` | `diciotto` | « diciott'anni » | 2.76 | 26,665 |
| `novant'` | `novanta` | « novant'anni » | 2.60 | 32,304 |
| `ogn'` | `ogni` | « ogn'uno », literary | 1.17 | — |

The two elided prepositions join D4's table of change 6: `coll'` → `con` + `lo`, sharing the span,
as `all'` is (« coll'aiuto », literary; 1.36), and `degl'` → `degli`, whole, as `dell'` is read
`dello` (« degl'Italiani »; 2.04): the `di` family stays whole (I7).

Every read word is already a form of the committed tables (`ultima` of *ultimo*, the others of
themselves), each of a ranked lemma. The plain `fin` stays a word — an apocope of *fino* written
without an apostrophe (« fin qui », « fin da »; Zipf 5.04) — and `qual` already is one (`APOCOPES`).

*Left out — `oltr'`.* « oltr'alpe » is 1.18 against « oltralpe » 3.04: Italian writes the word
whole, and a piece would read `oltre` + `alpe` for a word the dictionary has.

### D3 — Truncations: three more words, and a truncated articulated preposition read as two

**Three truncations**, one word each, added to `ITALIAN_TRUNCATIONS`:

| Written | Read as | Example | In change 10's corpus | Today |
|---|---|---|---|---|
| `de'` | `dei` | « Lorenzo de' Medici » — the `di` family's, read whole as `dell'` is | 613 | `de` |
| `que'` | `quei` | « que' due », *I promessi sposi* | 267 | `que` |
| `ca'` | `casa` | « Ca' Foscari », « Ca' d'Oro » | 17 | `ca` |

**A new case: a truncation read as two words**, as an articulated preposition is read (I7). The
books write `ai`, `coi`, `nei`, `sui` and `dai` truncated before a consonant; each is read as its
preposition and `i`, the two tokens sharing the span, apostrophe included, the first keeping the
written capital — the shape of change 6's `all'` (`a` + `lo`). `a'`, `co'`, `ne'` and `su'` are read
so **only when a word follows them**, nothing but a space between; `da'` by D4's context:

| Written | Read as | In change 10's corpus | Today |
|---|---|---|---|
| `ne'` | `in` + `i` | 106 | `ne` (the pronoun) |
| `a'` | `a` + `i` | 99 | `a` |
| `co'` | `con` + `i` | 81 | `co` |
| `da'` | `da` + `i` — but the verb in D4's contexts | 68, Manzoni's preposition (« da' primi suoi anni »), as change 10 measured | `dai` (*dare*) |
| `su'` | `su` + `i` | 13 | `su` |

They take a table of their own beside `ITALIAN_ELIDED_PREPOSITIONS` (written, preposition,
article), read as a truncation is — the seam keeps the apostrophe when no quote opened before the
word is still open — and their read words are final. `da'` moves there from `ITALIAN_TRUNCATIONS`.

**Only before a word** (decided at review, 2026-10-24). D4's own argument: the preposition always
has its noun phrase after it, and is never followed by punctuation. So `a'`, `co'`, `ne'` and `su'`
before a comma, a stop, a closing quote or the end of the text stay what they are today — apocopes,
read `a`, `co`, `ne`, `su`, the apostrophe outside the word — and that apostrophe closes a quote as
any does: « 'Vieni su', disse, e un po' » still closes after `su`, and `po'` still reads `poco`. A
quoted phrase ending on one of them is not read as a preposition with no noun. D5 counts the
occurrences of each before a word and before anything else.

**Change 10's apocopes, reconciled.** `ITALIAN_APOCOPES` listed `ne`, `de`, `co`, `a`, `su`, `que`
and `ca` only to keep its accent rule off them (« ne' » would read `né`, « su' » `sù`, « que' »
`qué`). The pre-pass now reads `de'`, `que'` and `ca'` wherever they stand, and the accent rule
never tries a word the pre-pass lists, so those three leave it. `a`, `co`, `ne` and `su` stay — the
pre-pass reads them only before a word, and with nothing after them they must still never read
`né` or `sù` —, beside the nine it never reads: `fe'`, `fra'`, `i'`, `me'`, `mi'`, `pe'`, `to'`,
`ve'`, `vo'` — thirteen in all. The accent requirement's list and its scenarios follow.

*Cost, accepted — `ne'` for *né*.* The web sometimes writes `ne'` for *né* (« ne' l'uno ne'
l'altro »): read `in` + `i`. Today it reads `ne`, the pronoun, which is wrong in both cases; the
books' `ne'` (*nei*) is now right.

*Left out — `to'`.* It is the interjection « tiens ! » (« to', guarda chi c'è »); reading it as
`togli` would show « enlever ». It stays an apocope, read `to`, its apostrophe outside the word.

### D4 — Two words read by their context

**`mo'`.** « a mo' di esempio » (*by way of*) is `modo`; « mo' arrivo », « e mo'? » (Roman and
southern Italian for *now*) is `mò` (Zipf 3.18; a form of itself in the committed tables, ranked
15,528). The rule: `mo'` is `modo` when the next word, as read, is `di` — so « a mo' d'esempio »,
whose `d'` reads `di`, is `modo` too — and `mò` otherwise. It is the second reading that looks at
the next word, after `diss'`; `mò` is written as the token's text whatever the pack lists, as every
listed truncation's word is.

**`dai` and `da'`.** Today `dai` is always `da` + `i` (I7), which hides *dare* in « Dai, andiamo! »
(*come on*), « mi dai una mano? », « non dai mai retta », « Ma dai! »; `da'` is always the verb,
where books write the preposition. The specialist's rule, for both: the verb, one token `dai`
spanning the written word, when the word just before it — nothing but a space between — is `non`
or a clitic pronoun (`mi`, `ti`, `ci`, `gli`, `le`, `lo`, `la`, `ne`, `me`, `te`, `ce`, and
`glielo`, `gliela`, `glieli`, `gliele`, `gliene`), or when it is followed by `,`, `!`, `?`, `.`,
`…`, a closing quotation mark (« “Ma dai” disse ») or the end of the text; the preposition otherwise. Why it holds: the
preposition always has its noun phrase after it — it is never followed by punctuation —, and a
clitic or `non` is never followed by a preposition. `dai` keeps Zipf 5.90 across both readings;
wordfreq cannot tell them apart, hence the measurement (D5). `me` and `te` are the doubtful
triggers: they are clitics only before another pronoun (« me lo dai », where the word before `dai`
is `lo`), and stressed pronouns elsewhere (« vieni con me dai nonni? »); D5 keeps them only on their
own figures.

*Cost — the verb with no sign.* « tu dai il libro a Maria », « dai retta a me » read as the
preposition, as today. The reverse error needs a clitic-shaped word right before a preposition or
punctuation right after one; the measurement looks for it.

*Cost, unchanged — `da'` for *dà*.* « me lo da' » reads `dai`, change 10's named cost: the lemma
*dare* is right, the token text is not.

*Cost, accepted — a selection is read alone.* The phrase gloss reads a selection as a text of its
own, so its end is the end of the text: a selection ending « … torno dai » reads the verb, and one
ending « a mo' » reads `mò`, where the page around them reads the preposition and `modo`. The rule
is not restricted for it: a selection cut inside a phrase is the reader's, and the word card, which
is keyed by its dictionary form, is unaffected.

**A word card has no context.** The extension hands the card the written word alone (« dai », «
mo' »), and `word_grammar` already answers the piece whose dictionary form the card asks. For a
word this rule reads two ways it asks the pre-pass for both readings and answers the one holding
the card's dictionary form: (« dai », `da`) the pieces `da` and `i`, (« dai », `dare`) the verb,
(« mo' », `modo`) `modo`, (« mo' », `mò`) `mò`. Where neither reading holds the card's form, the
word is answered as it reads alone — `dai` and `da'` the verb (the end of the text follows them),
`mo'` as `mò` —, as `word_grammar` already falls back to the written word's first token. Without
it, a click on the preposition `dai` would answer the verb, since a word alone ends the text.

**The expression keys.** Change 12 reads a headword through the same pre-pass, so « a mo' di »
keeps its key `a modo di` and « dai e dai » reads as a page does. Its split families and its
truncation list (its D6) read change 6's tables as they are when it is implemented: whichever of
the two is implemented second covers the other's tables — task 1.7 here if change 12 is on main
first; a one-line pointer in change 12's D6 the other way. Nothing of change 12's spec moves.

### D5 — The measurement the implementation runs first, by criteria written here

**Corpus.** Change 10's running text without its held-out parts — the `pages-it` baseline, change
8's two Gutenberg novels, its Italian Wikipedia sample, its 20,000-sentence Tatoeba sample, MarkIT's
training and development sections (990,502 words) — and the whole Tatoeba export beside it. **UD
Italian-PUD and MarkIT's test section are read by nothing this change decides**; they stay `it-ud.sh`'s
gates after the dispatch, as before.

**`dai` and `da'`.** These criteria were fixed in this proposal before any figure was read; the
review of 2026-10-24 only clarified them — the denominator of 3, the fallback, `me` and `te`, the
closing quote —, still before any measurement:

1. *Gold.* In MarkIT's training and development sections, UD writes the preposition as a
   multiword token (`da` + `i`) and the verb as one `VERB` token: that is the answer. In the other
   texts, every `dai` and `da'` the rule reads as the verb is read by hand, and so is a seeded
   random sample of 100 it reads as the preposition (the seed in the pull request).
2. *Each trigger kept on its precision.* `non` before, a clitic before, punctuation after: each is
   kept only if at least 95 % of the words it reads as the verb are the verb. A trigger under 95 %
   is dropped before the rule ships, and the pull request says so. The clitic trigger's precision
   is reported per clitic, not gated per clitic — but for `me` and `te`, which stay triggers only if
   their own precision, the two counted together, reaches the same 95 %.
3. *The prepositions kept.* The rule ships only if no more than 1 % of the prepositions are read as
   the verb, the share being: the prepositions found among the hand-read verb readings, plus
   MarkIT's gold prepositions read as the verb, over MarkIT's gold prepositions plus the other
   texts' preposition readings.
4. *Not tuned.* No trigger is added, no clitic list widened and no threshold moved after the figures
   are read. The verbs the rule finds and the ones it misses (`tu dai`, `li dai`, `vi dai`) are
   reported, not gated.
5. *The fallback.* If the rule fails 3, it does not ship: `dai` stays `da` + `i` everywhere, and
   `da'` keeps change 6's reading, the one token `dai`. If a trigger is dropped by 2, the rule ships
   without it. Either way, task 2.1 makes the spec's block and its scenarios follow before the
   implementation merges — « Me lo dai? » and « Ma dai! » each rely on two triggers, and a scenario
   whose triggers are all gone is rewritten to the preposition —, as task 2.2 does for `mo'`.

**`mo'`.** Every `mo'` of the corpus is read by hand (they are few). The rule ships if at least 9 in
10 of its readings are right; otherwise `mo'` stays `modo` everywhere, as today, and the pull request
says so. The corpus is mostly written, literary and encyclopaedic Italian: if it holds no `mò`
reading at all, the criterion measures the `modo` branch alone, and the `mò` branch ships unmeasured
— said so in the pull request, on the specialist's word.

**The new pieces and truncations** are counted, per piece, on the same corpus and listed in the pull
request with a sample of each, not gated: any occurrence where a piece splits a word Italian writes
whole is listed for the owner, who may drop the piece before the dispatch. `a'`, `co'`, `ne'` and
`su'` are counted twice: before a word (read as two words) and before anything else (left as
apocopes).

### D6 — The closed classes

Added, each by the cascade's lemma, in its sorted table of `function_words.rs`:

| Class | Words | Zipf | Example | The specialist's reason |
|---|---|---|---|---|
| prepositions | `davanti`, `dietro` | 5.23, 5.26 | « davanti alla porta », « dietro la casa » | prepositions of place, as `sopra`, `sotto`, `dentro`, `fuori` already are |
| determiners | `ambedue`, `entrambe`, `entrambi` | 3.47, 4.78, 5.07 | « entrambi i libri », « ambedue i figli » | *both*, before a noun with its article |
| determiners | `parecchi`, `parecchia`, `parecchie`, `parecchio` | 4.40, 3.47, 4.15, 4.44 | « parecchie volte » | a quantifier, as `molto`, `tanto`, `poco`, `troppo` already are |
| conjunctions | `anziché`, `bensì`, `nonché`, `ossia`, `pertanto`, `qualora` | 4.36, 4.31, 4.73, 4.60, 4.70, 4.41 | « anziché partire », « non oggi, bensì domani », « il pane nonché il vino », « qualora venisse » | conjunctions, as `ovvero`, `perciò`, `quindi`, `affinché` already are |

The pack files `entrambe` under *entrambi* and `parecchi`, `parecchia`, `parecchie` under
*parecchio*; the inflected forms are in the tables too, as change 7's are, for a pack that keeps one
as a lemma of its own. `davanti` and `dietro` are also adverbs (« sta davanti ») — the same cost
`sopra` and `fuori` already carry. `is_preposition` reads the prepositions' table, so
choose-lingua-overlapping-expressions' D2 reads `davanti` and `dietro` as prepositions too; the
golden shows whether a page's chosen expression moves.

**Unchanged and left out**, with the reasons:

- `stare` (5.27) — the specialist's: « sto bene », « stare a casa » are beginner vocabulary, and a
  French speaker must learn that Italian says « sto bene », never « sono bene »; greying `stare`
  would hide the very word to learn. Change 7's measurement (13 of 24 auxiliaries in MarkIT's test
  section) had kept it out already.
- `volere`, `secondo`, `lungo`, `mai`, `più`, `anche`, `salvo` — change 7's reasons, confirmed by the
  specialist: a content verb (« voglio un caffè »), as `vouloir` and `querer`; everyday adjectives
  a preposition's row would hide (« il secondo giorno », « un lungo viaggio »); adverbs with content
  (*never*, *more* or *no longer*, *also*); the adjective *safe*. `anche` is now named in the
  requirement's list, as change 7's design already left it out.

**Approved as they are**: `eccetto`, `colei`, `costui`, `costei`, `costoro`, `circa`, `oltre`,
`fuori` (change 7's D3 and its measurement); D4's names rule; D2's cascade choices (the enclitic
rule's bases, the unlisted plural left out).

*Cost, known and unchanged — the noun homographs.* The tables flag by lemma, and « il potere »,
« il dovere », « un essere umano » share their verb's: the noun is greyed as a function word. Spanish
(« el poder », « el deber », « el ser humano ») and French (« le pouvoir », « le devoir », « l'être
humain ») carry the same cost; nothing moves here.

### D7 — The reducer, and what the owner's dispatch moves

`reduce-it-fr.py` follows the tables it mirrors, and `test_the_closed_tables_are_change_6_s` holds
it to them, the new table and the context words included:

- `ELISIONS` gains D2's eighteen; `ELIDED_PREPOSITIONS` gains `coll'` (`con`, `lo`) and `degl'`
  (`degli`); `TRUNCATIONS` gains `ca'`, `de'`, `que'` and loses `da'`, which joins a new
  `TRUNCATED_PREPOSITIONS` (`a'`, `co'`, `da'`, `ne'`, `su'`): neither forms nor ranks, as the
  elided `all'` is; `CONTEXT_WORDS` gains `mò` (`dai` is a form of *dare* already, `DAI`).
- `APOCOPES` gains `fin` → *fino* (its reading today, by its own entry), so its stem stays a word.
- `WORDFREQ_STEMS` follows from the tables: the new pieces' stems are no ranked words, as change 9's
  rule has it. The truncations' stems `ca`, `de`, `que`, `co` keep their reading: each is a lemma of
  the section with a rank (`de` 94, `co` 991, `ca` 1,932, `que` 2,729), and change 9's rule leaves
  such a stem a word, as it leaves `da`, `di` and `fa`. Accepted: wordfreq counts
  `de'` and `que'` as `de` and `que`, so `de` (94) and `que` (2,729) stay ranked on counts that the
  pre-pass now reads as `dei` and `quei`.

**Change 9's requirement, amended in place.** *Italian's forms and frequencies* — change 9's, and
change 20's copy under MODIFIED — said `dai` is « the truncation `da'`'s word » and made wordfreq's
stems of the elided pieces and truncations « no words », `cent` and the apocopes excepted, which
read literally makes `ca`, `de` and `que` no words. Both changes are open, so both sentences are
amended in place in this pull request, as change 56 (#1031) amended the two open YouTube changes:
`dai` is « a form of *dare* with no rank, the verb the pre-pass reads `dai` and `da'` as by their
context », and the stems are « the stems … that are no word of their own ». Nothing else of either
requirement moves, and no MODIFIED block is needed here.

Measured on the committed tables (2026-10-24), the dispatch moves:

- **24 ranked words beginning with a new piece** leave the forms and the ranks — `trent'anni` 3,522,
  `qual'è` 4,273, `quarant'anni` 7,076, `cinquant'anni` 7,569, `tant'è` 7,781, `fin'ora` 11,870,
  `gliel'ho` 12,955, `sessant'anni` 13,788, `ultim'ora` 14,006, `gliel'ha` 15,707, `null'altro`
  16,864, `ottant'anni` 17,558, `settant'anni` 17,601, `neanch'io` 19,538, `prim'ordine` 20,093,
  `ventiquattr'ore` 20,952, `qual'e` 21,141, `gliel'hanno` 23,640, `diciott'anni` 26,665,
  `qual'era` 26,819, `novant'anni` 32,304, `gliel'hai` 38,790, `gliel'aveva` 45,495, `qual'é`
  50,051 —, their pieces read as words the tables rank;
- **7 ranked stems** leave the ranks — `coll` 10,555, `trent` 14,193, `tant` 16,946, `null` 18,253,
  `prim` 23,164, `degl` 25,356, `ogn` 40,228; the ranks close up;
- `prim'` moves from *prima* (the section's reading) to *primo*; `fin'`, `ultim'`, `ogn'`, `ottant'`,
  `ventiquattr'`, `degl'` and `que'` become forms of their words (`cinquant'`, `diciott'`, `gliel'`,
  `neanch'`, `novant'`, `null'`, `qual'`, `quarant'`, `sessant'`, `settant'`, `tant'` and `trent'`
  are forms already); `da'` stops being a form of *dare*;
- `tables/it-fr/manifest.json` records `1.4.0`, `pin.json` and the README follow.

Until the dispatch, `committed_tables.rs` and `test_pack_sources.py`'s record of the pin fail on
the committed tables, as they did for changes 10 and 11; the implementation's pull request runs the
dispatch on its branch before it merges.

**Before change 16.** it-fr's coverage is measured over the commonest ranked lemmas, and `trent'anni`
sits in the first 5,000 with no gloss possible. 11a is best implemented before change 16 commits its
coverage, so that the floor is measured on the re-reduced base; it is not a prerequisite.

### D8 — The version, the fixture, the probes, and what the golden shows

`ITALIAN_ANALYZER_VERSION` becomes `1.4.0` (the next minor, if no other Italian rule lands first);
the fixture's manifest with it. The fixture (`scripts/lingua-data/testdata/it-fr/`) gains the words
the pre-pass now writes that it lacks — D2's read words, `quei`, `mò`, and `pò` → *poco* for the
`informale` line — so that `the_fixture_lists_every_word_the_pre_pass_writes` holds; the
`italian_fixtures.rs` cases cover every new entry and each trigger. The probes gain « Dai,
andiamo! », « mi dai una mano? », « a mo' di esempio », « Lorenzo de' Medici », « ne' campi »,
« trent'anni fa », « gliel'ho detto », and « davanti alla porta » and « ossia il primo » for D6's
flags, and the grammar pairs (« dai », `dare`), (« dai », `da`) and (« mo' », `mò`).

The golden moves once: the version line, D1's lines, the probes, and whatever D1's new lines read.
A page's tokens carry no function-word flag — only a phrase gloss's do —, so D6's additions show in
the two phrase probes above, not on the pages. The pages' `dai` —
Manzoni's « dai molti suoi cocuzzoli », `grammatica`'s « dai nonni », `preposizioni`'s « dai cugini »
— stay `da` + `i`. The six goldens, S0 and every pin of the six pairs are byte for byte.

### D9 — The owner tasks closed here, and what the archive does at merge

This pull request ticks change 5's 5.1, change 6's 4.1 and change 7's 9.1, each with « Done
2026-10-24: read in an Italian-specialist review commissioned by the owner on 2026-10-24; approved, corrections in
refine-lingua-italian-reading ». Each was its change's last open task, so 5, 6 and 7 become complete,
and the `openspec-archive` workflow runs on the merge.

It archives nothing. For each complete change it reads `openspec_archive_order.py` first; exit 10
means it waits, and `openspec archive` never runs on it — so no MODIFIED header is ever looked up
and nothing can abort silently. Checked on this branch:

- change 5 waits for `add-lingua-french-baseline`, `add-lingua-french-analysis`,
  `add-lingua-native-language-sync-client` and `ignore-lingua-soft-hyphens`, all four in progress;
- change 6 waits for change 5; change 7 for 5, 6 and the same French changes and soft hyphens;
- changes 8 and 9, complete already, still wait — 8 for 5, 6 and 7, 9 for 5, 6 and two other open
  changes —; this change is in progress and waits for 6, 7 and 10.

The workflow prints a notice for each waiting change and succeeds. Later, as the French changes
archive, the order is 5 → 6 → 7 → 8 and 9 → 10 → this change, each MODIFIED block finding the
requirement its predecessor left.

## Risks / Trade-offs

- [`dai` read as the verb where it is the preposition] → D5's criteria, written before the figures:
  a trigger under 95 % precision is dropped, and the rule ships only if 99 % of the prepositions
  stay prepositions.
- [The verb `dai` with no sign before or after it] → read as the preposition, as today; reported by
  D5, not chased by tuning.
- [`ne'` written for *né*] → `in` + `i`, accepted (D3); today's `ne` is wrong as well.
- [A word card on a word read by context] → answered by the card's dictionary form (D4), else as the
  word alone reads; a test per reading.
- [A selection ending on `dai` or `mo'`] → read as at the end of a text, the verb and `mò`; accepted
  (D4), the card unaffected.
- [`a'`, `co'`, `ne'`, `su'` at the end of a quoted phrase] → read as two words only before a word
  (D3), so the quote still closes.
- [The re-reduction moves ranks] → 31 ranked entries leave (D7) and the ranks close up; listed in
  the dispatch's pull request; `it-ud.sh`'s gates are rerun on the new tables.
- [A new piece splitting a word Italian writes whole] → D5 lists every new piece's occurrences for
  the owner before the dispatch.
- [The noun homographs] → unchanged, shared with Spanish and French (D6).
- [Change 12 and this change implemented in either order] → whichever is second covers the other's
  tables (D4).

## Migration Plan

Nothing deploys and nothing is released: no listed pair studies Italian before change 20. The
implementation's pull request carries the code, the fixture, the golden and the owner's `reduce`
dispatch's commit; rollback is a revert of that pull request, tables included.
