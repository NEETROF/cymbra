## MODIFIED Requirements

### Requirement: Persisted Language Choice

A language chosen by the user SHALL be persisted locally and restored on the next
launch, taking precedence over the device locale. If the persisted language is no
longer supported, the app SHALL fall back to the device locale (or English) and
persist that fallback. The language selection SHALL be held in an injectable
provider backed by the local preferences store so tests can drive it with fakes.

For a **signed-in** account, the account's server-stored language SHALL reconcile
over the locally-persisted choice **after authentication resolves**: when the
account has a stored language the app can display, that language SHALL become the
active language and SHALL be persisted locally. Startup / cold-launch resolution is
unchanged (locally-persisted → device → English) and SHALL NOT block on any
server read; the server reconciliation happens only after sign-in. When the account
has no stored language, the locally-persisted choice SHALL be kept (and is adopted
by the account — see the `account-language-sync` capability).

#### Scenario: Language survives a restart

- **WHEN** the user selects Spanish and later relaunches the app
- **THEN** the app starts in Spanish regardless of the device locale

#### Scenario: Persisted language chosen over device locale

- **WHEN** a supported language has been persisted and differs from the device
  locale
- **THEN** the persisted language is used at startup

#### Scenario: Unsupported persisted language falls back

- **WHEN** the persisted language is no longer among the supported languages
- **THEN** the app resolves to the device locale (or English) and persists that
  as the new choice

#### Scenario: Startup does not block on the account language

- **WHEN** the app is launched before authentication resolves
- **THEN** the active language is resolved from the persisted choice (or device
  locale, or English) with no server read, and the account language is applied only
  after sign-in

#### Scenario: Account language reconciles over the persisted choice after sign-in

- **WHEN** a user whose account language is French signs in while the app is showing
  the locally-persisted English
- **THEN** the app switches to French and persists French locally
