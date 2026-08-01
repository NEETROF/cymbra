## Context

`persist-user-locale` gave the account a `locale` column and `UserPort::set_locale`/
`locale`, but the value is only ever **written** as a side effect of the auth email
flows and **read** only on the mail path. It is not on the public `UserService`
gRPC surface, so neither client can read or write it. Today each client owns its
language entirely on-device:

- **Flutter** (`apps/music`): `AppLocale` notifier (`state/app_locale.dart:42`),
  seeded from the device, restored from `PreferencesService` (`prefsKey =
  'app_language'`), mutated by `select(AppLanguage)` (`:81`); precedence
  `resolveLanguage(persisted → device → en)` (`state/app_language.dart:65`). The
  account is read after login by `SessionNotifier._resolveAuthenticated`
  (`state/session_notifier.dart:68`) via `AccountService.getAccount()`. Supported:
  `en/fr/it/es`.
- **Back-office** (`apps/back-office`): vue-i18n; `i18n/index.ts` `setLocale(locale)`
  (`:40`) updates `i18n.global.locale` + `localStorage['cymbra.bo.locale']`;
  `detectLocale()` = saved → `navigator.language` → en. The `user` gRPC client is
  already wired in `lib/transport.ts` behind the `api()`/`setClientsForTest` seam.
  Supported: `en/fr` only. Auth state (`stores/auth.ts`) is derived from JWT claims;
  it does **not** call `GetAccount` today.

## Goals / Non-Goals

**Goals:**
- Make the account the source of truth for the user's language across devices, for
  both UI and transactional email.
- Push the user's selection to the account; reconcile the account language into the
  UI after sign-in.
- Respect each client's supported set and never block pre-auth startup.

**Non-Goals:**
- No change to the email-rendering layer or precedence (request → stored → English
  stays; the explicit preference does not override the request locale).
- No per-device language override (a single account language) and no new email types.
- No backfill; accounts with `NULL` locale adopt the first client selection.

## Decisions

### D1 — Dedicated RPC surface, `SetLocale` returns the updated `Account`
Extend `backend/user-port/proto/user.proto`: add `optional string locale` to
`Account`; add `message SetLocaleRequest { string locale = 1; }` and
`rpc SetLocale(SetLocaleRequest) returns (Account)`. Returning the account (like
`UpdateAccount`) lets the client read back the canonical stored value in one call.
The caller is the authenticated identity (token extension), never the body — mirrors
every other write in `grpc.rs`. This keeps `locale` a **backend-owned column**, not
part of the client-owned `preferences` JSONB (persist-user-locale D1).
*Backend wiring:* add `locale: Option<String>` to the `Account` DTO
(`user-port/src/lib.rs`), have `get_account` (repo `Fake` + `pg.rs`) select/return it,
map it in `grpc.rs::to_proto`, and implement `set_locale` → `port.set_locale`
(already a no-op on empty).

### D2 — Flutter: keep `AppLocale` local-only; isolate the sync in a dedicated seam
`AppLocale` stays the pure local-language notifier. The server sync is a **separate,
isolated concern** so the four architecture rules hold (UI never calls a service; a
provider never pokes a sibling; no awaiting an action's return in the UI; listener
side effects isolated):
- A dedicated **language-sync listener** near the top of the app subtree
  `ref.listen`s `sessionProvider`. On the transition to *authenticated* with
  `account.locale`:
  - **set + displayable** → `AppLocale.applyFromAccount(lang)` (persists locally, no
    push);
  - **unset (`null`)** → push the current local language via `AccountService.setLocale`
    (adopt-local-when-unset);
  - **set but not displayable** → do nothing (UI keeps its fallback; stored value
    untouched).
- The **push on user change** fires from `AppLocale.select` when signed in: `select`
  is already the single mutation; it persists locally and, when a session is
  authenticated, calls `AccountService.setLocale`. (`AppLocale`, a notifier, may call
  the service.)
- **Echo guard:** `applyFromAccount` must not trigger a re-push. Track the
  last-known server value (in the sync seam) and skip a push when the change equals
  it; `SetLocale` is idempotent, so this is a noise/round-trip optimization, not a
  correctness lever.

