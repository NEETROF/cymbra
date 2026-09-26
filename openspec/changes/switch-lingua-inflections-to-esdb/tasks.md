## 1. Before any code

- [ ] 1.1 Commit the measurement harness (reducer imported unchanged, one parser swapped, `lingua-pack-build`, the extension's engine under Node) as a script, and its corpus manifest (article titles, fetch date)
- [ ] 1.2 Add a non-Wikipedia sample (tech and news prose) to the measurement; record glosses / resolved tokens / lemma changes per sample in design.md
- [ ] 1.3 Land after `pin-lingua-pack-sources` (committed tables, pinned sources, re-reduce and update modes)

## 2. ESDB as the inflection source

- [ ] 2.1 Pin `en-wl/wordlist` at `rel-2026.02.25` and the sha256 of its exported `scowl.txt` in `pin.json`; build it from that commit in re-reduce and update modes (D1)
- [ ] 2.2 `parse_esdb_relations`: the structure `parse_agid_relations` returns; POS and size filters, possessives out, primary and equal spellings only (D2); unit tests on real lines (`run`, `go`, `be`, `bear`, `lie`, `saw`, `focus`, `learn`, `datum`, `leaf`)
- [ ] 2.3 The rule that drops lesser variants made explicit and tested for AGID too (D2)
- [ ] 2.4 kaikki `form_of` relations, regular inflections only (D3); tests including *occupied*, *stocks*, *coats*, *born*
- [ ] 2.5 The regressions of D4 fixed by general rules, each with a test; the update report shows no other change the reviewer has not accepted

## 3. Notices and data

- [ ] 3.1 ESDB's notice in the reducer's NOTICE and the pack; the attributions page shows it; `SOURCES.md` updated (ESDB, kaikki `form_of`, AGID retired)
- [ ] 3.2 Re-reduce with ESDB + kaikki; review the `forms.tsv` and `freq.tsv` diff; merge it through the update flow of `pin-lingua-pack-sources`

## 4. Gates

- [ ] 4.1 `openspec validate switch-lingua-inflections-to-esdb --strict`
- [ ] 4.2 Reducer tests, `cargo test -p lingua-pack`, extension check lane green; the measurement re-run on the committed tables matches the design's figures
