## Why

"Remettre à apprendre" (put a word back to learn) is the only way to undo a "known" or
"ignored" decision. It is offered in the extension's marked-words list and in the word
popup for an ignored word. It does not do what it says. It removes the status, so the
word returns to "new". But a new word below the declared CEFR level (or within the
frequency calibration) is presumed known again, so the word stays unhighlighted. A word
confirmed by reading is worse: it already meets the distinct-day threshold, so the next
page read promotes it straight back. The undo also never leaves the device, because only
present statuses are pushed. Other devices keep the old `known`, and a re-pull after a
local wipe restores it.

## What Changes

- **Withdrawal.** Undoing a status becomes a *withdrawn decision*, recorded with the
  time it was made. A lemma with a withdrawal and no explicit status resolves as new: it
  is highlighted, and neither the declared level nor the calibration presumes it known.
- **Promotion.** Exposure promotion treats a withdrawal like any other user
  interaction: it permanently blocks re-promotion.
- **Sync.** A withdrawal is exported as a `cleared` status op stamped with its own time,
  and applying a pulled `cleared` records a withdrawal under last-write-wins. The undo
  now reaches every device.
- **Unchanged.** A status that was never stamped with a sync time (internal / pre-sync
  data), once cleared, still returns to plain "new" under calibration.
- **No wire change.** `cleared` is already a valid `KnownWordsService` status value, so
  the proto and the backend are unchanged.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-knowledge-model`:
  - A withdrawal is added to the status model and is never presumed known.
  - Exposure promotion is also blocked by a withdrawal.
  - Withdrawals are exported and applied as `cleared` status ops.

## Impact

- **Products**
  - **Lingua**: the only product affected. It *consumes* the existing
    `KnownWordsService` sync and wire vocabulary unchanged. It gains the withdrawal
    semantics in `lingua-core`, and its surfaces (extension; the Apple app once it
    ships) inherit them through the engine.
  - **ID / Music / Live / back office / site**: not affected.
- **Code**
  - `crates/lingua-core` (`knowledge/state.rs`): `clear_status_at`, withdrawal-aware
    resolution, promotion guard, export of withdrawals, LWW change reporting.
  - `crates/lingua-wasm` (`setStatusAt` clear path, `exportStatusOps`).
  - `apps/lingua-extension`: wire types and doc comments only.
- **Backend / proto / DB**: none. `lingua.word_statuses.status` is free text and
  `cleared` is already documented in `known_words.proto`.
- **Agent plugin**: none. It never syncs and never withdraws.
- **Data**
  - Words a reader already put back before this change carry a tombstone, so they
    resurface too. That is the effect they asked for at the time.
  - Implementation: NEETROF/cymbra#430.
