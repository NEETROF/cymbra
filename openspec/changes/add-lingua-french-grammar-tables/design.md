# Design — add-lingua-french-grammar-tables

## Context

See proposal.md (Why). Where French's word grammar stands, and what this design measured:

| What | Where, today |
|---|---|
| Word grammar in a pack | `add-lingua-word-grammar` (archived): the vocabulary — the 17 UPOS and `FEATURES` (`Mood` Cnd/Imp/Ind/Sub, `Tense` Fut/Imp/Past/Pqp/Pres, `VerbForm` Fin/Ger/Inf/Part, `Gender`, `Number`, `Person`, `Degree`, `Case`, `PronType`, `Reflex`, `Definite`); `grammar.tsv` (`form<TAB>lemma<TAB>tag<TAB>other\|-`); the builder files each reading under its lemma, a reading marked `other` as an « also » entry under the lemma the core reads the form as, and drops a reading whose lemma the lexicon does not hold; `word_grammar` answers the readings of a form as the card's lemma, and the other lemmas with theirs |
| Spanish's precedent | `add-lingua-spanish-grammar-tables` (archived): kaikki's tags as UD tags, a lemma's table first and a form's own entry for the pairs no table lists, `other` on every relation, attested forms only; 149,279 readings in 88 tags, the pack +199,380 B. `add-lingua-spanish-word-card` (archived): the card's Spanish names, persons merged within a tense, no line for the dictionary form itself, a letter's name giving no reading of its plural |
| French's tables | change 43 (`add-lingua-french-forms-tables`, proposal merged, implementation not on `main`): `reduce-fr-en.py` writes `tables/fr/forms.tsv` (124,040 forms in its design, 124,050 as implemented) and `freq.tsv` (60,000 lemmas) from the English Wiktionary's French section; `tags.tsv` and `lexical.tsv` committed empty, « filled by 45 and 48 » (its Non-Goals and D1); as implemented, a ranked lemma's own form reads as itself; its D12 hands this change « `reduce-fr-en.py`'s pass over the same entries, the forms it reads the readings of, `tags.tsv` to pin » |
| The tag pool | `crates/lingua-pack/src/tags.rs` `tag_pool`: the pin in its own order, then the readings' tags it lacks, then the tags only senses carry, each part sorted. English's (27 tags) and Spanish's (106) pins are the pools en-fr's and es-fr's packs already carried, sense tags included. No reducer writes a pin (`pack_sources.py KEPT_INPUTS`); `record-build` and the committed-tables check require the file |
| The card | `apps/lingua-extension/src/reading/grammar-description.ts`: readings merged by tag (one reading per tag, persons merged), `nameReadings` merging the persons of one tense and number, `tenseOrder` per studied language, `formKind` naming a participle only when its tense is `Past`; the extension's `StudiedLanguage` is `en \| es` until changes 51 and 52 |
| The French golden | `crates/lingua-wasm/tests/baseline/fr-en.golden`, over the hand-written fixture pack (no grammar, change 39 D5): its 31 `word-grammar` probes answer `"readings":[]`. French is at `0.2.0` since change 40's pre-pass merged (#821): an elided piece, `au` and an inversion are tokens of their own |
| The measurement inputs | change 43's: the French section derived on 2026-10-08 from the English dump of 2026-10-03 (403,269 entries, 510,058,226 B, sha256 `2d7bbe5f…`), UD French-PUD and GSD's test section at the commits change 43 pins; the forms and ranks of change 43's prototype tables (its design's S4: 124,040 forms, 60,000 lemmas) |
| How it was measured | a prototype of this design's reducer in the scratchpad (never committed) over those tables; its tables built into a fr-en pack by `lingua-pack-build` from `origin/main`; the card's answer asked through `lingua_core::engine::word_grammar` by a scratch binary on `origin/main` 35faf774, French at `0.2.0`; each rule switched off alone to measure what it does; the same rules over change 43's implemented tables (branch `claude/add-lingua-french-forms-tables-impl` at `2f5fc741`) as a cross-check (D9) |

