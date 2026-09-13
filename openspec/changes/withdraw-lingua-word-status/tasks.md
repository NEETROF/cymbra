## 1. Knowledge model (lingua-core)

- [x] 1.1 Add `KnowledgeState::clear_status_at(lang, lemma, at_ms)`: clear the explicit status and stamp the tombstone.
- [x] 1.2 Make `resolve_lemma` return new (`None`) for a withdrawn lemma (tombstone, no explicit status) before the declared-level / calibration fallback. `band_stats` then counts it to-learn.
- [x] 1.3 Make `promote_by_exposure` promote exactly the lemmas that resolve to `Known(Calibration)`, so a withdrawal blocks re-promotion.
- [x] 1.4 Change `StatusRecord.status` to `Option<Status>` and have `export_statuses` emit withdrawals (`None`) in deterministic order alongside explicit statuses.
- [x] 1.5 Make `apply_status_lww` also report a change when the withdrawn flag flips (a first pulled `cleared`).
- [x] 1.6 Document the withdrawal meaning of the `updated` tombstone (field, `clear_status`, module precedence).

## 2. WASM surface (lingua-wasm)

- [x] 2.1 Route the `setStatusAt` clear path to `clear_status_at` with the caller's timestamp.
- [x] 2.2 Emit `"cleared"` (provenance `manual`) from `exportStatusOps` for a withdrawal.

## 3. Extension (lingua-extension)

- [x] 3.1 Update the `StatusOp` wire comments (`cleared` status, `exposure` provenance) and the marked-words / popup doc comments.
- [x] 3.2 Cover `markedWords` dropping a `cleared` op.

## 4. Verification

- [x] 4.1 Core tests:
  - a withdrawn exposure promotion resurfaces and is not re-promoted;
  - a withdrawn manual known below the level is unknown and counted to-learn;
  - a withdrawal exports as `cleared` at its own time and beats a stale `known`;
  - a pulled withdrawal stops the presumption until a newer decision;
  - an unstamped clear returns to calibration.
- [x] 4.2 Native wasm tests (`crates/lingua-wasm/tests/statuses.rs`) over the real `setStatusAt(…, "clear", …)` path: the word resurfaces, and the undo exports and converges on a second engine.
- [x] 4.3 Gates:
  - `cargo test -p lingua-core -p lingua-wasm`;
  - `cargo clippy -p lingua-core -p lingua-wasm --all-targets -- -D warnings`;
  - `cargo fmt --all --check`;
  - extension `yarn test`, `yarn typecheck`, `yarn lint`.
