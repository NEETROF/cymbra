## Context

- **Today.** Every surface — the content script, the popup, the side panel, the in-page drawer, the background's two engines — reads and writes `chrome.storage.local` directly through `AsyncStorageArea` (`get`/`set`), and follows the others through `chrome.storage.onChanged` on the backup key.
- **What that costs.** The area is capped (single-digit megabytes, browser-dependent) and the cap is enforced on **write**: a full store fails every mutation, which is how a sync, an erasure and a word-marking all broke at once.
- **What IndexedDB gives.** A quota proportional to the disk, negotiated with the browser, plus `navigator.storage.persist()` to ask that the data not be evicted under pressure.
- **The constraint that shapes everything.** A content script runs in the **visited page's** origin. `indexedDB` there is the *site's* database, not the extension's. The reading engine on Chromium lives in the content script, and the in-page drawer (Firefox, Safari) renders there too.

## Goals / Non-Goals

**Goals:**
- The reader's state lives where it can grow: deck, review history, statistics.
- Every surface sees the same state and follows its changes, as today.
- No data lost in the move, and a rollback that still finds the data.
- The engine, sync and review code do not change: they consume a seam.

**Non-Goals:**
- Moving the session tokens or the reader's toggles.
- Changing the backup schema, the sync protocol or the pack.
- Giving each surface its own database and reconciling them.

## Decisions

### D1 — The background owns the store; surfaces reach it by message

A content script cannot open the extension's IndexedDB, so either the state is duplicated per context — which is a synchronisation problem we would then own — or one context holds it. The background already hosts the sync engine and, on Safari and Firefox, the reading engine; it is the only context every surface can talk to. It owns the database.

**Alternative rejected:** extension pages (popup, side panel, account) *can* open the database directly, so they could bypass the messages. Two writers with different latencies to the same records is exactly the echo-suppression problem the current `storage.onChanged` code carries; one owner removes it.

### D2 — The seam stays `AsyncStorageArea`

The surfaces consume `get`/`set` today. They keep consuming it:

- in the background, an implementation over IndexedDB;
- everywhere else, an implementation that sends `store:get` / `store:set` and awaits the answer.

Nothing in the reading, review, stats or sync code changes. The messaged implementation is also what keeps the background alive while it works, the way the sync request already does (`fix-…-signouts` reasoning: a pending response is what a suspended page needs).

### D3 — Changes are pushed over a port, not observed

`storage.onChanged` fired because every surface wrote the same key. With one owner, the background announces: each surface opens a long-lived port (`chrome.runtime.connect`) and receives the keys that changed, filtered to the ones it asked for.

**Alternatives rejected:** `chrome.tabs.sendMessage` to every tab needs tab enumeration and a permission the extension does not have; polling wastes wakes on a browser that suspends the background.

### D4 — What moves, and what does not

**Moves** (the reader's data, which grows): the engine backup (`lingua`), the daily statistics, the sync cursors and the install's device id — the cursors travel with the data they describe, so a restored or migrated store stays consistent.

**Stays in `chrome.storage.local`**: the session tokens (their own reasoning, and they must be readable before the background answers anything), the highlighting and pill toggles, the last-sync time and the lost-session mark. They are a few hundred bytes, read by surfaces that must render before any round-trip, and several are written by the background itself.

### D5 — Migration copies, and keeps the copy for one release

On the background's first start after the update, each moved key found in `chrome.storage.local` is written into IndexedDB, then marked migrated. The originals are **left in place** for one release: the state is small now, and a rollback to the previous build must still find a deck. A later change deletes them.

Migrating is idempotent, and a partly-migrated store converges: the mark is written last, after every key.

### D6 — A store that will not open must not take the extension down

If IndexedDB cannot be opened, the background falls back to `chrome.storage.local` — the bounded state fits — and says so once in the log. The reader keeps reading, marking and reviewing.

`navigator.storage.persist()` is requested once. A refusal changes nothing beyond eviction risk under disk pressure, which for a signed-in reader is recoverable from the server.

## Risks / Trade-offs

- **[The store is the reader's deck]** → The migration copies rather than moves (D5), the fallback keeps the extension usable (D6), and the existing backup/restore file stays the reader's own escape hatch.
- **[A round-trip on every read and write]** → The state is small and the surfaces already message the background for the engine on Safari and Firefox. Reads happen at surface open and on change, not per keystroke.
- **[The background is suspended mid-write]** → An IndexedDB transaction either commits or does not; a lost write is re-done by the next save, exactly as with `storage.local` today.
- **[Two stores to reason about]** → D4 draws the line once: the reader's data on one side, session and toggles on the other.

## Migration Plan

1. Ship the store, the owner and the migration together; the first start after the update copies the keys.
2. Verify on device (deck, statistics and a sync after the move), then the release that follows deletes the `chrome.storage.local` copies.
3. **Rollback:** the previous build reads the copies it left behind.

## Open Questions

None.
