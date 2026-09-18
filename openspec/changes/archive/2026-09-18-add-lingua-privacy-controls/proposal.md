## Why

Cymbra Lingua is ready for the App Store, but its App Review pass (`add-lingua-connected-clients` 4.7) turned up three gaps:

1. **Page addresses on the server.** A deck card carries the address of the page where the word was captured (`location.href`), and sync sends it to the server. Apple counts that as *Browsing History*. No screen uses it: it only appears in the local export.
2. **No way to delete from inside the app.** Lingua lets a reader create a Cymbra account, but offers no way to delete it, which Apple 5.1.1(v) requires. The account is also Music's, so the only deletion we have — the whole Cymbra account — takes Music with it. A reader who only wants their Lingua data gone has no option.
3. **Privacy policy silent on Lingua.** The published policy has an annex for Music and says nothing about Lingua.

## What Changes

- **A card's page address stays on the device.**
  - The extension sends an empty source when it pushes a card.
  - A pulled card no longer blanks a local source.
  - The server stops storing the field: the `lingua.cards.source` column is dropped, which erases the addresses already stored.
  - `CardOp.source` is kept in the `.proto` but deprecated and ignored, so the change is not breaking for installed extensions.
  - The `add-lingua-backend` allow-list requirement that let the card source go up is amended to match.
- **« Effacer mes données Lingua ».** A signed-in reader can erase their Lingua data (word statuses, declared levels, deck, daily stats) on every device, without touching the Cymbra account.
  - A new `LinguaDataService` erases the caller's `lingua.*` rows and records an **erasure mark**.
  - Every sync reads the mark **before pushing**. A device that has not yet seen it wipes its local store and cursors first, so it cannot re-upload what was erased.
  - The server also drops any op dated before the mark, which covers extensions installed before this change.
- **« Supprimer mon compte Cymbra ».** The extension links to `cymbra.app/suppression-compte` (`/en/delete-account/`), warning that the Cymbra account and every Cymbra app's data go with it, Music included.
- **Privacy disclosures.**
  - The Cymbra privacy policy gains « Annexe B — Cymbra Lingua » (fr + en): what is synced, what never leaves the device, the erasure. The deletion page names Lingua and the Lingua-only erasure.
  - The App Store privacy label answers are recorded with the Apple app: e-mail, user ID, device ID, other user content, product interaction; **no** browsing history, no tracking.

## Capabilities

### New Capabilities
- `lingua-privacy`: what Lingua keeps on the device versus sends to the server, the Lingua-only erasure across devices, the path to Cymbra account deletion from Lingua, and Lingua's privacy disclosures (policy annex, deletion page, App Store label).

### Modified Capabilities
_None in `openspec/specs/`._ The sync protocol capability (`lingua-sync`) exists only in the in-flight `add-lingua-backend` change. Its allow-list and card-sync requirements are edited in place, in that change, so both changes archive consistently.

## Impact

- **Products**
  - **Lingua (new):** extension account screen (erase + delete link), sync (erasure check first, empty card source), `lingua-wasm` card export/apply, `backend/lingua` (new service, column dropped, op filtering), Apple host app (privacy label record).
  - **Cymbra ID (consumed as-is):** `DeleteAccount` and the `purge_user` job, which already erase `lingua.*`. Identity comes from the bearer token.
  - **Site (new copy):** privacy policy annex B (fr/en) and the deletion page (fr/en). No new page, no new island.
  - **Music, Live, back office:** untouched. The Lingua back-office usage figures read `lingua.daily_stats` and simply see fewer rows after an erasure.
- **Tree**
  - `backend/lingua`: proto `lingua_data.proto`, migration `0003`, `data.rs` / `pg_data.rs` / `data_grpc.rs`, push filtering in deck, known words and stats.
  - `backend/server`: registers the service.
  - `crates/lingua-wasm`: card export/apply.
  - `apps/lingua-extension`: `sync/`, `account/`, popup.
  - `apps/site/src/pages`: privacy and deletion pages.
  - `apps/lingua-apple/README.md`.
- **API**
  - New `LinguaDataService` (`EraseMyData`, `GetDataState`), additive.
  - `CardOp.source` deprecated: no field removed, so no `buf breaking` failure.
- **Data:** migration `0003` drops `lingua.cards.source`, irreversibly deleting the page addresses stored so far, which is intended. It also adds `lingua.data_erasures`.
- **Out of scope**
  - **Music follow-up (separate `music-` change):** Music's deletion dialog should also say the account serves every Cymbra app, including Lingua. A « Effacer mes données Music » is a larger question (scores, plays, contributions, subscriptions).
  - The in-flight Chromium/Firefox store listings, whose privacy forms follow the same answers.
