# Cymbra Lingua — the Spanish programme (ES→FR)

Spanish for French speakers, at the perimeter English has today. This file holds what the
33 changes of the programme share: the product owner's decisions, the architecture they build on,
their order, and their status. Each change still goes through OpenSpec; its proposal points here
instead of re-arguing a decision.

- Study: 2026-09-29/30, read-only, figures checked online (data, licences, translation models).
  Coverage figures come from throwaway prototypes: they are orders of magnitude, not targets.
- Decisions: Guillaume Fortin (product owner), 2026-09-30.
- Status: as of 2026-10-03 (change 8). Update the programme table in the pull request that moves a line.

## Scope

In scope, as for English: highlighting on pages and in the EPUB reader, the word card and its
grammar, statuses and calibration, deck and review, reading statistics, sync, the back office,
read-aloud, extended translation, the Safari / Apple app, and the agent plugin.

Out of scope: YouTube captions (the open change is amended to say so, nothing more) and remote
translation (not built for English either).

Expected result, measured against English: 47 features, of which 15 reach full parity, 20 parity
with a measured reservation, 4 are degraded (glosses, expressions, translation through a pivot,
single-word translation), 6 depend on the CEFR decision below, and 2 are out of scope. Word grammar comes out better
than English (≈435,000 readings, 75 tags, gender on 99.5 % of nouns).

## The product owner's decisions (2026-09-30)

| # | Decision | Consequence |
|---|---|---|
| D1 | **Estimated CEFR levels for Spanish.** A table derived from frequencies, labelled « niveau estimé » wherever a level is shown. If the derived scale is not monotone, fall back to three bands. | No commercially usable Spanish CEFR list exists (ELELex is NC, PCIC all rights reserved, CEFR-J Spanish unpublished). For Spanish this supersedes D2 of the archived `add-lingua-cefr-levels`, which refused computed levels; English keeps its CEFR-J levels. Measured on English: 40 % exact, 83 % within one level. Only `add-lingua-spanish-levels` depends on it. |
| D2 | **Pivot-marking threshold, fixed before measuring.** ≥ 90 % correct marks and ≤ 25 % withheld: the mark is shown. 75–90 %: the sentence is translated, without bold. Under 75 % correct, or over 30 % empty marks: Spanish translation is withdrawn from releases. | English reference: 83/87 (95 %). Mozilla has no direct es→fr model, so Spanish goes es→en→fr. The study's indicative sample scored ≈12–13/20 marked against ≈17/20 for English. `release-lingua-spanish-translation` measures with a committed corpus, Spanish and English, per platform. |
| D3 | **Translation ships per platform.** Spanish is announced at the reading release; translation follows platform by platform once measured. | The cost is stated like-for-like in the listings. |
| D4 | **Data gaps accepted, plus curated verbal locutions.** Thinner glosses and expressions ship with their numbers published. A curated list adds missing locutions (« hay que », « tener en cuenta »…). A gloss is never English, and never machine-pivoted. | French glosses cover ≈85/73/58 % of the top 5k/10k/20k lemmas without fallback, against 95/90/79 for English. Expressions: ≈3,000, 17 % of the English table. |
| D5 | **es-ES voice by default**, no accent setting. | The voice code needs a `PREFERRED_REGIONS.es`; today es-ES comes first by ordering luck. |
| D6 | **The agent plugin is in scope, last**, and the first thing cut. | `add-lingua-agent-languages`. |
| D7 | **YouTube captions out of scope**: an amendment to `add-lingua-youtube-captions` only. | 0/38 tasks built for English. |
| D8 | **Mixed review queue** ordered by due date, with a language filter. | Not one session per language. |
| D9 | **The language choice is not synced.** It lives in state and backup, and is offered on a new device. | A synced profile can be added later, additively. |
| D10 | **One store listing per store**, reworded. The final wording is the owner's. | `add-lingua-spanish-listings`. |

Not decided yet:

- LLM-assisted pack data (levels or glosses). Recommendation: no at launch.
- If the Spanish tables exceed ≈10 MB in git: commit them, attach them to a release snapshot, or
  keep only attested forms. Decide on S1's measurement; attested forms preferred.

## Architecture

Decided by the study; a change that departs from one of these says so in its design.

