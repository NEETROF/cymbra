# Cymbra Lingua data-pack sources

The offline pipeline (`build.sh` → `lingua-pack-build`) turns three upstream
datasets into one versioned `pack.lingua` per language pair. Only sources whose
licence permits commercial use are allowed; the builder additionally enforces
the denylist and refuses to build on a denied source.

## EN → FR (the MVP pair)

| Table | Upstream source | Licence | Reduction |
|---|---|---|---|
| `forms.tsv` (`form → lemma`) | **AGID** (Automatically Generated Inflection Database, SCOWL/aspell family) | Permissive — "use, copy, modify, distribute and sell", notices retained (WordNet, 2of12id, ENABLE, … upstream) | invert `infl.txt` to form→lemma pairs, **one lemma per form**: a kept lemma maps to itself, any other form to the base Wiktionary names, else one with a gloss, else the most frequent (see *Words AGID gets wrong*); a listed hyphenated compound also gets its inflections (`t-shirts`, `mothers-in-law`) |
| `freq.tsv` (`lemma → rank`) | **wordfreq** English large list | CC BY-SA 4.0 (incl. SUBTLEX with Brysbaert's permission) | top 40k canonical lemmas, dense rank; CEFR words the pack would otherwise lack are added too — a hyphenated compound at its rarest part's rank, any other word with the lemmas of its frequency (after the list when rarer than all of them) |
| `gloss.tsv` (`lemma → gloss`) | **kaikki.org** extract of the French Wiktionary (`frwiktionary`) | CC BY-SA 4.0 + GFDL | one short French gloss per lemma, top ~20–30k lemmas (arbitrated by the 5 MB budget) |
| `level.tsv` (`lemma → CEFR`) *(optional)* | **CEFR-J Wordlist v1.5** (A1–B2, Tono Lab / TUFS) + **Octanove Vocabulary Profile C1/C2 v1.0** (C1–C2, Octanove Labs), both from the Open Language Profiles repo | CEFR-J: commercial use allowed with acknowledgement; Octanove: CC BY-SA 4.0 | lowest CEFR level per kept lemma across POS rows; the only pairing that covers A1→C2 with commercial-redistribution rights (Octanove was built to extend CEFR-J past B2) |
| `mwe.tsv` (`expression → gloss`) *(optional)* | **kaikki.org** extract of the French Wiktionary (`frwiktionary`), its multi-word entries | CC BY-SA 4.0 + GFDL | the same sense picker and cuts as `gloss.tsv`, over the entries whose headword holds a space; proper-noun-only entries and form-of senses dropped. The **keys are computed by the builder**, not here: each word goes through lingua-core's own lemmatiser against the lexicon that build assembled (`starting point` → `start point`), so a key is what the reader's cascade produces. Python cannot do it — it mirrors the analyser's irregulars but not its morphy rules or its out-of-lexicon plural. |
| `NOTICE` | all of the above | — | the full attribution stack, embedded in the pack and shown on the extension's Attributions page |

### Words AGID gets wrong

AGID generates inflections mechanically, so some "inflections" are words of their own: it
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

The CEFR level table is **optional and additive**: a pair without licence-clean CEFR data ships no `level.tsv`, the builder emits no `levels` section, and the reader falls back to frequency bands. Adding the section does **not** bump `analyzer_version` — it is behaviour-preserving (level-based presumed-known only activates once the user declares a level), so old cores load a level-bearing pack and simply ignore the section.

## Allowed vs denied licences

- **Allowed** (commercial use OK): permissive (AGID/WordNet family), CC BY, CC BY-SA (the derived tables are published, satisfying share-alike).
- **Denied** (never enter a pack): **GPL / AGPL** (viral — e.g. Apertium, FreeLing dictionaries), **any non-commercial** (CC BY-NC-*, e.g. Lemlat, LatMor, SUBTLEX-ESP as distributed, UD Italian-ISDT).

The denylist is code, not just prose: `lingua_pack::licence::is_denied` fails the build on `Gpl`/`Agpl`/`NonCommercial`, and the build also fails if a source is missing from the `NOTICE`.

## Rules

- **Raw sources are never committed.** They download into `scripts/lingua-data/work/` (git-ignored), dated.
- **The pack is never committed.** CI rebuilds it and caches it; the reproducibility test (`crates/lingua-pack/tests/pipeline_testdata.rs`) proves two builds over the same tables are byte-identical.
- **Adding a pair is data, not code**: drop a new pair's tables + manifest and build; the format and reader are already pair-keyed. Candidate sources for the Romance pairs: Morphalou (fr, LGPL-LR), morph-it! (it, CC BY-SA 2.0/LGPL), MorphoBr (pt, Apache-2.0), kaikki/frwiktionary glosses.

`testdata/en-fr/` holds a tiny hand-made fixture (a few lines, not real source data) so the pipeline and its reproducibility test run in CI without any download.
