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
| `gloss.tsv` (`lemma → gloss`) | **kaikki.org** extract of the French Wiktionary (`frwiktionary`) | CC BY-SA 4.0 + GFDL | one short French gloss per lemma, top ~20–30k lemmas (arbitrated by the 5 MB budget) |
| `level.tsv` (`lemma → CEFR`) *(optional)* | **CEFR-J Wordlist v1.5** (A1–B2, Tono Lab / TUFS) + **Octanove Vocabulary Profile C1/C2 v1.0** (C1–C2, Octanove Labs), both from the Open Language Profiles repo | CEFR-J: commercial use allowed with acknowledgement; Octanove: CC BY-SA 4.0 | lowest CEFR level per kept lemma across POS rows; the only pairing that covers A1→C2 with commercial-redistribution rights (Octanove was built to extend CEFR-J past B2) |
| `mwe.tsv` (`expression → gloss`) *(optional)* | **kaikki.org** extract of the French Wiktionary (`frwiktionary`), its multi-word entries | CC BY-SA 4.0 + GFDL | the same sense picker and cuts as `gloss.tsv`, over the entries whose headword holds a space; proper-noun-only entries and form-of senses dropped. The **keys are computed by the builder**, not here: each word goes through lingua-core's own lemmatiser against the lexicon that build assembled (`starting point` → `start point`), so a key is what the reader's cascade produces. Python cannot do it — it mirrors the analyser's irregulars but not its morphy rules or its out-of-lexicon plural. |
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
| `forms.tsv` (`form → lemma`) | **kaikki.org** extract of the English Wiktionary (`enwiktionary`), Spanish section — our snapshot of its 2026-09-28 dump; **UD Spanish-GSD** (train and dev, at a commit) for homographs | kaikki: CC BY-SA 4.0 + GFDL; GSD: CC BY-SA 4.0, read for counts only | the inflections a lemma's entry lists and the form-of links of a form's own entry: lowercased, NFC, single Spanish words. Never a verb with its clitics — a `combined-form`, or a sense naming the pronoun (`object-…` tags); the analyser's enclitic rule reads those — but a string that is also a plain form keeps it (`principales` → *principal*). **One lemma per form**: a reviewed override (`OVERRIDES` in the reducer, each with its reason; none yet, since an override takes the other lemma out of the pack), then GSD's counts of the form under each lemma, then the form's own entry, then the lemma's frequency, then the alphabet. Only the forms of kept lemmas that wordfreq attests, and every lemma's own form |
| `freq.tsv` (`lemma → rank`) | **wordfreq** Spanish list | CC BY-SA 4.0 | the top 60k canonical lemmas, dense rank; inflected forms, and combined forms that are no word of their own, skipped |
| `grammar.tsv` (`form → readings`) | **kaikki.org**, the same extract: the tags of each form | CC BY-SA 4.0 + GFDL | the readings of the forms `forms.tsv` holds, under kept lemmas, as Universal Dependencies tags (add-lingua-spanish-grammar-tables). A verb form takes its mood, tense, person and number — the conditional as a mood, the *usted* imperative as a third person, the negative imperative left to the subjunctive it repeats — or its form (`Inf`, `Ger`, an agreed `Part`). A noun takes its gender (`es-noun`'s argument, else its senses' tags) on its own form and its plural; an adjective, determiner or pronoun its agreement. A pronominal form (`azotarse`) reads from its own entry. Each reading of another lemma than the form's is marked `other`: kaikki's tables are structured, so the card may name it |
| `gloss.tsv` (`lemma → gloss`) | **kaikki.org**: the French Wiktionary's Spanish entries; else the French translations the Spanish Wiktionary's Spanish entries list; else the French Wiktionary's French entries whose translation tables list the word. The last two are derived from kaikki's dumps of the whole editions (`pack_sources.py DUMPS`) | CC BY-SA 4.0 + GFDL | the shared rules on the Spanish entries: up to eight senses grouped by part of speech (add-lingua-spanish-gloss-tables). A fallback gloss is up to three French words per part of speech, the commonest first from a table read backwards. A proper noun's translation glosses nothing; a gloss is never English and never a machine translation. Coverage below |
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

## What a pack studies, whatever it glosses

Two packs of one studied language must analyse it alike whatever native language they are glossed
in (add-lingua-pack-lexical-layer). So **a studied language's tables are kept once**
(split-lingua-pack-tables-by-language, the language matrix programme's M24), in `tables/<studied>/`
— `tables/en/`, `tables/es/` — and every pair studying that language is built from that folder
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
es-fr for Spanish, and for a language studied later the first pair reduced for it. A reducer still
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
| `reduce_edition_en.py` | English (enwiktionary) | es-en (change 21) | Senses tagged `form-of` or `alt-of`, naming their word in `form_of` or `alt_of`; untagged « plural of », « inflection of », « alternative form of », « synonym of », « only used in », « see »; no placeholder (an undefined sense has no gloss, tagged `no-gloss`, and is left out); a letter's name; glosses stay in lower case, as the edition writes a foreign word's senses. Long parentheses (M20) are kept until es-en's review settles them (`long_parenthesis`, 0) |
| `reduce_edition_es.py` | Spanish (eswiktionary) | en-es (change 22) | Untagged « Forma del plural de », « Grafía obsoleta de », « Participio pasado del verbo (to) read », a tense or a person followed by « de » or « del » — the « de » is required, so « Femenino. » stays a meaning; sense-link subscripts taken out whole — one, a range or two (« dejar₉ », « Madrid₁₋₂ », « bottom₉ o ₁₀ »), after a lower-case letter, the word's period or a stray space, never after a capital (« C₄H₁₀ » keeps its digits) nor a preposition (« similar a ₁ » names one of the entry's senses) — and « Véase también »; a letter's name |

Each module's docstring holds the census its rules come from, measured on the data es-fr pins. The
French-native pairs' reducers bind the French edition; `reduce_common.py` imports no edition, so a
shared function is always told which edition it cleans.

**The digest covers what a reducer loads.** A pair's rules (`pin.json` `reducer.files`) are its
reducer and every `reduce_*.py` module importing it loads, read from `sys.modules`
(`pack_sources.py rule_files`): en-fr's and es-fr's are `reduce-<pair>.py`, `reduce_common.py` and
`reduce_edition_fr.py`. Editing the English or Spanish edition re-pins no pair glossed in French;
editing `reduce_common.py` re-pins every pair. `check-reducer` fails, naming the module, when a
reducer loads a rule module its record does not name, and tests refuse a `reduce_*` import a
reducer would make later than at import time, and any module loaded other than by an import
statement (`importlib`, `__import__`, `exec`) in a reducer or a rule module.

**Translation tables wherever the edition writes them.** `derive` reads an entry's table for the
whole entry and each of its senses' (`translations_of`), with the sense a table names. The French and
Spanish Wiktionaries write one table per entry, so es-fr's derived files are unchanged; the English
one writes them under its senses (68,579 English entries list Spanish translations under a sense,
5,080 for the whole entry).

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
