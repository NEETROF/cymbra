# Design — add-lingua-analysis

## Context

A full product exploration was run up front (2026-08-10): competitive study
(LingQ/Readlang/Migaku/Lute/jpdb), per-surface feasibility, and a **working preshot**
(`~/workspace/lingua-preshot`, a plain-JS MV3 extension) that validated the product
loop on real pages. This change industrialises the heart of that preshot: the analysis
pipeline moves to Rust, into a monorepo crate, with determinism as a contract.

Constraints inherited from the monorepo: host-testable cores (`*_core.rs`), coverage
≥ 80%, no business logic in the shells. The product constraint that is settled: honest
per-lemma counting.

## Goals / Non-Goals

**Goals:**
- A complete, deterministic analysis pipeline in `crates/lingua-core`: text → tokens →
  lemmas → classification → % known, tested on mini-fixtures.
- The monorepo foundations of the Lingua product: OpenSpec prefix, crate in the
  workspace, CI/coverage lane.

**Non-Goals:**
- Statuses, calibration, the L1/L2 profile (`add-lingua-knowledge-model`); decks/FSRS
  (`add-lingua-decks-review`); the real pack (`add-lingua-data-pack`); the WASM target
  (`add-lingua-wasm`); every user-facing surface (extension, app, plugin — later
  changes in the stack).

## Decisions

### D1 — Tree: `crates/lingua-core` in the root Cargo workspace
The crate joins the root Cargo workspace and is therefore covered by the existing
llvm-cov lane (fmt/clippy/coverage ≥ 80% with no new lane). The module layout is laid
down now — `analysis/`, `knowledge/`, `decks/`, `packs/` — so the following changes
fill in slots that already exist instead of reorganising. The logic stays pure and
host-tested; only the future bindings (wasm-bindgen) will be excluded from coverage,
an exclusion added in this change to `--ignore-filename-regex` (the CI lane **and**
the command documented in CLAUDE.md). No Flutter, no Tauri.

### D2 — Tokenisation: UAX #29 + a pre-pass per studied language
`unicode-segmentation` tokenisation (UAX #29), preceded by a **per-language** pre-pass
that absorbs surface quirks — for English: contractions (`don't` → `do` + `not`), edge
apostrophes stripped, single-letter words counted only if they belong to the lexicon
("I", "a"). The pre-pass is the extension point for the Romance languages
(elisions/clitics): adding a language means adding a pre-pass, not touching the
tokeniser.

### D3 — English lemmatisation: AGID FST + morphy as fallback + out-of-lexicon plural fallback
The pipeline: exceptions/irregulars → FST lookup (AGID inverted forms→lemmas, ~0.5 MB)
→ morphy (WordNet rules, the `wordnet-lemmatizer` crate or an internal port) → simple
plural fallback for words outside the lexicon (the preshot's lesson:
`endeavors`→`endeavor`, otherwise the count lies). Rejected alternatives:
`rust-stemmers` (stems, not lemmas — never showable), `nlprule` (heavy LGPL binaries),
spacy-lookups-data EN (coverage below AGID). In this change the FST is read from a
slice (`include_bytes!`-compatible) over test mini-FSTs; the container format and the
real pack arrive with `add-lingua-data-pack`.

### D4 — Per-block language detection: `whichlang`
Detection runs **per block of text**, not per document: real pages mix languages
(French UI, quotations, code). A block outside the studied language is excluded from
the analysis — including blocks in the native language; a document without enough
content in the studied language is "not analysable". `whichlang`: pure Rust, fast, no
external data — compatible with the future WASM target.

### D5 — Determinism as a contract: `analyzer_version`
At equal `analyzer_version` and equal pack, the output is byte-for-byte identical
whatever the compilation target. That is a contract, not a wish: the future community
TextProfile, the sync and the merging of stores depend on it (an analysis that cannot
be reproduced makes counts incomparable). Tested from this change on by a determinism
test over a fixture corpus (double native run); the **native/WASM cross-parity** test
arrives with `add-lingua-wasm` (same `analyzer_version` ⇒ identical outputs).

## Risks / Trade-offs

- [v1 lemmatiser quality (AGID+morphy, no POS)] → good enough for counting (ambiguity
  is resolved on the knowledge-model side, in the learner's favour); the residual
  errors are what the v2 pack differentiates on, not an MVP blocker. Non-regression
  fixtures from day 1 (min. 100 cases).
- [Tests on mini-fixtures only] → accepted: the quality *of the pipeline* (cascade,
  determinism) is testable on synthetic fixtures; the quality *of the data* will be
  validated with the real pack (`add-lingua-data-pack`), which reuses the same
  non-regression fixtures.
- [Module layout laid down before its changes] → near-zero cost (directories + empty
  or minimal `mod`s) against a real benefit: the 11 following changes move no code.

## Migration Plan

Nothing to migrate (new crate, no surface). Rollback = remove the crate from the
workspace.

## Open Questions

- `wordnet-lemmatizer` (an existing crate) vs an internal port of the morphy rules —
  settle at implementation time depending on the state of the crate (Apache-2.0
  licence confirmed, maintenance to be checked); the cascade's contract does not move.
