# Design — add-lingua-decks-review

## Context

The third storey of the Lingua stack: the core can analyse (`add-lingua-analysis`) and
knows the learner's state (`add-lingua-knowledge-model`); this change adds decks, cards
and review scheduling on top. The knowledge model's decisions — the `(studied language,
lemma)` key, multi-word expressions as lemmas with spaces, the statuses and the
`known_source` field (`manual | calibration | srs | import`) — are inherited from
`add-lingua-knowledge-model` and are not reopened here. Like the rest of the core: pure
host-tested logic (`*_core.rs`), coverage ≥ 80%, no UI in this change.

## Decisions

### D1 — Card schema: provenance mandatory, media in the schema from day 1
A card carries the lemma, the encountered form, the originating context sentence, the
source of the encounter (URL or agent session identifier, timestamp) and the gloss. The
winning sentence-mining card is word + form + sentence; the originating sentence can only
be captured at the moment of the encounter — so provenance is non-negotiable at creation
time and can never be reconstructed after the fact. The media slot (`media`, with
`source: capture|stock|generated` and `sync_policy`) is present in the schema **without
being populated**: image capture is deferred, but the versioned serialised schema is a
contract that the export (D4) and the future sync will freeze — adding it later would be
a migration, adding it now is an empty column. Multi-word expressions are cards in their
own right.

### D2 — Review: FSRS-5 scheduling reimplemented in the core (not the `fsrs` crate)
The FSRS state (stability, difficulty, due date) lives on the card, computed in
`lingua-core` — the same scheduling everywhere (WASM extension, Apple app, native plugin,
in the following changes). Grading is `again/hard/good/easy`; the due-card count is
computable at any moment by the core.

**Deviation from the original wording ("via the `fsrs` crate"), decided at implementation
time.** The published `fsrs` crate (v6) is the FSRS *optimiser*: it pulls ~63 transitive
dependencies including `ndarray`, `rand` and `getrandom` — precisely the ML/`getrandom`
weight the stack's north star rejects (change 1 chose pure, data-free, WASM-clean deps).
Pulling it into the core that must compile small to WASM would break that invariant. So
the **scheduling** algorithm (FSRS-5 forgetting curve, stability/difficulty updates, next
interval at a target retention) is reimplemented directly in the core with zero new
dependencies and the published FSRS-5 default weights. The full 19-weight parameter vector
plus request-retention and max-interval are **stored on the state**, so the *optimiser* —
the `fsrs` crate, run server-side, outside the WASM core — can refit them per user later
with no schema change. Scope: scheduling only; parameter optimisation is explicitly out of
this crate.

The review's UI surfaces (side panel, drawer, icon popup) belong to
`add-lingua-extension-review` — the icon badge stays dedicated to the page percentage, the
due count lives in the popup and the side panel (no "due" badge and no alarm in v1).

### D3 — "I know this" closes the loop with the knowledge model
Marking "I know this" during review moves the lemma to `known` with provenance `srs` —
`add-lingua-knowledge-model`'s `known_source` field was preparing exactly this
Migaku-style inference — and removes the card from the queue **without deleting it or
erasing its history**: the FSRS state remains, and the word can return to learning later
with nothing lost.

### D4 — Backup/restore first, Anki export deferred
Throughout the stack's local phase (before `add-lingua-backend`/
`add-lingua-connected-clients`), the state lives in a browser profile's storage —
fragile (a profile reset loses everything). The MVP's safety net is therefore a **full
backup/restore** (a versioned file: cards, statuses, calibration, FSRS parameters; a
lossless round trip, tested), which doubles as pre-sync device migration. **Anki-format
export is deferred** (a user decision): what is expensive to catch up on is not the
serialiser but the schema (the Lute lesson) — the schema stays exportable by construction
(clean fields, provenance, gloss kept separate), and the Anki CSV will come as a dedicated
change when users ask for it.

## Risks / Trade-offs

- [The `fsrs` crate evolves (default parameters)] → pin the version; FSRS states store
  their parameters.
- [The "right next to the reading" requirement is specified before its surfaces] →
  accepted: it is the capability's contract; `add-lingua-extension-review` realises it,
  and this change delivers everything it needs (dues, state transitions, a review session
  on the core side).
- [The `media` field is inert in this change] → near-zero cost (an empty column in the
  export); the gain: neither a schema migration nor an export break when image capture
  arrives.
