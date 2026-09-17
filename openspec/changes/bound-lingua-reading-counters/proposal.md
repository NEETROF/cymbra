## Why

Dogfooding TestFlight 70/71 (Safari, macOS): the extension could no longer write anything — `Invalid call to browser.storage.local.set(). Exceeded storage quota.` Every write failed: the sync, and the local wipe that follows a « Effacer mes données Lingua » (the server erased, the device could not, and the reader was told the erasure had failed).

The store was full of reading counters. The extension keeps one per **distinct lemma encountered**, ~200 bytes each, for ever:

| local store | size |
|---|---|
| 31 marked words + 27 cards | 20 KB |
| + 5,000 distinct words read | 1 MB |
| + 20,000 | 4 MB |
| + 40,000 | 8 MB |

Browsers cap an extension's `storage.local` in the single-digit megabytes, so weeks of reading are enough to wedge the extension.

The counters exist for exactly one purpose: confirming a word the declared level already presumes known, after reading it on several distinct days. So **most of what was stored could never be used** — and with no declared level (« Débutant »), *none* of it could: promotion is a no-op without a level.

## What Changes

- **Count only what reading could confirm.** The extension records an exposure for a lemma only while it is promotable: a level is declared and the lemma resolves as presumed known (below that level, with no explicit or withdrawn status). Nothing is recorded otherwise.
- **Bound what is kept.** Counters not read for 90 days are dropped, and the kept set is capped at the 5,000 most recently read lemmas.
- **Heal a saturated device.** Restoring a backup prunes it, so a store filled by an older build shrinks on the first load after the update instead of failing to be written.
- **Name the failure.** A write the browser refuses is categorized as `storageFull` and the reader is told the extension's memory is full and what to do, instead of "sync failed" or "the erasure did not go through".

The agent plugin's ingestion path is untouched: it records into the same core counters through its own call, which still records what it is given.

## Capabilities

### Modified Capabilities
- `lingua-knowledge-model`: the exposure-counter requirement gains what a client may record and must bound.

## Impact

- `crates/lingua-core`: `ExposureCounters::retain` / `cap_by_recency`, `KnowledgeState::promotable_by_exposure` (the promotion rule, minus the day count, now named and reused).
- `crates/lingua-wasm`: filtering at record time, pruning on record and on restore.
- `apps/lingua-extension`: the `storageFull` category and its copy.
- No wire, pack or backup-format change: an old backup restores, and is pruned.
- **Out of scope:** moving the store to IndexedDB, which is a separate change; bounding it is what keeps the state small either way.
