# Design — generalise-lingua-gloss-reducer

## Context

See proposal.md (Why). Today, on origin/main:

| Where | What it does |
|---|---|
| `scripts/lingua-data/reduce_common.py` | Shared rules for « studied → FR ». Form-of regexes `_FORM_OF` (~47-65) also feed `wiktionary_signals`, so they shape en-fr's studied tables. MWE pointers `_MWE_FORM_OF` (~60). Notes and placeholders `_WIKI_NOTES` (~244-270). Dangling coordinators, joining, capitalisation. `alt_of` is never read. |
| `reduce-en-fr.py`, `reduce-es-fr.py` | Each pair's reducer. es-fr's native-side helpers (translation fallbacks, letter removal, casing, French-frequency order) live in its own file. |
| `pack_sources.py` | `rule_files` (~396-409) is the pair's reducer plus every top-level `reduce_*.py` (a glob). `derive()` (~198-237) reads `entry["translations"]` only. `check-reducer` compares one digest string. |
| `pack_report.py` | Reports per-table key changes. It fails only on a missing table or a loss beyond `--max-loss`. |
| `lingua-pack-update.yml` | Reduces one pair per dispatch and re-blesses both goldens on each run. |
| `lingua-extension-check.yml` | Rebuilds each pack from committed tables and compares the digest string. Nothing re-reduces from the pinned sources. |

Measured in a scratch copy:
- main's reducers reproduce all nine committed files of both pairs on macOS arm64;
- the prototype, with `Edition`, reproduces the seven tables and NOTICE of both pairs, and moves
  only `pack_version`;
- reduction takes 10 s for en-fr and 31 s for es-fr, with a 1.44 GB peak; fetching the pinned
  sources takes about 1 minute.

## Goals / Non-Goals

**Goals:**
- The English and Spanish editions' cleaning rules exist and are tested on their real data, ready
  for changes 21 and 22.
- en-fr and es-fr move once, with byte-identical tables, and never again for another edition's
  tuning.
- A pull request cannot commit tables the rules do not make, or a digest written by hand.

**Non-Goals:**
- Any new pair, and its sources or `DUMPS` entries: es-en and en-es are changes 21 and 22.
- M20, the long-parenthesis rule. It is an English-edition field, settled with change 21's review
  and re-pinning no French-native pair.
- Fixing the French rules' known defects (Proposal, Not here). Each would move tables, so it gets
  its own measured change.
- Merging same-part-of-speech etymologies before the round-robin: it affects 532 of the top-10k
  es-en lemmas. It is measured and decided in change 21, as an English-edition flag.
- Splitting the studied tables by language: change 7. It must move no digest.

## Decisions

### D1 — An `Edition` object, French first and unchanged

`Edition` (in `reduce_common.py`) holds:
- `code`;
- `form_of`, `mwe_form_of`, `pointer_tags`, `pointer_fields`;
- `notes`;
- `letter`;
- `dangling`;
- `capitalised`;
- `long_parenthesis`, 0 until M20 is settled.

Every function that cleans or classifies a gloss takes an edition, as a required keyword: the
shared module has no default to fall back on, since it imports no edition module (D2). The
French-native reducers bind the French edition (`functools.partial` and thin aliases), so their
names, their tests and their behaviour stay as they were.

The French instance holds today's regexes by identity, so en-fr and es-fr reduce exactly as
before. That includes the studied-side signals of en-fr, which read the same form-of test.

The English and Spanish instances come from the census of their kaikki data:

| Edition | Pointers recognised | Cleaning | Casing |
|---|---|---|---|
| English | `form-of` / `alt-of` tags and fields; untagged « (alternative \| obsolete \| …) (form \| spelling) of », « (plural \| inflection \| …) of », « synonym of », « only used in », « see » | no placeholder (none in 875,591 senses) | lower case, as written |
| Spanish | « Forma (del \| de la \| flexiva \| verbal \| …) … de », « grafía », « variante », tense and person names followed by « de » or « del » (the « de » is required, so « Femenino. » stays a meaning) | sense-link subscripts, taken out whole — one, a range (« Madrid₁₋₂ ») or two (« bottom₉ o ₁₀ ») — after a lower-case letter, the word's period or a stray space; never after a capital (case-sensitive, so « C₄H₁₀ » keeps its digits) nor a preposition (« similar a ₁ » names one of the entry's own senses, and the sense would not read without it); « Véase también » | — |

Native-side helpers that only es-fr had move into the shared module, generalised over studied
language and edition: translation fallbacks, letter removal, casing, frequency order. es-fr keeps
thin aliases, so its tests are untouched.

