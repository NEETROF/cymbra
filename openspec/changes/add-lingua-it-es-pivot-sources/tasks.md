# Tasks

## 1. The pipeline (scripts/lingua-data)

- [ ] 1.1 `pack_sources.py` `derive`: the kind `("pivot", lang, "it", "es")` — for each entry of the edition's language, its Italian and Spanish translations grouped by the table's `sense`, else `sense_index`, else the sense it sits under, else the whole entry; one record per group with the entry's word, part of speech, `sense`, `sense_index`, whether the entry has several senses, the Italian words and the Spanish words (D2); `test_pack_sources.py`: a homograph's two senses kept apart, a sense with one language alone giving nothing, a table numbering its senses only, a table naming none, a table nested under a sense, an entry of another language.
- [ ] 1.2 `EDITIONS`: `["es"]` gains `kaikki-es-Italiano.jsonl` and `kaikki-es-traductions-it.jsonl`, `["it"]` `kaikki-it-traductions-es.jsonl` and `kaikki-it-Spagnolo.jsonl`, `["en"]` and `["fr"]` their pivot files; `"de"`, `"pt"`, `"ru"`, `"pl"`, `"nl"` with their dumps' URLs and one pivot file each (D1–D3); the tests that pin the editions' addresses and files (`test_pack_sources.py`'s edition list and the English and French catalogues) follow.
- [ ] 1.3 The `"wikidata"` edition (D3): the latest dated Lexemes dump, its identity recorded, `wikidata-lexemes-it-es.jsonl` derived (P5137 pairs, P5972 links, Spanish glosses of Italian senses) and published in it-es's release; a test on a fixture of three lexemes.
- [ ] 1.4 `DUMPS["it-es"]` (D4); `SOURCES.md`: the sources, their licences, CC0 among the licences allowed, the pivot's exception under I11, the cost of D5; `python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"`.

## 2. Gates

- [ ] 2.1 Nothing else moves: `git diff --stat origin/main -- scripts/lingua-data/tables` empty; every other pair's `DUMPS` and pin unchanged.
- [ ] 2.2 `openspec validate add-lingua-it-es-pivot-sources --strict`; the archive order; `docs/lingua/italian-programme.md` row 27a's status.
