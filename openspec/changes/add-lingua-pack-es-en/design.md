# Design — add-lingua-pack-es-en

## Context

See proposal.md (Why). What stage 1 left in place, and what the pipeline does today:

| What | Where |
|---|---|
| The studied side of Spanish, once | `tables/es/` (forms, freq, grammar, level, tags, lexical, `studied.json` → es-fr), written by es-fr's reduction alone (change 7); any other pair "SHALL read them as committed and write only its own folder" |
| The native side of a pair | `tables/<pair>/` gloss, senses, mwe, NOTICE, manifest, pin, README; `check_committed_tables` and `committed_tables.rs`' exact file set for the shipped pairs |
| The English edition's rules | `reduce_edition_en.py` (`EN`: form-of pointers, letters, `capitalised=False`; `long_parenthesis` 0), tested on recorded senses |
| The sources es-en needs | es-fr's `kaikki` (the English Wiktionary's Spanish section, 1,054,565,864 B, asset of `lingua-pack-sources-es-fr-2026.10.03`) and `kaikki-es` (the Spanish Wiktionary's whole-edition dump, from which `kaikki-es-traductions.jsonl` es→fr is derived) |
| `pack_sources.py` | `KAIKKI`/`DUMPS`/`PINNED` per pair; `fetch_pinned` reads each source's `release` from the pin (another pair's release is followable); `fetch_live` always downloads the extract and writes `release_tag(pair, snapshot)`; the update publishes every asset under one tag and refuses an existing tag; `derive()` one pass per dump; `record_build` writes snapshot, pack, reducer, sources; `check_reducer` of a reader pair checks the reference's pin and rules too; `pairs --after`; `rule_files` = the reducer plus the `reduce_*.py` modules it loads, and a guard test refuses `importlib` in a reducer |
| `build.sh --reduce` | fetch-pinned → `pack_version = snapshot + rules[:7]` → split → build `--studied tables/es` → record-build; `max_lemmas` 60,000 for es-fr, 40,000 otherwise; the pack's sha256 fails the build when a studied table moved under a pair |
| `reduce-es-fr.py` | the Spanish studied side (forms, GSD, readings, `noun_class_runs`) and the native side bound to `french.FR`; the native tables from `native_tables(entries, ranks, studied=ES, edition=FR, fallbacks=[(direct, list), (inverted, by_french_frequency(…))])`, locutions winning; `_join_senses` (the round-robin) in `reduce_common.py` |
| The reduce job | pairs from `pairs`, `work/<pair>` removed after each, `timeout-minutes: 45`; the extract would be fetched once per pair |
| The pack | `MAX_PACK_BYTES` 5 MiB; es-fr 2,190,188 B; the lexical section written only when the dictionary words differ from the glossed lemmas (for es-en, 10,259 lemmas glossed but not dictionary words) |
| The extension | `packs.json` ["en-fr", "es-fr"], `check_variants` `SHIPPED_PAIRS` equal to it; `nativeLanguageOf` resolves a native only for a listed pair |
| Measured | es-en 93.6 / 87.0 / 77.1 %; M20 5,724 of 147,653 senses; 532 top-10k lemmas affected by etymology merging (change 6) |

## Goals / Non-Goals

**Goals:**
- A committed, pinned, reproducible es-en, reduced from the committed studied side and its own
  native sources.
- Nothing of es-fr, en-fr or the shipped package moves; no reducer loads another.
- The rules change 7 left open, settled once for every reader pair to come.

**Non-Goals:**
- Shipping (34); the card's English wording (23); marks (26); the site's figures (34 publishes
  them with the pair).
- An inverted table from the English Wiktionary's English entries (en→es): change 22's source.
- Computing the Spanish studied side again: it is committed.

## Decisions

### D1 — `reduce-es-en.py` builds the native side alone, from the committed studied tables

