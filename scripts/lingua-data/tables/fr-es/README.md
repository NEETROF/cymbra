# The fr→es dictionary tables

The reduced tables Cymbra Lingua's French→Spanish data pack is built from: French glossed in
Spanish, for Spanish speakers studying French (add-lingua-pack-fr-es, change 49 of
`docs/lingua/language-matrix-programme.md`). This folder holds the native side alone; French's own
tables are in `../fr/`, written by fr-en's reduction.

No tables yet: fr-es's first `lingua-pack-update` dispatch, on the change's branch, publishes the
files it derives from the Spanish and French Wiktionaries' dumps under
`lingua-pack-sources-fr-es-<snapshot>`. Its coverage is the committed measurement, held to the floor
fixed before it (`gloss_coverage.py` `FLOORS["fr-es"]`: 81.4 / 68.8 / 54.5 % of the 5,000 / 10,000 /
20,000 commonest lemmas): at or above it, the pinned reduction that follows commits the tables here
with their pin, and this file then describes them; under it, nothing is committed and no package
lists fr-es.
