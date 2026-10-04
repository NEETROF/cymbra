# Cymbra Lingua data-pack sources

The pipeline turns upstream datasets into one versioned `pack.lingua` per language pair, in two
stages (pin-lingua-pack-sources):

1. **Reduce** (`reduce-<pair>.py`, with the rules every `<studied>->FR` pair shares in
   `reduce_common.py`): the raw sources become the tables of `tables/<pair>/`, which
   are **committed** with `pin.json` — the record of each raw source (at a commit and by sha256,
   or as our own snapshot for kaikki, which upstream regenerates daily) and of the pack they
   build. Only `build.sh --update` (today's sources), `--reduce` (the pinned sources, after a
   change to the rules) and `--dry` (the monthly check) read raw sources, and only the
   `lingua-pack-update` workflow or a person runs them.
2. **Build** (`lingua-pack-build`): the committed tables become the pack, offline, checked against
   `pin.json`. That is all a release or a pull request runs.

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
proper nouns are left out. The 2026-10-03 tables pass the programme's gates:
- 99.38 % of 19,276 words resolve in the lexicon (gate 98.5 %);
- 95.92 % of 9,439 content words take PUD's lemma (gate 93.5 %);
- 97.95 % of 634 auxiliaries take PUD's lemma (gate 97 %).

**French glosses**, the share of the commonest lemmas glossed on the 2026-10-03 tables:

| Lemmas | French Wiktionary | with the translations |
|---|---|---|
| top 5,000 | 82.9 % | 87.7 % |
| top 10,000 | 69.6 % | 77.3 % |
| top 20,000 | 54.4 % | 63.8 % |
| all 60,000 | 29.0 % | 38.0 % |

22,826 lemmas are glossed, 17,420 from the French Wiktionary. There are 15,133 expressions: 2,952
from the French Wiktionary's Spanish entries and 12,181 from the translations.

**Estimated levels**, the rule measured on English's 8,302 CEFR lemmas, ranked the same way: 39.8 %
take their list's level, and 82.6 % are within one level of it. The scale is monotone (the mean true
level rises from 1.67 at A1 to 5.03 at C2), so the three-band fallback of the programme's decision
D1 is not needed. The pack is 2,188,994 B, with the grammar, the glosses and the levels.

## Allowed vs denied licences

- **Allowed** (commercial use OK): permissive (ESDB/SCOWL and WordNet), CC BY, CC BY-SA (the derived tables are published, satisfying share-alike).
- **Denied** (never enter a pack): **GPL / AGPL** (viral — e.g. Apertium, FreeLing dictionaries), **any non-commercial** (CC BY-NC-*, e.g. Lemlat, LatMor, SUBTLEX-ESP as distributed, UD Italian-ISDT).

The denylist is code, not just prose: `lingua_pack::licence::is_denied` fails the build on `Gpl`/`Agpl`/`NonCommercial`, and the build also fails if a source is missing from the `NOTICE`.

## Rules

- **Raw sources are never committed.** They download into `scripts/lingua-data/work/` (git-ignored), dated.
- **The pack is never committed.** CI rebuilds it and caches it; the reproducibility test (`crates/lingua-pack/tests/pipeline_testdata.rs`) proves two builds over the same tables are byte-identical.
- **Adding a pair is a reducer, not a format change**: the container and the builder are pair-keyed, and what a `<studied>->FR` pair does with the French Wiktionary and with its forms is shared (`reduce_common.py`, driven by a `Studied` — the language's word pattern, form-of target wording, coordinators and wordfreq code). A new pair writes `reduce-<pair>.py` for its own inflection and level sources and registers its sources in `pack_sources.py`; the analyser must also know the language (lingua-core). Candidate sources for the Romance pairs: Morphalou (fr, LGPL-LR), morph-it! (it, CC BY-SA 2.0/LGPL), MorphoBr (pt, Apache-2.0), kaikki/frwiktionary glosses.

`testdata/en-fr/` holds a tiny hand-made fixture (a few lines, not real source data) so the pipeline and its reproducibility test run in CI without any download.