`AccountService`/`Account` (`services/account_service.dart`) gain a `locale` field +
`setLocale(String)`, implemented in `GrpcAccountService` (`services/grpc_client.dart`,
mapping the new proto field in `_toAccount`, adding a `setLocale` adapter via
`_authed`). Injected through the existing `accountServiceProvider` seam (fakes in
tests).

### D3 — Back-office: a `locale` Pinia store; `i18n` stays pure
`i18n/index.ts::setLocale` remains a pure UI mutation (no API call — vue rule).
Add a `stores/locale.ts` Pinia store mirroring `stores/roles.ts`: a `choose(locale)`
action that calls `api().user.setLocale(...)` then `setLocale(locale)`, and a
`reconcile()` action that reads `api().user.getAccount()` and, when the returned
locale is in `SUPPORTED_LOCALES`, applies it via `setLocale`. Request state is one
`Async<T>` union (`lib/async.ts`), matched exhaustively. `App.vue`'s language buttons
call the store's `choose` instead of `setLocale` directly. `reconcile()` is triggered
after login from `stores/auth.ts` `bootstrap()`/`refresh()`. The Playwright fake-client
seam (`lib/e2e-seam.ts`) is extended with the new `setLocale`/`getAccount(locale)`.

### D4 — Reconciliation policy: server-wins-when-set, local-wins-when-unset
The account value overrides the local choice **only when it is set and displayable**;
a `NULL` account never blanks or resets the UI (local wins and is pushed up). An
account language a client can't display leaves both the UI fallback and the stored
value alone. This is the same last-writer-wins column from persist-user-locale, now
also written by an explicit user action.

### D5 — No pre-auth server read (inherent)
`GetAccount`/`SetLocale` are authenticated, so the account language simply cannot be
read on the login screen / cold start; the first frame uses the locally-persisted
(or device/browser) language and the account reconciles only after auth resolves.
This is a property of the surface, recorded as a rule so no one adds a blocking
pre-auth fetch.

### D6 — Email precedence unchanged; the explicit choice feeds it for free
`SetLocale` writes the same column the email flows already read as their fallback
(request → stored → English). So a user's explicit selection automatically improves
server-initiated mail without any change to the rendering layer or precedence.

### D7 — Send the client's short language code; no server-side enum
Clients send their own code (`en`/`fr`/`it`/`es`). The column stays free-text and the
email layer's `SupportedLocale::parse` already tolerates tags/unknowns (→ English),
so the server does not validate against a fixed set (keeps the app's and back-office's
differing supported sets independent of the backend).

## Risks / Trade-offs

- **Reconcile ↔ push echo/loop.** → Split methods (`applyFromAccount` = no push) +
  last-server-value guard; `SetLocale` is idempotent regardless.
- **Extra `GetAccount` in the back-office** (which today only decodes JWT claims). →
  One authenticated read after login; acceptable. *Alternative (rejected):* put locale
  in the token claims — token bloat + staleness across a mid-session change.
- **Undisplayable account locale** (e.g. `es` on the `en/fr` back-office). → UI falls
  back locally; the stored value is preserved so another client can still honor it.
- **Signed-out change to an account that already has a server value** is local-only
  until the next signed-in change (see Open Questions).
- **Multi-device/tab flapping** → last-writer-wins by design (persist-user-locale D3).

## Migration Plan

1. Proto: add `Account.locale` + `SetLocale`; `melos run gen-grpc` (Dart + Vue).
2. Backend: `Account` DTO `locale`, `get_account` select/return, `grpc.rs`
   `set_locale` + mapping; unit tests; coverage ≥ 80%.
3. Flutter: `AccountService`/`Account`/`GrpcAccountService` + `AppLocale.select` push,
   `applyFromAccount`, the language-sync listener; widget/unit tests with fakes.
4. Back-office: `stores/locale.ts` + `App.vue` wiring + `auth.ts` reconcile trigger +
   e2e-seam; unit/Playwright tests.
*Rollback:* additive proto field + additive RPC + additive client code; revert code,
the column/field can stay harmlessly.

## Open Questions

- **Queue a signed-out change to push on next sign-in?** Default: no — the
  adopt-local-when-unset path covers fresh accounts; an existing account with a server
  value keeps it until the user changes the language while signed in. Revisit if the
  gap bites.
- **Server-side locale validation/normalization** (reject/normalize unknown tags)?
  Default: accept any tag (D7); clients own their supported sets.
