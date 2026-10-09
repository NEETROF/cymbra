# Cymbra Lingua data-pack sources

The pipeline turns upstream datasets into one versioned `pack.lingua` per language pair, in two
stages (pin-lingua-pack-sources):

1. **Reduce** (`reduce-<pair>.py`, with the rules every pair shares in `reduce_common.py` and the
   rules of the Wiktionary edition its glosses come from in `reduce_edition_<native>.py`; see
   *The Wiktionary editions' rules*): the raw sources become the tables of `tables/<pair>/` and,
   when the pair is its studied language's reference, of `tables/<studied>/` (see *What a pack
   studies, whatever it glosses*), which are **committed** with the pair's `pin.json` — the
   record of each raw source (at a commit and by sha256, or as our own snapshot for kaikki, which
   upstream regenerates daily) and of the pack they build. Only `build.sh --update` (today's
   sources), `--reduce` (the pinned sources, after a change to the rules) and `--dry` (the monthly
   check) read raw sources, and only the `lingua-pack-update` workflow or a person runs them.
2. **Build** (`lingua-pack-build`): the pair's committed tables and its studied language's become
   the pack, offline, checked against the pair's `pin.json`. That is all a release or a pull
   request runs.

Raw sources and built packs are never committed. Only sources whose licence permits commercial use
are allowed; the builder additionally enforces the denylist and refuses to build on a denied source.

## EN → FR (the MVP pair)