The reducer reads `tables/es/forms.tsv` and `freq.tsv` as committed — the lemmas and their
ranks the native side is matched against — and builds `gloss.tsv`, `senses.tsv` and `mwe.tsv`
with `common.native_tables(entries, ranks, studied=ES, edition=EDITION, fallbacks=[(direct,
list)])`: entries = the English Wiktionary's Spanish section (es-fr's
`kaikki` file, letters removed, reduced by `EDITION = english.EN`), direct = the Spanish
Wiktionary's English translations (`kaikki-es-traductions-en.jsonl`, `("translations", "es",
"en")`, derived from es-fr's `kaikki-es` dump); no inverted table. Nothing of the studied side
is computed: no forms, no readings, no levels, no `noun_class_runs` (the gender comes from the
readings, change 5 D5). Its rule digest is `reduce-es-en.py`, `reduce_common.py` and
`reduce_edition_en.py`.

Why not load es-fr's reducer: a reducer loads code by import statements alone (the guard test
and `rule_files`' rule), a dashed file cannot be imported, moving es-fr's studied side into a
module would move es-fr's digest and bytes, and the main spec says a reader pair reads the
committed tables. A change to es-fr's rules reaches es-en through the tables: when es-fr is
re-reduced and a studied table moves, es-en's build fails on its pack's sha256 until it is
re-reduced (`pairs --after es-fr`), and D3 names the table.

### D2 — Shared sources, one release per pair

`KAIKKI["es-en"]` names the same extract as es-fr's. `fetch_live` for a reader pair reuses the
extract its reference fetched in the same run (the asset cache, keyed by sha256) and records the
reference's release; dispatched alone, it downloads the live extract and records its own release
(kaikki regenerates daily, so es-fr's pinned extract is rarely the live one). `fetch_pinned`
follows a record's `release`, whichever pair's it is. `DUMPS["es-en"]` lists the `kaikki-es` dump with one
derived file, `kaikki-es-traductions-en.jsonl`, published under `lingua-pack-sources-es-en-
<snapshot>`. The update step tags `release_tag(pair, snapshot)` — never `sources.kaikki.release`, which for
a reader pair may be its reference's — and publishes a pair's own assets only: each source record
carries its release, and `assets --pin` lists the records whose release is that tag. The reduce job keeps
fetched assets in `work/cache/<sha256>` across pairs, so the 1.05 GB extract is fetched once per
job. A dispatch names one pair; es-fr's update brings es-en along in reduce mode from es-en's
own pin, as change 7 D5 has it.

Bootstrap: es-en's first update is dispatched alone on this change's pull request branch, so it
downloads the live extract and publishes it with its derived file under es-en's own release; the
pinned reduction that commits the tables runs after it, on the same branch, within the reduce
job's 45-minute timeout (the extract's fetch and one reduction fit; measured on the pull request).

### D3 — A reader pair's pin records the studied tables it read

`record-build` writes `"studied": {"reference": "es-fr", "tables": {"forms.tsv": "<sha256>", …}}`
for the six studied tables — `STUDIED_TABLES` (forms, freq, grammar, level), `LEXICAL` and the
pinned pool (`studied.json` is the record, not a table) — found through `studied_dir(pin)`, and
`save()` keeps the `studied` key beside `snapshot`, `pack`, `reducer` and `sources`; `check-reducer` compares them with `tables/es/` and fails naming the
pair and the table; `pack_report` names the table whose move left a pair behind. The pack's
sha256 catches the move already (change 7); the record names what moved. No snapshot of the
reference is recorded: a rules-only change of es-fr keeps its snapshot, and the tables say the
truth. `pack_version` is `<es-en's snapshot>+<its digest[:7]>`.

### D4 — Credits

`NOTICE` names both sides: the English Wiktionary's Spanish section (CC BY-SA 4.0 + GFDL) for
the forms, readings and glosses; the Spanish Wiktionary for the translations used as glosses;
wordfreq; UD Spanish-GSD for the readings' counts; the levels as es-fr estimated them, from
French glosses, and said so. `manifest.json` lists the same sources and `levels_estimated: true`.

### D5 — The two settings, decided in the pull request, without touching `reduce_common.py`

`long_parenthesis` is a field of `Edition`; `EN`'s value lives in `reduce_edition_en.py`. The
etymology merging is `merge_same_pos_etymologies(entries)` in `reduce_edition_en.py`, a pre-pass
over the entries that `reduce-es-en.py` calls before `native_tables`, behind an `EN` setting; the
round-robin in `reduce_common.py` is not edited. Editing `reduce_edition_en.py` re-pins the pairs
that load it — es-en alone (change 6 D2). The reducer runs both ways on the top 10,000 and the
pull request shows two samples of 100 glosses each; the owner picks; the values are committed
before the pin is recorded.

### D6 — Not shipped, measured against a floor

`packs.json`, `SHIPPED_PAIRS`, the listings and the site are untouched. `gloss_coverage.py`
gains `--pair` (measure one pair) and `--floor 87.6 77.2 63.7` (fail under); the reduce job runs
it for es-en; the figures are not published until change 34 lists the pair. `testdata/es-en/` lets
`gen:pack` build it then. `cross_native.rs` compares the real es-en pack's studied sections with
es-fr's, replacing the synthetic `SPANISH_IN_ENGLISH`.

## Risks / Trade-offs

- **The English extract is 1.05 GB** → fetched once per job (the cache) and never twice at an
  update (the reference's fetch in the same run reused); the reduce job's 45-minute timeout is measured
  against it on the pull request.
- **A gloss in a third language** (M5) → the direct table is the Spanish Wiktionary's English
  translations only; `test_reduce_editions` asserts the pairing.
- **The English native becomes resolvable** → `packs.json` is unchanged, so no listed pair is
  glossed in English and `nativeLanguageOf` keeps resolving `fr`.
- **A setting the owner rejects** → the setting stays 0 and the pin records that.

## Migration Plan

No release: tables and tooling only. The first `lingua-pack-update` dispatch for es-en runs on
the pull request branch and publishes its derived file's release.
