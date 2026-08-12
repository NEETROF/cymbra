## Why

A notification that arrives while the user is **in the app** currently does one of
three different things depending on the platform: Android drops it into Dart's
`onMessage` where nothing handles it, iOS suppresses it, and macOS shows a system
banner. Nobody chose that — it is three OS defaults leaking through.

Two problems follow. The behaviour is **inconsistent**, so a category that reads
well on macOS is invisible on the other two. And it is **uniform per app** when the
right answer is per *category*: an evening "you're about to lose your streak"
reminder is absurd while the user is practising — they are doing the thing — but
"someone just beat your score" is worth surfacing the moment it happens.

The push platform (`platform-push-notifications`) deliberately left this out: with
no concrete notification type declared, there was nothing to decide. The first
types are now coming, so the decision needs a home.

## What Changes

- **Per-category foreground behaviour, as a back-office flag** — a third
  per-category key, `notifications.category.<id>.foreground`, beside the `.enabled`
  and `.hour` keys that already exist. Default: **not** surfaced, which is today's
  behaviour on iOS/Android and the safer choice. Hot-reloadable, so revisiting
  "do we interrupt someone mid-practice?" is a click, not an app release.
- **In-app presentation, not a system banner** — a foreground notification is shown
  as an **in-app** surface (a dismissible banner inside the app), not as an OS
  notification. The user is already looking at the app; a system banner over the app
  it came from is redundant, and this avoids a native local-notification dependency
  entirely.
- **One consistent baseline across platforms** — macOS is brought in line with iOS
  and Android by suppressing the system banner while the app is in the foreground,
  so all three platforms behave identically and the *category* decides.
- **The decision travels with the message** — the dispatcher resolves the flag and
  attaches it to every message it sends, so the client presents what it is told
  rather than deriving it from a compiled-in list. A category the running app has
  never heard of still behaves correctly.
- **Tap routing** — an in-app banner honours the same `data` payload the platform
  already transports (e.g. `{"route": "/practice"}`), so tapping it goes where the
  system notification would have.
- **The message is already localized** by the feature that owns the category (the
  platform transports copy, it does not translate), so the banner renders it as-is.

Out of scope: OS-level local notifications (`flutter_local_notifications`) and any
new native dependency; a notification centre or history; grouping, rate-limiting or
coalescing of foreground banners; changing anything about background delivery.

## Capabilities

### New Capabilities
<!-- None. This extends the existing push platform. -->

### Modified Capabilities
- `platform-push-notifications`: gains foreground presentation — a per-category
  declaration, an in-app banner as the presentation surface, a consistent
  cross-platform baseline, and tap routing through the existing `data` payload.

## Impact

- **App** (`apps/music`): a foreground message listener (a dedicated listener
  widget, per the Riverpod rules) reacting to `onMessage`; an in-app banner widget;
  the Apple foreground presentation options set to suppress, so macOS matches iOS.
  `PushCategory` is **unchanged** — presentation is not a client constant.
- **Backend**: one flag-key builder beside the existing two, and the dispatcher
  attaching the resolved value to each message. The selection core, the send seam
  and the wire format are untouched.
- **Back office**: the *generic* flags panel picks the new key up on its own
  (prefix discovery), but the dedicated notifications panel renders a structured
  per-category row (enabled / hour) and silently dropped unknown suffixes — it
  gains a **Foreground** toggle column so a category's three controls read as one
  definition. (Discovered during the manual pass; the original assumption of "no
  BO change" held only for the generic panel.)
- **Coverage**: Rust unit tests for the resolved flag reaching the message; Flutter
  widget/state tests over the faked push seam — banner shown vs suppressed by what
  the message says, tap routing, dismissal, and the default (attribute absent ⇒
  nothing shown). No new native code, so nothing lands outside the measured surface.
