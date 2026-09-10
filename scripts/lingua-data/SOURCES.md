# Cymbra Lingua data-pack sources

The offline pipeline (`build.sh` → `lingua-pack-build`) turns three upstream
datasets into one versioned `pack.lingua` per language pair. Only sources whose
licence permits commercial use are allowed; the builder additionally enforces
the denylist and refuses to build on a denied source.

## EN → FR (the MVP pair)

| Table | Upstream source | Licence | Reduction |
|---|---|---|---|
| `forms.tsv` (`form → lemma`) | **AGID** (Automatically Generated Inflection Database, SCOWL/aspell family) | Permissive — "use, copy, modify, distribute and sell", notices retained (WordNet, 2of12id, ENABLE, … upstream) | invert `infl.txt` to form→lemma pairs |
| `freq.tsv` (`lemma → rank`) | **wordfreq** English large list | CC BY-SA 4.0 (incl. SUBTLEX with Brysbaert's permission) | top ~50k lemmas, dense rank |
| `gloss.tsv` (`lemma → gloss`) | **kaikki.org** extract of the French Wiktionary (`frwiktionary`) | CC BY-SA 4.0 + GFDL | one short French gloss per lemma, top ~20–30k lemmas (arbitrated by the 5 MB budget) |
| `NOTICE` | all of the above | — | the full attribution stack, embedded in the pack and shown on the extension's Attributions page |

## Allowed vs denied licences

- **Allowed** (commercial use OK): permissive (AGID/WordNet family), CC BY, CC BY-SA (the derived tables are published, satisfying share-alike).
- **Denied** (never enter a pack): **GPL / AGPL** (viral — e.g. Apertium, FreeLing dictionaries), **any non-commercial** (CC BY-NC-*, e.g. Lemlat, LatMor, SUBTLEX-ESP as distributed, UD Italian-ISDT).

The denylist is code, not just prose: `lingua_pack::licence::is_denied` fails the build on `Gpl`/`Agpl`/`NonCommercial`, and the build also fails if a source is missing from the `NOTICE`.

## Rules

- **Raw sources are never committed.** They download into `scripts/lingua-data/work/` (git-ignored), dated.
- **The pack is never committed.** CI rebuilds it and caches it; the reproducibility test (`crates/lingua-pack/tests/pipeline_testdata.rs`) proves two builds over the same tables are byte-identical.
- **Adding a pair is data, not code**: drop a new pair's tables + manifest and build; the format and reader are already pair-keyed. Candidate sources for the Romance pairs: Morphalou (fr, LGPL-LR), morph-it! (it, CC BY-SA 2.0/LGPL), MorphoBr (pt, Apache-2.0), kaikki/frwiktionary glosses.

`testdata/en-fr/` holds a tiny hand-made fixture (a few lines, not real source data) so the pipeline and its reproducibility test run in CI without any download.
