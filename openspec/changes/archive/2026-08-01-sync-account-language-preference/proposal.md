## Why

`persist-user-locale` added a `locale` column on the account plus `UserPort::set_locale`/
`locale`, but nothing **exposes** it: it is only written as a side effect of the
email flows and read only on the mail path. The user's language today lives purely
on-device (Flutter persists it locally; the back-office keeps it in `localStorage`),
so a user who picks a language on one device sees the default on the next, and a
server-initiated email still has only a stale heuristic to go on. Making the account
the **source of truth** for language — written when the user picks one, read back
after sign-in — gives a single, deterministic language across devices for both the
UI and transactional email.

## What Changes

- **Expose the account locale over RPC.** `UserService` gains a `SetLocale` RPC
  (writes the caller's account locale via the existing `UserPort::set_locale`) and
  `GetAccount` starts returning the stored `locale`. Dedicated field/RPC — **not**
  folded into the client-owned `preferences` JSONB (persist-user-locale D1 keeps
  `locale` backend-owned).
- **Clients push the user's selection to the server.** When the user changes the
  interface language (Flutter settings drawer; back-office language control), the
  client calls `SetLocale` so the account remembers it.
- **Clients reconcile the server language after sign-in.** Once authenticated, a
  client reads the account `locale` and applies it as the active UI language. The
  **server value wins** over the locally-persisted choice when it is set; when the
  account has **no** stored locale (`NULL`), the local selection wins and is pushed
  up to the server (so a fresh account never blanks the UI).
- **First frame stays local.** Reading the server locale needs authentication, so
  the login screen / cold start still uses the locally-persisted (or device/browser)
  language; the server value is applied only *after* auth resolves, as a
  reconciliation step — never a blocking pre-auth fetch.
- **A client applies only what it can display.** Each client maps the account locale
  through its own supported set (Flutter `en/fr/it/es`; back-office `en/fr`); an
  account language the client cannot render falls back to that client's default for
  the UI **without** overwriting the stored account locale.
- **Email precedence is unchanged.** Transactional mail stays request → stored →
  English; the explicit preference does **not** override the request locale for
  client-triggered mail (persist-user-locale D3/D4).

## Capabilities

### New Capabilities
- `account-language-sync`: the account language is exposed over the authenticated
  user RPC (read via `GetAccount`, written via `SetLocale`); clients push the user's
  selection to the server and, after sign-in, reconcile the server language into the
  active UI language (server-wins-when-set, local-wins-and-push-up when unset),
  applying only a language the client can display and never a pre-auth blocking read.

### Modified Capabilities
- `app-localization`: the persisted-language precedence gains a post-sign-in
  reconciliation tier — for a signed-in account whose server locale is set, that
  language reconciles over the locally-persisted choice (and is persisted locally),
  whereas startup/cold-launch behavior (local persisted → device → English) is
  unchanged.

<!-- The backend `locale` column + `UserPort::set_locale`/`locale` come from the
     in-flight `persist-user-locale` change (its `user-locale-preference` capability),
     which is a dependency; this change only adds the RPC surface + `Account.locale`
     field on top, so it is covered by the new capability's requirements rather than a
     delta against that unarchived spec. -->

## Impact

- **Proto/API** (`backend/user-port/proto/user.proto`): `Account` gains
  `optional string locale`; new `SetLocaleRequest { string locale }` +
  `SetLocaleResponse {}` + `rpc SetLocale`. Regenerate the gRPC clients
  (Dart + Vue) via `melos run gen-grpc`.
- **Backend**:
  - `backend/user-port/src/lib.rs` — add `locale: Option<String>` to the `Account`
    DTO.
  - `backend/user/src/repo.rs` + `pg.rs` — `get_account` also selects/returns
    `locale` (Fake + Postgres).
  - `backend/user/src/grpc.rs` — implement `set_locale` (identity from the internal
    token → `port.set_locale`) and carry `locale` in `to_proto`/`get_account`.
- **Flutter** (`apps/music`): the language notifier calls `SetLocale` on change; a
  dedicated post-login listener reads `GetAccount.locale` and applies/persists it
  (flutter-riverpod-architecture rules — UI never calls the service; only the
  notifier does; side effects isolated in a listener widget). Reuses the existing
  user gRPC client seam + local-preferences store.
- **Back-office** (`apps/back-office`): a Pinia store/composable behind the
  injectable client seam calls `SetLocale` on `setLocale` and applies the account
  locale after login; async state as one `ts-pattern` `Async<T>` union
  (vue-frontend-architecture rules).
- **Depends on** `persist-user-locale` (the `locale` column + `UserPort::set_locale`/
  `locale`) already on this branch.
- **No** email-rendering change, no new email types, no per-device language override
  (a single account language).
