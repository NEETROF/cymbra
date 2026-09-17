# Tasks — move-lingua-store-to-indexeddb

## 1. The store

- [ ] 1.1 `state/idb-store.ts`: an `AsyncStorageArea` over IndexedDB (one database, one object store, keys as today), opened once and reused; open failure surfaces to the caller
- [ ] 1.2 `state/messaged-store.ts`: an `AsyncStorageArea` that sends `store:get` / `store:set` and awaits the answer (the pending response keeps a suspended background alive)
- [ ] 1.3 Background: owns the IndexedDB area, answers the two messages, and falls back to `chrome.storage.local` when the database will not open (logged once)
- [ ] 1.4 `navigator.storage.persist()` requested once, its refusal ignored

## 2. Following changes

- [ ] 2.1 Background: a long-lived port per surface; every `set` announces the changed keys to the others
- [ ] 2.2 Surfaces: subscribe to the keys they care about and drop their `storage.onChanged` listeners on the moved keys (the popup's toggles keep theirs)

## 3. Migration

- [ ] 3.1 On the background's first start after the update: copy the moved keys into the store, mark migrated last (idempotent, converges after a partial run), leave the `chrome.storage.local` copies in place
- [ ] 3.2 vitest: a fresh install, an install with state to move, a partly-migrated store, and a second start doing nothing

## 4. Gates

- [ ] 4.1 vitest over the store, the messaged area, the migration and the subscription (fake IndexedDB + fake messaging)
- [ ] 4.2 `typecheck`, `lint`, `format:check`, `build`, `check:variants`
- [ ] 4.3 `openspec validate move-lingua-store-to-indexeddb --strict`
- [ ] 4.4 On device (TestFlight): deck and statistics intact after the update, a sync, an erasure, and the panel following a change made in the page
