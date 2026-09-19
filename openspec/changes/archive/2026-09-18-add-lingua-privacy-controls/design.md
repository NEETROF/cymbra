## Context

- **Sync.** The Lingua sync protocol (`add-lingua-backend`, in flight) sends three families of data: lemma statuses, declared levels and cards, and per-(day, language, device) stat aggregates.
  - Every op is last-write-wins on a client timestamp, which the server clamps at receipt.
  - The extension's `SyncEngine.sync()` **pushes, then pulls** (`apps/lingua-extension/src/sync/sync.ts`).
  - The Safari, Chromium and Firefox variants share that engine.
- **Card source.**
  - `crates/lingua-wasm` `exportCardOps` puts `EncounterSource::Web { url }` into `CardOp.source`, and `applyCardOps` writes the pulled `source` back into the card.
  - The server stores it in `lingua.cards.source` (`backend/lingua/migrations/0001_lingua.sql`) and returns it on pull.
  - No screen displays it: it only survives in the local JSON export.
- **Deletion.**
  - `DeleteAccount` (`account-management`) runs the `purge_user` worker job, which already deletes the four `lingua.*` tables.
  - The Cymbra account is shared: deleting it from Lingua also deletes the reader's Music account and data.
  - The site's `/suppression-compte` (`/en/delete-account/`) runs the web deletion with re-authentication, and describes Music-only partial deletions.
- **Constraints.**
  - Apple 5.1.1(v) requires account deletion reachable in the app. The privacy label must match what is actually collected.
  - Installed extensions (Chrome, Firefox, Safari TestFlight) keep sending `CardOp.source`, and never call a service that doesn't exist yet.

## Goals / Non-Goals

**Goals:**
- No page address on the server, old or new, and nothing on the wire from current clients.
- A reader can erase **only** their Lingua data, on every signed-in device, without a later sync resurrecting it.
- From Lingua, a reader can reach deletion of the whole Cymbra account, warned that Music goes too.
- The policy, the deletion page and the App Store label describe Lingua accurately.

**Non-Goals:**
- A Music-only erasure, and Music's deletion copy: a separate `music-` change.
- Erasing the Claude Code plugin's local store (`~/.lingua/`): it never syncs.
- Deleting the Cymbra account from inside the extension: it links to the site instead (D5).
- Selective erasure (per word, per card): statuses and cards can already be changed one by one.

## Decisions

### D1 — The page address stays in the local card, never on the wire
- **Export.** `exportCardOps` emits an empty `source` for every card; the local `EncounterSource::Web { url }` is untouched, so the JSON export keeps it.
- **Apply.** `applyCardOps` keeps the local source when the pulled op's `source` is empty. A card edited on another device therefore doesn't lose its address on the device that captured it.
- **Server.**
  - Migration `0003` drops `lingua.cards.source`, which erases the stored addresses in the same step.
  - The deck repository neither writes nor reads it; the gRPC adapter ignores the incoming field and returns it empty.
- **Proto.** `CardOp.source` is kept with `[deprecated = true]`.

**Alternatives.**
- **Keep only the host:** Apple still counts a visited site as browsing history.
- **Remove the field:** `buf breaking` (FILE) refuses a deleted field. The break would buy nothing, since installed clients still send it.
- **Keep and declare it:** it would label the app « Historique de navigation » for data no screen uses.

### D2 — An erasure mark, not just a delete
Deleting the rows is not enough.
- **The problem.** Another signed-in device still holds its local store, and its next sync pushes before it pulls. The deck and statuses would come straight back.
- **The mark.**
  - `EraseMyData` deletes the caller's rows in `lingua.word_statuses`, `lingua.declared_levels`, `lingua.cards` and `lingua.daily_stats`.
  - In the same transaction, it upserts `lingua.data_erasures(user_id, erased_at)`, where `erased_at` is the server time in epoch milliseconds.
  - It runs on the `lingua_svc` pool, synchronously: four indexed deletes for one user. A job would add latency and a "still erasing" state for nothing, and `purge_user` belongs to account deletion.
- **`GetDataState`** returns `erased_at` (0 when never erased).
- **Push filtering (defense in depth, also covers installed extensions that never read the mark).**
  - Status, level and card ops whose clamped `client_ts` ≤ `erased_at` are dropped.
  - Daily stats for a day before the erasure's UTC day are dropped.
  - Ops are still acknowledged, so an old client's outbox drains instead of retrying forever.

**Alternatives.**
- **Client-only wipe:** the other devices re-upload.
- **A per-user version counter:** equivalent, but ops already carry timestamps, and filtering by time needs no client change for old clients.
- **Extending `purge_user`:** it is keyed on account deletion and runs asynchronously in the worker.

### D3 — Every sync reads the mark first; the erasing device wipes itself
- **Every sync.** `SyncEngine.sync()` starts with `GetDataState`.
  - If `erased_at` is newer than the device's stored mark (`cymbra-lingua-erased-at`, `chrome.storage.local`), the device runs the **local wipe**, stores the mark, then continues the sync.
  - After a wipe the push is empty and the pull fills in anything created elsewhere since.
