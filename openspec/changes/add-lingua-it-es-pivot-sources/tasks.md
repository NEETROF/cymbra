# Tasks

## 1. The pipeline (scripts/lingua-data)

- [ ] 1.1 `pack_sources.py` `derive`: the kind `("pivot", lang, "it", "es")` — for each entry of the edition's language, each translation table's sense listing both an Italian and a Spanish word, one record with the entry's word, part of speech, sense, Italian words and Spanish words (D1); `test_pack_sources.py`: a homograph's two senses kept apart, a sense with one language alone giving nothing.
- [ ] 1.2 `EDITIONS`: `kaikki-en-pivot-it-es.jsonl`, `kaikki-fr-pivot-it-es.jsonl`; `"de"`, `"pt"`, `"ru"`, `"pl"`, `"nl"` with their dumps' URLs and one pivot file each; `EDITIONS["it"]` gains `kaikki-it-Spagnolo.jsonl` (D1, D2).
- [ ] 1.3 The Wikidata Lexemes source (D3): the derivation of `wikidata-lexemes-it-es.jsonl` (Italian and Spanish senses linked to one item), `PINNED["it-es"]` naming the mirrored asset; a test on a fixture of three lexemes.
- [ ] 1.4 `DUMPS["it-es"]`; `SOURCES.md` (the sources, their licences, the pivot's exception under I11, the cost of D5); `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`.

## 2. Gates

- [ ] 2.1 Nothing else moves: `git diff --stat origin/main -- scripts/lingua-data/tables` empty; every other pair's `DUMPS` and pin unchanged.
- [ ] 2.2 `openspec validate add-lingua-it-es-pivot-sources --strict`; the archive order; `docs/lingua/italian-programme.md` row 27a's status.

## 3. Owner

- [ ] 3.1 [manual] The owner publishes the dated Wikidata Lexemes dump as the release asset `lingua-pack-source-wikidata-lexemes-<date>` (D3).
