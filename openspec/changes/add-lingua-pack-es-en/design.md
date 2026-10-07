# Design — add-lingua-pack-es-en

## Context

See proposal.md (Why). What stage 1 left in place:

| What | Where |
|---|---|
| The studied side of Spanish, once | `tables/es/` (forms, freq, grammar, level, tags, lexical, `studied.json` → es-fr), written by es-fr's reduction alone (change 7) |
| The native side of a pair | `tables/<pair>/` gloss, senses, mwe, NOTICE, manifest, pin, README; the committed-tables check (`crates/lingua-pack/src/tables.rs`) and `committed_tables.rs`' exact file set for the shipped pairs |
| The English edition's rules | `reduce_edition_en.py` (`EN`: form-of pointers, letters, lower case; `long_parenthesis` 0), tested on recorded senses |
| The sources es-en needs | es-fr's `kaikki` (the English Wiktionary's Spanish section, 1.05 GB, `lingua-pack-sources-es-fr-2026.10.03`) and `kaikki-es` (the Spanish Wiktionary's whole-edition dump, from which `kaikki-es-traductions.jsonl` es→fr was derived) |
| `pack_sources.py` | `KAIKKI`/`DUMPS` per pair; `fetch_pinned` fails for a pair not in `KAIKKI`; `derive()` one pass per dump; `record_build` writes snapshot, pack, reducer, sources; `check_reducer` checks a non-reference pair's reference too; `pairs --after` |
| `build.sh --reduce` | fetch-pinned → `pack_version = snapshot + rules[:7]` from the pair's reducer → split → build `--studied tables/es` → record-build; `max_lemmas` 60,000 for es-fr, 40,000 otherwise |
| The reducers | `reduce-es-fr.py` holds the Spanish studied side (forms, GSD, readings, `noun_class_runs`) and binds `EDITION = french.FR`; `reduce-en-fr.py` likewise for English |
| The rule digest | `rule_files` = the reducer plus the `reduce_*.py` modules it loads (`sys.modules`); editing a module re-pins only the pairs loading it |
| The pack | `MAX_PACK_BYTES` 5 MiB; es-fr 2,190,188 B; the lexical section written only when the dictionary words differ from the glossed lemmas (change 5: for es-en, 10,259 lemmas glossed but not dictionary words) |
| The extension | `packs.json` ["en-fr", "es-fr"], `check_variants` `SHIPPED_PAIRS` equal to it; `gen:pack` builds every pair of `packs.json` from `testdata/<pair>/`; a French reader never loads a pair glossed in another native (change 4) |
| Measured | es-en 93.6 / 87.0 / 77.1 % glossed; M20 5,724 of 147,653 senses; 532 top-10k lemmas affected by etymology merging (change 6) |

## Goals / Non-Goals

**Goals:**
- A committed, pinned, reproducible es-en: tables, reducer, pin, reduce job.
- Nothing of es-fr, en-fr or the shipped package moves.
- The rules change 7 left open, settled once for every reader pair to come.

**Non-Goals:**
- Shipping (34); the card's English wording (23); marks (26); the site's figures (34 publishes
  them with the pair).
- An inverted table from the English Wiktionary's English entries (en→es): change 22's source.

## Decisions

### D1 — `reduce-es-en.py` loads es-fr's studied side and binds the English edition

The Spanish studied side is `reduce-es-fr.py`'s code. `reduce-es-en.py` loads it by
`importlib` (a file named with a dash) and calls its studied-side functions, then binds
`EDITION = english.EN` and builds the native tables with `native_tables(entries, direct)`:
entries = the English Wiktionary's Spanish section (es-fr's `kaikki` file, letters removed as
es-fr does), direct = the Spanish Wiktionary's English translations (`kaikki-es-traductions-en.
jsonl`, derived `("translations", "es", "en")` from es-fr's `kaikki-es` dump); no inverted table
(the English Wiktionary's English entries are change 22's source). The studied-side output the
reducer writes stays in `work` (change 7's `split`).

Why by import and not by moving the studied side into a shared module: moving it would change
es-fr's `reducer.files` and digest, hence its `pack_version` and pack bytes — a re-bless the
programme's rules forbid outside change 6. Loaded modules enter es-en's digest, so es-en re-pins
when es-fr's rules change, which D3 wants.

### D2 — The sources: shared snapshot, one new derived file

`KAIKKI["es-en"]` points at the same extract as es-fr's (`kaikki.org-dictionary-Spanish.jsonl`)
and `pin.json` records the same release asset; `DUMPS["es-en"]` lists the `kaikki-es` dump with
one derived file, `kaikki-es-traductions-en.jsonl`. A reduction of es-en from pinned sources
fetches es-fr's assets (already published) plus that file, published by the first `update` of
es-en under `lingua-pack-sources-es-en-<snapshot>`. `fetch_live` derives both pairs' files from
one pass over the dump when both are asked.

### D3 — A non-reference pair's pin and `pack_version`

`pin.json` of es-en gains `"studied": {"reference": "es-fr", "snapshot": "<es-fr's>", "tables":
{"forms.tsv": "<sha256>", …}}`, written by `record-build` from the tables it read. Its
`pack_version` is `<es-en's snapshot>+<rules[:7]>`, where the rules are es-en's digest — its
reducer, `reduce_common`, `reduce_edition_en`, and `reduce-es-fr.py` loaded for the studied
side. When es-fr's rules change, es-fr is re-pinned and `pairs --after es-fr` re-reduces es-en,
whose digest moved too: both pins move in one pull request, as change 7 D5 intends. The
committed-tables check compares the recorded studied sha256s with `tables/es/`: a studied table
that moved without es-en re-pinned fails, naming the pair (the "pair left behind" of change 7,
now recorded rather than inferred).

