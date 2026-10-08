# Design — migrate-lingua-pack-sources-to-raw-dumps

## Context

See proposal.md (Why). What the pipeline does today, and what was measured for this design
(HEAD requests to kaikki.org on 2026-10-08; the pins as committed):

| What | Where |
|---|---|
| Two ways of reading kaikki | `pack_sources.py KAIKKI` (one per-language extract per pair, fetched live, kept whole as a zstd asset) and `DUMPS` (per pair, the whole-edition dumps it reads and the files it derives from each; the dump deleted after one pass) |
| What is pinned today | en-fr: `kaikki-Anglais.jsonl` 201,084,572 B (kaikki's regeneration of 2026-09-24, snapshot 2026.09.26). es-fr: `kaikki-Spanish.jsonl` 1,054,565,864 B (2026-09-28, snapshot 2026.10.03) and three derived files — `kaikki-fr-Espagnol.jsonl` 212,331,331 B, `kaikki-fr-traductions.jsonl` 6,191,621 B (both from the French dump of 2026-10-02 00:10), `kaikki-es-traductions.jsonl` 1,537,580 B (the Spanish dump of 2026-10-02 12:12). es-en: `kaikki-Spanish.jsonl` 1,054,565,723 B (2026-10-03, snapshot 2026.10.08) and `kaikki-es-traductions-en.jsonl` 2,200,504 B (the same Spanish dump). Three extracts, four derived files, three releases |
| The dumps | English edition `kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz` **2,981,058,381 B** gzipped (2.78 GiB; regenerated 2026-10-03 08:24); French `frwiktionary/raw-wiktextract-data.jsonl.gz` **736,590,407 B** (2026-10-02 00:10); Spanish `eswiktionary/raw-wiktextract-data.jsonl.gz` **103,226,106 B** (2026-10-02 12:12). The French and Spanish ones served today are the regenerations es-fr's and es-en's pins record |
| The extracts, as served (uncompressed) | English edition: Spanish 1,054,565,723 B (2026-10-03 10:55, the bytes es-en pins), French 583,677,104 B, English 3,335,546,346 B (change 22's `kaikki-en` source). French edition: Anglais 201,505,597 B, Espagnol 259,427,477 B, Français 3,215,393,854 B. Spanish edition: Francés 11,862,917 B, Inglés 47,311,153 B |
| A derived entries file is not the extract | The French dump's Spanish entries derive to 212,331,331 B; the Espagnol extract of the same regeneration is 259,427,477 B. The tables reduced from either were byte-identical (add-lingua-spanish-gloss-tables D1, measured): the proof lives at the table level, not the file level |
| Compression | zstd level 19: 197 MB → 13.6 MB for the Anglais extract (pin-lingua-pack-sources D3); 1.05 GB → about 52 MB for the Spanish section (`SOURCES.md`) |
| Runtimes | the `reduce` job: 3 min 38 s for en-fr, es-fr and es-en (2026-10-08), from release assets only; es-en's first update: 6 min with the 1.05 GB extract and the 98 MB Spanish dump — about 4 MB/s from kaikki; a local re-reduction of es-en: about 30 s |
| Limits | a GitHub release asset must be under 2 GiB; an `ubuntu-24.04` runner has about 14 GB of free disk; the `reduce` job has a 45-minute timeout, the update job none |
| `derive` | one pass per dump, `gzip.open`, a substring prefilter on `"lang_code": "<code>"` before `json.loads`; `entries` lines written as the dump writes them, `translations` lines cut to word, pos and the translations into one language, wherever the edition writes the table (`translations_of`, generalise-lingua-gloss-reducer D3) |
| The rule digest | a pair's rules are `reduce-<pair>.py` and the `reduce_*.py` it loads; `pack_sources.py` and `build.sh` are outside it, so this change re-pins nothing by itself |
| Stage 3's reads | fr-en: the English Wiktionary's French section (forms, readings, glosses), its English entries' French translations (inverted), the French Wiktionary's English translations (direct); fr-es: the Spanish Wiktionary's French section, the French Wiktionary's Spanish translations (direct, es-fr's inverted file), the Spanish Wiktionary's French translations (inverted, es-fr's direct file) |

## Goals / Non-Goals

**Goals:**
- Kaikki is read at three addresses, one per edition, and every file a pair reads is derived from
  them; no per-language extract is fetched by an update.
- Nothing committed moves: the pins, tables, packs and baselines of en-fr, es-fr and es-en (and
  en-es, once committed) are untouched, and keep reproducing from their releases.
- The equivalence of the two readings is measured on the real data before it is relied on.
- The six pairs of the programme read the catalogue; stage 3 adds no derivation.

**Non-Goals:**
- Re-pinning a pair, or taking in upstream drift: each pair's next update.
- Publishing or archiving a dump; a mirror of kaikki.
- A per-edition derive job in the update workflow (see D4's alternative and the Risks).
- The French studied side's sources (change 43 may add a file to the catalogue).

## Decisions

### D1 — One catalogue per edition; a pair names what it reads; the extract registry is retired

`pack_sources.py` gains `EDITIONS`: for each edition — `en`, `fr`, `es` — the dump's address and
the **catalogue** of files derivable from it, each `("entries", lang)` or `("translations", lang,
into)` as `derive` already reads them. `DUMPS[pair]` becomes the files the pair reads, by edition,
and `KAIKKI` is deleted: `fetch_live` requires `pair in DUMPS` and reads nothing else of kaikki.
Existing file names are kept, since a name is a reducer's input and part of its rules; new ones
carry the edition, in ASCII alone — GitHub renames a release asset whose name holds another
character on upload, so `kaikki-es-Frances.jsonl`, not kaikki's « Francés ». The catalogue, with the
pairs that read each file:

| Edition (dump, gzipped, 2026-10-08) | File | Kind | Read by |
|---|---|---|---|
| `en`, 2,981,058,381 B | `kaikki-Spanish.jsonl` | entries `es` (1,054,565,723 B raw, about 52 MB compressed) | es-fr (Spanish's studied side: forms, readings, the gender of nouns), es-en (glosses) |
| | `kaikki-French.jsonl` | entries `fr` (583,677,104 B raw) | fr-en (French's studied side and glosses; changes 43, 45, 48) |
| | `kaikki-en-traductions-es.jsonl` | translations `en` → `es` | en-es (direct; change 22's `kaikki-en-traductions-es.jsonl`, now from the dump) |
| | `kaikki-en-traductions-fr.jsonl` | translations `en` → `fr` | fr-en (inverted; change 48) |
| `fr`, 736,590,407 B | `kaikki-Anglais.jsonl` | entries `en` (201,505,597 B as the extract; the derived file is smaller) | en-fr (glosses, expressions, form links) |
| | `kaikki-fr-Espagnol.jsonl` | entries `es` (212,331,331 B pinned) | es-fr (glosses) |
| | `kaikki-fr-traductions.jsonl` | translations `fr` → `es` (6,191,621 B pinned) | es-fr (inverted), fr-es (direct; change 49) |
| | `kaikki-fr-traductions-en.jsonl` | translations `fr` → `en` | fr-en (direct; change 48) |
| `es`, 103,226,106 B | `kaikki-es-English.jsonl` | entries `en` (47,311,153 B as the Inglés extract) | en-es (glosses; change 22) |
| | `kaikki-es-Frances.jsonl` | entries `fr` (11,862,917 B as the Francés extract) | fr-es (glosses; change 49) |
| | `kaikki-es-traductions.jsonl` | translations `es` → `fr` (1,537,580 B pinned) | es-fr (direct), fr-es (inverted; change 49) |
| | `kaikki-es-traductions-en.jsonl` | translations `es` → `en` (2,200,504 B pinned) | es-en (direct), en-es (inverted; change 22) |

Three pairs are re-registered here (en-fr, es-fr, es-en); change 22 registers en-es against the
catalogue instead of its `kaikki-en` extract-as-dump source if it lands after this change, and is
re-registered here if it lands before; changes 48 and 49 register fr-en and fr-es. A change that
needs a file the catalogue lacks adds it there, in the edition it comes from, and nowhere else.

*Rejected — keep the extracts while kaikki serves them, and switch each address when it goes.*
That is what kaikki's notice allows, and it is what stage 3 would multiply: five extracts for six
pairs, two of them the same 1.05 GB address fetched and published twice (es-fr and es-en today),
and change 22's English extract at 3,335,546,346 B served plain, larger than the whole English
edition gzipped. The dumps are the one address kaikki says it will keep.

### D2 — The dump is never kept, never published; what a pair reads is, under the pair's release

An edition's dump is downloaded to `work/dumps/<edition>-<snapshot>.jsonl.gz`, read once (D4),
and deleted. A pair's derived files are zstd-compressed and published as assets of the pair's own
release, `lingua-pack-sources-<pair>-<snapshot>`, pinned by the sha256 of their decompressed bytes
and fetched by `fetch-pinned` from the release the record names — change 21's D2 unchanged, so
*A reader pair's pin records the studied tables it read* stands as written: each pair pins its own
fetch, an update publishes a pair's own assets only, a pair brought along by its reference is
reduced from its own pin.

The asset sizes are those of the derived files: the largest is the English Wiktionary's Spanish
section, about 52 MB compressed; a release is about 70 MB for es-fr (its four files), 53 MB for
es-en, 14 MB for en-fr, a few MB for en-es and fr-es, and about 35 MB for fr-en (the French
section, at the Spanish section's ratio). Every one is far under the limit.

*Rejected — publish each edition's dump as the snapshot's asset, so that any pair, present or
future, derives from a pinned snapshot.* The English edition's dump is 2,981,058,381 B gzipped:
above the 2 GiB a release asset may weigh, and a zstd re-compression is not sure to bring it under
(gzip already yields about 8:1 on this JSON). Splitting it into parts would work and was
considered; but the only consumer of a kept dump would be a new kind of derived file wanted for an
old snapshot, and tomorrow's dump serves that just as well, since any update moves the tables
anyway. Six pairs would also store about 3.6 GiB of dumps per update day. Not worth it.

### D3 — The dump's identity in the pin

A dump record keeps today's fields — `release`, `url`, `fetched`, `last_modified`, `files` — and
gains `dump`: the sha256 and size of the decompressed bytes and the size of the gzipped file,
computed in the same pass as the derivation (the gzip stream is hashed as it is read: about a
minute more over the English edition, nothing over the others). Example, es-fr updated on
2026-10-08 from the English dump served that day — its address, headers and gzipped size as
measured; the decompressed size is D4's estimate until T2.3 measures it, the derived file's size
an estimate until T2.2 does, the sha256 elided:

```json
"kaikki-en": {
  "release": "lingua-pack-sources-es-fr-2026.10.08",
  "url": "https://kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz",
  "fetched": "2026-10-08",
  "last_modified": "Sat, 03 Oct 2026 08:24:38 GMT",
  "dump": { "sha256": "…", "size": 23000000000, "compressed_size": 2981058381 },
  "files": { "kaikki-Spanish.jsonl": { "asset": "kaikki-Spanish.jsonl.zst", "sha256": "…", "size": 1060000000 } }
}
```

No `kaikki` extract record is written by an update any more; `kaikki-en`, the key change 22's
design gave the English extract, names the English edition's dump from here on.

Why record what is not kept: two pairs updated from one regeneration carry one dump sha256 and
derive files of one sha256, so the statement "the same snapshot" is checkable from the pins and
the asset cache fetches the file once; a reader who obtains the same dump elsewhere can check it;
and a monthly report can say which regeneration drifted. The cost is one field.

### D4 — A dump is read once per run; the monthly check is one job

`fetch_live` derives an edition's **whole catalogue** at its first read of a run, into
`work/editions/<edition>-<snapshot>/`, and a later pair of the same run copies what it reads from
there instead of downloading the dump again. The pass is the cost; writing a file no pair of the
run reads is not (the French section of the English edition, 584 MB, when only es-fr is updated).
`work/editions/` is removed with the run; `work/<pair>/` is removed after each pair — today the
workflow removes it in reduce mode only (`lingua-pack-update.yml`, the Reduce step), since an
update's own pair publishes from it; the single job removes it in dry mode too, once the pair's
pack is built, since a dry run publishes nothing and the next pair needs the room (D8).

The monthly dry run therefore becomes **one job over every pair**, in `pairs` order, instead of a
matrix of one job per pair: each dump is fetched once a month — about 3.6 GiB and three passes —
instead of once per pair, about 14 GiB and four passes over the English edition with six pairs. An
update dispatched for one pair fetches only the dumps of the editions that pair reads; the pairs
it brings along read no dump (their own pins).

Two things the matrix gave for free, the loop must keep:

- **Each pair reduces into a dry root of its own.** `build.sh --dry` removes `<root>/<pair>` and
  `<root>/<studied>` and lays the committed studied folder down again before every pair (the
  `dry` branch of `build.sh`), so in one job over en-fr, es-fr, en-es, es-en the later pairs would
  reset `work/dry/en` and `work/dry/es` — the references' drift — before the Report step reads
  them; the matrix hid this by giving each pair a runner of its own. The loop sets
  `LINGUA_DRY_ROOT=scripts/lingua-data/work/dry/<pair>` (a variable `build.sh` already reads) for
  each pair, and the Report step reads a pair's folder under its own root and a studied language's
  folder under its reference's root — the first pair of that language in `pairs` order. The roots
  hold tables only, tens of megabytes.
- **A failing pair does not stop the loop.** The matrix runs `fail-fast: false`, so one pair's
  failure never hid another's report; the loop keeps that: a pair whose `build.sh` fails is named,
  the loop goes on to the next pair, the Report step skips the folders that were not written, and
  the job fails at the end.

Estimated: at kaikki's observed 4 MB/s, the English dump downloads in about 12 minutes, the
French in 3, the Spanish in under 1; `derive`'s pass over the English edition — an estimated 20–25
GB decompressed (its English, Spanish and French sections alone are 5 GB), streamed through
Python's gzip with the substring prefilter — in about 5–15 minutes. An update of es-fr would take
25–40 minutes, the monthly job 35–50 for four pairs. Both are measured on the implementation pull
request (T2.3); the budget is an update within 45 minutes.

*Rejected — a per-edition derive job that publishes artifacts the per-pair jobs read.* The
cleanest shape for the monthly run, and the right one if the single job measures too long; it
restructures the workflow (two jobs, artifacts of about 120 MB, the publish step moved) for about
a day, outside the programme's estimate. The single job gets the same download count for a loop.

### D5 — No pair is re-pinned; legacy records read as recorded; the move is each pair's next update

The pins of en-fr, es-fr and es-en are not touched. `fetch-pinned` reads every kaikki record of a
pin by its shape — an `asset` at the record's top (an extract) or `files` (derived files) — from
the release the record names, checked by sha256, with no registry check; `assets --release` lists
both shapes. Two lines of today's `fetch_pinned` would break that silently once `KAIKKI` is gone,
so the design names them:

- **A legacy `kaikki` record is kept, not pruned.** `fetch_pinned` builds the set of records it
  reads from the registry (`PINNED`, `ESDB`, the literal `kaikki`, `DUMPS`, `wordfreq`), deletes
  every other record as "no longer read" and saves the pin before fetching anything. With `KAIKKI`
  deleted, that set must still hold `kaikki` when the pin has such a record: otherwise the record
  is pruned, the rewritten pin shows in `git status`, and the reduce job's gate
  (`lingua-extension-check.yml`, *Reduce every pair again from its pinned sources*) fails on a pin
  that moved. A legacy record is read as an extract, and the pin's bytes are unchanged after the
  fetch.
- **The extract's raw name comes from the record.** Today the raw file is
  `work / KAIKKI[pair]["file"]`; after this change it is the record's `asset` name without its
  `.zst` (`kaikki-Spanish.jsonl.zst` → `kaikki-Spanish.jsonl`), which is the name the reducer reads.

The reduce job keeps reproducing every committed table, manifest and pin from the pinned
extracts, and the `check` job keeps building the pinned packs. A pair moves to the dumps when it
is next updated: its pin then records the editions' dumps and the files derived from them, names
no extract, and its report shows the upstream drift beside what D6 measured for that pair — nothing
else for en-fr and es-en, whose tables are the same both ways; es-fr's four readings and en-es's
order of the translation tables otherwise.

Why not re-reduce from the dumps "at the same snapshot": the dumps of 2026-09-24, 2026-09-28 and
2026-10-03 were never kept (add-lingua-spanish-gloss-tables D1 keeps no dump, and kaikki serves
only its latest regeneration); the pinned extracts are the only copy of those regenerations. The
one exception is the present: on 2026-10-08 kaikki still serves the French and Spanish dumps of
2026-10-02 that es-fr and es-en pin, and an English dump of 2026-10-03 08:24 — the regeneration
the Spanish extract es-en pins (2026-10-03 10:55) most likely came from. While that holds, es-en's
committed tables can be reproduced from the dump itself (T2.2), and es-fr's derived files
re-derive to their pinned sha256. It is an opportunity, not a plan and not a gate: T2.2 tries it
if the dumps are still served that day and skips it otherwise, saying so in `SOURCES.md`; it ends
with kaikki's next regeneration, and D6's both-ways measurement is what each pair's move relies on
either way.

Why not re-pin every pair now from today's dumps: that is an update — en-fr's dictionary has
drifted since 2026-09-24, es-fr's since 2026-09-28 — and the programme keeps a dictionary update
(its own pull request, its report, the baselines re-blessed with the owner's approval) apart from
a tooling change that must move nothing. The owner decides when each pair's next update is
dispatched (T5.1); this change does not require one.

### D6 — The measurement: both readings of one regeneration, reduced both ways

Before the first update under D1, each kind of file that replaces an extract is measured against
it on the real data, at the table level — the file level is already known not to hold (212 MB
against 259 MB for the Spanish section of the French edition; what differs is measured and said in
`SOURCES.md`, T2.2):

- the English edition's Spanish section: es-fr and es-en reduced from their pinned sources with
  `kaikki-Spanish.jsonl` taken once from today's extract and once derived from today's dump, the
  other sources pinned; `pack_report.py --identical` on both pairs' tables, both folders;
- the French edition's English section: en-fr likewise with `kaikki-Anglais.jsonl`;
- the English entries' Spanish translations: en-es's tables both ways once change 22 has committed
  them, else the two derived files compared as sets of lines (`derive` writes in input order, and
  the extract's order is not the dump's).

Both readings must come from the same regeneration: kaikki writes the dump first and the
per-language files from it within hours, so the measurement runs on one day and checks the two
`Last-Modified` dates. Measured on 2026-10-08 (`SOURCES.md`, *Extract and dump are measured against
each other*): en-fr's and es-en's tables are byte for byte the same both ways. es-fr's
`es/grammar.tsv` gains 4 readings from the dump — the feminine plurals of *beta*, *delta*, *kappa*
and *zeta* —: the dump leaves the page's categories on the entry, where the extract assigns them to
the senses, and es-fr's letter-name rule reads a sense's. en-es's `gloss.tsv` and `mwe.tsv` take
186 and 55 entries' words in another order or another third word: the dump keeps each translation
table where the page writes it, where the extract moves it under the sense it translates. Each
difference is recorded in `SOURCES.md` with its cause and carried by the pair's next update, which
names it in its report — this change re-pins nothing. es-fr's breaks *A letter's name gives no
reading of its plural* (`betas` read as the plural of the letter *beta*), so a change of its own
fixes it before es-fr's next update is merged (Open Questions); en-es's calls for no reducer fix —
the derived file has no sense glosses to order the words by — and the owner judges the 186 glosses
at en-es's next update.

### D7 — The publish step and its notes

The step keeps its shape: `release-tag` and `assets --release` list the pair's own assets, the
tag refuses to exist twice, the release is created once. Its notes no longer name an extract
(`sources.kaikki.url` is gone): a new `pack_sources.py dumps --pin` lists each edition's dump the
pin records — address, regeneration date, decompressed sha256 and size — and the notes say that
the assets are what the pair derives from them. `test_reduce_loops.py` doubles `gh` and `build.sh`
as it does — never `curl`: the workflow's steps fetch nothing themselves — and checks the notes
and the asset list for a pin with dump records only, for a legacy pin, and for one of each. That
a dump is fetched once per run is `fetch_live`'s doing, not the workflow's, so its test lives in
`test_pack_sources.py` beside *One pass per edition per run*, counting the calls of the injected
`fetch`.

### D8 — Cache keys, disk and the reduce job

- `work/cache/<sha256>` keeps fetched release assets by the sha256 of their decompressed bytes,
  unchanged: an extract and a derived file are both release assets, so the reduce job's fetches,
  cache hits and runtime are what they are today (3 min 38 s for three pairs). A pinned
  re-reduction never reads a dump.
- A dump is not cached: an update runs on a fresh runner, and its only consumer is `derive`'s one
  pass. A developer updating the same pair twice in a day downloads it twice; accepted.
- `work/editions/<edition>-<snapshot>/` is the run's own reuse (D4), keyed by edition and snapshot
  day, never read across runs: a later run of the same day could face a regeneration in between,
  and the pin, not the folder, is the record.
- Disk, worst moment (es-fr's update, or the monthly job while es-fr runs): the English dump
  2.8 GiB, the French 0.7, the Spanish 0.1, each deleted after its pass; the editions' derived
  files about 2.5 GB raw at most; the pair's work folder — its derived files copied from the
  editions, about 1.3 GB raw for es-fr, and its compressed assets about 0.1 GB; the pack builder's
  cargo target, about 1–2 GB (`cargo run --release -p lingua-pack`, restored by `rust-cache`);
  the dry roots and the committed copy, tens of megabytes. About 9 GB — under about 10 GB of the
  runner's 14 GB. The monthly job reads the editions in the order the pairs need them, deletes
  each dump after its pass and each pair's work folder after its pack (D4): without that removal
  the four pairs' work folders alone would add about 2.8 GB.
- The English edition's Spanish section (1.05 GB raw) was zstd-compressed once per pair that
  reads it — twice per monthly job, by es-fr and es-en, minutes each at level 19. T2.3 measured it
  at about 3 minutes of the monthly job, so `pack_asset` computes the sha256 first and copies the
  entry the asset cache already holds under it — those bytes, compressed by an earlier pair of the
  run or fetched and checked — instead of compressing again (the pin records the decompressed
  sha256, the same either way).

### D9 — What the documents say

`SOURCES.md` gains a section on the editions' dumps: the catalogue (D1), the sizes measured here
and at the first update, the equivalence measurements (D6), and the rule for legacy records (D5);
the per-pair sections stop calling a source "the extract" and name the section of the edition it
is. The three READMEs' *Take in upstream changes* describe the dumps and the pair's release. The
update workflow's header comment follows. Change 22's design note — "change 38 switches the
address" — is honoured: its `kaikki-en` source becomes the English edition's dump, and its
`derive` reading a plain or a gzipped file is kept (kaikki serves its extracts plain, its dumps
gzipped; the magic tells them apart).

## Risks / Trade-offs

- **The equivalence does not hold for a section** (an extract carries lines or fields the dump
  lacks, or the reverse) → measured before anything depends on it (D6); the difference is
  explained and carried by that pair's next update, reviewed; nothing committed moves here.
- **The English dump's pass is slower than estimated** → the measurement (T2.3) sets the figure;
  above 45 minutes for one update, the prefilter is tightened (an entry's `"lang_code"` sits near
  the end of its line today; a cheaper mark exists) before a per-edition job is considered. The
  update job's timeout is 90 minutes, five times what T2.3 measured, and a stalled transfer is cut
  and retried (`STALL`).
- **kaikki's bandwidth** → an update reads 3.6 GiB at most, the monthly job the same once; the
  per-pair matrix would have read four times as much. The sequence of the monthly job (D4) is the
  mitigation.
- **kaikki regenerates between two pairs' updates on one day** → each pair pins its own
  regeneration, as today (D2); the dump sha256 says so (D3).
- **Change 22 lands first with its `kaikki-en` extract source** → re-registered here (D1); its
  pin, like the others', moves at its next update.
- **A runner out of disk** → the order and deletions of D8, measured on the monthly job.

## Migration Plan

No release, no table: tooling and documents only. The implementation pull request carries the
measurements (T2.2, T2.3) and leaves every pin as it is. After the merge, the first update of a pair — the owner's
dispatch, when the monthly report or a reader's bug calls for it — moves that pair to the dumps in
its own reviewed pull request; stage 3's pairs are born on them.

## Open Questions

- es-fr's four readings from the dump (D6) break *A letter's name gives no reading of its plural*:
  a change of its own must fix them before es-fr's next update is merged (T5.1). Reading the
  entry's categories would be wrong — the dump puts the section's categories on every entry of the
  page, so the rule would drop the committed `Masc|Plur` readings of a page's other nouns; the
  likely fix is a rule on the sense's gloss (« the Greek letter … », « Greek letter delta »,
  « the letter Z »), tested on a fixture shaped as the dump writes an entry.
- Whether the owner wants the shipped pairs moved soon after the merge (an update of es-fr, which
  brings es-en along, then of en-fr — two reviewed dictionary updates, baselines re-blessed), or
  left to the monthly report (T5.1).
- The monthly job's duration as one sequential job (D4): acceptable up to what figure, before the
  per-edition derive job is worth its day.
- The names of the new files (`kaikki-French.jsonl` against `kaikki-es-Frances.jsonl`): the
  existing convention is kept for the English edition and the edition-marked one elsewhere; a
  single convention would rename files reducers read and move their digests, so it is not done here.