- **One engine, several packs, the language on every call.** One engine per language would
  overwrite the other's backup, and Firefox and Safari share one engine across tabs, so a pack
  cannot be swapped in. The engine reads the language from the pack (`Pack::studied()`); no
  `const EN`.
- **The language is per document** (a page, or an EPUB section), detected among the enabled
  languages. It comes back in an envelope **outside** the canonical `PageAnalysis` JSON, so English
  output does not move. Routing per block comes later.
- **The analyser version is per language** from the first core change. English stays `1.1.0`, so a
  fix to Spanish analysis never invalidates the English pack. NFC normalisation is Spanish-only.
- **TypeScript port:** a view bound to a language (`port.for(language)`) for everything that reads
  a pack. A root with no language serves backup, restore, review and export/apply.
- **Cards carry their language, server first** (done in `add-lingua-card-language`). A client that
  does not announce its languages receives English only, because installed clients label every
  card they receive `en`. Pull cursors reset when the declared set of languages grows.
- **Enclitics are a rule in the core**, never a table (combinatorial). The rule runs only when the
  whole form is absent from the lexicon, and is guarded: a listed word is never split, the accent
  must stay consistent, and monosyllabic imperatives are a closed list. « al » and « del » split
  the way « don't » does. No suffix rules (Wiktionary already lists diminutives and -mente forms).
- **Homographs: one form maps to one lemma.** The reducer picks the winner (corpus counts plus a
  hand-reviewed override file); the card shows the other reading (« peut aussi être… de *ir* »). A
  multi-lemma format is costed as a branch (3–5 days) if lemma agreement stays under 91 %.
- **Backup v2 is written only when a non-English language is present.** New builds read the
  version first. Released builds deserialise before checking the version, so they report a v2 file
  as « Malformed »; the spec states that behaviour.
- **Translation:** a model catalogue per pair with a route per language (`es → [es-en, en-fr]`),
  completeness per model, deletion with reference counting (en-fr is shared), and cost computed
  from the catalogue. The pinned Bergamot build already exports `translateViaPivoting`.
- **Detection:** a Catalan/Galician guard, a vote at document level, and the DOM `lang` hint. The
  remaining leak is written into the spec, not promised away. Without the guard, 40–55 % of Catalan
  or Galician blocks read as Spanish; with a function-word guard, ≈16–18 %.
- **Packs stay embedded.** Downloading them would not reduce memory: a loaded pack is 16 MiB of
  WASM linear memory however it arrived. The translation engine is what exposes the extension to
  the OS (223 MiB with one model, 322 MiB with the pivot). Real levers: load only the active
  language's pack, decode the pack lazily (≈⅓ of the heap), a sync engine with no pack, and a mobile
  policy for the translation engine.
- **Logic goes in `lingua-core`**, which is host-testable. `lingua-wasm` is excluded from the 80 %
  coverage gate.

## Data sources (checked 2026-09-29)

| Need | Source | Licence | Measure |
|---|---|---|---|
| Forms and grammar | kaikki, English Wiktionary, Spanish section | CC BY-SA 4.0 + GFDL (already accepted) | 811,049 entries, ≈8,800 conjugated verbs, every form tagged. French Wiktionary has no structured tags. |
| Frequencies | wordfreq `es` 3.1.1 (already pinned) | data CC BY-SA 4.0, credit Robyn Speer | 82.9 % of the top 40k tokens resolved, 96.6 % of the mass. Frozen since 2021. |
| Lemma choice | UD Spanish-GSD as tie-breaker; AnCora (licence trail to clarify); PUD kept for tests | CC BY-SA 4.0 / CC BY 4.0 | English rule alone: 88–90 % agreement; pinned table and frequency rule: 91–92 %; with corpus counts: 92–95 %. |
| Glosses and expressions | kaikki, French Wiktionary, Spanish entries; fallbacks from Spanish Wiktionary translations and inverted French tables | CC BY-SA | ≈23,700 glossed lemmas (English: 56,400). |
| Translation | Mozilla es-en `base-memory` 2.0 + en-fr | MPL-2.0 | +26.2 MB to download; latency ×2; BLEU 27.0 against 27.4 for a large direct model. |
| Detection | `whichlang` 0.1.1 (already a dependency) | MIT | Spanish recall 93–99 % above 40 characters; no Catalan or Galician class. |

