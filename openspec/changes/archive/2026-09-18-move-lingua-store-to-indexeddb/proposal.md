## Why

The extension keeps the reader's whole Lingua state in `chrome.storage.local`. That area is a settings store: browsers cap it in the single-digit megabytes, and when the cap is reached **every write fails** — sync, erasure, even marking a word (dogfooding, TestFlight 70/71; `bound-lingua-reading-counters` removed what had filled it).

Bounding the state fixed the incident, and the state is now small. But the home is still wrong: a deck, its review history and daily statistics are data that grows with the reader, and they live in an area sized for preferences. IndexedDB is the browser's store for exactly that — quota measured against the disk rather than a fixed few megabytes, and a persistence request the browser can honour.

## What Changes

- **The reader's state moves to IndexedDB**: the engine backup, the daily statistics and the sync cursors (with the device id). One database, owned by the extension's background.
- **One owner, one seam.** A content script cannot reach the extension's IndexedDB — it runs in the visited page's origin — so the background owns the store and the surfaces reach it through the storage seam they already consume (`AsyncStorageArea`), whose calls become messages. No surface changes shape.
- **Changes are announced, not observed.** Surfaces follow the store through a subscription the background pushes to, replacing the `storage.onChanged` events that only existed because every surface wrote the same key.
- **What stays in `chrome.storage.local`**: the session tokens, the reader's toggles (highlighting, the in-page pill), the last-sync time and the lost-session mark. They are tiny, several of them are read before the background answers anything, and the session's storage has its own reasoning (`fix-interrupted-refresh-signouts`).
- **Migration on first run**: the moved keys are copied into IndexedDB; the copies in `chrome.storage.local` are left in place for one release, so a rollback finds them.

## Capabilities

### Modified Capabilities
- `lingua-browser-extension`: where the versioned local state lives, who owns it, and how the surfaces reach and follow it.

## Impact

- `apps/lingua-extension`: a new store module (the IndexedDB implementation and the messaged one), the background as owner and broadcaster, the migration, and the surfaces' change subscription. The engine, the sync protocol and the review code are untouched: they consume the seam.
- No wire, pack or backup-format change; the same versioned schema is stored, in a different place.
- **Risk to hold**: the store is the reader's deck. The migration copies rather than moves, and a store that cannot be opened falls back to `chrome.storage.local` so the extension keeps working.
- **Out of scope**: moving the tokens; the agent plugin's own local store (`~/.lingua/`), which is a different runtime.
