## ADDED Requirements

### Requirement: The app logs through one hierarchical logger with levels

The app SHALL log through a single root logger (`cymbra`) with per-area child loggers
and four levels — `FINE` (development detail), `INFO` (milestones), `WARNING` (handled
failure, with the exception), `SEVERE` (unexpected failure, incl. uncaught Flutter and
platform errors). Production code under `lib/` MUST NOT use `print` / `debugPrint`
directly (lint gate); a `catch` that swallows an error SHALL log at `WARNING` or higher
with the cause, or at `FINE` when the swallow is intentional and named.

#### Scenario: A handled platform failure leaves a trace

- **WHEN** a native sign-in SDK throws before any token exists
- **THEN** a `WARNING` line with the exception is emitted on the `cymbra.auth` logger and the user still sees the generic message

#### Scenario: Ad-hoc prints are refused

- **WHEN** a `debugPrint(` or `print(` call is added under `lib/` outside the sink
- **THEN** analysis / CI fails

### Requirement: Release builds write to the platform log; debug builds to the console

In release/profile builds the sink SHALL forward `INFO`+ lines to the platform's log
(iOS/macOS `os_log`, subsystem `app.cymbra.music`, category = area; Android logcat, tag
`cymbra.<area>`; stderr on Windows/Linux) so they are readable on TestFlight / store
builds with the OS tools; `FINE` is dropped. In debug builds the sink SHALL print to the
console. A failing platform sink MUST NOT throw or block the caller.

#### Scenario: TestFlight build

- **WHEN** a `WARNING` is logged in a store build on macOS
- **THEN** it appears in Console.app / `log stream` under subsystem `app.cymbra.music`

#### Scenario: Sink failure is harmless

- **WHEN** the platform channel is unavailable
- **THEN** logging still returns immediately and the app keeps running

### Requirement: Secrets never reach a log line

The sink SHALL redact, before formatting, bearer tokens, JWT-shaped strings, e-mail
addresses and access-code-shaped strings; call sites SHALL log identifiers and outcomes,
never payloads. Redaction SHALL be unit-tested against fixtures.

#### Scenario: A token in a message is masked

- **WHEN** a message contains `Bearer eyJ…` or an e-mail
- **THEN** the written line contains the redacted forms only

### Requirement: The user can copy or share a diagnostic journal from Help

The app SHALL keep an in-memory ring buffer of the last 500 `INFO`+ lines and expose it
on the Help screen as a "Diagnostic log" with **Copy** (clipboard) and, when a share
sheet is available, **Share**. The journal SHALL start with a header (app version and
build, platform and OS version, locale, plan kind) and MUST NOT contain the user id,
e-mail or any token. Nothing is transmitted automatically.

#### Scenario: A tester hands over the journal

- **WHEN** the tester opens Help → Diagnostic log and taps Copy
- **THEN** the clipboard holds the header + the buffered lines, redacted, newest last

#### Scenario: Buffer is bounded

- **WHEN** more than 500 lines have been logged
- **THEN** only the most recent 500 remain, in order
