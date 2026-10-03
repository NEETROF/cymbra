# Design — generalise-lingua-analysis-by-language

## Context

See proposal.md (Why). The analysis of `lingua-core` is English-only at four places, and nowhere
else:

| Seam | Today |
|---|---|
| `analysis/language.rs` | `StudiedLanguage { English }`; detection maps it to whichlang `Eng` |
| `analysis/tokenize.rs` | `let StudiedLanguage::English = language;` then the pre-pass, whose `n't` split is English |
| `analysis/function_words.rs` | `let StudiedLanguage::English = studied;` then the English tables |
| `analysis/lemmatize.rs` | `lemmatize(form, lexicon)`, no language: irregulars, then the pack's forms, morphy rules, plural fallback |

The pipeline (`pipeline.rs`) and the engine (`engine.rs`) already take a `StudiedLanguage` on
every call, and the knowledge model already keys its maps by it. `Pack::load` compares the pack's
`analyzer_version` with the single `ANALYZER_VERSION` (`1.1.0`). Three readers outside the core
use that constant:
- the pack builder (`lingua-pack`) writes it into test manifests;
- the reducer (`reduce-en-fr.py`) reads it with `pub const ANALYZER_VERSION: &str = "…";`;
- the extension build (`build.mjs`) reads it with `ANALYZER_VERSION:\s*&str…`, which is unanchored.

`lemmatize` has two callers outside the analysis: the pack builder's grammar paradigms and
expression keys (`lingua-pack/src/lib.rs`). The wasm engine (`EN`, 33 uses) and the agent plugin
(`EN`) pass English explicitly.

The English invariance baseline (`crates/lingua-wasm/tests/english_baseline.rs`) records what the
extension receives from the engine over a fixed corpus with the real en-fr pack: 142 probes. The
en-fr pack's sha256 is pinned in `scripts/lingua-data/tables/en-fr/pin.json`, and
`lingua-extension-check` rebuilds the pack against that pin.

## Goals / Non-Goals

**Goals:**
- A second variant the whole core compiles and dispatches on, with English's behaviour moved
  verbatim and proven unmoved.
- A Spanish arm that is honest: the pack's forms and nothing borrowed from English.
- Analyser versions and pack compatibility per language.

**Non-Goals:**
- Spanish analysis rules: pre-pass, enclitics, accent retry, plurals, function words, NFC
  (`add-lingua-spanish-analysis`).
- Detection between enabled languages, and the language envelope beside `PageAnalysis`
  (`add-lingua-language-routing`).
- The wasm engine reading `Pack::studied()` and its exports' `"language"`
  (`generalise-lingua-wasm-engine`).
- Backup v2, and a reader choosing Spanish (later changes of the programme).

## Decisions

### D1 — Spanish is appended; serialised names and order stay

`StudiedLanguage::Spanish` goes after `English`. The derived `Ord` keys the knowledge model's
`BTreeMap`s, and serde writes variant names (`"English"`), so a state holding only English
serialises byte for byte as before. The type gains:
- `tag()` / `from_tag()`: ISO 639-1, exact match;
- `ALL`, the list of languages the core can analyse;
- whichlang `Spa` for detection.

*Rejected — serialise as ISO codes (`"en"`).* That rewrites every stored state and backup for no
gain. The tags serve packs and the wire; the variant names serve state.

### D2 — One `match` per seam; English's arm is the current code

Each seam `match`es on the language. English's arm is the existing code, moved without edits, so
the diff of the English path is a signature change and an indentation. The changes per seam:
- **Tokenisation:** the pre-pass takes a `contractions` flag, true for English.
- **Function words:** the tables are selected by language.
- **Lemmatisation:** `lemmatize(form, studied, lexicon)` dispatches to `lemmatize_english` (the
  current cascade) or `lemmatize_baseline`.

*Rejected — a `LanguageRules` trait object (or a table of function pointers) per language.* With
two languages it adds indirection for nothing, and it makes "English did not move" harder to see
in review. A third Romance language can revisit this (the study counts 28–48 days for one, after
this programme).

### D3 — The Spanish baseline keeps only what belongs to no language

