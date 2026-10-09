# Tasks

## 1. The catalogue (apps/lingua-extension)

- [ ] 1.1 Download fr-en's three files once from Mozilla's registry (run `retrain_hr_EFgIftH_RrCyzl5gjemVNg`); compute each gzip's sha256 and length and each decompressed file's sha256 and length; check the model against the registry's `uncompressedHash` and the three against Firefox's Remote Settings for fr→en 2.0; stop if any differs from the design's D1 table (D1).
- [ ] 1.2 `model-manifest.json`: `fr-en/base-memory/2.0` after `en-es/base-memory/2.1` (paths, sizes, digests, source, mirror `lingua-model-fr-en-base-memory-2.0`, MPL-2.0), and the routes `fr-en` (the fr-en model) and `fr-es` (fr-en then en-es) after `en-es` (D1, D2). `node tool/assemble_model_site.mjs <dir>` run locally keeps every file of the four models.
- [ ] 1.3 `test/model-manifest.spec.ts`: four models in pin order and six routes, fr-en's digests, sizes (26,234,715 B to download, 37,200,311 B unpacked), paths, sources and mirror, fr-es's 51,608,069 B, fr-en's vocabulary equal to en-fr's once decompressed and its gzip file not, every native language's routes needing two models together, the no-route test on `de-fr` and `fr` alone, *A route of a pair not shipped* over fr-en and fr-es (D2, D3, D8; *Every reader today*, *Pinned against Mozilla's publications*, *Every native language's pairs*, *A vocabulary fr-en shares with en-fr*, *A route of a pair studying French*).
- [ ] 1.4 `test/model-residency.spec.ts`: D3's sequences through the committed catalogue's routes — a Spanish-native reader alternating en-es and fr-es, an English-native reader alternating es-en and fr-en, nothing deleted; a native language changed from French while the worker lives, two models held after every load (*A Spanish-native reader of English and French*).

## 2. The corpus and the soak

- [ ] 2.1 `tool/marks/pud.mjs`: UD French-PUD at `db260db10fe728853c549760801229ef4e7b16e1`, `fr_pud-ud-test.conllu`, sha256 `4dfed37b83d76e77fd2e9963d0be00d723e9a010e7e2a746f8b7640c48063c10`, as change 43 pins it. `select_corpus.mjs` over en, es and fr: the next sentence for every language, the rule's text saying so; `corpus.json` regenerated — its diff adds the 100 French items and changes the rule's text, nothing else (checked in the pull request: the English and Spanish items equal the committed ones) (D4).
- [ ] 2.2 `test/translate-marks.spec.ts`: the corpus's languages are en, es and fr, 100 items each, and each step's three items share `k` and `id` (*A studied language added to the corpus*).
- [ ] 2.3 `tool/soak_engine.mjs --pair fr-en` and `--pair fr-es`, `--isolate` and in one instance, over the French selections with the models assembled locally; the runs recorded in `TRANSLATION.md` beside en-es's (D5; change 9's *A route can be soaked by hand*).

## 3. The marks

- [ ] 3.1 `tool/measure_marks.mjs`: the gloss experiment reads `tables/<pair>/gloss.tsv` only when it holds a gloss, its columns left empty otherwise (`existsSync` kept); a test with an empty table (D6).
- [ ] 3.2 `tool/marks/README.md`'s criteria gain a French example per native language and say that fr-es crosses two alignments through English, before any run (D6).
- [ ] 3.3 `measure_marks.mjs --pair fr-en` and `--pair fr-es` with the models assembled locally; `results-fr-en.jsonl` and `results-fr-es.jsonl` committed. The results of en-fr, es-fr, es-en and en-es are not run again nor rewritten (D6).
- [ ] 3.4 `judged-fr-en.tsv` and `judged-fr-es.tsv` judged in English and in Spanish and committed, a doubtful line saying so; the figures in the README (corpus, files, results, what they say) and the programme's table; the gloss experiment's columns filled if changes 48 and 49 merged first, else left empty (D6).
- [ ] 3.5 `MARKED_PAIRS` gains each pair on the first tier; `test/translate-marks.spec.ts` holds each measured pair's totals and tier and the list's exact value; `test/translate-relay.spec.ts` holds the list and marks a French selection through fr-en in the English sentence; a pair under 75 % correct or over 30 % withheld is reported to the owner (D7; *A French selection, judged in English*, *French through English, judged in Spanish*).

## 4. Gates and docs

- [ ] 4.1 `TRANSLATION.md` (the routes, the shared vocabulary, the models per native language, the soak, the measured pairs), `REVIEWERS.md` (four models and the routes each serves; nothing downloaded for French until it ships), the usage comments of `soak_engine.mjs` and `measure_marks.mjs` (D8).
- [ ] 4.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`. In `apps/site` (its check watches the catalogue): `site-check`'s steps pass unchanged.
- [ ] 4.3 Nothing else moves: `git diff --stat origin/main -- crates scripts/lingua-data apps/site apps/lingua-extension/test/baseline apps/lingua-extension/tool/marks/results-en-fr.jsonl apps/lingua-extension/tool/marks/results-es-fr.jsonl apps/lingua-extension/tool/marks/results-es-en.jsonl apps/lingua-extension/tool/marks/results-en-es.jsonl apps/lingua-extension/tool/marks/judged-en-fr.tsv apps/lingua-extension/tool/marks/judged-es-fr.tsv apps/lingua-extension/tool/marks/judged-es-en.tsv apps/lingua-extension/tool/marks/judged-en-es.tsv apps/lingua-extension/packs.json` is empty, and the goldens' tests pass without re-blessing.
- [ ] 4.4 `openspec validate add-lingua-french-translation --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-french-translation` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 50's row in `docs/lingua/language-matrix-programme.md` says where it stands.

## 5. Owner

- [ ] 5.1 [manual] The owner dispatches `lingua-model-deploy` after the merge: it creates `lingua-model-fr-en-base-memory-2.0` (and `lingua-model-en-es-base-memory-2.1`, if change 25's 4.1 has not run by then), deploys the host with every model of the catalogue, and `check_model_host` passes from outside (M18).
- [ ] 5.2 [manual] The owner reads the judgments, re-judging any line — the doubtful ones are named (design, Open Question 1) — and settles M15 for fr-en and fr-es before change 52 (Open Question 2); this change decides neither.