What the English Wiktionary's French section says of grammar, as measured: 7,395 verbs carry a
conjugation table (the study's « ≈ 7,380 fully tagged verbs »), and 7,058 of them list every one of
the 45 simple finite cells (six persons of the present, imperfect, passé simple and future
indicative, the conditional, the present and imperfect subjunctive, and the imperative's three) —
the others are defective or impersonal (`falloir`, `pleuvoir`, `pouvoir` without an imperative). A
table tags the passé simple `historic past`, the present participle `gerund participle present`,
and lists one past participle, bare, the masculine singular: the agreed participles (`dirigée`,
`dirigés`, `dirigées`) hang under the participle's own entry (`dirigé`, a verb entry). It lists the
compound tenses as constructions (`avoir + past participle`), and a pronominal verb's forms with
their pronoun (`m'évanouis`, `nous évanouissons`, `évanouis-toi`). A form's own entry sometimes
merges persons and moods in one sense (`parle`: « first/third-person singular present
indicative/subjunctive »). A noun's head (`fr-noun`) gives its gender — `m`, `f`, `mf`,
`mfbysense`, `m-p`, `f-p` —; a noun whose plural is spelled like its singular lists none (`bras`,
tagged `invariable`); a feminine noun may list its masculine counterpart (`déesse`: masculine
`dieu`). An adjective lists its feminine, its plurals and its masculine before a vowel (`beau`:
`bel`).

## Goals / Non-Goals

**Goals:**
- Readings for the forms French's table holds, in the vocabulary as it stands — no change to the
  container, the core or the card's code —, measured against held-out treebanks.
- Everything the word card (change 51) needs to merge moods on five-reading forms (M21), and
  nothing merged here.
- French's tag pool pinned, so fr-en and fr-es store a form's readings alike.
- One form, one lemma (M8): readings filed under the form's one lemma; another kept word named,
  never counted.
- en-fr, es-fr, es-en and en-es byte for byte, and the French golden unmoved.

**Non-Goals:**
- The card's French: a renderer per native language, tense names keyed by pair, the merge of
  moods, a name for the present participle (51, « formes françaises nommées en anglais et en
  espagnol »).
- Readings of forms the table does not hold (`vînmes`, which wordfreq never met): a conjugation
  table, which no change builds.
- Choosing a reading from context, as in `add-lingua-word-grammar`.
- Lemma alternatives (M8: the optional `add-lingua-lemma-alternatives`, outside the counts).
- The glosses, the sense runs and the dictionary words (48), fr-es (49), the levels (46).
- A fix of `Pack::readings` for a lemma that is a form of another word (D12, open question 4).

## Decisions

### D1 — fr-en's reducer writes French's readings

`reduce-fr-en.py`, French's reference reducer (change 43 D1), reads the readings in the same pass
over `kaikki-French.jsonl` as the forms, after the forms and ranks are chosen, and writes
`grammar.tsv` into its work folder; `split` files it into `tables/fr/` as it files Spanish's into
`tables/es/`. The rules live in that file, as Spanish's live in `reduce-es-fr.py`: no shared module
is edited, so no other pair's rule digest moves (D12), and fr-es (49) reads `tables/fr/grammar.tsv`
as committed, never the rules. The forms and ranks do not move: the pass reads them, it does not
choose them.

### D2 — Which forms get readings

The forms `tables/fr/forms.tsv` holds, under the lemmas `tables/fr/freq.tsv` ranks — Spanish's D1:
a card opens on a form the analysis resolved through the table, so a form outside it never reaches
one, and every form of every kept lemma would add, for French, 88,678 forms nobody writes (change
43 D8) for a conjugation table nothing builds.

Measured: **125,193 readings of 88,579 of the 124,040 forms**, in 79 tags; 115,909 under the form's
own lemma, which 88,328 forms carry, and 9,284 marked `other` (D6). Own readings per form:

| Readings | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| Forms | 69,438 | 15,026 | 1,470 | 101 | 2,153 | 140 |

— 1.31 on average. By part of speech: 70,583 verb readings, 34,967 noun, 19,560 adjective, 39
determiner, 30 pronoun, 14 numeral. The 35,712 forms without a reading of their own are, for
35,312 of them, forms of a lemma the dictionary holds as no noun, verb, adjective, determiner,
pronoun or numeral — adverbs, prepositions, conjunctions, interjections, names, words it does not
know —; the rest are a determiner's, pronoun's or numeral's dictionary form (`le`, `je`, `deux`),
which names nothing on its own card (Spanish's D4 of the word card), old spellings the dictionary
marks doubtful (`avoit`, D5), and 146 participles change 43 files under a noun's or a pronoun's
spelling (`citée` → *cité*, `tues` → *tu*), which name their verb instead (D5). Two function words
do carry readings, their dictionary noun's: `un` and `nous`, read in both numbers (D4).

**Verb paradigms.** The 60,000 ranked lemmas hold 3,339 verbs, 3,285 of them with all 45 simple
finite cells in their table; those tables list 151,363 form–cell pairs, of which 53,080 (35.1 %) are
forms the table holds and so carry a reading. The rest are forms wordfreq never met (`vînmes`), left
out by change 43's attested cut: on UD French-PUD, 99.94 % of the finite verbs whose form the tables
map to the treebank's lemma carry a reading (D10).

### D3 — Verb forms

| kaikki | UD |
|---|---|
| `indicative` + `present` / `imperfect` / `future` | `Mood=Ind`, `Tense=Pres` / `Imp` / `Fut` |
| `historic past` (with or without `indicative`) | `Mood=Ind`, `Tense=Past` — the passé simple |
| `subjunctive` + `present` / `imperfect` | `Mood=Sub`, `Tense=Pres` / `Imp` |
| `conditional` | `Mood=Cnd`, no tense, as Spanish writes it (UD French writes `Tense=Pres`; D7) |
| `imperative` | `Mood=Imp`, no tense (likewise); second person singular, first and second plural |
| `first-person`… and `singular` / `plural` | `Person`, `Number`, with `VerbForm=Fin` |
| `infinitive` | `VerbForm=Inf` |
| `gerund participle present` | `VerbForm=Part`, `Tense=Pres`, as UD French writes it: French has no gerund form of its own, « en parlant » is the participle after « en » |
| `participle past`, the verb table's bare row | `VerbForm=Part`, `Tense=Past`, `Gender=Masc`, `Number=Sing`: in French's tables it is the masculine singular, not a repeat of it (Spanish dropped the row) |
| a past participle's agreed forms | its gender and number, through the participle's entry (D5) |

Every verb is `VERB`: `AUX` is a role in a sentence, which a card cannot see. A sense of a form's
own entry that merges persons or moods reads as each combination (`parle`'s « first/third-person
singular present indicative/subjunctive » is four readings); a table row names one. A participle
sense that names an agreement without `past` (`feminine singular of dirigé`) is the past one.

**Never a reading**: a compound tense (`avoir + past participle` and the like): it is not one token.

**A pronominal verb's forms read without their pronoun.** Its table writes `m'évanouis`, `nous
évanouissons`, `évanouis-toi`; change 40's pre-pass splits the pronoun off what a page writes, and
change 43 keeps the bare form, so the bare form takes the reading: `évanouit` is *s'évanouir*'s
present indicative third person singular, and `évanouis` carries the imperative a page writes as
`évanouis-toi`. Measured: 12,023 rows read without their pronoun; 37 more ranked verbs get a full
paradigm (3,248 → 3,285), most of their forms having had readings through their own entries
already. Through the engine on `main` (change 40's pre-pass merged), the card on `s'évanouit`
(pieces `se`, `évanouit`) reads `évanouit`'s present and passé simple, and the card on
`évanouis-toi` (`évanouis`, `toi`) carries the imperative among its six readings.

### D4 — Nouns, adjectives, determiners, pronouns, numerals

- **A noun's gender** comes from `fr-noun`'s first argument, or else from its senses' tags. Its own
  form reads `NOUN|Gender=…|Number=Sing` (`Plur` for a plural-only noun, `m-p`/`f-p`), its plural
  `Number=Plur`; a noun of both genders (`mf`, `mfbysense`: `enfant`, `artiste`) has a reading per
  gender. 17,122 of the 17,132 ranked nouns (99.94 %) carry a gender on their own form — the
  readings change 48's sense runs take their « masculine noun » from (`A noun's gender comes from its
  readings`).
- **A noun the dictionary gives one form for** — its head's plural `#`, or a sense tagged
  `invariable` and no plural listed: `temps`, `fois`, `bras`, `vis` — reads singular and plural, as
  Spanish's `crisis` does: 549 readings; without them 97.80 % of PUD's nouns agree with the
  treebank, 98.85 % with them (D10). The card says « may also be the plural of *temps* » on its
  own form, as it does of `crisis` — and of `un` and `nous`, whose dictionary nouns (« un », « le
  nous ») are invariable: the article and the pronoun get that line (open question 3).
- **A feminine noun's masculine row gives no reading** (`déesse`: masculine `dieu`; `sainte`:
  `saint`): the masculine is its own dictionary form, not an inflection of the feminine, and the card
  of `dieu` would otherwise say « may also be the masculine singular of *déesse* »: 146 readings,
  140 of them `other` marks.
- **An adjective's own form** reads `ADJ|Gender=Masc|Number=Sing` when it has a feminine of its own
  (`grand`), `ADJ|Number=Sing` when one form serves both (`rapide`), and `Number=Plur` when every
  sense is plural (`plusieurs`); its forms read by their tags (`grandes`: `Gender=Fem|Number=Plur`;
  `rapides`: `Number=Plur`); the masculine before a vowel (`bel`, `nouvel`, `vieil`) is a masculine
  singular; a comparative or superlative reads `Degree=Cmp` or `Sup`.
- **Determiners, articles among them, pronouns and numerals** read their gender and number:
  `cette` of *ce*, `la` of *le* both as a determiner and as a pronoun, `celle` of *celui*, `une` of
  *un*. An article is a determiner, as UD writes it. There is no `PronType`, as in Spanish: kaikki
  does not state it per form.
- **A pronoun's form names its gender, or gives no reading.** A personal pronoun's head lists its
  other persons, numbers and cases as forms — `il`: plural `ils`, dative `lui`, emphatic `lui`,
  possessive determiner `son`; `nous`: singular `je` —, which are other words, not inflections; read
  as forms, `je` would be « the singular of *nous* ». **A plural-headed determiner's or pronoun's
  table gives none either** (`tes`, which lists `ton` as its masculine). Both measured: 10
  readings, three of them a pronoun's own forms the first rule also leaves out, for want of a
  gender (`tous` of *tout*, `ce`, `s'`; D10).

### D5 — Where a reading comes from, and what never gives one

- **A lemma's table first** (116,593 readings), **a form's own entry** for the pairs no table lists
  (552), as Spanish's D4.
- **A form of a form, along one part of speech**, as change 43's D3 maps the form: when a form
  reads as a form of a word that is itself a form of another, of the same part of speech, it reads
  as that other word with its own agreement — `dirigée`, the feminine of the participle `dirigé`,
  which is *diriger*'s past participle, reads `VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part`
  of *diriger*; `faites` adds the feminine plural participle of *faire* to its finite readings.
  8,048 readings; without them 75.93 % of PUD's participles carry a reading and 98.10 % of those
  agree with the treebank, 100 % and 99.38 % with them. Such a reading is added only where the
  form has no reading of that lemma in that part of speech and verb form already: `les`, the plural
  of *le*, does not also read feminine through `la`.
- **A reading's part of speech is one its lemma's entries hold.** A past participle whose spelling
  change 43 keeps as a noun's, an adjective's or a pronoun's lemma (`cité`, `privé`, `mort`, `tu`)
  gives its agreed forms no verb reading of that word: they read through it as forms of a form,
  toward the verb, which the card names (`citée` → *citer*, `privée` → *priver*, `morte` →
  *mourir*, `tues` → *taire*); and a link toward a word the dictionary holds as no verb gives no
  verb reading (`venait` would name *came*, an English gloss the French section also holds as a
  noun; `partie` *parti*, `rendez-vous` *se*). Measured: 2,098 readings out — 1,992 a verb's under a
  noun's, an adjective's or a pronoun's lemma (« the past participle of *tu* »), 16 an adjective's
  or a noun's under a lemma held otherwise (`meilleures` of the noun *meilleur*, which now names
  *bon*), 90 `other` marks — and 1,404 `other` marks in, 1,293 of them a participle naming its
  verb; 146 forms keep no reading of their own (`citée`, `tues`), their lemma being change 43's
  choice (D6, open question 6). A lemma the dictionary holds as none of the parts of speech read
  keeps its readings (`infos` of *info*).
- **A spelling variant** reads as the word it spells (change 43 D7): `coeurs` is the masculine
  plural of *cœur*: 534 readings of 337 forms.
- **Never a reading**:
  - an inflection or a sense the dictionary marks doubtful — the tags change 43 leaves out of the
    forms (alternative, obsolete, archaic, rare, dated, misspelt, nonstandard, proscribed,
    abbreviated, a pronunciation spelling): a form a source marks doubtful is no inflection
    (*Licence hygiene*), so no reading either;
  - a form's own sense marked as a region's or a register's (Louisiana, Quebec, slang…): `été` is
    Louisiana's past participle of *aller*, and a standard card would otherwise say so;
  - a capitalised headword, which is another word (Spanish's word-card D7): `CE`, `LE`, `Mes`
    (« Maîtres ») would read `ce`, `le` and `mes` as nouns. 3,592 entries; reading them adds 1,510
    readings, lowers PUD's adjective agreement by 0.2 point and raises its nouns' by 0.4;
  - an entry whose every sense is an alternative form of another word or a neologism: `estre`,
    « archaic spelling of *être* », carries a conjugation table that would make `est` and `suis`
    its forms; `lea`, `man`, gender-neutral neologisms, would make `les` and `mes` theirs. 2,548
    entries, 696 readings. An entry that is a word of its own, however rare or proscribed
    (`médire`, `impacter`), keeps its readings;
  - a letter's name toward its plural (Spanish's word-card D7): `elles` is no plural of `elle`, the
    name of the letter L;
  - a link toward the word a reviewed override row of change 43 sets aside as a copy error:
    `fatiguée`, whose verb entry says « feminine singular of parlé », names no *parler* (one reading
    on change 43's implemented tables; Known data defects).

### D6 — `other` marks, and one form, one lemma (M8)

The forms table gives each form one lemma (change 43 D5). Its readings of that lemma are its own; a
reading of another ranked lemma is marked `other`, and the card names that word below, as English's
`leaves` names *leaf* and Spanish's `vino` *venir*: **9,284 marks on 4,999 forms** — `fils` the
plural of *fil*, `couvent` *couver*'s third person plural, `vis` *vivre*'s present and *voir*'s
passé simple, `suis` *suivre*'s, `sommes` the plural of *somme*. kaikki's tables are structured
paradigms, so the relations stand (Spanish D5), with two French filters: the other word is an
entry of the dictionary — `irait`'s form link lists « would » after *aller*, and wordfreq ranks the
English word —, and not one whose every sense is regional — *vader*, a Louisiana verb, would be
named on `va`, `vais` and `vont`. 88 readings.

**M8 keeps one lemma per form.** The mark changes no count: the form's status, exposures, level and
review belong to its one lemma; the card names a word, it offers no second lemma, which is the
optional `add-lingua-lemma-alternatives`. **M8's cost on the card**: a dictionary noun whose own
form the forms table reads as another lemma is no lemma of the tables (change 43 D5), so no reading
of it is kept, and the builder would drop one: 543 nouns, 292 of them forms of one of the 5,000
commonest lemmas — `fait`, `été`, `porte`, `demande`, `élève`, `nouvelle`, `bonne`, `reste`. « La
porte » opens *porter*'s card with five verb readings; the door returns with the lemma alternatives,
or with an override row of change 43 (its open question 1), which this reduction follows without a
change of its own.

### D7 — What the word card needs to merge moods (M21)

M21 settles « moods merged on five-reading forms ». Change 40 (D1) places it on the word card:
the core answers readings as tags, unmerged, and the card's description merges them; « not change
41: the cascade picks one lemma per form and never sees a reading ». The merge is change 51's rule.
It can only merge what the readings say, so this change gives the card exactly what it needs, and
says so in its requirement:

1. **One reading per mood, person and number, never merged here.** `parle` carries five tags:
   `Mood=Ind|Person=1`, `Mood=Ind|Person=3`, `Mood=Sub|Person=1`, `Mood=Sub|Person=3`, each
   `Number=Sing|Tense=Pres|VerbForm=Fin`, and `Mood=Imp|Number=Sing|Person=2|VerbForm=Fin`. A table
   that stored « present indicative or subjunctive » as one reading would leave the card nothing to
   merge — and the vocabulary has no such value.
2. **The two presents carry `Tense=Pres`, the imperative and the conditional no tense** — Spanish's
   encoding, not UD French's `Mood=Imp|Tense=Pres`. The card already merges the persons of one tense
   and number (`describeReadings`, `nameReadings`), which leaves three groups: present indicative
   first and third singular, present subjunctive first and third singular, imperative second
   singular. The moods' merge then joins the two that share a tense, persons and number, and leaves
   the imperative, whose tense and person differ, apart. A tense on the imperative would hand a merge
   keyed by tense a third mood to fold in. The treebank measurement maps UD French's tense away
   (D10).
3. **All five under the form's own lemma**, none marked `other`: they come from the verb's own
   table, and a form whose lemma is the verb keeps them whatever noun it also spells (`porte`, D6).
4. **No flag.** A five-reading form is recognised from its readings alone; the pack stores nothing
   more and `word_grammar`'s answer keeps its shape. The pinned pool lays the five out imperative,
   indicative, subjunctive (byte order, D8); the card orders tenses itself, as Spanish's does
   (`tenseOrder`), and change 51's goldens lock the order.

Measured: **2,142 forms of 2,113 verbs** carry exactly these five readings — first-group presents
first/third singular, and the *ouvrir*, *offrir*, *cueillir* families (13 verbs) —: 40
of them forms of the 1,000 commonest lemmas (`parle`, `passe`, `trouve`, `donne`, `joue`,
`laisse`, `demande`, `reste`, `pense`, `porte`…), 348 of the 5,000, 1,057 of the 20,000. They are
180 of PUD's 3,282 verb tokens (5.5 %) and 92 of GSD test's 1,180 (7.8 %). The other five-reading
forms are 11: 8 like `dis` (present and passé simple first and second singular, and the imperative:
two tenses of the indicative, nothing for the moods' merge) and 3 forms that are nouns and
adjectives at once; 140 forms have six readings, 136 like `finis`, which adds the masculine
plural participle.

Wider than M21's words, **6,147 forms read the indicative and the subjunctive of one tense, person
and number**: the 2,142 five-reading forms, 2,729 third persons plural (`parlent`, `disent`), 1,080
second persons singular (`parles`), 162 like `finissent` (also an imperfect subjunctive), 33 whose
imperfect indicative is also an imperfect subjunctive (`finissions`, `finissiez`) and one other
— 292 of PUD's verb tokens (8.9 %), 128 of GSD test's (10.8 %). Whether change 51 merges the
moods on those 4,005 too is its call; the readings serve either.

### D8 — French's tag pool, pinned

`tables/fr/tags.tsv` holds **the 79 tags French's readings carry**, one per line, each once, no
other: in byte order as the first reduction gives them (`cut -f3 grammar.tsv | LC_ALL=C sort -u`),
and a tag a later reduction adds appended after them, so that a pinned tag never changes index. A
person writes it and reviews it in the pull request; no reducer writes it (`KEPT_INPUTS`), so no rule
digest moves with it and `split` never overwrites it. The committed-tables check holds it to the
readings' tags: an update whose readings carry a new tag, or no longer carry a pinned one, fails
naming it, and its pull request appends or removes the line.

What it buys, measured. The builder lays a pack's pool out as the pin, then the readings' tags it
lacks, then the tags only senses carry (`tags.rs`); with no pin file at all, as one sorted pool, where
a sense tag can fall between two reading tags. Change 43's empty pin already keeps the senses after
the readings; this pin names the readings' tags, so the pool's first 79 tags are French's readings'
and nothing a native's senses add — fr-en's `NOUN|Gender=Fem` runs and bare `ADV` (48), fr-es's
(49) — can move them, as *A form's readings do not depend on the pack's native language* requires;
a tag a later dump introduces takes the next index instead of shifting every index after it; and
the folder is what a studied folder is (SOURCES.md, *What a pack studies, whatever it glosses*): a
pool a person wrote. Today the pin moves no byte: fr-en's pack built with it has the sha256 the empty
pin gives (both measured), since the readings' tags in byte order are what the empty pin lays out
too.

*Rejected — the full pool, as English's and Spanish's pins hold it.* Those pins are the pools the
shipped en-fr and es-fr packs carried, sense tags included, so that those packs kept their bytes.
No French pack has shipped, and a sense tag inside the pin buys nothing: a pack's sense runs are its
own.
*Rejected — a pin the reducer writes.* It would follow every reduction, which is what a pin must
not do, and the folder contract says a person writes it (SOURCES.md, *What a pack studies, whatever
it glosses*).

### D9 — The table's size, in git, and the pack

`grammar.tsv` is 7,938,182 B of text, 125,193 lines such as
`parlions<TAB>parler<TAB>VERB|Mood=Ind|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin<TAB>-`;
752,281 B compressed (gzip -9). The three tables of `tables/fr/` then hold 11,038,651 B of text
(forms 2,255,591, ranks 844,878, readings), 1,635,505 B compressed file by file. Spanish's
`grammar.tsv` is 9,190,541 B (898,285 compressed), and its D6 committed its tables past the ≈ 10 MB
the programme had named; French's are committed for its reasons — git stores them compressed, an update's diff stays readable line by
line, a release reads nothing but the tables — and attested forms are already the rule (D2).

**The pack** built from `tables/fr/` grows from 1,239,104 B (change 43's, forms and ranks) to
**1,396,520 B (+157,416 B)**, Spanish's grammar having added 199,380 B. Each French pack carries it.
With change 46's levels (+60,035 B, its measurement) and change 48's glosses, sense runs and
expressions, fr-en is about 2.42 MB (change 48's design measures 2,421,321 B on this design's
tables before D5's part-of-speech rule, which adds 1,796 B): 46 % of the 5 MiB budget.

**On change 43's implemented tables** (124,050 forms, every ranked lemma its own form's), the same
rules give 125,191 readings of 88,588 forms, 9,268 `other` marks, the same 79 tags and 2,142
five-reading forms, M8's cost 549 nouns (295 of them forms of the 5,000 commonest lemmas), and a
pack growing from 1,239,671 B to 1,396,257 B (+156,586 B). Task 2.1 records the figures again at the
pin.

*Rejected — a compact table* (one line per form, tags pooled): it changes the builder's input,
which every pair shares, to save what git already saves (Spanish's D6).

### D10 — Measured against held-out treebanks

`scripts/lingua-data/measure/fr_readings.py`, run by `measure/fr-ud.sh` (change 43 D9) after its
gates, over the two files that script fetched: for each part of speech, the words whose form the
tables map to the treebank's lemma, the share that carry a reading — any reading, what the card
shows —, and the share of those whose treebank part of speech (`VERB` for a verb or an auxiliary)
and features are among their readings — the conditional's and the imperative's tense left aside
(D3), and a participle without a tense, as GSD's recent releases write it, read either way. It
reads the committed tables — the reduction never reads either treebank — and reports without
deciding the exit status: a treebank's own errors would otherwise fail an update.

| | Words | Read | Agree |
|---|---|---|---|
| PUD, finite verbs | 1,742 | 99.94 % | 99.60 % |
| PUD, participles | 968 | 100 % | 99.38 % |
| PUD, infinitives | 466 | 100 % | 100 % |
| PUD, nouns | 4,490 | 99.09 % | 98.85 % |
| PUD, adjectives | 1,477 | 98.85 % | 96.44 % |
| PUD, determiners | 3,587 | 61.36 % | 85.42 % |
| PUD, pronouns | 522 | 16.67 % | 27.59 % |
| GSD test, finite verbs | 637 | 100 % | 99.53 % |
| GSD test, participles | 298 | 100 % | 99.33 % |
| GSD test, infinitives | 190 | 99.47 % | 100 % |
| GSD test, nouns | 1,772 | 98.65 % | 98.68 % |
| GSD test, adjectives | 562 | 98.75 % | 95.68 % |
| GSD test, determiners | 1,350 | 57.85 % | 78.10 % |
| GSD test, pronouns | 292 | 20.89 % | 19.67 % |

A determiner's or pronoun's dictionary form names nothing (D2), hence their « read » column, and
what they read is mostly a dictionary noun's spelling — `un` (241 of PUD's determiners read, the
noun « un »), `son` (« sound », 74), `nous` and `ça` among the pronouns —, whose readings are the
noun's: the card shows no article's or pronoun's line for them, hence their « agree » column. Among
verbs, what disagrees on PUD is mostly the treebank's own annotation — `prit` and `crée` as
participles, `attirerait` as an imperfect, `représente` as a plural, `fussent` as a present —; among
nouns and adjectives, words the two sources class differently (`millions` a numeral, `tous` and
`toute` determiners, `autres` an adjective, `meilleure` a noun only, `tel`, `maximum`). Counting a
nominal reading of any part of speech as agreeing, as a first draft did, gave 99.84 % for nouns and
99.50 % for determiners — `un`'s noun standing for the article.

**Each rule, measured** (switched off alone from the whole; D4's plural-only adjective rule changes
one reading, `plusieurs`, and lifts PUD's adjectives from 95.14 to 96.44 %):

| Rule switched off | Readings | Forms with one | `other` | Five-reading forms | What it moves on PUD |
|---|---|---|---|---|---|
| — (this design) | 125,193 | 88,328 | 9,284 | 2,142 | — |
| a reading's part of speech its lemma's (D5) | 125,887 | 88,474 | 7,970 | 2,142 | adjectives agree 96.44 → 96.58 % |
| a form of a form (D5) | 117,138 | 82,200 | 7,657 | 2,142 | participles read 100 → 75.93 %, agree 99.38 → 98.10 % |
| both numbers on an invariable noun (D4) | 124,644 | 88,328 | 9,284 | 2,142 | nouns agree 98.85 → 97.80 % |
| spelling variants (D5) | 124,659 | 87,991 | 9,187 | 2,131 | nouns read 99.09 → 99.02 % |
| merged senses read as each (D3) | 125,083 | 88,304 | 9,284 | 2,128 | — |
| a pronominal verb without its pronoun (D3) | 125,143 | 88,294 | 9,275 | 2,142 | infinitives read 100 → 99.79 %; 3,285 → 3,248 full paradigms |
| a participle sense without `past` (D3) | 125,175 | 88,317 | 9,280 | 2,142 | — |
| articles as determiners (D4) | 125,187 | 88,328 | 9,281 | 2,142 | determiners agree 85.42 → 9.59 % |

| Exclusion switched off | Readings | Forms with one | `other` | What it moves on PUD |
|---|---|---|---|---|
| capitalised headwords read (D5) | 126,703 | 88,579 | 9,322 | adjectives agree 96.44 → 96.24 %, nouns 98.85 → 99.26 %, determiners 85.42 → 83.93 % |
| alternative-only and neologism entries read (D5) | 125,889 | 88,764 | 9,388 | nouns agree 98.85 → 98.81 % |
| `other` toward a link-only or a regional word (D6) | 125,281 | 88,328 | 9,372 | — |
| a feminine noun's masculine row read (D4) | 125,339 | 88,334 | 9,424 | — |
| a composed reading over a direct one (D5) | 125,208 | 88,328 | 9,287 | — |
| a letter's plural read (D5) | 125,206 | 88,328 | 9,285 | — |
| a pronoun's form without a gender read (D4) | 125,201 | 88,330 | 9,289 | determiners agree 85.42 → 83.93 %; pronouns read 16.67 → 23.37 %, agree 27.59 → 48.36 % (`tous`, `ce`) |
| a plural-headed table read (D4) | 125,195 | 88,328 | 9,286 | — |
| a regional sense of a form's entry read (D5) | 125,196 | 88,328 | 9,287 | — |

### D11 — The tests

- **`test_reduce_fr_en.py`**, each rule on fixture entries shaped as the English dump writes them
  (task 1.2).
- **`crates/lingua-pack/tests/fr_en_grammar.rs`** builds the fr-en pack from the committed tables
  and asks `word_grammar` what the card would show, as `es_fr_grammar.rs` does: `fut`, `parle` (the
  five readings), `parlerait`, `parlant`, `dirigée`, `faites`, `été` as *être*, `évanouit`,
  `maisons`, `temps`, `la` as *le*, `couvent` naming *couver*, `fils` *fil*, `vis` *vivre* and
  *voir*, `porte` as *porter* naming nothing, `est` and `va` naming nothing, `citée` as *cité*
  with no reading of its own and naming *citer*, `venait` naming nothing. Every probe asks a lemma
  the tables keep (D12).
- **`committed_tables.rs`**: `tables/fr/tags.tsv` holds exactly the tags of `tables/fr/grammar.tsv`,
  canonical, each once, and the check names a tag missing or left over; a pack built from
  `tables/fr/` with a sense run tagged `INTJ` stores its paradigms byte for byte as without it.
- **`measure/fr_readings.py`** on a hand-written CoNLL-U fixture: a verb read, a participle
  without a tense, the conditional's tense left aside, a determiner whose form reads only as a noun
  (not agreeing), a word whose lemma the tables do not give.

### D12 — What moves, and what cannot

- **`fr-en.golden` does not move.** The French baseline runs over its fixture pack until change 48
  switches it to the committed tables (change 39 D5, change 43 D11); the fixture has no grammar and
  keeps none: a hand-written reading table would test the fixture, not the reduction, and its
  `porte` noun — a lemma of the fixture, not of the tables (M8) — would show a reading the real pack
  never has. The readings are pinned instead by `fr_en_grammar.rs` over the committed tables.
- **What change 48's switch will show** of this change, measured on the prototype's pack with the
  core on `main` (French at `0.2.0`): 24 of the 31 `word-grammar` probes gain readings —
  `est`, `sont`, `était`, `fut`, `a`, `ai`, `as`, `eût`, `pût` one each, `été` (*être*) its
  participle, `soyez`, `va`, `allez`, `fait`, `prenez`, `vis` (*voir*), `couvent` (*couver*) two,
  `faites`, `dit`, `vis` (*vivre*) three, `porte` (*porter*) five; `couvent` (*couvent*) and `fils`
  read their own noun and name *couver* and *fil*; `l'homme` (*homme*), split by change 40, reads
  its piece's own noun reading, which the card leaves unnamed. `vînmes` (a form the table does not
  hold), `au` (*au* and *à*), `du` and `l’` stay empty. Two probes ask a lemma the committed tables
  do not keep, `été été` and `porte porte` (the fixture's nouns, M8's cost): `Pack::readings` looks
  a lemma up through `id_of`, which maps a form to its lemma's id, so they read *être*'s and
  *porter*'s paradigms against the wrong word (`porte` as *porte*: `NOUN|Gender=Masc|Number=Sing`
  and `VERB|VerbForm=Inf`), and the golden would record it — as `en-fr.golden` and `en-es.golden`
  already record `saw saw`, `lay lay` and `more more` read off *see*'s, *lie*'s and *many*'s
  paradigms, and `thought thought` with *think*'s gloss. French's card at `0.2.0` never asks so: its lemma is the pack's,
  or a word the pack does not hold. English's and Spanish's do, through their out-of-lexicon plural,
  whose singular the cascade does not look up — measured on the released packs, 2,002 Spanish forms
  the tables lack (`ablativas` → *ablativa*, read « masculine plural » off *ablativo*'s paradigm; 433
  feminine plurals read masculine only) and 12,129 English ones, mostly no words (`abandoneds`;
  `lefts` → *left*, read « plural » and « third person singular » off *leave*'s). The builder has
  the twin: it files a reading, a rank, a level or a gloss under `id_of(lemma)`, so a ranked lemma
  whose own form the forms table maps elsewhere lands on that other word — none in any committed
  table or in change 43's implemented tables, which forbid it; six in its prototype tables, where
  *venir*'s rank became `venue`'s 1,637 and `venue`'s noun readings joined *venir*'s paradigm. A core
  change of its own (open question 4).
- **en-fr, es-fr, es-en and en-es cannot move**: no file of their rule digests changes
  (`reduce-fr-en.py` is fr-en's alone; `reduce_common.py` and the `reduce_edition_*.py` are not
  edited), no file of `tables/en/`, `tables/es/` or their pairs' folders, and nothing of lingua-core,
  lingua-wasm, the builder or the extension. Their packs rebuild to the sha256 their pins record,
  and the five goldens and `cross_native` pass without re-blessing (task 4.1).
- **fr-en moves**, and nothing else of French: `tables/fr/grammar.tsv` and `tags.tsv` are new, its
  `manifest.json` (`pack_version`, the rule digest) and `pin.json` are recorded again; `forms.tsv`,
  `freq.tsv`, `lexical.tsv` and `studied.json` are byte for byte change 43's.

OpenSpec: three ADDED requirements in `lingua-data-packs`, none MODIFIED. *French's word grammar*
builds on change 43's *French's forms and frequencies* (the forms, ranks and folder it reads), on
change 38's dump-derived source and on change 39's French baseline (its golden unmoved); the three
are in `archiveAfter`, and `openspec_archive_order.py` exits 10 naming them while they are open.
Change 43's scenario *The reference pair writes French's folder* says `tables/fr/tags.tsv` is empty:
read as « until change 45 », as its requirement's text says (« an empty tag pool … until fr-en's
readings and glosses are reduced ») — open question 1.

### D13 — What the later changes of the stage take from here

| Change | Takes |
|---|---|
| 46 levels | nothing: readings move no rank. Both add to `reduce-fr-en.py` and re-record fr-en's manifest, pin and README: the second to merge reduces again on top of the first |
| 48 fr-en | the readings its noun runs take their gender from (17,122 nouns); the pin, ahead of its sense tags; the golden's switch (D12) |
| 49 fr-es | `tables/fr/grammar.tsv` and `tags.tsv` as committed: its readings byte for byte fr-en's, its noun runs gendered alike |
| 51 word card | the five readings and D7's encoding for the moods' merge; French's tense names keyed by pair; a name for the present participle (`VerbForm=Part\|Tense=Pres`, which `formKind` leaves unnamed today); `finiteKey` and `NAMES_INFINITIVE` for French; the invariable noun's plural (open question 3) |
| `add-lingua-lemma-alternatives` (optional) | the nouns M8 leaves without a reading (D6) |

## Known data defects

What reads wrong in the source, left as it is, each with where its fix lives:
- **`fatiguée` → *parler*.** The dictionary's participle entry `fatiguée` says « feminine singular
  of parlé », a copy error; change 43's prototype tables follow it, so its readings are *parler*'s
  feminine participle, and *fatiguer*'s is marked `other`. Change 43's implementation overrides the
  form (`fatiguée` → *fatiguer*), after which the copy still names *parler* on the card (measured):
  the reduction gives no reading toward the word an override row of change 43 sets aside as a copy
  error (task 1.1). The source's fix is the dictionary's.
- **`supe`** reads as the verb *super* (« to sip »), which shares its lemma with the common
  adjective *super*: one of the 40 five-reading forms among the 1,000 commonest lemmas is this one.
- **`meilleure`** reads as the noun *meilleur* only: the adjective `meilleur` is a form entry of
  *bon*, and this design does not read a form entry's own forms (doing so for adjectives and nouns
  adds 123 readings, measured) — left to a later reduction, as no card is wrong without them.

## Risks / Trade-offs

- **[kaikki's tags drift]** → fr-en's `pin.json` holds the source; an unknown combination gives
  no reading rather than a wrong one, a tag outside the vocabulary fails the build, and D10's report
  shows a fall of agreement on the next update.
- **[The card names nothing French until change 51]** → no package carries a French pack before
  change 52, which comes after the card.
- **[A common noun has no reading]** (`porte`, `élève`) → M8 accepts it (D6); an override row of
  change 43 brings it back without a change here.
- **[A reading tag a later dump introduces]** → the committed-tables check fails, naming it; the
  update's pull request appends it to the pin, so no pinned tag changes index, and its report lists
  the readings that move.
- **[A large text table]** → D9; the compact table stays the way out, and the pack does not change.
- **[The golden's non-lemma probes at change 48]** → D12, open question 4.

## Migration Plan

Nothing to migrate: new files in `tables/fr/`, no stored format, no reader holds a French pack.
Rollback is a revert; change 43's empty `tags.tsv` returns with it.

## Effort

3.5–5.5 ideal days, against the programme's 3.5–6: the reducer's readings 1.5–2.5, its tests
0.75–1, the tables, pin, README and SOURCES 0.5–0.75, the Rust tests 0.5, the measurement 0.25–0.5,
spec and programme 0.25.

## Open Questions

For the owner, none blocking:
1. **Change 43's scenario *The reference pair writes French's folder*** says `tables/fr/tags.tsv`
   is empty. Read as « until change 45 », as its requirement says; its words are best made so when
   change 43 is archived, as change 43's open question 4 does for change 39.
2. **The conditional and the imperative with no tense** (Spanish's encoding, not UD French's
   `Tense=Pres`): what D7's merge needs; the treebank measurement maps it.
3. **An invariable noun read in both numbers** (`temps`, `fois`): its card says « may also be the
   plural of *temps* », as Spanish's `crisis` does — and the article `un` (rank 7) and the pronoun
   `nous` (rank 34) get the same line from their invariable dictionary nouns (D4). Change 51 may
   leave a plural spelled like its singular unnamed, which removes it from all three; the readings
   keep it either way, since `les temps` is a plural.
4. **`Pack::readings` and a lemma that is another word's form** (D12). For change 48: drop the
   French golden's `été été` and `porte porte`, or ask the lemma the tables keep. The lookup is
   wrong in every language and reached today on the released en-fr and es-fr packs (`ablativas` read
   masculine): a core change of its own — every lemma-keyed read and write by the lemma's own id, a
   string the pool does not hold as a lemma reading nothing, a ranked lemma whose own form reads as
   another word refused by the builder — best proposed before change 41 gives French an
   out-of-lexicon rule; it would move `en-fr.golden` and `en-es.golden` on four probes each.
5. **M8's cost on the card** (D6): 543 nouns without a reading, `fait`, `élève`, `porte`,
   `nouvelle` among them; an override row of change 43 brings one back.
6. **Participles change 43 files under a noun's or a pronoun's spelling** (`citée` → *cité*,
   `tues` → *tu*, `quise` → *qui*, `marchée` → *marché*): D5 gives them no verb reading of that
   word and names the verb (*citer*, *taire*, *quérir*, *marcher*); 146 forms keep no reading of
   their own. Reading them as their verb is change 43's lemma choice — an override row, or its form
   of a form followed past a ranked lemma —, for the owner and change 43.
