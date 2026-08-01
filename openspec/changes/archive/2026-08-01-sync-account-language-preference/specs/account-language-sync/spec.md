## ADDED Requirements

### Requirement: Account language is readable and writable over the authenticated RPC

The user service SHALL let an authenticated caller write and read **their own**
account language. A `SetLocale` operation SHALL persist the supplied locale on the
caller's account (via the existing account-locale store), and the account-read
operation (`GetAccount`) SHALL return the stored locale (absent when none has been
recorded). The account whose locale is written or read SHALL be the authenticated
caller's, resolved from the request's identity — never from the request body. An
empty supplied locale SHALL be a no-op that leaves any stored value unchanged.

#### Scenario: A written locale is read back

- **WHEN** an authenticated caller invokes `SetLocale` with `fr` and then reads their account
- **THEN** the account read reports the stored locale as `fr`

#### Scenario: The write targets only the caller's account

- **WHEN** an authenticated caller invokes `SetLocale`
- **THEN** only the caller's own account locale is written, identified from the request identity rather than any account id in the body

#### Scenario: An empty locale does not clear the stored value

- **WHEN** an account whose stored locale is `fr` receives a `SetLocale` with an empty locale
- **THEN** the stored locale remains `fr`

#### Scenario: An account with no recorded locale reads as absent

- **WHEN** an authenticated caller whose account has never recorded a locale reads their account
- **THEN** the returned locale is absent

### Requirement: A language selection is recorded on the account

A client SHALL record the user's interface-language selection on the account when
the user is **signed in**: on such a change the client calls `SetLocale` so the
account remembers the user's most recent explicit choice. A language change made
while the user is **not** signed in SHALL be applied and persisted locally only,
with no account write (there is no account to write to).

#### Scenario: Signed-in selection is pushed to the account

- **WHEN** a signed-in user selects a different interface language
- **THEN** the client calls `SetLocale` with the chosen language so the account stores it

#### Scenario: Signed-out selection stays local

- **WHEN** a user who is not signed in selects a different interface language
- **THEN** the selection is applied and persisted locally and no account write is attempted

### Requirement: The account language reconciles into the UI after sign-in

After authentication resolves, a client SHALL reconcile the account's stored
language into the active interface language:
- when the account has a stored locale that the client can display, that language
  SHALL become the active interface language — overriding the locally-persisted
  choice — and SHALL be persisted locally so it survives the next launch;
- when the account has **no** stored locale, the client SHALL keep the local
  selection and push it up to the account via `SetLocale`, so a fresh account never
  blanks or resets the user's language;
- when the account's stored locale is one the client **cannot** display, the client
  SHALL keep its own resolved/fallback language for the UI and SHALL NOT overwrite
  the stored account locale.

This reconciliation SHALL be a side effect isolated from view rendering (not
performed inline during a build/render pass).

#### Scenario: Server language wins over the local choice

- **WHEN** a user whose account locale is `fr` signs in on a client currently showing English
- **THEN** the interface switches to French and French is persisted locally

#### Scenario: Cross-device propagation

- **WHEN** a user sets the language to French on one device and later signs in on a second device that had English persisted
- **THEN** the second device's interface becomes French after sign-in

#### Scenario: Unset account locale adopts the local choice

- **WHEN** a user whose account has no stored locale signs in with Spanish selected locally
- **THEN** the interface stays Spanish and the client writes `es` to the account via `SetLocale`

#### Scenario: Undisplayable account locale does not disturb the UI or the stored value

- **WHEN** a user whose account locale is `es` signs in on a client that supports only `en`/`fr`
- **THEN** the client renders its own fallback language for the UI and leaves the stored account locale as `es`

### Requirement: The account language is never read before authentication

The interface language SHALL be resolvable before authentication with no read of
the account language: at cold start / on the pre-authentication screen, resolution
depends only on locally-persisted and device/browser inputs. The account language
SHALL be consulted only after authentication has resolved.

#### Scenario: Cold start does not block on the server

- **WHEN** the app is launched and the sign-in screen is shown before authentication
- **THEN** the interface language is resolved from the locally-persisted choice (or device/browser locale, or the default) without any pre-auth account read