### D4 — Credits

`NOTICE` names both sides: the English Wiktionary's Spanish section (CC BY-SA 4.0 + GFDL) for
the forms, readings and glosses; the Spanish Wiktionary for the translations used as glosses;
wordfreq; UD Spanish-GSD for the readings' counts; the levels as es-fr estimated them (from
French glosses — noted as such: the levels are the studied language's, shared). `manifest.json`
lists the same sources and `levels_estimated: true`.

### D5 — The two measurements, decided in the pull request

`long_parenthesis` (M20) and the etymology-merging flag are English-edition settings. The
reducer runs both ways on the es-en top-10,000 and the pull request shows two samples of 100
glosses each (before/after). The owner picks; the chosen values are committed in
`reduce_edition_en.py` (re-pinning no French-native pair, change 6 D4) before the pin is
recorded. The spec states the mechanism, not the values.

### D6 — Not shipped

`packs.json`, `SHIPPED_PAIRS`, the listings and the site are untouched; `gloss_coverage.py`
gains `--pair` to measure an unshipped pair without publishing it. `testdata/es-en/` lets
`gen:pack` build it when change 34 lists it. `cross_native.rs` compares the real es-en pack's
studied sections with es-fr's, replacing the synthetic `SPANISH_IN_ENGLISH`.

## Risks / Trade-offs

- **The English extract is 1.05 GB and the reduce job 45 minutes** → the sources are shared
  with es-fr (fetched once per job); es-en's reduction is ≈ 30 s.
- **A gloss in a third language** (M5) → the direct table is the Spanish Wiktionary's English
  translations only; no pivot, no machine translation; `test_reduce_editions` asserts the pairing.
- **The English native becomes resolvable** → a backup naming English as native with es-en
  listed in `packs.json` would resolve to it; `packs.json` is unchanged here, so it cannot.
- **A sample the owner rejects** → the setting stays 0 and the pin records that.

## Migration Plan

No release: tables and tooling only. The first `lingua-pack-update` dispatch for es-en publishes
its derived file's release.
