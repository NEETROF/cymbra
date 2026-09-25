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

## 3. Merges and the reader's statuses

- [ ] 3.1 The reducer writes `merges.tsv` from the previous and the new tables; no one- or two-letter target
- [ ] 3.2 `lingua-pack-build` writes an optional merges section; the pack format stays readable by the current core (absent section = no merge); `cargo test -p lingua-pack`
- [ ] 3.3 `lingua-core` applies merges once per `pack_version`: explicit status only, onto a lemma without one, as a status change at that time; the applied version recorded in the state; tests per scenario of `lingua-knowledge-model`
- [ ] 3.4 The extension and the Safari app carry the change through their existing engine; sync tests: a carried-over status reaches another device like any other

## 4. Notices and data

- [ ] 4.1 ESDB's notice in the reducer's NOTICE and the pack; the attributions page shows it; `SOURCES.md` updated (ESDB, kaikki `form_of`, AGID retired)
- [ ] 4.2 Re-reduce with ESDB + kaikki; review the `forms.tsv`, `freq.tsv` and `merges.tsv` diff; merge it through the update flow of `pin-lingua-pack-sources`

## 5. Gates

- [ ] 5.1 `openspec validate switch-lingua-inflections-to-esdb --strict`
- [ ] 5.2 Reducer tests, `cargo test -p lingua-pack -p lingua-core`, extension check lane green; the measurement re-run on the committed tables matches the design's figures