Alternative: a copy of the reducer per native language. The fallbacks and cleaning would drift
between copies, and the French bug fixes would land three times.

### D2 — Each edition is its own rule module, and the digest covers what a reducer loads

The editions' rules live in `reduce_edition_fr.py`, `reduce_edition_en.py` and
`reduce_edition_es.py`.
- A pair's reducer imports the shared module and its native language's edition module.
- `pack_sources.rule_files` becomes the pair's reducer plus every `reduce_*` module it loads, read
  from `sys.modules` after importing it.
- A test fails when importing a reducer loads a `reduce_*` module that is not in its rule files.
- `reduce_common.py` imports no edition module.

The effect is that tuning the English rules re-pins en-es and es-en, and never en-fr or es-fr.

Alternative: the glob, as the programme first assumed. Every English or Spanish tweak — M20, the
committed measurements of changes 21 and 22, dogfood fixes — would re-pin both French-native pairs
and re-bless both goldens. The Spanish programme needed about twelve such fixes.

### D3 — Sense-level translation tables

`derive()` also reads `senses[].translations`, writing the same line shape with the sense the table
names. Measured as a no-op on es-fr's dumps: frwiktionary has 197,400 French entries with
translations, and eswiktionary 32,488 Spanish entries, all with entry-level tables. Both pinned
derived files re-derive to their sha256. `pack_sources.py` is outside the digest, so this re-pins
nothing. A pinned re-reduction never calls `derive()`, so unit tests cover it.

### D4 — The proof

**`pack_report.py --identical`.** It exits 1, naming the pair and the file, when:
- any of the seven tables, the kept inputs (`tags.tsv`, `lexical.tsv`) or NOTICE differs;
- `manifest.json` differs anywhere but `meta.pack_version`;
- `pin.json` differs anywhere but `pack.sha256` and `reducer.{sha256,files}`.

**The `reduce` job in `lingua-extension-check`.** It runs on `ubuntu-24.04` with Python 3.12 and
the hashed requirements, and is filtered on `scripts/lingua-data/**` and on its own workflow file,
so a pull request that edits the job runs it.
- It re-reduces every `tables/*/` pair from its pinned sources with `build.sh --reduce`.
- It fails when `git status` shows any file under `scripts/lingua-data/tables` changed or added.

Linux reproducibility has only been measured on macOS so far. This pull request is the first Linux
run, and the design says so.

**`lingua-pack-update`.**
- `pair: all` reduces every pair in one leg and pushes one branch, blessing both goldens once.
- `expect: identical` passes `--identical`, and also fails when the coverage JSON or any golden
  line other than the `### pack` / `### beside` lines moves.

### D5 — The re-pin the owner approves

The re-pin reduces en-fr and es-fr again with the new rules. In each pair:
- `manifest.json` moves `meta.pack_version` only;
- `pin.json` moves `pack.sha256`, `reducer.sha256` and `reducer.files`.

Three golden lines move, each only in its 7-hex digest token:
- `en-fr.golden:4`;
- `es-fr.golden:4`;
- `es-fr.golden:6`.

The byte counts stay, because the metadata and NOTICE are stored uncompressed and `pack_version`
keeps its length.

Unchanged and checked: the seven tables, `tags.tsv`, NOTICE, `apps/site/src/data/lingua-coverage.json`,
every other golden line, and `ENGLISH_TYPICAL_VOCABULARY`'s provenance comment (it names the
version it was frozen from, which stays true).

## Risks / Trade-offs

- **[The Linux re-reduction does not reproduce macOS bytes.]** → The job shows it on this pull
  request. A real platform difference would be a reproducibility bug to fix before merging, since
  `lingua-pack-update` already reduces on Linux.
- **[The Spanish edition's wording regex drops a real meaning that starts « Forma de … ».]** → The
  « de » or « del » is required, and samples are reviewed in change 22, when the edition is first
  read for real.
- **[The CI job depends on upstream hosts]** (release assets, pinned raw URLs). → `curl --retry`,
  pinned sha256 on every fetch. A flake is re-run; it never passes silently.
- **[A rule module escapes the digest.]** → The `sys.modules` test, a test that every `reduce_*`
  import in a pair's rules names one of its rule files, and a test that no reducer or rule module
  loads code other than by an import statement (`importlib`, `__import__`, `exec`).

## Migration Plan

None for readers. The re-pinned packs ship whenever the owner next releases. Rollback is reverting
the change: the tables are the same.