| Table | Upstream source | Licence | Reduction |
|---|---|---|---|
| `forms.tsv` (`form → lemma`) | **ESDB** (English Speller Database, SCOWLv2, `en-wl/wordlist` `rel-2026.02.25`, the maintained successor of AGID), completed by **kaikki.org**'s Wiktionary form links | ESDB: permissive, Kevin Atkinson's notice and WordNet's (used by ESDB for parts of speech) in every copy; kaikki: CC BY-SA 4.0 + GFDL | ESDB's derived forms of `n`, `v`, `m`, `n_v`, `aj`, `av`, `a` and the comparisons of `d`, sizes ≤ 80, primary and equal spellings only (never a lesser variant: `born` is no form of `bear`, `art` none of `be`), no possessive; kaikki's form links only where the inflection is regular and the target is longer than two letters (recent plurals ESDB lacks: `smartphones`, `influencers`, `apps`). Then **one lemma per form**: a kept lemma maps to itself, any other form to the base Wiktionary names, else one with a gloss, else the most frequent (see *Words the inflection source gets wrong*); a listed hyphenated compound also gets its inflections (`t-shirts`, `mothers-in-law`) |
| `freq.tsv` (`lemma → rank`) | **wordfreq** English large list | CC BY-SA 4.0 (incl. SUBTLEX with Brysbaert's permission) | top 40k canonical lemmas, dense rank; CEFR words the pack would otherwise lack are added too — a hyphenated compound at its rarest part's rank, any other word with the lemmas of its frequency (after the list when rarer than all of them) |
| `gloss.tsv` (`lemma → gloss`) | **kaikki.org**: the French Wiktionary's (`frwiktionary`) English entries (`kaikki-Anglais.jsonl`), derived from the edition's dump (*The editions' dumps*; pinned as kaikki's per-language extract of 2026-09-24 until en-fr's next update) | CC BY-SA 4.0 + GFDL | one short French gloss per lemma, top ~20–30k lemmas (arbitrated by the 5 MB budget) |
| `level.tsv` (`lemma → CEFR`) *(optional)* | **CEFR-J Wordlist v1.5** (A1–B2, Tono Lab / TUFS) + **Octanove Vocabulary Profile C1/C2 v1.0** (C1–C2, Octanove Labs), both from the Open Language Profiles repo | CEFR-J: commercial use allowed with acknowledgement; Octanove: CC BY-SA 4.0 | lowest CEFR level per kept lemma across POS rows; the only pairing that covers A1→C2 with commercial-redistribution rights (Octanove was built to extend CEFR-J past B2) |
| `mwe.tsv` (`expression → gloss`) *(optional)* | **kaikki.org**: the same English entries of the French Wiktionary (`frwiktionary`), its multi-word ones | CC BY-SA 4.0 + GFDL | the same sense picker and cuts as `gloss.tsv`, over the entries whose headword holds a space; proper-noun-only entries and form-of senses dropped. The **keys are computed by the builder**, not here: each word goes through lingua-core's own lemmatiser against the lexicon that build assembled (`starting point` → `start point`), so a key is what the reader's cascade produces. Python cannot do it — it mirrors the analyser's irregulars but not its morphy rules or its out-of-lexicon plural. |
| `NOTICE` | all of the above | — | the full attribution stack, embedded in the pack and shown on the extension's Attributions page |

### Words the inflection source gets wrong

Inflections replaced AGID (last release 2016) with ESDB on 2026-09-26
(switch-lingua-inflections-to-esdb). Two rules were added for ESDB, the rest predates it:

- **An adjective of its own**: a form ESDB lists as an adjective in a commoner size class than the
  line deriving it is no inflection (`renowned`, an adjective at 35, is no form of the verb
  `renown`, listed at 80; `sophisticated`, `outstanding`, `situated` likewise). At an equal size the
  derivation stands (`tired` stays a form of `tire`).
- **No lemma behind it**: a form none of whose bases — nor any base of a base — is a kept lemma
  reads as a word of its own. ESDB knows rare bases AGID did not (`grandkid`, `policymaker`,
  `uprise`, `gree`), whose forms wordfreq ranks far above them: `grandkids`, `policymakers`,
  `uprising`, `greed` would otherwise drop out of the pack.

Measured on 200 documents (150 random and 10 recent Wikipedia articles, 20 Wikinews articles, 20
MDN pages; `measure/`): +411 glosses shown, +134 tokens resolved, 0.41 % of tokens changing lemma
— `header` no longer read as a comparison of `head` (AGID's), recent plurals resolved, UK
spellings joined to their lemma (`travelled`, `realised`).

What follows was written for AGID and holds for ESDB. AGID generated inflections mechanically, so some "inflections" are words of their own: it
lists `butter` as the comparative of `but`, `number` of `numb`, `his` as the plural of `hi`.
Other inflections are real but are the word readers actually meet (`ground` is far commoner
than the past of `grind`). Treated as inflections, those words never became lemmas, so
reading "butter" showed the gloss and CEFR level of "but". `reduce-en-fr.py::own_words`
keeps such a form as a lemma when the other sources contradict AGID:
- a CEFR list teaches it as a modal, or as a part of speech the relation cannot produce
  while its base is not taught as the part of speech the relation needs;
- Wiktionary gives it a meaning without calling it a form of that base, and the relation is
  not believable;
- or it is far commoner than its base.

-ing/-ed forms only qualify through the CEFR test, and the analyser's own irregular forms
are never kept apart. A hyphenated compound added to the pack is a lexical unit: statuses
set on its parts no longer reach it.

Before this rule, 1,186 of the 8,679 CEFR-J/Octanove words (14 %) had no level in the pack:
525 AGID-listed inflections, 590 rare words beyond the 40k frequency cut, and 71 hyphenated
compounds wordfreq never ranks. The pack now carries 8,299 of them. The 380 left out are
inflections that read as their base, and 356 of those bases have a level of their own.

The rule was chosen, and its thresholds set, by comparing candidate rules on the ~3,100 forms
whose lemma they disagreed on. Each form was labelled by two independent LLM reviewers, and
a third settled the 96 disagreements; the question was "which headword should a learner's
tracker count this form under, for its dominant use?". The pack built with the rule agrees
with those labels on 91 % of the frequency-weighted forms, against 45 % for the old
behaviour. Most remaining misses keep the old behaviour: lexicalised -ing nouns such as
`building` still map to their verb. The labels are a tuning aid, not a reference; they are
not shipped and nothing is built from them.

The expression table is **optional and additive** in the same way: a pair whose sources hold no multi-word entry ships no `mwe.tsv`, the builder emits no `expr`/`expr.zst` section, and the reader reports no expression. Only a new interface reads it, so it does **not** bump `analyzer_version`; it does bump `pack_version`, as any data change does. Measured on the 2026-09-22 snapshot: 17 437 entries, 345 KB of sections, a pack of 1 543 992 B (29.4 % of the 5 MiB budget).

The grammar tables are **optional and additive** too (add-lingua-word-grammar). `grammar.tsv` says
what each inflected form is — `form<TAB>lemma<TAB>tag<TAB>other|-` — from the slot ESDB lists it in
(its README, "The derived forms are as follows": a verb's past, past participle, -ing and -s, the
participle left out when spelled like the past; a noun's plural; an adjective's comparative then
superlative) and, for the regular forms kaikki's links add, from their ending. `senses.tsv` gives
the part of speech of each run of a gloss's senses, from kaikki's `pos`; the reducer groups a
gloss's senses by it and turns a `;` inside a sense into a comma, so "; " only ever separates
senses. The fourth field of `grammar.tsv` marks the relations `is_believable` accepts: only those
may be named on a card as another reading of a form the analysis reads as something else. An
acronym's entries (`AND`, `WHO`, `US`: headwords all in capitals) no longer gloss the common word
the reducer lowercases them into when that word has an entry of its own — the card of `and` read
« verbe Faire le ET de » — and keep glossing it when they are its only entries (`nato` « OTAN »).
A word's gloss holds up to eight senses, each within 300 characters and all of them within 800 —
the card pages it — and a cut falls on a word boundary with an ellipsis, never mid-word
(`cut_at_word`); expressions keep 42 / 80. The Wiktionary's notes to its own readers — a pointer
« → voir … » and the placeholder « Définition manquante ou à compléter. (Ajouter) » — are taken out
of every sense (`strip_wiki_notes`), and a sense made of nothing else is left out. A kept word
whose every sense only says which word it is a form of (`catacombs`, « Pluriel de catacomb ») takes
the senses of that word in the same part of speech. No new source and
no new licence: the tables derive from ESDB and kaikki. Neither moves `analyzer_version`. Measured
on the 2026-09-26 snapshot: 45 684 readings and 30 798 runs, a pack of 1 835 497 B (35.0 % of the
5 MiB budget) against 1 543 687 B before.

The tags are **Universal Dependencies** part-of-speech tags and features — tag names only, no UD
data. The vocabulary (`lingua-core` `packs::grammar`) was checked on 2026-09-27 against the UD
documentation and the feature statistics of the Romance treebanks, so that a Romance pack fits it
with no change to the container:

| What the learner meets | Tag | Checked against |
|---|---|---|
| passé simple, pretérito indefinido, passato remoto | `Mood=Ind\|Tense=Past` | `u/feat/Tense` ("the simple past … `Tense=Past`"); fr_gsd (`fut`, `firent`); it_isdt (`fu`, `ebbe`) |
| imparfait, imperfecto, imperfetto | `Mood=Ind\|Tense=Imp` | fr_gsd (`était`); es_gsd (`tenía`, `era`); it_isdt (`era`, `aveva`) |
| subjonctif imparfait | `Mood=Sub\|Tense=Imp` | es_gsd (`tuviera`, `tuviese`, `fuera`) |
| futur du subjonctif | `Mood=Sub\|Tense=Fut` | pt_bosque (`for`, `fôr`) |
| plus-que-parfait synthétique | `Tense=Pqp` | `u/feat/Tense` ("applies e.g. to Portuguese"); pt_bosque (`fora`, `fizera`) |
| conditionnel | `Mood=Cnd` | fr_gsd (`serait`, `pourrait`); pt_bosque |
| infinitif personnel | `VerbForm=Inf` with `Person`, `Number` | pt_bosque (`termos`, `terem`) |
| gérondif | `VerbForm=Ger` | `u/feat/VerbForm`; pt_bosque |
| clitic pronoun | `PRON\|PronType=Prs` with `Case`, `Person`, `Number`, `Reflex=Yes` for *se* | es_gsd (`lo` Acc, `le` Dat, `se` Reflex) |

Two points the check raised, neither a container change. es_gsd also marks the comitative
(`conmigo`: `Case=Com`), now in the vocabulary. And it gives `me`, `te`, `nos`, `os` the multi-value
`Case=Acc,Dat`; the vocabulary reads one value per feature, so a Spanish pack writes such a form as
two readings, one per case — or the parser learns UD's comma-separated values when that pack is
built. Either way the container stays as it is.

The CEFR level table is **optional and additive**: a pair without licence-clean CEFR data ships no `level.tsv`, the builder emits no `levels` section, and the reader falls back to frequency bands. Adding the section does **not** bump `analyzer_version` — it is behaviour-preserving (level-based presumed-known only activates once the user declares a level), so old cores load a level-bearing pack and simply ignore the section.

## ES → FR

Reduced by `reduce-es-fr.py` (add-lingua-spanish-forms-tables), for the Spanish programme
(`docs/lingua/spanish-programme.md`). No extension package carries the pack yet
(`enable-lingua-spanish`).

| Table | Upstream source | Licence | Reduction |
|---|---|---|---|
| `forms.tsv` (`form → lemma`) | **kaikki.org**: the English Wiktionary's (`enwiktionary`) Spanish section (`kaikki-Spanish.jsonl`), derived from the edition's dump (*The editions' dumps*; pinned as kaikki's per-language extract of 2026-09-28 until es-fr's next update); **UD Spanish-GSD** (train and dev, at a commit) for homographs | kaikki: CC BY-SA 4.0 + GFDL; GSD: CC BY-SA 4.0, read for counts only | the inflections a lemma's entry lists and the form-of links of a form's own entry: lowercased, NFC, single Spanish words. Never a verb with its clitics — a `combined-form`, or a sense naming the pronoun (`object-…` tags); the analyser's enclitic rule reads those — but a string that is also a plain form keeps it (`principales` → *principal*). **One lemma per form**: a reviewed override (`OVERRIDES` in the reducer, each with its reason; none yet, since an override takes the other lemma out of the pack), then GSD's counts of the form under each lemma, then the form's own entry, then the lemma's frequency, then the alphabet. Only the forms of kept lemmas that wordfreq attests, and every lemma's own form |
| `freq.tsv` (`lemma → rank`) | **wordfreq** Spanish list | CC BY-SA 4.0 | the top 60k canonical lemmas, dense rank; inflected forms, and combined forms that are no word of their own, skipped |
| `grammar.tsv` (`form → readings`) | **kaikki.org**, the same section: the tags of each form | CC BY-SA 4.0 + GFDL | the readings of the forms `forms.tsv` holds, under kept lemmas, as Universal Dependencies tags (add-lingua-spanish-grammar-tables). A verb form takes its mood, tense, person and number — the conditional as a mood, the *usted* imperative as a third person, the negative imperative left to the subjunctive it repeats — or its form (`Inf`, `Ger`, an agreed `Part`). A noun takes its gender (`es-noun`'s argument, else its senses' tags) on its own form and its plural; an adjective, determiner or pronoun its agreement. A pronominal form (`azotarse`) reads from its own entry. Each reading of another lemma than the form's is marked `other`: kaikki's tables are structured, so the card may name it |
| `gloss.tsv` (`lemma → gloss`) | **kaikki.org**: the French Wiktionary's Spanish entries; else the French translations the Spanish Wiktionary's Spanish entries list; else the French Wiktionary's French entries whose translation tables list the word. All three are derived from the French and Spanish editions' dumps (*The editions' dumps*) | CC BY-SA 4.0 + GFDL | the shared rules on the Spanish entries: up to eight senses grouped by part of speech (add-lingua-spanish-gloss-tables). A fallback gloss is up to three French words per part of speech, the commonest first from a table read backwards. A proper noun's translation glosses nothing; a gloss is never English and never a machine translation. Coverage below |
| `senses.tsv` (`lemma → runs`) | the same | CC BY-SA 4.0 + GFDL | the parts of speech of each gloss's runs of senses; a fallback gloss has one sense per part of speech. A noun's runs carry its gender, from the `es-noun` heads its readings read (`NOUN\|Gender=Fem`), unless it has both (add-lingua-spanish-word-card) |
| `mwe.tsv` (`expression → gloss`) | the same | CC BY-SA 4.0 + GFDL | the Spanish multi-word entries, then the multi-word headwords the translations give; `LOCUTIONS` in the reducer, written by a person, wins (empty so far). The builder keys them through the lexicon |
| `level.tsv` (`lemma → CEFR`) | none: derived from `freq.tsv` and `gloss.tsv` | that of `freq.tsv` | **estimated**, since no Spanish CEFR list can be shipped (ELELex is NC, the PCIC all rights reserved). The commonest lemmas whose French gloss is not only a proper noun's take, in rank order, English's band sizes (1,020 A1 … 876 C2): 8,302 lemmas. The manifest says `levels_estimated`, and the extension labels the levels « estimé » (add-lingua-spanish-levels) |
| `NOTICE` | all of the above | — | the attribution stack, embedded in the pack |

**Measured on UD Spanish-PUD** (CC BY-SA 3.0), never committed and never read by the reduction.
`measure/es-pud.sh` fetches it at a commit, builds the pack from the committed tables, and runs the
real analyser over it (`lingua-pack-measure`). Punctuation, numbers, symbols, foreign words and
proper nouns are left out. The 2026-10-04 tables pass the programme's gates:
- 99.38 % of 19,276 words resolve in the lexicon (gate 98.5 %);
- 95.79 % of 9,439 content words take PUD's lemma (gate 93.5 %). PUD keeps `gran` as its own lemma
  but takes `primer` to *primero*: reading apocopes as their full words moved it from 95.92 %;
- 97.95 % of 634 auxiliaries take PUD's lemma (gate 97 %).

**French glosses**, the share of the commonest lemmas glossed on the 2026-10-04 tables:

| Lemmas | French Wiktionary | with the translations |
|---|---|---|
| top 5,000 | 82.7 % | 87.6 % |
| top 10,000 | 69.4 % | 77.2 % |
| top 20,000 | 54.2 % | 63.7 % |
| all 60,000 | 28.9 % | 37.9 % |

22,755 lemmas are glossed, 17,341 from the French Wiktionary. There are 15,133 expressions: 2,950
from the French Wiktionary's Spanish entries and 12,183 from the translations.

**Estimated levels**, the rule measured on English's 8,302 CEFR lemmas, ranked the same way: 39.8 %
take their list's level, and 82.6 % are within one level of it. The scale is monotone (the mean true
level rises from 1.67 at A1 to 5.03 at C2), so the three-band fallback of the programme's decision
D1 is not needed. The pack is 2,190,188 B, with the grammar, the glosses and the levels.

## ES → EN

Reduced by `reduce-es-en.py` (add-lingua-pack-es-en): Spanish glossed in English, the first pair of a
studied language's second native language (`docs/lingua/language-matrix-programme.md`, change 21).
No extension package carries the pack: `packs.json` does not list it until change 34.

es-en reads Spanish's tables in `tables/es/` as committed — es-fr's reduction writes them — and
writes its native side alone: its lemmas and their ranks are `tables/es/forms.tsv` and `freq.tsv`
(capped at 60,000, as for every pair studying Spanish), its readings, levels and dictionary words
es-fr's. Its reducer loads `reduce_common.py` and `reduce_edition_en.py`, not es-fr's reducer nor
the French edition, so nothing of es-fr moves with it.

| Table | Upstream source | Licence | Reduction |
|---|---|---|---|
| `gloss.tsv` (`lemma → gloss`) | **kaikki.org**: the English Wiktionary's (`enwiktionary`) Spanish section — the file es-fr reads for Spanish's forms, derived from the edition's dump (pinned as kaikki's per-language extract of 2026-10-03 until es-en's next update); else the English translations the Spanish Wiktionary's Spanish entries list (`kaikki-es-traductions-en.jsonl`, derived from the Spanish edition's dump; *The editions' dumps*) | CC BY-SA 4.0 + GFDL | the shared rules on the English Wiktionary's Spanish entries, read first as meanings, not as the page's layout (`reduce_edition_en.read_as_meanings`, below), and cleaned by the English edition's (`reduce_edition_en.EN`): up to eight senses grouped by part of speech, in lower case as the edition writes them. A fallback gloss is up to three English words per part of speech, in the table's order. No inverted table (the English Wiktionary's English entries are en-es's), no pivot, no machine translation |
| `senses.tsv` (`lemma → runs`) | the same | CC BY-SA 4.0 + GFDL | the parts of speech of each gloss's runs; no gender — the builder gives a noun's runs the gender of its readings in `tables/es/grammar.tsv` |
| `mwe.tsv` (`expression → gloss`) | the same | CC BY-SA 4.0 + GFDL | the Spanish multi-word entries, then the multi-word headwords the translations give; `LOCUTIONS` in the reducer is empty |
| `NOTICE` | both sides' sources | — | the English Wiktionary's Spanish section (forms, readings, glosses), the Spanish Wiktionary's translations, the French Wiktionary (es-fr's glosses decide the dictionary words and which lemmas take a level), wordfreq and UD Spanish-GSD; the levels, es-fr's estimate from its French glosses, said so. The manifest says `levels_estimated` |

**English glosses**, the share of the commonest lemmas glossed on the 2026-10-08 tables, held by the
`reduce` job to es-fr's published figures (`gloss_coverage.py --pair es-en`, its floor in `FLOORS`)
and published nowhere until the pair ships:

| Lemmas | English Wiktionary | with the translations | es-fr (the floor) |
|---|---|---|---|
| top 5,000 | 92.9 % | 93.0 % | 87.6 % |
| top 10,000 | 86.3 % | 86.5 % | 77.2 % |
| top 20,000 | 76.3 % | 76.5 % | 63.7 % |
| all 60,000 | 52.9 % | 53.1 % | 37.9 % |

31,885 lemmas are glossed, 31,756 from the English Wiktionary. 9,928 of them are no dictionary word
of Spanish (es-fr glosses none of them), and 798 dictionary words have no English gloss, so the pack
carries a lexical section: its dictionary words are es-fr's. There are 15,515 expressions: 14,803
from the English Wiktionary's Spanish entries and 712 from the translations. The pack is 2,567,804 B.
A letter glosses no word: the English edition's letter rule takes « the letter r » and the Spanish
spelling alphabet's « the letter E in … » (36 glosses ended on one), a pre-pass of
`reduce_edition_en.py` drops the entries written under a single capital letter (`A` « bishop », `C`
« abbreviation of caballo »), and the direct table leaves out the letters the Spanish Wiktionary
translates as themselves (`b` « b »).

**Each pair pins its own fetch, one release per pair** (D2). es-en's Spanish section is es-fr's
file, derived when es-en is updated and published under es-en's own release, `release_tag(pair,
snapshot)` (`pack_sources.py release-tag`); an update publishes a pair's own assets only (`assets
--release`), and its release step fails when it cannot list them. When es-fr's update brings es-en
along, es-en is reduced from its own pin and nothing of it is published. Fetched release assets are
kept in `work/cache/<sha256>` (written whole or not at all; an entry that does not decompress to its
name is deleted, the error naming it), so an asset two pins name is fetched once per run and a pair
reduced again on the same machine fetches nothing again. es-en's first update was dispatched alone
(2026-10-08), before the dumps: it fetched kaikki's per-language extract of the section and
published it, with its derived translations, as `lingua-pack-sources-es-en-2026.10.08`. Its
extract is kaikki's regeneration of 2026-10-03, not the one es-fr pins, so the `reduce` job fetches
both (52 MB compressed each); each pair's next update moves it to the English edition's dump. es-en's re-reduction from
its pinned sources takes about 30 s on a laptop, fetch and pack build included; its first update run
took 6 minutes, and the `reduce` job reduced en-fr, es-fr and es-en again in 3 min 38 s of its
45-minute timeout (2026-10-08).

**Meanings, not the page's layout** (refine-lingua-es-en-glosses). The English Wiktionary nests a
sense under its parents' glosses, writes a shortened form as a pointer that carries its meaning, and
has its own typography: read as written, es-en glossed « venir » by two sense-group labels, « su »
« apocopic form of suyo », « lo » by its article alone, « como » by the city of Como. A pre-pass of
the English edition, `read_as_meanings`, run after the letters' passes and before the etymology
merging, reads a sense nested under a label or a pointer by its own gloss (D2); a shortened or
respelled form, known by its wording and never by a tag alone, as the meaning it carries or its
target's senses in the same part of speech from a target of three letters or more, and a pronoun's
case form as the meaning after its colon or semicolon (D3); a function word's gloss without a place
of the same spelling first, an acronym's lines in their place (D4); one English typography — no text
after a line break, no source's numbered sense, the edition's descriptions in lower case, one
ellipsis « … », straight double quotes paired “ ” (D5). es-en was reduced again from its pinned
release, its pin's snapshot, studied record and sources byte for byte: 268 rows change (111 of the
top 10,000), 9 lemmas and 25 expressions gain a gloss, none loses one, 332 expressions change, and
the coverage holds to the decimal. Left, measured, for the owner or a shared fix (its design, D6):
the labels the packs do not carry, the part of speech a row opens on, a proper noun before a common
word, « q », « k » and « t » borrowing through an abbreviation, upstream text, and « etc »'s period,
which `reduce_common.clean_gloss` takes from every pair.

**The English edition's two settings** are es-en's alone (D5, M20): `LONG_PARENTHESIS` (a
parenthesis of 40 characters or more taken out: 1,057 of the top 10,000 glosses would change) and
`MERGE_SAME_POS_ETYMOLOGIES` (a word's entries of one part of speech merged before the round-robin:
247 would change). Both are committed at their defaults, off, until the owner picks them on samples
of the top 10,000; a value chosen re-pins es-en alone.

## EN → ES

Reduced by `reduce-en-es.py` (add-lingua-pack-en-es): English glossed in Spanish, the first pair
glossed in a native language no shipped pack speaks — Spanish speakers studying English
(`docs/lingua/language-matrix-programme.md`, change 22, decision M1). No extension package carries
the pack: `packs.json` does not list it until change 35.

en-es reads English's tables in `tables/en/` as committed — en-fr's reduction writes them — and
writes its native side alone: its lemmas and their ranks are `tables/en/forms.tsv` and `freq.tsv`
(the commonest 40,000, as en-fr keeps them, and the level lists' words of `level.tsv` beyond them:
40,685 lemmas), its readings, levels and dictionary words en-fr's. Its reducer loads
`reduce_common.py` and `reduce_edition_es.py`, not en-fr's reducer nor the French edition, so
nothing of en-fr moves with it (the digest test says so against en-fr's, es-fr's and es-en's pins).

| Table | Upstream source | Licence | Reduction |
|---|---|---|---|
| `gloss.tsv` (`lemma → gloss`) | **kaikki.org**: the Spanish Wiktionary's English entries (`kaikki-es-English.jsonl`, derived from kaikki's dump of the whole edition, `pack_sources.py DUMPS`); else the Spanish translations the English Wiktionary's English entries list (`kaikki-en-traductions-es.jsonl`, derived from the English edition's dump; pinned from kaikki's extract of that section, served uncompressed, until en-es's next update); else the English translations the Spanish Wiktionary's Spanish entries list, read backwards (`kaikki-es-traductions-en.jsonl`, es-en's derivation run again on en-es's own snapshot of the dump) | CC BY-SA 4.0 + GFDL | the shared rules on the Spanish Wiktionary's English entries, cleaned by the Spanish edition's (`reduce_edition_es.ES`): up to eight senses grouped by part of speech, opening on a capital as the edition writes them. A fallback gloss is up to three Spanish words per part of speech — in the table's order from the direct table, the commonest Spanish word first (wordfreq) from the inverted one, as es-fr orders its inverted table by French frequency. A letter glosses no word in either direction: a single letter is glossed only by a sense that is neither the letter nor a name borrowed through it (a `character` entry, a one-letter word translated as itself or under a noun entry — the letter's name, a note, a grade —, « Nombre de la letra Q »; read backwards, no one-letter English word at all, since the entry is the Spanish word's and says nothing of the English side: the Spanish Wiktionary's `do` lists « C »). No pivot, no machine translation. Coverage below |
| `senses.tsv` (`lemma → runs`) | the same | CC BY-SA 4.0 + GFDL | the parts of speech of each gloss's runs; English's readings come from `tables/en/grammar.tsv` |
| `mwe.tsv` (`expression → gloss`) | the same | CC BY-SA 4.0 + GFDL | the Spanish Wiktionary's multi-word English entries, then the multi-word headwords the translations give; `LOCUTIONS` in the reducer is empty |
| `NOTICE` | both sides' sources | — | the studied side as en-fr's notice credits it (ESDB with its WordNet notice, wordfreq, the French Wiktionary's form links and dictionary words, CEFR-J, Octanove), and the native side (the Spanish Wiktionary's definitions and English translations, the English Wiktionary's Spanish translations). The levels are CEFR-J's and Octanove's, not estimated |

**Spanish glosses**, the share of the commonest lemmas glossed on the 2026-10-08 tables, held by the
`reduce` job to a floor the owner sets on the pull request (`gloss_coverage.py --pair en-es`
against `FLOORS["en-es"]`, the one place the value lives, no `--floor` passed; D3, the programme's
risk 5 and M6's rule) and published nowhere until the pair ships:

| Lemmas | Spanish Wiktionary | with the translations | the floor (proposed) | the study |
|---|---|---|---|---|
| top 5,000 | 80.4 % | 93.0 % | 91.4 % | 93.4 % |
| top 10,000 | 62.9 % | 85.0 % | 83.2 % | 85.2 % |
| top 20,000 | 42.5 % | 71.7 % | 69.9 % | 71.9 % |
| all 40,685 | 25.2 % | 54.0 % | — | — |

21,964 lemmas are glossed: 10,247 from the Spanish Wiktionary's definitions,
11,213 from the English Wiktionary's Spanish translations and 504 from the
Spanish Wiktionary's English translations read backwards. **The translation-table share** (D4): of
the 8,495 glossed lemmas among the 10,000 commonest, 26.0 % come from a translation
table rather than from a definition (2,135 direct, 73 inverted) — the reducer
measures it into `work/en-es/measures.json`, stored in no pack, and `pack_report.py --measures`
shows it beside the coverage in the update's summary. 2,501 glossed lemmas are no
dictionary word of English and 5,336 dictionary words have no Spanish gloss, so
the pack carries a lexical section: its dictionary words are en-fr's. There are 17,094
expressions. The pack is 1,688,931 B. A sample of 100 glosses, marked by source, is in
`tables/en-es/README.md`.

**Meanings and the translators' words** (refine-lingua-en-es-glosses). The Spanish Wiktionary writes
notes to its readers into its senses, opens an entry on its oldest sense, writes a surname's note
under the capitalised headword spelled like a common word and heads a possessive « adjetivo »; the
English Wiktionary's translators write their notes inside the Spanish words they list; read
backwards, a Spanish word is listed once per part of speech of its own. Read as written, en-es glossed
« will » « Deseo, inclinación, disposición; …; Apellido; Hipocorístico de William », « smith »
« Apellido; Herrero », « a » « Un, una. A veces se omite en la traducción », « orchestra » « Orquesta,
orquestra (disused) » and « lengthy » « Largo; Largo; Largo; Largo ». The Spanish edition takes out
its notes to its readers — a maintenance template, a disambiguation note, a reference to numbered
senses, the expansion notice, and a usage note after the meaning from a closed list of openers — and
its pre-pass `read_as_meanings` writes the senses it marks obsolete or outdated after the others and
one Spanish typography (one ellipsis « … », « » for straight double quotes) (D2, D5, D6); en-es's
`english_entries`, run after it, leaves a surname's or a given name's note off a word that has an
entry of its own in lower case, reads a possessive or demonstrative adjective as a determiner and
names the -ing form « forma en -ing », as the card does (D3, D4, D6); `read_translated` leaves out
the direct table's disused words and the notes its translators wrote inside a word — a loanword's
respelling, a label holding no Spanish word, a sense number, an English usage note — and lists an
inverted Spanish word once, under the first of its parts of speech the English word's readings name
(D7). en-es was reduced again from its pinned release, its pin's snapshot, studied record and sources
byte for byte: 295 rows change (149 of the top 10,000), the first sense of 140 (63), 64 expressions;
« malign », whose one translation is disused, and two expressions glossed by a disused word alone lose
their gloss, none gains one, and the coverage holds to the decimal. Left, measured, for the owner or a
shared fix (its design, D8): the labels the packs do not carry, the part of speech a row opens on, the
quantifiers' heading, the inverted table's regional words, a name's own row, the direct table's
repeated word across the English word's parts of speech, upstream wording, and « etc »'s period,
which `reduce_common.clean_gloss` takes from every pair.

**A pair whose sources are dumps alone** (D2). en-es was the first pair with no extract of its own
and no `sources.kaikki` in its pin: everything it reads is derived at an update, in one pass per dump, and kept as the
assets of its own release, `lingua-pack-sources-en-es-<snapshot>` — `kaikki-es-English.jsonl` and
`kaikki-es-traductions-en.jsonl` from the Spanish Wiktionary's dump (103 MB gzipped, es-fr's and
es-en's address, en-es's own snapshot of it), `kaikki-en-traductions-es.jsonl` from the English
Wiktionary's English extract (`kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl`,
served uncompressed: 3,335,546,346 B as served, measured and recorded at the first update,
kaikki's regeneration of 2026-10-03). `derive` reads a plain or a gzipped dump alike, told apart by the gzip magic; the
release notes name no extract; a pinned reduction fetches the three derived files and nothing
larger. Why the extract and not the raw English dump the programme's risk 6 names: the raw dump is
several times the extract and holds every language's entries; `derive` reads both, so the address is
the only difference. Change 38 (`migrate-lingua-pack-sources-to-raw-dumps`) switched the address:
`kaikki-en` names the English edition's dump from en-es's next update on; the file derived from
it holds the extract's entries, with some tables in the dump's order, and 60 entries repeat 77
translations they already list (the extract lists each once; *Extract and dump are measured against
each other*). en-es's first update was dispatched alone on its pull request branch
(2026-10-08, run [37771510878](https://github.com/NEETROF/cymbra/actions/runs/37771510878), 4 min 57 s whole, 2 min 16 s of it the extract's fetch and derivation): it fetched the dump and the extract, published the derived files as
`lingua-pack-sources-en-es-2026.10.08`, and the pinned reduction that committed the tables followed on the same branch
(`build.sh --reduce en-es`, about 6 s on a laptop, fetch and pack build included).

## FR → EN: French studied

Reduced by `reduce-fr-en.py` (add-lingua-french-forms-tables): fr-en is **French's reference pair**,
whose reduction writes `tables/fr/` (`tables/fr/studied.json`), and which every later pair studying
French reads as committed (fr-es, change 49). It writes French's forms and ranks alone today: its
glosses, readings and levels are changes 48, 45 and 46, so `tables/fr-en/gloss.tsv`,
`tables/fr/lexical.tsv` and the pinned tag pool `tables/fr/tags.tsv` are committed empty. No extension
package carries the pack: `packs.json` does not list it until change 52. Its reducer loads
`reduce_common.py` alone; no shared module was edited for it, so no other pair's rule digest moved.

| Table | Upstream source | Licence | Reduction |
|---|---|---|---|
| `tables/fr/forms.tsv` (`form → lemma`) | **kaikki.org**: the English Wiktionary's (`enwiktionary`) French section (`kaikki-French.jsonl`, `DUMPS["fr-en"]`), derived from the English edition's dump — the catalogue's file, no new derivation; **UD French-GSD**'s training and development sections at a commit (`PINNED["fr-en"]`), for homographs | kaikki: CC BY-SA 4.0 + GFDL; GSD: CC BY-SA 4.0, read for counts only | the inflections a lemma's entry lists and the first word of each form-of target of a form's own entry: lowercased, NFC, the typographic apostrophe read as `'`; never kaikki's bookkeeping, a multi-word construction, an inflection tagged alternative, obsolete, archaic, rare, dated, uncommon, misspelt, nonstandard, proscribed, abbreviated, clipped or a pronunciation spelling, nor a gender or number marker a head left among the forms (`m` under *Paris*). **A form of a form along one part of speech** (`dirigée` → `dirigé` → *diriger*; `étés`, a noun's plural, not to *être*). **French's tokenisation** (M21): the fourteen elided pieces by a reviewed table (`ELISIONS`, `l'` → *le*); no plain word beginning with a piece (`c'est`, `d'abord`, `l'on`), a hyphenated run may be one (`c'est-à-dire`); the dictionary's nouns, adjectives, adverbs, pronouns and prepositions ending in a pronoun listed whole (`rendez-vous`, `qu'en-dira-t-on`), the reduction failing if one is not; `au` and `aux` no form; `du` and `des` words of their own; a verb joined to its pronouns by hyphens (`souviens-toi`) no form. **A spelling variant** (`coeur`, `connait`) reads as the word it spells. **One lemma per form**: a name and a commoner word keep the word, then a reviewed override (`OVERRIDES`: no homograph; two rows mend the source's copy errors, `fatiguée` « feminine singular of parlé » and `bridée`), GSD's counts, the form's own entry, the lemma's frequency, the alphabet. Only the forms of kept lemmas that wordfreq attests, every lemma's own form, and the pieces |
| `tables/fr/freq.tsv` (`lemma → rank`) | **wordfreq** French list; **UD French-GSD**'s frequency of a hyphenated lemma | CC BY-SA 4.0 | the top 60,000 lemmas, dense rank: wordfreq's order, inflected forms skipped, and its elision stems (`l`, `d`, `qu`, `jusqu`, …), which carry the pieces' frequency, and `au`/`aux`. A word whose own form reads as another gives its rank to the next (`tenue`, whose every form reads as *tenir*; `donnée`, read as *donner*): a pack finds a lemma by its own form, so the builder would key its rank on the other word. A hyphenated word only when GSD attests it, at the lower of wordfreq's estimate and GSD's own frequency, after wordfreq's words of the same frequency (`peut-être`, 941; never an inversion such as `est-il`); the nouns ending in a pronoun GSD never meets at the cut's last ranks |
| `tables/fr-en/gloss.tsv` | none yet | — | empty: the glosses are change 48's |
| `NOTICE` | all of the above | — | the attribution stack, embedded in the pack |

**The sources pinned** (`tables/fr-en/pin.json`): the French section derived on 2026-10-08 from the
English edition's dump regenerated on 2026-10-03 08:24 (decompressed sha256 `93b79aac…`,
25,614,284,530 B; 2,981,058,381 B gzipped) — 403,269 entries, 510,058,226 B, sha256 `2d7bbe5f…`,
26,410,463 B as its zstd level-19 asset `kaikki-French.jsonl.zst`, under fr-en's own release
`lingua-pack-sources-fr-en-2026.10.08`; GSD's `fr_gsd-ud-train.conllu` and `fr_gsd-ud-dev.conllu` at
`94d5b68e185fc22a9ef292040e84f476d36d9b0e` (25,555,018 and 2,573,677 B), the default branch's head of
2026-05-06; wordfreq 3.1.1. An update reads no dump but the English edition's, which es-fr and es-en
read already: the monthly dry run derives the French section in the same pass, one more reduction and
no more download.

**The cut**: 60,000 lemmas and their attested forms, as Spanish — 124,050 forms, `forms.tsv`
2,255,819 B and `freq.tsv` 844,901 B. On the design's prototype tables, 40,000 lemmas passed the
gates too, 0.16 points of resolution lower on PUD, and every form nobody writes would have added
88,678 rows for 0.02 points. The pack built from the two tables alone is 1,239,671 B
(`tables/fr-en/README.md`, with M8's cost — the 135 dictionary nouns among wordfreq's 5,000 commonest
words that read as a verb — and the twelve determiners and pronouns ranked on their own).

**Measured on UD French-PUD and on GSD's test section** (`measure/fr-ud.sh`), neither committed nor
read by the reduction, each fetched at a commit and checked by sha256 — PUD
(`db260db10fe728853c549760801229ef4e7b16e1`, sha256 `4dfed37b…`, CC BY-SA 3.0) held to Spanish's
gates, GSD's test section reported:
- PUD: 99.12 % of 20,232 words resolve (gate 98.5 %), 96.30 % of 9,573 content words and 99.90 % of
  1,030 auxiliaries take PUD's lemma (gates 93.5 % and 97 %);
- GSD's test section: 98.89 % of 8,049 words resolve, 95.67 % of 3,791 content words and 99.72 % of
  359 auxiliaries take its lemma.

French's analyser at `0.2.0`; it reads the treebanks' words alike at `0.1.0`.

## The editions' dumps

kaikki is read at **three addresses, one dump per Wiktionary edition**
(migrate-lingua-pack-sources-to-raw-dumps, change 38 of `docs/lingua/language-matrix-programme.md`):
kaikki has marked its per-language files deprecated and keeps its dumps of whole editions. Every file
a pair reads of kaikki is derived from the dump of the edition that writes it, in one pass per
edition (`pack_sources.py derive`): a language's entries as the dump writes them, or the
translations an edition's entries list into one language, cut down to those. `pack_sources.py
EDITIONS` is the catalogue — each edition's address and every file derivable from it that a pair of
the programme reads — and `DUMPS[pair]` names the files a pair reads, by edition; a pair that needs a
file the catalogue lacks adds it there, in the edition that writes it, and nowhere else. The files
derived from the dumps served on 2026-10-08:

| Edition: the dump, as served on 2026-10-08 | File | Kind | Derived | Read by |
|---|---|---|---|---|
| **English** (`kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz`): 2,981,058,381 B gzipped, 25,614,284,530 B decompressed, regenerated 2026-10-03 08:24 | `kaikki-Spanish.jsonl` | Spanish entries | 928,737,891 B | es-fr (forms, readings, genders), es-en (glosses) |
| | `kaikki-French.jsonl` | French entries | 510,058,226 B | fr-en (change 48) |
| | `kaikki-en-traductions-es.jsonl` | English entries' Spanish translations | 13,762,773 B | en-es (direct) |
| | `kaikki-en-traductions-fr.jsonl` | English entries' French translations | 13,003,913 B | fr-en (inverted) |
| **French** (`kaikki.org/frwiktionary/raw-wiktextract-data.jsonl.gz`): 736,590,407 B gzipped, 6,865,136,428 B decompressed, regenerated 2026-10-02 00:10 | `kaikki-Anglais.jsonl` | English entries | 145,705,523 B | en-fr |
| | `kaikki-fr-Espagnol.jsonl` | Spanish entries | 212,331,331 B | es-fr |
| | `kaikki-fr-traductions.jsonl` | French entries' Spanish translations | 6,191,621 B | es-fr (inverted), fr-es (direct) |
| | `kaikki-fr-traductions-en.jsonl` | French entries' English translations | 14,573,988 B | fr-en (direct) |
| **Spanish** (`kaikki.org/eswiktionary/raw-wiktextract-data.jsonl.gz`): 103,226,106 B gzipped, 1,233,016,167 B decompressed, regenerated 2026-10-02 12:12 | `kaikki-es-English.jsonl` | English entries | 36,426,539 B | en-es |
| | `kaikki-es-Frances.jsonl` | French entries | 7,438,610 B | fr-es (change 49) |
| | `kaikki-es-traductions.jsonl` | Spanish entries' French translations | 1,537,580 B | es-fr (direct), fr-es (inverted) |
| | `kaikki-es-traductions-en.jsonl` | Spanish entries' English translations | 2,200,504 B | es-en (direct), en-es (inverted) |

The existing names are kept — a name is a reducer's input — and the new ones carry their edition,
in ASCII alone: GitHub renames a release asset whose name holds another character on upload
(`kaikki-es-Frances.jsonl`, not kaikki's « Francés »). fr-en and fr-es register what they read and
derive nothing new.

**A dump is recorded, never kept.** It is fetched into `work/dumps/`, read once and deleted; the
English edition's alone is above the 2 GiB a release asset may weigh. A pair's derived files are
zstd-compressed and published under the pair's own release, `lingua-pack-sources-<pair>-<snapshot>`,
pinned by the sha256 of their decompressed bytes, as before (the largest, the Spanish section, about
52 MB compressed). The pin records each edition's dump as a source, `kaikki-<edition>`: its address,
the day it was fetched, kaikki's regeneration date (`last_modified`), `dump` — the sha256 and size
of its decompressed bytes, hashed in the pass that derives from it, and its size as served — and the
`files` derived from it. Two pairs updated from one regeneration therefore carry one dump sha256 and
files of one sha256, which the asset cache fetches once; a re-reduction fetches the derived files and
never a dump. The release notes name each dump the same way (`pack_sources.py dumps --pin`).

**A dump is read once per run.** At the first read of a run, the edition's whole catalogue is
derived into `work/editions/<edition>-<snapshot>/` (`build.sh` passes `LINGUA_EDITIONS`), and a later
pair of the run copies what it reads from there: the pass is the cost, writing a file no pair of the
run reads is not. The folder counts once it holds `dump.json`, written last, so a pass cut short is
started again; it is the run's own and `lingua-pack-update` removes it at the end. On a laptop it
outlives the run, one folder per edition and day: a folder of the same day counts only while its
`dump.json` records today's address and catalogue and every file of it is there, else it is derived
again, and `work/editions` can be removed at any time — the next update fetches the dumps again.
An update of one pair fetches only the dumps of the editions it reads (es-en: the English and
Spanish ones); the pairs it brings along read their own pins. The monthly dry run checks every pair
**in one job**, in `pairs` order, so each dump is fetched once a month rather than once per pair reading its edition; each pair
reduces into a dry root of its own (`work/dry/<pair>`), so a later pair never lays the committed
studied folder over the drift its reference wrote, and a pair that fails is named, the loop goes on,
and the job fails at the end.

**A pin written before the dumps stays readable** (D5). en-fr's, es-fr's and es-en's pins name
kaikki's per-language extract (`kaikki`: `asset`, `sha256`, the extract's address); en-es's names
files derived from the Spanish dump and from the English extract, read as a dump is. `fetch-pinned`
reads a record by its shape — an extract's `asset` or derived `files` — from the release it names,
checked by sha256, keeps a legacy `kaikki` record (nothing is pruned, the pin's bytes do not move)
and fetches the extract under the name its asset gives (`kaikki-Spanish.jsonl.zst` →
`kaikki-Spanish.jsonl`). The `reduce` job reproduces every committed table, manifest and pin from
them as before, and no pin, table, pack or baseline moved in this change. A pair moves to the dumps
at its next update, the owner's dispatch: its pin then names the editions' dumps and no extract.

**Extract and dump are measured against each other** (D6). On 2026-10-08 kaikki served, side by
side, the English dump of 2026-10-03 08:24 and the Spanish extract of 2026-10-03 10:55 (the bytes
es-en pins), the French dump of 2026-10-02 00:10 and the Anglais extract of 00:18, and the English
extract of 2026-10-03 11:09 that en-es's direct translations were derived from. Each pair was reduced
from its pinned sources twice, the one file read once as the extract gives it and once as the dump
gives it (`pack_report.py --identical` on every folder):

| Section | The two files | Reduced |
|---|---|---|
| The English edition's Spanish entries | the same 811,049 entries; the extract adds an `id` to each sense and assigns the page's categories to the senses by disambiguation, as objects, the dump keeps them on the entry as names; 37 entries or runs of entries stand elsewhere in the file. 1,054,565,723 B against 928,737,891 B | es-en: identical. es-fr: `es-fr/` identical; `es/` identical but `grammar.tsv`, which gains 4 readings from the dump — the feminine plurals of *beta*, *delta*, *kappa* and *zeta* (`betas beta NOUN\|Gender=Fem\|Number=Plur`). es-fr's letter-name rule (`_names_a_letter`) reads a sense's categories, where the extract puts « Greek letter names » and the dump does not |
| The French edition's English entries | the same 194,304 entries, differing the same way (sense ids, categories as objects); 201,505,597 B against 145,705,523 B | en-fr: `en-fr/` and `en/` identical |
| The English entries' Spanish translations | the same 68,058 words and parts of speech, the dump writing 3 more entries for them (`do` and `ceno-`); 1,703 entries list the same translations in another order — the extract assigns each table to the sense it translates, the dump keeps it where the page writes it — and 60 repeat 77 translations they already list (the extract lists each once). 13,757,388 B against 13,762,773 B | en-es: 186 glosses and 55 expressions take their words in another order or another third word (the direct fallback takes the table's first three); no row added or removed, the coverage the same (93.0 / 85.0 / 71.7 %); the same 186 and 55 under refine-lingua-en-es-glosses's rules |

So the dumps give en-fr's and es-en's tables byte for byte. The two pairs that differ keep their
pins, and each difference reaches the committed tables only through the pair's next update, whose
report names it beside the upstream drift (*A pair whose two readings differ*). es-fr's four
readings break *A letter's name gives no reading of its plural* — `betas` read as the plural of the
letter *beta* —, so a change of its own must fix them before es-fr's next update is merged. Not by
reading the entry's categories: the dump puts the section's categories on every entry of the page
(« Greek letter names » on *beta* the letter and on *beta*, a matter, masculine), and the rule
would drop the committed `Masc|Plur` readings of those other nouns; the likely fix is a rule on the
sense's gloss (« beta; the Greek letter Β, β », « Greek letter delta », « the letter Z »), tested on
a fixture shaped as the dump writes an entry. en-es's 186 glosses and 55 expressions call for no
reducer fix — the derived file keeps no sense glosses to order the words by —: en-es's next update
carries them, and the owner judges the 186 glosses there. Neither difference is a defect of the
dumps: kaikki's post-processing of the extract moves categories and translation tables.

The regenerations the pins record were still served that day, so the opportunity was taken too:
es-fr's three derived files and es-en's and en-es's derived from the French and Spanish dumps re-derive
to their pinned sha256 byte for byte (the new `derive`, hashing in the same pass, writes what the old
one wrote); es-en's committed tables reproduce byte for byte from the English dump of 2026-10-03
08:24 (the Spanish section derived from it, Spanish's committed tables, every other source pinned) —
and again under refine-lingua-es-en-glosses's rules, whose D3 lending, D4 and the round-robin read
file order: es-en's re-pinned tables reduced from that Spanish section (sha256 `7622948c…`, the dump
of 08:24 still served on 2026-10-08) are identical (`pack_report.py --identical`);
and en-es's direct translations re-derive from today's English extract to the sha256 its pin
records. Under refine-lingua-en-es-glosses's rules too, whose direct-table rules act on each word the
file lists: en-es's re-pinned tables reduced with the direct table derived from the English dump of
08:24 (`kaikki-en-traductions-es.jsonl`, 13,762,773 B, sha256 `2723f3e2…`; the dump still served on
2026-10-08, its `Last-Modified` and size unchanged), every other source pinned, differ from the
pinned ones (`pack_report.py --identical`) in the same 186 glosses and 55 expressions as before them
— words in another order or another third word —, no row added or removed and no run moved, the
coverage the same: en-es's next update carries them, and the owner judges them there.

**Measured** (D4, D8). On a laptop (Apple silicon), the pass of `derive` over each edition's dump,
deriving its whole catalogue and hashing the decompressed stream: the English 2 min 39 s (25.6 GB
decompressed), the French 52 s, the Spanish 9 s, at about 90 MB of memory; the downloads, at the
laptop's throughput, 2 min 3 s, 1 min 1 s and 12 s. On the implementation pull request's runners:
the monthly job as one job over en-fr, es-fr, en-es and es-en — run
[37805366125](https://github.com/NEETROF/cymbra/actions/runs/37805366125), `ubuntu-24.04`,
2026-10-08 — took 17 min 11 s, its loop 16 min 39 s (en-fr 3 min 39 s, es-fr 9 min 25 s, en-es 21 s,
es-en 3 min 13 s), each dump fetched and read once: the French one fetched in 29 s and its catalogue
derived in 57 s, the English one in 1 min 48 s (about 28 MB/s from kaikki, not the 4 MB/s the
design feared) and 2 min 55 s, the Spanish one in 6 s and 9 s. A dry run of es-fr alone — an
update's cost but its release and branch — run
[37805406472](https://github.com/NEETROF/cymbra/actions/runs/37805406472), took 16 min 57 s, its
reduction 16 min 14 s (the English dump fetched in 2 min 27 s and derived in 4 min 6 s, the French
in 35 s and 1 min 27 s, the Spanish in 7 s and 14 s): an update stays well within the 45 minutes, and
the prefilter needs no tightening. What a pair then spends is mostly compression: es-en's 3 min 13 s
were about 2 min 50 s of zstd at level 19 over the 929 MB Spanish section, which es-fr compresses
too; since this measurement it is compressed once per run (D8): es-en copies the compressed bytes
es-fr left in the asset cache under their sha256, about 3 minutes less for the monthly job.

The disk at the run's fullest — the end of the English pass, its 2.98 GB dump and 1.47 GB
catalogue beside the French catalogue and the pack builder — held 68.2 GB used, about 4 GB more than
during the French pass. Most of it is not the run's: about 63 GB is the runner's own image, used
before the job starts, so the run itself peaks about 5 GB above it; `df` showed 82 GiB or more free
after each pair — far more room than the 14 GB the design assumed.

## What a pack studies, whatever it glosses

Two packs of one studied language must analyse it alike whatever native language they are glossed
in (add-lingua-pack-lexical-layer). So **a studied language's tables are kept once**
(split-lingua-pack-tables-by-language, the language matrix programme's M24), in `tables/<studied>/`
— `tables/en/`, `tables/es/`, `tables/fr/` — and every pair studying that language is built from that folder
together with its own:

| Folder | Holds |
|---|---|
| `tables/<studied>/` | the studied tables `forms.tsv`, `freq.tsv`, `grammar.tsv` and `level.tsv`, the pinned tag pool `tags.tsv`, the dictionary words `lexical.tsv`, and `studied.json`, which names the language's reference pair — nothing else |
| `tables/<pair>/` | `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE`, `manifest.json`, `pin.json` and `README.md` — no table of its studied language |

A folder is a pair's when its name is `<studied>-<native>`; any other is a studied language's. The
builder reads both (`lingua-pack-build --studied tables/<studied> tables/<pair> <out>`; `build.sh`
passes them) and refuses, by path, a table on the wrong side, and a pair whose manifest studies
another language than its studied folder's. The `testdata/` fixtures stay one folder, passed twice.
The checks (`lingua_pack::tables`, `tests/committed_tables.rs`) refuse a folder that is neither, a
studied table in a pair's folder — naming the pair, the file and the reference pair — anything else
in a studied folder, and a pair whose studied language has no folder; they hold en-fr's and es-fr's
packs, built from the two folders, to the sha256 their pins record.

**Only the reference pair's reduction writes a studied language's tables** — en-fr for English,
es-fr for Spanish, fr-en for French: for a language studied later, the first pair reduced for it. A reducer still
writes every table into its work folder; `pack_sources.py split` then files them by side: the pair's
own into `tables/<pair>/`, and the studied tables into `tables/<studied>/` only when `studied.json`
names the pair. Any other pair reads the studied folder as committed and never writes it. The
reference's reduction reads its native side too — the French Wiktionary's form links give English
forms, French glosses decide which Spanish lemmas take a level, and its glossed lemmas are the
dictionary words — so its `pin.json` and its rules are the record of its studied language's tables:
one pin per pair, none in a studied folder. Hence:
- a change to the reference's rules fails `check-reducer` for every pair of its language, naming the
  reference's rule files, until the reference is reduced again;
- a run that reduces several pairs reduces each reference first (`pack_sources.py pairs`), and the
  `reduce` job names, for each file that moved, the pair whose reduction writes it
  (`pack_sources.py moved`: `en-fr: en/forms.tsv`);
- an update of a reference reduces every other pair of its language again, from its own pinned
  sources, on the same branch (`pairs --after`), and its report lists what the studied tables change
  — forms, ranks, levels, readings and dictionary words — and names every pair whose pack moves.

**A reader pair's pin records the studied tables it read** (add-lingua-pack-es-en D3). The pin of a
pair that is not its language's reference — es-en today — holds, beside its snapshot, pack, rules and
sources, `studied`: the reference pair and the sha256 of each of the six studied tables its build
read (`forms.tsv`, `freq.tsv`, `grammar.tsv`, `level.tsv`, `lexical.tsv`, `tags.tsv`;
`pack_sources.py RECORDED_STUDIED`), written by `record-build`. When the reference's reduction moves
one, the pair's pack no longer matches its pin, and `check-reducer` and `pack_report.py` name the
pair and the table (`es-en: es/level.tsv`) until it is reduced again. A rules-only change of the
reference that moves no studied table leaves the reader's pin and pack as they are. A reader's
`pack_version` names its own snapshot and rule digest and a digest of that record
(`2026.10.08+ee357fe.08034dc`, `pack_sources.py version`), in an update as after a re-reduction:
it moves when a studied table does.

Two inputs of a studied folder are **written by no reducer**, so no rule digest moves with them:
`split` never overwrites them, a dry run copies the studied folder into its scratch root
(`pack_sources.py keep`, `KEPT_INPUTS`), and `pack_sources.py record-build` refuses tables whose
studied folder lacks `tags.tsv`. The dictionary words are written by `split`, not by a reducer,
either.

| File (in `tables/<studied>/`) | What it holds | Who writes it |
|---|---|---|
| `tags.tsv` | The studied language's **pinned tag pool**: one canonical Universal Dependencies tag per line, each once. The builder lays the pack's pool out as the pin in its own order, then the readings' tags the pin lacks, then the tags only senses carry, each part sorted, so a form's readings index the same tags whatever parts of speech a native language's senses use (en-es's senses use `NUM`, en-fr's none). English's and Spanish's are the pools en-fr's and es-fr's packs already carried (27 and 106 tags), so both keep their bytes; every pair of the language reads the same file. A pack built without one (the `testdata/` fixtures, the tests' packs) keeps a single sorted pool. | A person. The checks fail when a studied language's folder lacks it (`lingua_pack::tables`). |
| `lexical.tsv` | The language's **dictionary words**, one lemma per line, byte-sorted: the lemmas its reference pair glosses. They are what the vocabulary estimate and a CEFR list's typical vocabularies count, and what the Spanish names rule keeps as words. The builder writes them as a `lexical` section, one bit per lemma id, only when they differ from the lemmas the pack glosses; it then refuses, by name, a dictionary word or a glossed lemma that is neither the lemma of a form nor a ranked lemma, so no native language's glosses add a lemma. A pack without the section — or read by a core that predates it — reads its glossed lemmas as its dictionary words. en-fr and es-fr, the references, gloss exactly these, so their packs carry none; a pair glossed in another native language carries the section. | `pack_sources.py split`, when the reference is reduced: its `gloss.tsv` lemmas, read as the builder reads them (the text before the first tab, trimmed, non-empty), byte-sorted, each once. The checks fail, naming the reference and a lemma, when it is not the reference's glossed lemmas. |
| `studied.json` | `{"reference": "<pair>"}`: the pair whose reduction writes the folder, and whose `pin.json` is its provenance. | A person, or `split` for a language's first pair. |

**A noun's gender comes from its readings.** The builder gives a noun's sense runs the gender its
dictionary form, read as itself in `grammar.tsv`, has when that is a single one, and refuses a run
naming the other. A noun read with both genders (`estudiante`) gets a bare run, whatever its sense
table says; a noun read with no gender keeps its runs as the sense table writes them. es-fr's 13,435
noun runs, which its reducer genders the same way, are reproduced exactly, so a pack glossed in
another language shows the same « nom féminin » with no gender in its own sense tables.

**Two pairs of one studied language cannot disagree on its studied tables**: there is one copy.
With the pinned pool and the dictionary words, `forms.tsv`, `freq.tsv`, `level.tsv` and
`grammar.tsv` decide the lemma ids and how a form's readings are stored, so a pair glossed in
another native language stores them byte for byte as its reference does.

## The Wiktionary editions' rules

A pack's glosses are in the reader's native language, **written by a person**: never in the studied
language, never in a third language, never machine-translated, and never pivoted through a third
language. A word a person wrote into a Wiktionary translation table qualifies when the table pairs
the studied and the native language directly — the native words a studied entry lists, or the
native entry whose table lists the studied word (`pack_sources.py DUMPS`: every translation file a
pair derives pairs its two languages). Where no such source glosses a lemma, it has no gloss.

The glosses come from the Wiktionary written in the native language, and each edition writes its
own notes around them. Its cleaning rules are an `Edition` (`reduce_common.py`), one module per
edition (generalise-lingua-gloss-reducer):

| Module | Edition | Read by | What it knows |
|---|---|---|---|
| `reduce_edition_fr.py` | French (frwiktionary) | en-fr, es-fr | Today's rules, unchanged: the form-of wordings (« Pluriel de », « Forme de », also read for en-fr's own forms), « Présent », « Graphie » for expressions, the pointers and placeholders (« → voir », « Définition manquante ou à compléter »), a coordinator left hanging (« ou », « et »), a letter's name; a gloss of translation-table words opens on a capital |
| `reduce_edition_en.py` | English (enwiktionary) | es-en | Senses tagged `form-of` or `alt-of`, naming their word in `form_of` or `alt_of`; untagged « plural of », « inflection of », « alternative form of », « synonym of », « only used in », « see »; no placeholder (an undefined sense has no gloss, tagged `no-gloss`, and is left out); a letter's name; glosses stay in lower case, as the edition writes a foreign word's senses. Its senses read as meanings before the shared rules read them (`read_as_meanings`, refine-lingua-es-en-glosses): a sense nested under a label or a pointer by its own gloss; a shortened or respelled form (« apocopic form of », « pronunciation spelling of »…, an untagged one a pointer too) by the meaning it carries or its target's senses, a pronoun's case form by its meaning; a place's name after a function word spelled like it; the edition's descriptions in lower case, one ellipsis, curly double quotes, no numbered sense, nothing after a line break. Two settings, es-en's alone and off until the owner picks them: long parentheses (M20, `LONG_PARENTHESIS` as `EN.long_parenthesis`, 0 keeps them) and the merging of a word's same-part-of-speech etymologies before the round-robin (`MERGE_SAME_POS_ETYMOLOGIES`, a pre-pass `reduce-es-en.py` runs) |
| `reduce_edition_es.py` | Spanish (eswiktionary) | en-es | Untagged « Forma del plural de », « Grafía obsoleta de », « Participio pasado del verbo (to) read », a tense or a person followed by « de » or « del » — the « de » is required, so « Femenino. » stays a meaning; sense-link subscripts taken out whole — one, a range or two (« dejar₉ », « Madrid₁₋₂ », « bottom₉ o ₁₀ »), after a lower-case letter, the word's period or a stray space, never after a capital (« C₄H₁₀ » keeps its digits) nor a preposition (« similar a ₁ » names one of the entry's senses) — and « Véase también »; a letter's name. Its notes to its readers (refine-lingua-en-es-glosses): a maintenance template (« ^([cita requerida]) »), a disambiguation note (« [sentido del sustantivo] »), a reference to numbered senses (« (definiciones [1,2]) »), the expansion notice, and a usage note after the meaning, a sentence opening on a closed list (« A veces », « Usado », « Se dice »…), a second sentence that carries the meaning kept. Its senses read in their order before the shared rules read them (`read_as_meanings`, a pre-pass en-es runs): the senses it marks obsolete or outdated after the others, nothing left out; one ellipsis « … », straight double quotes paired « » (`typography`, which en-es also applies to the translation tables' words) |

Each module's docstring holds the census its rules come from, measured on the data es-fr pins. The
French-native pairs' reducers bind the French edition; `reduce_common.py` imports no edition, so a
shared function is always told which edition it cleans.

**The digest covers what a reducer loads.** A pair's rules (`pin.json` `reducer.files`) are its
reducer and every `reduce_*.py` module importing it loads, read from `sys.modules`
(`pack_sources.py rule_files`): en-fr's and es-fr's are `reduce-<pair>.py`, `reduce_common.py` and
`reduce_edition_fr.py`; es-en's are `reduce-es-en.py`, `reduce_common.py` and
`reduce_edition_en.py`; en-es's are `reduce-en-es.py`, `reduce_common.py` and
`reduce_edition_es.py`. Editing the English edition re-pins es-en alone, the Spanish edition en-es
alone; editing `reduce_common.py` re-pins every pair. `check-reducer` fails, naming the module, when a
reducer loads a rule module its record does not name, and tests refuse a `reduce_*` import a
reducer would make later than at import time, and any module loaded other than by an import
statement (`importlib`, `__import__`, `exec`) in a reducer or a rule module.

**Translation tables wherever the edition writes them.** `derive` reads an entry's table for the
whole entry and each of its senses' (`translations_of`), with the sense a table names. The editions'
dumps write one table per entry: the French and Spanish ones, so es-fr's derived files are
unchanged, and the English one too — 68,582 of its English entries list Spanish translations, every
table on the entry in the page's order, none under a sense, a translation the page lists twice kept
twice. It is kaikki's per-language extract of the English Wiktionary's English section, which en-es's
pin was derived from, that moves each table under the sense it translates (about 65,752 entries with
a table under a sense, 5,074 for the whole entry). `derive` reads a dump as served, gzipped or plain, told apart by the
gzip magic and not by the address: kaikki serves an edition's dump gzipped and its per-language
extracts uncompressed — the English Wiktionary's English one, which en-es's first update read for
those tables (add-lingua-pack-en-es D2).

**The committed tables are what the rules make of the pinned sources.** The `reduce` job of
`lingua-extension-check` reduces every pair again from its pinned sources, each reference first,
with the pinned interpreter and dependencies, whenever `scripts/lingua-data` or the job changes, and
fails on any byte of the tables — both kinds of folder — `manifest.json` or `pin.json` that differs,
naming the pair whose reduction writes the file: a table or a digest written by hand cannot pass.
To change the rules without moving a table, dispatch `lingua-pack-update` with `mode=reduce`,
`pair=all` and `expect=identical`: one branch with every pair's tables, the baselines re-blessed
once, and a failure naming the pair and the file (a studied language's file with its reference:
`es-fr: es/level.tsv`) when a table, `tags.tsv`, `studied.json`, `lexical.tsv` or NOTICE moves, when `manifest.json` moves beyond `pack_version` or `pin.json` beyond the pack's sha256 and
the rules' record (`pack_report.py --identical`), when the site's coverage figures move, or when a
baseline moves beyond the lines naming the packs.

Measured when the editions came in: en-fr and es-fr reduced again from their pinned sources gave
their seven tables, `tags.tsv` and NOTICE byte for byte; only `pack_version` (`2026.09.26+55f480b`,
`2026.10.03+0d876dc`) and the pins moved.

## Allowed vs denied licences

- **Allowed** (commercial use OK): permissive (ESDB/SCOWL and WordNet), CC BY, CC BY-SA (the derived tables are published, satisfying share-alike).
- **Denied** (never enter a pack): **GPL / AGPL** (viral — e.g. Apertium, FreeLing dictionaries), **any non-commercial** (CC BY-NC-*, e.g. Lemlat, LatMor, SUBTLEX-ESP as distributed, UD Italian-ISDT).

The denylist is code, not just prose: `lingua_pack::licence::is_denied` fails the build on `Gpl`/`Agpl`/`NonCommercial`, and the build also fails if a source is missing from the `NOTICE`.

## Rules

- **Raw sources are never committed.** They download into `scripts/lingua-data/work/` (git-ignored), dated.
- **The pack is never committed.** CI rebuilds it and caches it; the reproducibility test (`crates/lingua-pack/tests/pipeline_testdata.rs`) proves two builds over the same tables are byte-identical.
- **Adding a pair is a reducer, not a format change**: the container and the builder are pair-keyed, and what a pair does with a Wiktionary's glosses and with its forms is shared (`reduce_common.py`, driven by a `Studied` — the language's word pattern, form-of target wording, coordinators and wordfreq code — and by the `Edition` of the Wiktionary its glosses come from, `reduce_edition_<native>.py`). A new pair writes `reduce-<pair>.py` for its own inflection and level sources and registers its sources in `pack_sources.py`; the analyser must also know the language (lingua-core). Candidate sources for the Romance pairs: Morphalou (fr, LGPL-LR), morph-it! (it, CC BY-SA 2.0/LGPL), MorphoBr (pt, Apache-2.0), kaikki/frwiktionary glosses.

`testdata/en-fr/` holds a tiny hand-made fixture (a few lines, not real source data) so the pipeline and its reproducibility test run in CI without any download.
