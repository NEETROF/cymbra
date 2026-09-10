# Design — add-lingua-data-pack

## Context

The fourth storey of the Lingua stack. The core (`add-lingua-analysis`) exposes the
lemmatisation cascade, the ranks and the `analyzer_version`; its tests run on tiny
fixtures. This change brings the real data (EN→FR) and the format that carries it.
Inherited and not re-litigated here: the AGID + morphy + plural-fallback cascade
(`add-lingua-analysis`) and the L1/L2 `native_language` profile that picks the pair
(`add-lingua-knowledge-model`).

## Decisions

### D1 — A container format versioned per pair (L2→L1)
`pack.lingua` = magic + TOC: `meta` (pair, versions, licences), `forms.fst`,
`lemmas.bin`, `freq.bin` (quantised wordfreq ranks), `gloss.zst` (French glosses
extracted from kaikki, offset-indexed, top ~30k lemmas), `NOTICE`. The reader (in
`lingua-core`, reading from an `include_bytes!`-compatible slice) refuses a pack whose
declared `analyzer_version` is incompatible: analysis is deterministic for a given
(version, pack) — the contract `add-lingua-analysis` sets — and a pack from another
generation must never produce a partial analysis. MVP ships a single pack (EN→FR), but
**all the code is pair-keyed** — adding (ES→FR) is data, not code.

### D2 — A reproducible offline pipeline; the pack is never committed
Built offline by `scripts/lingua-data` (reproducible, dated sources, raw data not
committed); the pack itself is **not committed**: CI rebuilds it (determinism tested) and
caches it, and a local dev builds it once via the script.

### D3 — Licence hygiene: sellable or nothing
Sources kept: AGID (permissive licence, its notice stack shipped — including the upstream
WordNet stack), wordfreq frequencies (CC BY-SA), kaikki glosses (CC BY-SA). Nothing
GPL/AGPL/NC enters the build (denylist documented in `scripts/lingua-data`). The `NOTICE`
is embedded in the pack and the derived tables are published (share-alike satisfied); the
attributions page on the extension side arrives with `add-lingua-extension-reading`.

### D4 — Budget: 5 MB, arbitrated on the glosses
The embedded pack must stay under 5 MB: that is the downstream constraint of the surfaces
(~1 MB of wasm code + a ≤ 5 MB pack instantiated per tab in the extension —
`add-lingua-wasm` / `add-lingua-extension-reading`). Going over fails the build; the
adjustment variable is gloss coverage (top 20k vs 30k lemmas), never the FST or the
frequencies: a lemma without a gloss is still counted correctly, whereas a lemma missing
from the FST corrupts the count.

## Risks / Trade-offs

- [Data licences (CC BY-SA for wordfreq/kaikki, AGID notices)] → `NOTICE` embedded in the
  pack + derived tables published (share-alike satisfied) + a documented denylist.
- [Living sources (kaikki is re-extracted continuously)] → sources are dated and archived
  locally; the reproducibility test compares two builds over the **same source files**,
  not over a re-download.
- [Quality of the v1 lemmatiser (AGID + morphy, no POS)] → good enough for counting
  (ambiguity resolved in the learner's favour); the residual errors are what a v2 pack
  differentiates on, not a blocker.

## Open Questions

- The exact gloss threshold to embed (top 20k vs 30k lemmas) — measure the real
  `gloss.zst` size and arbitrate under a 5 MB total pack.