Common to both languages: kaikki announces it will delete its per-language files, including the
URL pinned for English. The en-fr pin keeps its own snapshot as a release asset; the Spanish forms
change generalises the kaikki entry.

## Programme

In merge order (1 change = 1 pull request, none above ≈10 ideal days). Effort is in ideal days,
min–max.

| Stage | # | Change | Effort | Status |
|---|---|---|---|---|
| G0 | — | Archive sweep, in order: `add-lingua-backend` → `add-lingua-connected-clients`; `add-lingua-phrase-gloss` → `add-lingua-expression-table`; `add-lingua-translation-delivery` → `-android` → `-safari`; `add-lingua-reader`; `add-lingua-read-aloud`; `add-lingua-apple` | — | Open: each waits on manual release or verification tasks |
| G0 | — | Rewrite `add-lingua-connected-clients` in place | 1.5–3 | Done ([#607](https://github.com/NEETROF/cymbra/pull/607)) |
| G0 | — | Spikes S0–S8 (below) | ≈15 | S0 done ([#622](https://github.com/NEETROF/cymbra/pull/622): `crates/lingua-wasm/tests/english_baseline.rs`); S5 covered by [#619](https://github.com/NEETROF/cymbra/pull/619)'s reference run |
| G0 | — | Licence requests: UCLouvain (ELELex), Instituto Cervantes (PCIC), TUFS (CEFR-J Spanish) | — | Owner |
| R1 server first | 2 | `add-lingua-card-language` | 4.5–8 | Merged ([#609](https://github.com/NEETROF/cymbra/pull/609)); backend 0.35.0 deployed 2026-09-30. Tasks 5.1/5.2 (external checks) open |
| R1 | 3 | `add-admin-lingua-language-labels` | 1.5–2.5 | Not started |
| R2 silent English release A (shipped list `[en]`, asserted by `check_variants`) | 4 | `generalise-lingua-pack-reducer` | 5.5–9.5 | Merged ([#619](https://github.com/NEETROF/cymbra/pull/619)), archived 2026-10-03 |
| R2 | 5 | `generalise-lingua-analysis-by-language`: `Spanish` variant, dispatch, English arm moved verbatim, version per language, `Pack::studied()` | 6–10 | Done (proposal [#623](https://github.com/NEETROF/cymbra/pull/623)); English baseline unmoved, en-fr pack unchanged |
| R2 | 6 | `generalise-lingua-wasm-engine`: a pack per language in one engine, the language on every call, sync records in their own language | 4–7 | Done (proposal [#625](https://github.com/NEETROF/cymbra/pull/625)); English baseline unmoved, extension source unchanged |
| R2 | 7 | `generalise-lingua-extension-port` (first dogfood build): a root port and a language view, the language on every language-bound call and over the RPC | 4–7 | Done (proposal [#627](https://github.com/NEETROF/cymbra/pull/627)); asks in `en`, nothing a reader sees changed; R2 dogfood pass due before the release |
| R2 | 8 | `generalise-lingua-pack-build`: one list of shipped pairs (`packs.json`), a pack per pair at `assets/packs/<pair>.lingua`, each checked against its own pin and language | 4–7 | Done (proposal [#629](https://github.com/NEETROF/cymbra/pull/629)); same en-fr bytes, new path; `check_variants` refuses any list but en-fr |
| R2 | 9 | `package-lingua-packs-per-pair`: an engine starts with the default pair's pack and adds another listed pair's the first time its language is needed, synced records included | 2.5–4.5 | Done (proposal [#631](https://github.com/NEETROF/cymbra/pull/631)); en-fr alone still loads, at the same moment; a language nothing ships is refused before the engine |
| R2 | 10 | `add-lingua-studied-language-profile`: the reader's profile (the existing `Profile`) in the state and backup, never synced; backup schema version 2 only when a non-English language is present, read first; every surface reads in the reader's first shipped language | 3.5–6.5 | Done (proposal [#633](https://github.com/NEETROF/cymbra/pull/633)); English backups unchanged byte for byte (S0, pinned fixture); next: the R2 dogfood pass, then the silent English release |
| R3 silent English release B | 11 | `add-lingua-language-sync-client`: a device accepts the reader's shipped languages, names them in every card pull, files each card under its own language, pushes non-English cards only to a server that keys them by language, and pulls again from the start when the set grows | 4.5–8 | Done (proposal [#637](https://github.com/NEETROF/cymbra/pull/637)); every reader accepts `en`, so requests are unchanged in effect; no store package before `add-lingua-card-language` 5.2 |
| R3 | 12 | `add-lingua-language-routing`: each document read in its own language, chosen among the reader's by a length-weighted vote of its blocks, the page's `lang` as hint; the analysis JSON unchanged | 5.5–9.5 | Done (proposal [#639](https://github.com/NEETROF/cymbra/pull/639)); one accepted language asks for no detection |
| R3 | 13 | `add-lingua-language-choice`: « Langues étudiées » in the single settings builder, onboarding, level and voice per language, language labels module and lint | 6–10 | Done (proposal [#642](https://github.com/NEETROF/cymbra/pull/642)); hidden while en-fr ships alone; a voice kept before is the English one; `needsLevelChoice` per language |
| R3 | 14 | `add-lingua-language-stats-review`: one review queue across languages (due order when mixed) with a language filter, each card naming its language; daily statistics per day and language (`-v3`, the day-only `-v2` read as English), synced per language; a language selector in the statistics | 3.5–6.5 | Done (proposal [#644](https://github.com/NEETROF/cymbra/pull/644)); controls hidden while en-fr ships alone; the English review keeps its order (S0) |
| R3 | 15 | `add-lingua-reader-language`: a book section that declares no language takes the book's `dc:language`; `xml:lang`, and a declaration on the body, are read; the vote still decides | 2.5–4.5 | Done (proposal [#648](https://github.com/NEETROF/cymbra/pull/648)); one accepted language asks for no detection; archives after `add-lingua-reader` |
| R3 | 16 | `generalise-lingua-translation-catalogue`: `model-manifest.json` becomes a catalogue (models, decompressed sizes, a route per studied language to French); the runtime takes the English route; the setting's cost, the build, the variant check and the host tools (assembly, check, mirror releases via `tool/mirror_models.mjs`) read it | 3.5–6 | Done (proposal [#649](https://github.com/NEETROF/cymbra/pull/649)); same model, files and paths, a stored model stays complete; checked end to end on Chromium (download, translation) |
| R3 | 17 | `generalise-lingua-translation-model-state`: the device keeps the models its reader's languages need (union of routes, completeness per model, pruning, a shared model kept), one state naming the complete models and translatable languages with a new `missing` phase, a translation asked in its document's language | 4.5–7.5 | Done (proposal [#652](https://github.com/NEETROF/cymbra/pull/652)); one route, one model for every reader today; checked end to end on Chromium |
| G1 internal Spanish build (`[en, es]`, internal) | 18 | `add-lingua-spanish-analysis`: pre-pass, enclitics, accent retry, plurals, function words, 100+ fixtures | 7.5–13 | Not started |
| G1 | 19 | `add-lingua-spanish-detection-guard` | 2.5–4.5 | Not started |
| G1 | 20 | `add-lingua-spanish-forms-tables`: kaikki reducer, homograph policy, ranks per lemma, compact tables, measurement harness. Gates: 98.5 % of tokens resolved, 93.5 % content lemmas and 97 % AUX on PUD | 8.5–14 | Not started |
| G1 | 21 | `add-lingua-spanish-grammar-tables` | 3.5–6 | Not started |
| G1 | 22 | `add-lingua-spanish-gloss-tables` | 6–10.5 | Not started |
| G1 | 23 | `add-lingua-spanish-levels` (D1) | 3.5–7 | Not started |
| G1 | 24 | `add-lingua-spanish-word-card`: tense names in French school terms, gender and number, elision | 4–7 | Not started |
| G1 | 25 | `add-lingua-spanish-read-aloud` (D5) | 2–3.5 | Not started |
| R4 Spanish reading published | 28 | `enable-lingua-spanish`: pairs `[en, es]`, manifest summary ≤ 112 characters, dogfood on 5 targets, beta ring. The owner's go-ahead before merge and before each submission | 4–7.5 | Not started |
| R4 | 29 | `add-lingua-spanish-listings` (D10) | 2.5–4.5 | Not started |
| R4 | 30 | `add-site-lingua-spanish-pages` | 1.5–2.5 | Not started |
| R5 model host, then translation per platform | 26 | `add-lingua-spanish-translation-pivot`: es-en 2.0 pinned, mirror release, `translateViaPivoting`. After merge, deploy the host and run `check_model_host` from outside for es-en **and** en-fr 2.0, before any store package | 3–5.5 | Not started |
| R5 | 27 | `release-lingua-spanish-translation` (D2, D3): marking corpus committed in Spanish **and** English, confirmation pass per platform | 3.5–6.5 | Not started |
| R6 agent and parity | 31 | `add-lingua-agent-languages` (D6) | 5–8 | Not started |
| R6 | 32 | Amend `add-lingua-youtube-captions` (D7) | 0.5–1 | Not started |
| R6 | 33 | `refine-lingua-language-wording`: the only change allowed to modify the 12 requirements open changes held, after those archive | 1.5–3 | Not started |

The numbers are the study's: change 26 merges after 28–30, but keeps the number it was given.

**Spikes (G0).**
- S0 English invariance baseline (1).
- S1 prototype pack with glosses, at real weight (3.5).
- S2 pivot marking and memory on the weakest devices (4).
- S3 brief on estimated levels (1.5). D1 settles the decision; the monotonicity check that picks
  levels or bands remains.
- S4 detection guard on real documents (1.5).
- S5 does the raw kaikki dump reproduce en-fr byte for byte (0.5).
- S6 rehearsal with installed clients (1).
- S7 OpenSpec pre-flight (0.5).
- S8 homograph audit on real pages (1.5).

**Size.** 140–235 ideal days, most likely 180: 46 % platform generalisation, 38 % Spanish itself,
16 % process. Lingua English took 19 calendar days (182 pull requests, a median of 0.87 day per
change), so the calendar reading is ≈9 weeks (7–12), store reviews and server waits included.

## Rules every change of the programme follows

- **English does not move.** Each R2/R3 change keeps the English invariance baseline (S0)
  byte-identical, or bumps the English analyser version on purpose and says why. R2 and R3 ship
  as English releases with the shipped language list `[en]`.
- **Server first, filter by default.** No Spanish-capable client is built for a store before the
  server that filters cards is deployed and checked from outside.
- **OpenSpec: never MODIFY a requirement an open change holds.** Use ADDED only, or
  `archiveAfter`. Check the spec diff after each archive (the automatic archive ignores order, and
  aborts silently on a requirement it cannot find).
- **Never cut:** card language on the server with its filter and capability; the analyser version
  per language; the English invariance gates; the Catalan/Galician guard; the compact committed
  format; backup v2 only when needed.
- **If time runs short, cut in this order:** the YouTube amendment, the agent, the stats language
  selector, the reader language, the gloss fallback layers (gate relaxed to ≈84 %), translation on
  the weakest platforms then entirely, then levels.
- **Nothing is merged, deployed or published without the owner's go-ahead.**

## Main risks

1. An installed client receives a Spanish card, labels it `en` and sends it back. Mitigated by
   server first, an English-only default, the capability flag, and a rehearsal with released builds.
2. A released build meets Spanish state or a v2 backup and reports « Malformed ». Mitigated by
   writing v2 only when needed, reading the version first, and stating the behaviour in the spec.
3. Generalisation moves English output. Mitigated by moving the English arm verbatim, an analyser
   version per language, the envelope outside the JSON, the invariance gate on every pull request,
   and two silent releases.
4. The pivot costs too much memory (322 MiB) or marks too poorly, and loses « tu »/« usted » and
   gender through English. Mitigated by S2 before any code, the threshold fixed beforehand (D2),
   and per-platform offers (D3).
5. Estimated levels are judged misleading, or refused late. Mitigated by an isolated change, the
   « niveau estimé » label, and the band fallback (D1).
6. Frequent homographs (« fue », « como », « para », « creo »). Mitigated by corpus counts plus
   overrides, the other reading on the card, and PUD in tests.
7. A requirement is overwritten at archive. Mitigated by the OpenSpec rule above.
8. Thin glosses read as a lesser product. Mitigated by like-for-like numbers, written acceptance
   (D4), and curated locutions.
9. Catalan or Galician read as Spanish. Mitigated by the guard, the vote, the `lang` hint, and the
   residual leak stated in the spec.
10. Upstream data moves (kaikki deletions) and tables swell git. Mitigated by snapshots for both
    pairs, the compact format, and a monthly check per pair.
