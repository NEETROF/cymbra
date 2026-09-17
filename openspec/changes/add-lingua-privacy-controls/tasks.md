## 1. Backend — card sources and the erasure mark

- [ ] 1.1 Migration `0003`: drop `lingua.cards.source`; create `lingua.data_erasures(user_id TEXT PRIMARY KEY, erased_at BIGINT NOT NULL)` owned by `lingua_svc`
- [ ] 1.2 Deck: the repository neither writes nor reads `source`; the gRPC adapter ignores the incoming field and returns it empty; `CardOp.source` marked `[deprecated = true]` in `deck.proto` (no field removed, `buf breaking` clean); tests: a pushed address is not stored or returned
- [ ] 1.3 `lingua_data.proto` + `LinguaDataService`:
  - `EraseMyData` deletes the caller's four tables and upserts the mark in one transaction, and returns `erased_at`;
  - `GetDataState` returns `erased_at` (0 when never erased);
  - the caller comes from the token only;
  - mockall-backed module tests: erasure, idempotent replay, other users untouched, mark returned.
- [ ] 1.4 Push filtering against the mark:
  - `PushOps` (statuses and levels) and `PushCards` drop ops with clamped `client_ts` ≤ `erased_at`;
  - `UpsertDailyStats` drops days before the mark's UTC day;
  - all still acknowledge the push;
  - tests for each path, and for an op after the mark being kept.
- [ ] 1.5 Register `LinguaDataService` in `backend/server` behind the same `CYMBRA_LINGUA_DATABASE_URL` switch and gRPC-web routing as the other Lingua services; `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, coverage ≥ 80%

## 2. Engine — card export and apply

- [ ] 2.1 `crates/lingua-wasm` `exportCardOps` emits an empty `source`, and `applyCardOps` keeps the local `EncounterSource::Web` address when the pulled `source` is empty; Rust tests for both, and for the local export still carrying the address

## 3. Extension — sync, erasure and account section

- [ ] 3.1 gRPC-web stubs for `lingua_data.proto` (`gen:proto`) and a `data` client beside `deck`/`knownWords`/`stats`
- [ ] 3.2 `SyncEngine.sync()` starts with `GetDataState`:
  - when the mark is newer than `cymbra-lingua-erased-at`, it runs the local wipe (engine reset, exposure counters, local stats, `clearSyncCursors`), stores the mark, then pushes and pulls;
  - pushed ops are floored at `erased_at + 1`;
  - vitest: wipe before push, no wipe when the mark is known, the floor.
- [ ] 3.3 `account:eraseLinguaData` in the account host:
  - serialized with the sync loop;
  - calls `EraseMyData`, runs the local wipe and stores the mark;
  - a failure wipes nothing and replies with a category;
  - vitest for success, failure and serialization.
- [ ] 3.4 Account page signed-in view: a « Tes données » section (`#data`).
  - « Effacer mes données Lingua » with a confirmation stating it is irreversible, applies to every device and keeps the Cymbra account and Music.
  - « Supprimer mon compte Cymbra », preceded by the cross-app warning, opening `https://cymbra.app/suppression-compte/` (fr) or `/en/delete-account/` (other languages) in a tab.
  - French copy, Cymbra tokens; flow tests.
- [ ] 3.5 Popup: a « Gérer mes données » link under the signed-in account row, opening `account.html#data`
- [ ] 3.6 `yarn lint`, `yarn typecheck`, `yarn format:check`, `yarn test`, `yarn build`, `yarn check:variants`

## 4. Site — disclosures

- [ ] 4.1 « Annexe B — Cymbra Lingua » in `apps/site/src/pages/confidentialite.md` and `en/privacy.md`, with the update date moved:
  - stays on the device: page text and analysis, exposures, a card's page address;
  - synced: statuses, level, deck without addresses, daily stats, installation identifier;
  - removal: Lingua erasure, account deletion;
  - retention and legal basis: as the account.
- [ ] 4.2 `suppression-compte.astro` and `en/delete-account.astro`: state that the account serves every Cymbra app (Music, Lingua), and list « Effacer mes données Lingua » among the partial deletions; site checks green

## 5. Apple app — privacy label and review

- [ ] 5.1 `apps/lingua-apple/README.md`: the App Store privacy answers.
  - Declared, all linked to the user and never used for tracking: e-mail address, user ID, device ID, other user content, product interaction.
  - Purposes: app functionality for all of them, plus analytics for user ID and product interaction.
  - Not declared: no browsing history, no tracking.
- [ ] 5.2 Fill App Store Connect → Confidentialité de l'app from 5.1, and the privacy policy URL [manual, owner]
- [ ] 5.3 New TestFlight build (iOS + macOS, production extension) and a manual pass:
  - erase on the iPhone, then check the Mac wipes at its next sync;
  - the deletion link opens the site;
  - a new capture syncs without its address.

## 6. Specs and gates

- [x] 6.1 Amend `add-lingua-backend`'s `lingua-sync` requirements « Complete card synchronisation, media excluded » and « Strict allow-list of what reaches the server »: a card goes up without its page address (see D7)
- [ ] 6.2 `python3 scripts/check_ci_units.py`; `openspec validate add-lingua-privacy-controls --strict` and `openspec validate add-lingua-backend --strict`