Kept: UAX #29 segmentation; edge-apostrophe trimming; hyphenated compounds (whole when the pack
lists the whole, parts otherwise); the drop of words containing digits; single letters counted
only when the pack lists them; the pack's form→lemma lookup, else the lowercased form.

Dropped, because each is an English rule that misreads Spanish:
- the `n't` split;
- the irregulars table (« has », « are », « more », « ate » are also Spanish words);
- the morphy-style suffix rules and the regular-plural fallback (« mes » → « me »);
- the English function words.

Spanish runs at analyser `0.1.0` until `add-lingua-spanish-analysis` gives it a cascade of its own
and `1.0.0`. Nothing reaches a reader in Spanish before then: no shipped pack studies Spanish.

### D4 — An analyser version per language; `ANALYZER_VERSION` stays English's

`StudiedLanguage::analyzer_version()` returns `ANALYZER_VERSION` (English, `1.1.0`) or
`SPANISH_ANALYZER_VERSION` (`0.1.0`). Keeping the name and meaning of `ANALYZER_VERSION` means its
three outside readers keep reading English's version unchanged. A page analysis reports
`studied.analyzer_version()`.

*Rejected — one core version for every language.* A fix to a Spanish rule would invalidate the
English pack and move English counts, which is exactly what the programme forbids.

### D5 — A pack names its language; compatibility is checked within it

`Pack::load` maps `meta.studied` through `from_tag`. An unknown tag fails with
`PackError::UnknownLanguage` before any section is read. The pack's `analyzer_version` is then
compared with that language's version. `Pack::studied()` exposes the result. Tags are normalised
where they enter the system (the reducer writes `en`/`es`; the server normalises wire values,
#609), so `from_tag` matches exactly.

The builder maps `meta.studied` the same way, refuses an unknown language (a pack no core could
load), and passes the language to `lemmatize` for grammar paradigms and expression keys. For en-fr
that is English, so the pack's bytes do not change. One builder test builds an `es` pack stamped
with English's version; it takes Spanish's version, which is the point of this change.

### D6 — Callers keep passing the language explicitly, for now

The engine functions keep their `studied` argument. The wasm engine and the agent plugin keep
passing English. `generalise-lingua-wasm-engine` replaces the wasm constant with `Pack::studied()`
and owns the exports' `"language"` literals: keeping that out of this change keeps one invariance
argument per pull request. Until then nothing checks that a caller's language matches its pack's;
every caller passes English with the en-fr pack.

### D7 — Three gates prove English did not move

1. The English invariance baseline passes **without re-blessing**: a re-bless in this pull request
   is a review failure.
2. `build.sh en-fr` reproduces the sha256 in `pin.json`.
3. The existing English tests pass unchanged, apart from passing `English` where `lemmatize` now
   asks for a language: lemmatisation fixtures, determinism, engine and pipeline tests.

The extension's version reader is anchored (`\bANALYZER_VERSION`), so it can never pick up
`SPANISH_ANALYZER_VERSION`. The reducer's pattern (`pub const ANALYZER_VERSION:`) already cannot.

### D8 — OpenSpec: ADDED only

Both deltas are ADDED requirements with names no open change holds. `add-lingua-phrase-gloss`
and `add-lingua-expression-table` add different requirements to `lingua-analysis` and
`lingua-data-packs`, so no `archiveAfter` is needed and no requirement is rewritten.

## Risks / Trade-offs

- [An English assumption outside the four seams applies to Spanish, e.g. the proper-noun
  heuristic or the percentage's counting rules] → They are language-neutral by intent, and no
  Spanish analysis reaches a reader before G1. `add-lingua-spanish-analysis` revisits each one
  against Spanish fixtures.
- [A state naming Spanish reaches a released build, which reports it as malformed] → Nothing in
  this change passes Spanish outside tests. Backup v2 belongs to the change that lets a reader
  choose Spanish.
- [`ALL`, `tag()` and `from_tag()` drift from the variants] → A test round-trips every variant,
  and the `match`es are exhaustive, so a third variant fails to compile until each seam answers.
- [The baseline is mistaken for Spanish support] → It carries a `0.x` version and the rules
  dropped (D3) are listed in the code where Spanish's arm stands.

## Migration Plan

Nothing to migrate. No stored format, wire field or pack byte changes. Rollback is a revert.

## Open Questions

None.