- **The local wipe** reuses the full reset: engine `reset()` (statuses, deck, levels, calibration), exposure counters, local stat aggregates and `clearSyncCursors`. It is persisted through the backup key, which open tabs already follow via `storage.onChanged`.
- **The erasing device.**
  1. The background serializes the request (`account:eraseLinguaData`) with the sync loop, so no push slips in between.
  2. It calls `EraseMyData`, wipes locally and stores the returned mark.
  3. A failed call wipes nothing and reports a category, as the other account errors do.
- **Timestamp floor.** An op pushed after a wipe carries `max(client_ts, erased_at + 1)`, so a device clock running behind cannot get a fresh decision dropped by D2.

### D4 — Where the controls live
- **Account page.** The signed-in view (`account.html`) gains a « Tes données » section:
  - « Effacer mes données Lingua », which asks for confirmation. The dialog says it applies to every device, is irreversible, and keeps the Cymbra account and Music.
  - « Supprimer mon compte Cymbra », a link (D5).
- **Popup.** It stays compact: under the account row, a « Gérer mes données » link opens that section (`account.html#data`), the way sign-up opens the page today.
- **Signed out.** The existing settings reset already covers this device, so there is nothing to erase server-side.

### D5 — Account deletion is a link to the site
The site already implements deletion with re-authentication, including Google and Apple through the web flow; Safari's extension has no provider flow of its own.
- **Link.** The extension opens `https://cymbra.app/suppression-compte/`, or `/en/delete-account/` when the browser language isn't French, in a tab.
- **Warning.** Above the link: « Supprime ton compte Cymbra pour toutes les apps Cymbra, Music compris. Pour ne retirer que Lingua, utilise « Effacer mes données Lingua ». »
- **Apple 5.1.1(v)** accepts a direct link to a web deletion flow.

### D6 — Disclosures
- **Privacy policy.** `confidentialite.md` / `en/privacy.md` gain « Annexe B — Cymbra Lingua »:
  - **Local-only:** page text, reading exposures, and the page address of a card; the analysis runs on the device.
  - **Synced when signed in:** statuses, level, deck (word, sentence, translation, review state), daily stats, and a random installation identifier.
  - **Erasure:** « Effacer mes données Lingua » and account deletion.
  - **Retention and legal basis:** same as the account.
  - The update date moves.
- **Deletion page.** Its partial-deletion section names Lingua's erasure, and the page says the account serves every Cymbra app.
- **App Store label.** The answers are recorded in `apps/lingua-apple/README.md`, as the reference for the App Store Connect form:
  - e-mail address, user ID and device ID; other user content; product interaction;
  - all linked to the user, for app functionality, plus analytics for the user ID and product interaction (aggregated back-office figures);
  - no browsing history, no tracking.

### D7 — `add-lingua-backend` is amended in place
`lingua-sync` is not in `openspec/specs/` yet: it lives in the in-flight `add-lingua-backend`.
- **What changes there.** Its « Complete card synchronisation » and « Strict allow-list » requirements are edited in that change, so the card no longer carries its source.
- **Why not a delta here.** A `MODIFIED` delta cannot target a capability that isn't archived. Leaving the old text would archive a contradiction.

## Risks / Trade-offs

- **[Migration `0003` irreversibly drops the stored addresses]** → Intended; the pre-deploy database backup is the only copy. The column is not recreated on rollback.
- **[An old extension keeps showing erased data on its own device]** → It cannot re-upload old ops (D2). The fresh decisions it makes still sync, which is correct. The erasure dialog says other devices clear themselves at their next sync once updated.
- **[Device clocks disagree with the server's]**
  - A clock ahead is clamped to server receipt, so an old op pushed after the erasure is still dated at most at its receipt time. It is only kept if it arrives after the mark, which means it was pushed by a client that never read the mark (an installed old extension); the previous item covers that case.
  - A clock behind is handled by D3's floor.
- **[An erasure-day stats row from an old client survives]** → Bounded to one day of counts; the reader's next erasure removes it. Stricter filtering would need a timestamp that `DailyStat` doesn't carry.
- **[A card's page address no longer survives a reinstall or a new device]** → No screen shows it today. The local export still contains it, for a reader who wants to keep it.

## Migration Plan

1. **Backend first:** migration `0003` + `LinguaDataService` + push filtering. Installed extensions keep working, since `source` is ignored and they never call the new service.
2. **Extension builds** (Chromium, Firefox, Safari with a new TestFlight build): empty source, erasure check at sync start, the « Tes données » section.
3. **Site:** the policy annex and deletion page copy. Then the App Store Connect privacy form is filled from the README table.
4. **Rollback:** the new service and the filters can be reverted; the dropped column cannot, which is acceptable.

## Open Questions

None blocking. The Music-side copy and a possible Music-only erasure are tracked as a separate change.
